# AI Terminal Enhanced Features Design

## Overview
Transform the AI Terminal into a Warp-terminal-like experience with system access tools, mouse interaction, web search, and advanced code assistance.

## 1. System Access Tools Architecture

### Core Tool Categories

#### File System Tools
```rust
pub enum FileSystemTool {
    List { path: String, recursive: bool },
    Read { path: String, lines: Option<(usize, usize)> },
    Write { path: String, content: String, append: bool },
    Search { path: String, pattern: String, file_types: Vec<String> },
    Find { path: String, name_pattern: String },
    Tree { path: String, depth: Option<usize> },
    Info { path: String }, // size, permissions, modified time
}
```

#### Process Management Tools  
```rust
pub enum ProcessTool {
    List { filter: Option<String> },
    Kill { pid: u32, force: bool },
    Monitor { pid: Option<u32> }, // top-like monitoring
    Systemctl { action: String, service: String },
}
```

#### Network Tools
```rust
pub enum NetworkTool {
    Curl { url: String, method: String, headers: HashMap<String, String> },
    Ping { host: String, count: Option<u32> },
    Netstat { listening: bool },
    Port { port: u16 }, // check if port is open
}
```

#### System Information Tools
```rust
pub enum SystemTool {
    DiskUsage { path: Option<String> },
    Memory,
    CPU,
    Uptime,
    Users,
    Environment { var: Option<String> },
}
```

#### Git Operations
```rust
pub enum GitTool {
    Status,
    Add { files: Vec<String> },
    Commit { message: String },
    Push { remote: String, branch: String },
    Pull { remote: String, branch: String },
    Branch { list: bool, create: Option<String> },
    Diff { file: Option<String> },
    Log { count: Option<usize> },
}
```

#### Package Management
```rust
pub enum PackageTool {
    AptSearch { query: String },
    AptInstall { packages: Vec<String> },
    CargoSearch { query: String },
    CargoInstall { package: String },
    NpmSearch { query: String },
    NpmInstall { package: String, global: bool },
}
```

### Tool Execution Framework

```rust
pub struct ToolExecutor {
    allowed_tools: HashSet<String>,
    security_settings: SecuritySettings,
    execution_timeout: Duration,
}

impl ToolExecutor {
    pub async fn execute(&self, tool: SystemTool) -> Result<ToolResult> {
        // Security check
        self.check_permissions(&tool)?;
        
        // Execute with timeout
        timeout(self.execution_timeout, self.run_tool(tool)).await
    }
    
    pub fn is_dangerous(&self, tool: &SystemTool) -> bool {
        // Check if tool modifies system state
        matches!(tool, 
            SystemTool::Process(ProcessTool::Kill { .. }) |
            SystemTool::FileSystem(FileSystemTool::Write { .. }) |
            // ... other dangerous operations
        )
    }
}
```

## 2. Tavily Web Search Integration

### API Integration
```rust
pub struct TavilyClient {
    api_key: String,
    client: reqwest::Client,
}

#[derive(Serialize)]
pub struct TavilySearchRequest {
    query: String,
    search_depth: String, // "basic" or "advanced"
    include_domains: Option<Vec<String>>,
    exclude_domains: Option<Vec<String>>,
    max_results: Option<u32>,
    include_answer: bool,
    include_raw_content: bool,
}

#[derive(Deserialize)]
pub struct TavilySearchResult {
    query: String,
    answer: Option<String>,
    results: Vec<SearchResultItem>,
}

#[derive(Deserialize)]
pub struct SearchResultItem {
    title: String,
    url: String,
    content: String,
    score: f64,
    published_date: Option<String>,
}
```

### Integration in Python Bridge
```python
# Add to python-bridge/tavily_search.py
import requests
import os

class TavilySearch:
    def __init__(self):
        self.api_key = os.getenv('TAVILY_API_KEY')
        self.base_url = "https://api.tavily.com/search"
    
    def search(self, query, max_results=5):
        if not self.api_key:
            return {"error": "TAVILY_API_KEY not set"}
        
        payload = {
            "api_key": self.api_key,
            "query": query,
            "search_depth": "basic",
            "include_answer": True,
            "max_results": max_results
        }
        
        response = requests.post(self.base_url, json=payload)
        return response.json()
```

