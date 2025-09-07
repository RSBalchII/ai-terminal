//! Layout module for the AI Terminal UI
//! 
//! This module provides layout management functionality for the terminal UI.

pub mod manager;
pub mod pane;
pub mod tab;

pub use manager::LayoutManager;
pub use pane::{PaneManager, SplitOrientation};
pub use tab::TabManager;