//! Widgets module for the AI Terminal UI
//! 
//! This module provides custom widgets for the terminal UI.

pub mod command_palette;
pub mod confirmation_modal;
pub mod command_block;

pub use command_palette::{CommandPalette, Command};
pub use confirmation_modal::{ConfirmationModal, ModalButton};
pub use command_block::CommandBlock;