### Search Commands
- `search: <query>` - General web search
- `search code: <query>` - Code-focused search  
- `search docs: <query>` - Documentation search
- `search github: <query>` - GitHub repository search

## 3. Mouse Interaction System

### Clickable UI Elements
```rust
pub struct ClickableArea {
    pub rect: Rect,
    pub action: ClickAction,
    pub hover_text: Option<String>,
}

pub enum ClickAction {
    // Model management
    SelectModel(String),
    RefreshModels,
    
    // Message interactions
    CopyMessage(usize),
    RegenerateResponse(usize),
    EditMessage(usize),
    
    // Tool actions
    ExecuteTool(String),
    CancelOperation,
    
    // Settings
    ToggleOffline,
    OpenSettings,
    ChangeTheme(String),
    
    // Navigation
    ScrollUp,
    ScrollDown,
    GoToBottom,
    
    // File operations (when displaying file content)
    OpenFile(String),
    EditFile(String),
}
```

### Mouse Event Handling
```rust
impl TerminalApp {
    fn handle_mouse_event(&mut self, mouse: MouseEvent) -> Result<()> {
        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                if let Some(action) = self.get_click_action(mouse.column, mouse.row) {
                    self.execute_click_action(action).await?;
                }
            }
            MouseEventKind::Down(MouseButton::Right) => {
                self.show_context_menu(mouse.column, mouse.row);
            }
            MouseEventKind::Moved => {
                self.update_hover_state(mouse.column, mouse.row);
            }
            _ => {}
        }
        Ok(())
    }
}
```

### Hover Effects and Tooltips
```rust
pub struct HoverState {
    pub current_area: Option<ClickableArea>,
    pub tooltip: Option<Tooltip>,
    pub cursor_style: CursorStyle,
}

pub struct Tooltip {
    pub text: String,
    pub position: (u16, u16),
    pub style: Style,
}
```

### Context Menus
```rust
pub struct ContextMenu {
    pub items: Vec<ContextMenuItem>,
    pub position: (u16, u16),
    pub selected_index: usize,
}

pub struct ContextMenuItem {
    pub label: String,
    pub action: ClickAction,
    pub enabled: bool,
    pub shortcut: Option<String>,
}
```

## 4. Configuration Management System

### Configuration Structure
```rust
#[derive(Serialize, Deserialize, Clone)]
pub struct AppConfig {
    // Appearance
    pub theme: Theme,
    pub font_size: u16,
    pub show_line_numbers: bool,
    
    // Behavior
    pub auto_save: bool,
    pub confirm_dangerous_operations: bool,
    pub max_history_size: usize,
    
    // API Keys
    pub tavily_api_key: Option<String>,
    pub openai_api_key: Option<String>,
    
    // Ollama settings
    pub ollama_url: String,
    pub default_model: Option<String>,
    pub request_timeout: u64,
    
    // Tool permissions
    pub allowed_tools: HashSet<String>,
    pub require_confirmation: HashSet<String>,
    
    // Shortcuts
    pub keybindings: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Theme {
    pub name: String,
    pub background: Color,
    pub foreground: Color,
    pub accent: Color,
    pub error: Color,
    pub success: Color,
    pub warning: Color,
    pub borders: Color,
}
```

### Settings UI
```rust
pub enum SettingsPage {
    General,
    Appearance,
    Behavior,
    Security,
    APIKeys,
    Keybindings,
}

pub struct SettingsUI {
    pub current_page: SettingsPage,
    pub config: AppConfig,
    pub modified: bool,
    pub selected_item: usize,
}
```

### Persistent Storage
- Configuration stored in `~/.config/ai-terminal/config.toml`
- Themes in `~/.config/ai-terminal/themes/`
- History in `~/.config/ai-terminal/history.json`

## 5. Enhanced Code Assistance Features

### Syntax Highlighting
```rust
pub struct CodeBlock {
    pub language: String,
    pub content: String,
    pub highlighted: Vec<HighlightedLine>,
    pub line_numbers: bool,
}

pub struct HighlightedLine {
    pub content: String,
    pub spans: Vec<HighlightSpan>,
}

pub struct HighlightSpan {
    pub text: String,
    pub style: SyntaxStyle,
}
```

