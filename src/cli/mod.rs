//! CLI module for the Pomodoro Timer.
//!
//! This module provides the command-line interface:
//! - `commands`: Command definitions using clap derive
//! - `client`: IPC client for daemon communication
//! - `display`: Output formatting and display logic

pub mod client;
pub mod commands;
pub mod display;

pub use client::IpcClient;
pub use commands::{Cli, Commands, StartArgs};
pub use display::Display;
