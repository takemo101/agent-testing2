//! Daemon module for the Pomodoro Timer.
//!
//! This module contains the core daemon functionality:
//! - `timer`: Timer engine with state transitions and countdown logic

pub mod timer;

pub use timer::{TimerEngine, TimerEvent};