### Code Execution Sandbox
```rust
pub struct CodeExecutor {
    pub supported_languages: HashSet<String>,
    pub execution_timeout: Duration,
    pub max_output_size: usize,
}

impl CodeExecutor {
    pub async fn execute(&self, code: &str, language: &str) -> Result<ExecutionResult> {
        match language {
            "rust" => self.execute_rust(code).await,
            "python" => self.execute_python(code).await,
            "javascript" => self.execute_js(code).await,
            "bash" => self.execute_bash(code).await,
            _ => Err(anyhow!("Unsupported language: {}", language))
        }
    }
}

pub struct ExecutionResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub execution_time: Duration,
}
```

### File Editing Integration
```rust
pub struct FileEditor {
    pub current_file: Option<PathBuf>,
    pub content: String,
    pub modified: bool,
    pub cursor_position: (usize, usize),
    pub syntax_highlighting: bool,
}

pub enum EditAction {
    Open(PathBuf),
    Save,
    SaveAs(PathBuf),
    Undo,
    Redo,
    Find(String),
    Replace { find: String, replace: String },
    GoToLine(usize),
}
```

## 6. Implementation Phases

### Phase 1: Core System Tools (Week 1)
- Implement basic file system tools (list, read, write)
- Add process management (ps, kill)
- Create security framework for tool execution
- Basic tool command parsing

### Phase 2: Web Search & API Keys (Week 2)  
- Integrate Tavily search API
- Implement secure API key storage
- Add search result formatting
- Create search command interface

### Phase 3: Mouse Interaction Foundation (Week 3)
- Enable mouse events in crossterm
- Implement clickable areas system
- Add basic hover effects
- Create context menu framework

### Phase 4: Configuration System (Week 4)
- Build configuration structure
- Implement settings UI
- Add theme system
- Create persistent storage

### Phase 5: Advanced Code Features (Week 5)
- Add syntax highlighting
- Implement code execution sandbox
- Create file editing capabilities
- Add project analysis tools

### Phase 6: Polish & Integration (Week 6)
- Refine mouse interactions
- Add more clickable elements
- Implement advanced tooltips
- Performance optimization

## 7. UI Mockups

### Enhanced Chat Interface
```
┌─ AI Terminal ─────────────────────────── [⚙️ Settings] [🔍 Search] ─┐
│ [👤 user] How do I check disk usage?                                │
│                                                                      │  
│ [🤖 assistant] I can help you check disk usage! Here are options:   │
│ [💡 Run Tool] df -h                   [📋 Copy] [🔄 Regenerate]      │
│ [💡 Run Tool] du -sh /home/user       [📋 Copy] [🔄 Regenerate]      │
│                                                                      │
│ Would you like me to run one of these commands?                     │
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘
┌─ Input ──────────────────────────────────────────────────────────────┐
│ > search: rust async best practices                                  │
└──────────────────────────────────────────────────────────────────────┘
┌─ Status ─ Model: mistral-nemo [🔄] │ 🌐 Online │ ⏳ Searching... ─────┐
```

### Settings Panel
```
┌─ Settings ───────────────────────────────────────────────────────────┐
│ ┌─ Navigation ────┐ ┌─ General ──────────────────────────────────────┐│
│ │ › General       │ │ Theme:           [Dark ▼] [Preview]           ││
│ │   Appearance    │ │ Font Size:       [14 ─────○──] 18             ││
│ │   Behavior      │ │ Auto-save:       [✓] Enable                   ││
│ │   Security      │ │ History Size:    [1000 ─○────] entries        ││
│ │   API Keys      │ │ Confirm Actions: [✓] Dangerous operations     ││
│ │   Keybindings   │ │                                               ││
│ └─────────────────┘ │ [Apply] [Reset] [Cancel]                      ││
│                     └───────────────────────────────────────────────────│
└──────────────────────────────────────────────────────────────────────────┘
```

This design provides a roadmap for transforming your AI Terminal into a powerful, interactive development tool with Warp-like capabilities while maintaining the robust architecture we've built.

## Next Steps
1. **Fix model switching freeze** ✅ - Done!
2. **Start with Phase 1** - Implement core system tools
3. **Set up Tavily API key management** 
4. **Begin mouse interaction foundation**

Would you like me to start implementing any of these features, or would you prefer to test the model switching fix first?
