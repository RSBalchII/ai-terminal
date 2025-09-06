use anyhow::{Context, Result};
use portable_pty::{CommandBuilder, NativePtySystem, PtySize, PtySystem};
use std::io::{BufReader, Read};
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

use crate::command_block::{BlockState, CommandBlock};

/// Events that can occur during command execution
#[derive(Debug, Clone)]
pub enum ExecutionEvent {
    /// Command has started running
    Started,
    
    /// Output received from stdout
    StdoutData(String),
    
    /// Output received from stderr
    StderrData(String),
    
    /// Command completed with exit code
    Completed { exit_code: i32, duration: Duration },
    
    /// Command failed to execute
    Failed(String),
    
    /// Command was cancelled
    Cancelled,
}

/// PTY-based command executor
#[derive(Debug, Clone)]
pub struct PtyExecutor {
    /// Current working directory
    working_dir: String,
    
    /// Shell to use (bash, zsh, etc.)
    shell: String,
}

impl PtyExecutor {
    /// Create a new PTY executor
    pub fn new() -> Result<Self> {
        // Get current working directory
        let working_dir = std::env::current_dir()
            .context("Failed to get current directory")?
            .to_string_lossy()
            .to_string();
            
        // Detect shell from environment
        let shell = std::env::var("SHELL").unwrap_or_else(|_| {
            if cfg!(windows) {
                "cmd.exe".to_string()
            } else {
                "/bin/bash".to_string()
            }
        });
        
        Ok(Self {
            working_dir,
            shell,
        })
    }
    
    /// Execute a command and stream events
    pub async fn execute(
        &self,
        command: &str,
        event_tx: mpsc::UnboundedSender<ExecutionEvent>,
    ) -> Result<()> {
        info!("Executing command: {}", command);
        let start_time = Instant::now();
        
        // Send start event
        event_tx.send(ExecutionEvent::Started)
            .map_err(|e| anyhow::anyhow!("Failed to send start event: {}", e))?;
        
        // Set up PTY
        let pty_size = PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        };
        
        let pty_system = NativePtySystem::default();
        let pair = pty_system
            .openpty(pty_size)
            .context("Failed to open PTY")?;
        
        // Build the command
        let mut cmd = CommandBuilder::new(&self.shell);
        cmd.arg("-c");
        cmd.arg(command);
        cmd.cwd(&self.working_dir);
        
        // Spawn the child process
        let mut child = pair.slave.spawn_command(cmd)
            .context("Failed to spawn command")?;
        
        // Set up readers for stdout/stderr
        let reader = pair.master.try_clone_reader()
            .context("Failed to clone PTY reader")?;
        
        // Create a thread to read output
        let event_tx_clone = event_tx.clone();
        let read_thread = std::thread::spawn(move || {
            let mut buf_reader = BufReader::new(reader);
            let mut buffer = vec![0u8; 4096];
            
            loop {
                match buf_reader.read(&mut buffer) {
                    Ok(0) => break, // EOF
                    Ok(n) => {
                        let raw_bytes = &buffer[..n];
                        
                        // Strip ANSI escape codes for cleaner output
                        let cleaned_bytes = strip_ansi_escapes::strip(raw_bytes);
                        let cleaned = String::from_utf8_lossy(&cleaned_bytes).to_string();
                        
                        // Send output event
                        if let Err(e) = event_tx_clone.send(ExecutionEvent::StdoutData(cleaned)) {
                            error!("Failed to send output event: {}", e);
                            break;
                        }
                    }
                    Err(e) => {
                        error!("Error reading PTY output: {}", e);
                        break;
                    }
                }
            }
        });
        
        // Wait for the process to complete
        let wait_result = tokio::task::spawn_blocking(move || {
            child.wait()
        }).await;
        
        match wait_result {
            Ok(Ok(status)) => {
                let exit_code = if cfg!(unix) {
                    // On Unix, extract exit code
                    status.exit_code() as i32
                } else {
                    // On Windows, assume success if we get here
                    0
                };
                
                let duration = start_time.elapsed();
                
                // Send completion event
                event_tx.send(ExecutionEvent::Completed { exit_code, duration })
                    .map_err(|e| anyhow::anyhow!("Failed to send completion event: {}", e))?;
                    
                info!("Command completed with exit code {} in {:?}", exit_code, duration);
            }
            Ok(Err(e)) => {
                error!("Process wait failed: {}", e);
                event_tx.send(ExecutionEvent::Failed(format!("Process error: {}", e)))
                    .map_err(|e| anyhow::anyhow!("Failed to send error event: {}", e))?;
            }
            Err(e) => {
                error!("Task join error: {}", e);
                event_tx.send(ExecutionEvent::Failed(format!("Execution error: {}", e)))
                    .map_err(|e| anyhow::anyhow!("Failed to send error event: {}", e))?;
            }
        }
        
        // Wait for read thread to complete
        if let Err(e) = read_thread.join() {
            warn!("Read thread panicked: {:?}", e);
        }
        
        Ok(())
    }
    
    /// Execute a command and update a CommandBlock
    pub async fn execute_block(&self, block: &mut CommandBlock) -> Result<()> {
        let (tx, mut rx) = mpsc::unbounded_channel();
        
        // Start execution
        block.start_execution();
        
        // Spawn execution task
        let command = block.command.clone();
        let executor = self.clone();
        let exec_handle = tokio::spawn(async move {
            executor.execute(&command, tx).await
        });
        
        // Process events and update block
        while let Some(event) = rx.recv().await {
            match event {
                ExecutionEvent::Started => {
                    debug!("Command started: {}", block.command);
                }
                ExecutionEvent::StdoutData(data) => {
                    block.append_output(&data, false);
                }
                ExecutionEvent::StderrData(data) => {
                    block.append_output(&data, true);
                }
                ExecutionEvent::Completed { exit_code, duration } => {
                    block.complete(exit_code, duration);
                    break;
                }
                ExecutionEvent::Failed(error) => {
                    block.state = BlockState::Failed;
                    block.append_output(&format!("\n[Error: {}]", error), true);
                    break;
                }
                ExecutionEvent::Cancelled => {
                    block.state = BlockState::Cancelled;
                    break;
                }
            }
        }
        
        // Wait for execution to complete
        exec_handle.await??;
        
        Ok(())
    }
    
    /// Change the working directory
    pub fn set_working_dir(&mut self, dir: String) {
        self.working_dir = dir;
    }
    
    /// Get the current working directory
    pub fn working_dir(&self) -> &str {
        &self.working_dir
    }
}