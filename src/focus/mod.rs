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

pub use config::FocusModeConfig;
pub use error::FocusModeError;
pub use shortcuts::{
    disable_focus, disable_focus_with_timeout, enable_focus, enable_focus_with_timeout,
    shortcuts_exists,
};

#[allow(async_fn_in_trait)]
pub trait FocusModeController {
    async fn enable(&self) -> Result<(), FocusModeError>;
    async fn disable(&self) -> Result<(), FocusModeError>;
    fn is_available(&self) -> bool;
}

#[derive(Debug, Clone)]
pub struct ShortcutsFocusController {
    config: FocusModeConfig,
}

impl ShortcutsFocusController {
    #[must_use]
    pub fn new(config: FocusModeConfig) -> Self {
        Self { config }
    }
}

impl FocusModeController for ShortcutsFocusController {
    async fn enable(&self) -> Result<(), FocusModeError> {
        if !self.config.enabled {
            return Ok(());
        }
        enable_focus_with_timeout(
            &self.config.enable_shortcut_name,
            self.config.timeout_seconds,
        )
        .await
    }

    async fn disable(&self) -> Result<(), FocusModeError> {
        if !self.config.enabled {
            return Ok(());
        }
        disable_focus_with_timeout(
            &self.config.disable_shortcut_name,
            self.config.timeout_seconds,
        )
        .await
    }

    fn is_available(&self) -> bool {
        shortcuts_exists()
    }
}

#[derive(Debug, Default)]
pub struct MockFocusModeController {
    enabled_calls: std::sync::Mutex<Vec<()>>,
    disabled_calls: std::sync::Mutex<Vec<()>>,
    available: std::sync::atomic::AtomicBool,
    should_fail_enable: std::sync::atomic::AtomicBool,
    should_fail_disable: std::sync::atomic::AtomicBool,
}

impl MockFocusModeController {
    #[must_use]
    pub fn new() -> Self {
        Self {
            enabled_calls: std::sync::Mutex::new(Vec::new()),
            disabled_calls: std::sync::Mutex::new(Vec::new()),
            available: std::sync::atomic::AtomicBool::new(true),
            should_fail_enable: std::sync::atomic::AtomicBool::new(false),
            should_fail_disable: std::sync::atomic::AtomicBool::new(false),
        }
    }

    pub fn set_available(&self, available: bool) {
        self.available
            .store(available, std::sync::atomic::Ordering::SeqCst);
    }

    pub fn set_should_fail_enable(&self, should_fail: bool) {
        self.should_fail_enable
            .store(should_fail, std::sync::atomic::Ordering::SeqCst);
    }

    pub fn set_should_fail_disable(&self, should_fail: bool) {
        self.should_fail_disable
            .store(should_fail, std::sync::atomic::Ordering::SeqCst);
    }

    #[must_use]
    pub fn enable_call_count(&self) -> usize {
        self.enabled_calls.lock().unwrap().len()
    }

    #[must_use]
    pub fn disable_call_count(&self) -> usize {
        self.disabled_calls.lock().unwrap().len()
    }

    pub fn reset_counts(&self) {
        self.enabled_calls.lock().unwrap().clear();
        self.disabled_calls.lock().unwrap().clear();
    }
}

impl FocusModeController for MockFocusModeController {
    async fn enable(&self) -> Result<(), FocusModeError> {
        if self
            .should_fail_enable
            .load(std::sync::atomic::Ordering::SeqCst)
        {
            return Err(FocusModeError::ExecutionFailed(
                "mock".to_string(),
                "simulated failure".to_string(),
            ));
        }
        self.enabled_calls.lock().unwrap().push(());
        Ok(())
    }

    async fn disable(&self) -> Result<(), FocusModeError> {
        if self
            .should_fail_disable
            .load(std::sync::atomic::Ordering::SeqCst)
        {
            return Err(FocusModeError::ExecutionFailed(
                "mock".to_string(),
                "simulated failure".to_string(),
            ));
        }
        self.disabled_calls.lock().unwrap().push(());
        Ok(())
    }

    fn is_available(&self) -> bool {
        self.available.load(std::sync::atomic::Ordering::SeqCst)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_public_api_exports() {
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
            assert!(error.is_recoverable());
            assert!(!error.suggestion().is_empty());
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
