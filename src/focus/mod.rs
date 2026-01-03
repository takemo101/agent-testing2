//! Focus mode integration for macOS.
//!
//! This module provides integration with macOS Focus Mode through
//! Shortcuts.app. It allows the Pomodoro timer to automatically
//! enable focus mode during work sessions and disable it during breaks.
//!
//! # Overview
//!
//! macOS does not provide a public API for controlling Focus Mode directly.
//! Instead, this module uses Shortcuts.app as an intermediary:
//!
//! 1. User creates shortcuts that enable/disable Focus Mode
//! 2. This module executes those shortcuts via `/usr/bin/shortcuts`
//! 3. Shortcuts.app triggers the Focus Mode change
//!
//! # Requirements
//!
//! - macOS 12 (Monterey) or later
//! - Two shortcuts created in Shortcuts.app:
//!   - "Enable Work Focus" - enables focus mode
//!   - "Disable Work Focus" - disables focus mode
//!
//! # Example
//!
//! ```no_run
//! use pomodoro::focus::{enable_focus, disable_focus, shortcuts_exists};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Check if Shortcuts.app is available
//! if shortcuts_exists() {
//!     // Enable focus mode when work starts
//!     enable_focus("Enable Work Focus").await?;
//!     
//!     // ... work session ...
//!     
//!     // Disable focus mode when break starts
//!     disable_focus("Disable Work Focus").await?;
//! }
//! # Ok(())
//! # }
//! ```
//!
//! # Error Handling
//!
//! All errors in this module are recoverable - the timer should always
//! continue even if focus mode operations fail. Use the
//! [`FocusModeError::is_recoverable`] method to check this.

pub mod config;
pub mod error;
pub mod shortcuts;

// Re-export main types for convenience
pub use config::FocusModeConfig;
pub use error::FocusModeError;
pub use shortcuts::{
    disable_focus, disable_focus_with_timeout, enable_focus, enable_focus_with_timeout,
    shortcuts_exists,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_public_api_exports() {
        // Verify that all expected types are re-exported
        let _config = FocusModeConfig::default();
        let _error = FocusModeError::ShortcutsNotFound;
        let _exists = shortcuts_exists();
    }

    #[test]
    fn test_config_default_values() {
        let config = FocusModeConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.enable_shortcut_name, "Enable Work Focus");
        assert_eq!(config.disable_shortcut_name, "Disable Work Focus");
        assert_eq!(config.timeout_seconds, 5);
    }

    #[test]
    fn test_error_types() {
        let errors = vec![
            FocusModeError::ShortcutsNotFound,
            FocusModeError::ShortcutNotFound("test".into()),
            FocusModeError::ExecutionTimeout("test".into(), 5),
            FocusModeError::ExecutionFailed("test".into(), "error".into()),
            FocusModeError::Other("other".into()),
        ];

        for error in errors {
            // All errors should be recoverable
            assert!(error.is_recoverable());

            // All errors should have a suggestion
            assert!(!error.suggestion().is_empty());

            // All errors should display properly
            let _ = error.to_string();
        }
    }

    #[tokio::test]
    async fn test_enable_focus_graceful_degradation() {
        // Even if shortcuts doesn't exist, the function should return
        // an error without panicking
        let result = enable_focus("Test").await;

        // On macOS 12+, this might succeed or fail depending on shortcut existence
        // On other platforms, it should fail with ShortcutsNotFound
        #[cfg(not(target_os = "macos"))]
        {
            assert!(result.is_err());
        }

        // Regardless of result, we should be able to check recoverability
        if let Err(e) = &result {
            assert!(e.is_recoverable());
        }
    }

    #[tokio::test]
    async fn test_disable_focus_graceful_degradation() {
        // Even if shortcuts doesn't exist, the function should return
        // an error without panicking
        let result = disable_focus("Test").await;

        // On macOS 12+, this might succeed or fail depending on shortcut existence
        // On other platforms, it should fail with ShortcutsNotFound
        #[cfg(not(target_os = "macos"))]
        {
            assert!(result.is_err());
        }

        // Regardless of result, we should be able to check recoverability
        if let Err(e) = &result {
            assert!(e.is_recoverable());
        }
    }
}
