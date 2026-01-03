//! Pomodoro Timer Library
//!
//! This library provides the core functionality for the Pomodoro Timer CLI.
//! It includes:
//! - Timer engine for managing Pomodoro sessions
//! - IPC server/client for daemon-CLI communication
//! - CLI command parsing and display utilities
//! - Type definitions for configuration and state

pub mod cli;
pub mod daemon;
pub mod types;

// Re-export commonly used types for convenience
pub use types::{
    IpcRequest, IpcResponse, PomodoroConfig, ResponseData, StartParams, TimerPhase, TimerState,
};
