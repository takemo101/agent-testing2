//! Pomodoro Timer Library
//!
//! This library provides the core functionality for the Pomodoro Timer CLI.
//! It includes:
//! - Timer engine for managing Pomodoro sessions
//! - IPC server/client for daemon-CLI communication
//! - CLI command parsing and display utilities
//! - Type definitions for configuration and state
//! - Native macOS notification system (macOS only)
//! - Menu bar UI with tray icon (macOS only)
//! - Sound playback for timer notifications

pub mod cli;
pub mod daemon;
pub mod launchagent;
pub mod menubar;
pub mod sound;
pub mod types;

// macOS-specific notification system
#[cfg(target_os = "macos")]
pub mod notification;

// Re-export commonly used types for convenience
pub use types::{
    IpcRequest, IpcResponse, PomodoroConfig, ResponseData, StartParams, TimerPhase, TimerState,
};

// Re-export notification types on macOS
#[cfg(target_os = "macos")]
pub use notification::{
    NotificationActionEvent, NotificationError, NotificationManager, NotificationType,
};

// Re-export menubar types
pub use menubar::{
    EventHandler, IconManager, MenuAction, MenuBuilder, MenuConfig, MenuItemConfig, MenuItemId,
    TrayIconManager, TrayUpdate,
};

// Re-export sound types
pub use sound::{
    discover_system_sounds, get_default_sound, play_notification_sound, RodioSoundPlayer,
    SoundError, SoundSource,
};

// Re-export launchagent types
pub use launchagent::{
    install, is_installed, uninstall, LaunchAgentError, PomodoroLaunchAgent, ServiceStatus,
};
