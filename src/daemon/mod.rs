//! Daemon module for the Pomodoro Timer.
//!
//! This module contains the core daemon functionality:
//! - `timer`: Timer engine with state transitions and countdown logic
//! - `ipc`: Unix Domain Socket IPC server for client communication

pub mod ipc;
pub mod timer;

pub use ipc::{IpcError, IpcServer, RequestHandler, DEFAULT_SOCKET_PATH};
pub use timer::{TimerEngine, TimerEvent};
