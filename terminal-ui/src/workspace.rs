use super::TerminalSession;

pub struct Tab {
    pub app: TerminalSession,
    pub title: String,
}

impl Tab {
    pub fn new(app: TerminalSession, title: String) -> Self {
        Self { app, title }
    }
}
