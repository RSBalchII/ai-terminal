use anyhow::{Context, Result};
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    prelude::*,
    widgets::{block::Title, Block, Paragraph, Wrap},
};
use std::{
    io::{self, stdout, Write},
    process::Command,
    time::Duration,
};

fn main() -> Result<()> {
    let mut app = App::new();
    app.run()
}

struct App {
    should_quit: bool,
    input_buffer: String,
    command_history: Vec<String>,
    output_blocks: Vec<String>,
}

impl App {
    fn new() -> Self {
        Self {
            should_quit: false,
            input_buffer: String::new(),
            command_history: Vec::new(),
            output_blocks: vec!["Welcome to AI-Terminal!\nType a command and press Enter to execute.".to_string()],
        }
    }

    fn run(&mut self) -> Result<()> {
        // Try to set up the terminal UI
        if let Err(e) = self.setup_terminal() {
            eprintln!("Failed to setup terminal UI: {}", e);
            eprintln!("Running in fallback mode...");
            return self.run_fallback();
        }

        let result = self.run_ui();
        
        // Always try to restore terminal state
        let _ = self.restore_terminal();
        
        result
    }

    fn setup_terminal(&mut self) -> Result<()> {
        stdout().execute(EnterAlternateScreen)?;
        enable_raw_mode()?;
        Ok(())
    }

    fn restore_terminal(&self) -> Result<()> {
        disable_raw_mode()?;
        stdout().execute(LeaveAlternateScreen)?;
        Ok(())
    }

    fn run_ui(&mut self) -> Result<()> {
        let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))
            .context("Failed to create terminal")?;

        while !self.should_quit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }

        Ok(())
    }

    fn run_fallback(&mut self) -> Result<()> {
        println!("AI-Terminal (Fallback Mode)");
        println!("===========================");
        println!("Type 'quit' to exit");
        println!();

        loop {
            print!("> ");
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;

            let input = input.trim();
            if input == "quit" {
                break;
            }

            if !input.is_empty() {
                self.input_buffer = input.to_string();
                self.execute_command();
                if let Some(last_output) = self.output_blocks.last() {
                    println!("{}", last_output);
                }
            }
        }

        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(1), // Output area
                Constraint::Length(3), // Input area
            ])
            .split(frame.area());

        // Render output blocks
        let output_text = self.output_blocks.join("\n---\n");

        let output = Paragraph::new(output_text)
            .block(
                Block::bordered()
                    .title(Title::from("AI-Terminal Output".to_string()).alignment(Alignment::Center)),
            )
            .wrap(Wrap { trim: false });
        frame.render_widget(output, layout[0]);

        // Render input area
        let input = Paragraph::new(self.input_buffer.as_str())
            .block(Block::bordered().title("Input"));
        frame.render_widget(input, layout[1]);
    }

    fn handle_events(&mut self) -> Result<()> {
        if event::poll(Duration::from_millis(16))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') => {
                            if self.input_buffer.is_empty() {
                                self.should_quit = true;
                            } else {
                                self.input_buffer.push('q');
                            }
                        }
                        KeyCode::Char(c) => {
                            self.input_buffer.push(c);
                        }
                        KeyCode::Backspace => {
                            self.input_buffer.pop();
                        }
                        KeyCode::Enter => {
                            self.execute_command();
                        }
                        _ => {}
                    }
                }
            }
        }
        Ok(())
    }

    fn execute_command(&mut self) {
        if self.input_buffer.is_empty() {
            return;
        }

        // Add command to history
        self.command_history.push(self.input_buffer.clone());

        // Execute the command and capture output
        let output = match self.run_shell_command(&self.input_buffer) {
            Ok(output) => output,
            Err(e) => format!("Error executing command: {}", e),
        };

        self.output_blocks.push(format!("$ {}\n{}", self.input_buffer, output));

        // Clear input buffer
        self.input_buffer.clear();
    }

    fn run_shell_command(&self, command: &str) -> Result<String> {
        // Split command into program and arguments
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() {
            return Ok("".to_string());
        }

        let program = parts[0];
        let args = &parts[1..];

        // Execute command
        let output = Command::new(program)
            .args(args)
            .output()
            .context("Failed to execute command")?;

        // Format output
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        let mut result = String::new();
        if !stdout.is_empty() {
            result.push_str(&stdout);
        }
        if !stderr.is_empty() {
            if !result.is_empty() {
                result.push('\n');
            }
            result.push_str(&stderr);
        }

        if output.status.success() {
            Ok(result)
        } else {
            Ok(format!("{}\nCommand failed with exit code: {:?}", result, output.status.code()))
        }
    }
}