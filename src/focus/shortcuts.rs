//! Shortcuts.app execution logic for focus mode integration.
//!
//! This module handles the execution of macOS Shortcuts.app shortcuts
//! for enabling and disabling focus mode. It provides:
//!
//! - Detection of Shortcuts.app availability
//! - Asynchronous shortcut execution with timeout
//! - Graceful error handling that doesn't block the timer

use std::path::Path;
use std::process::Command;
use std::time::Duration;

use tokio::time::timeout;
use tracing::{error, info, warn};

use super::error::FocusModeError;

/// Path to the shortcuts command-line tool.
const SHORTCUTS_PATH: &str = "/usr/bin/shortcuts";

/// Default timeout for shortcut execution in seconds.
const DEFAULT_TIMEOUT_SECONDS: u64 = 5;

/// Checks if Shortcuts.app is available on this system.
///
/// Shortcuts.app is available on macOS 12 (Monterey) and later.
///
/// # Returns
///
/// `true` if `/usr/bin/shortcuts` exists, `false` otherwise.
///
/// # Example
///
/// ```
/// use pomodoro::focus::shortcuts_exists;
///
/// if shortcuts_exists() {
///     println!("Shortcuts.app is available");
/// } else {
///     println!("Shortcuts.app is not available (requires macOS 12+)");
/// }
/// ```
#[must_use]
pub fn shortcuts_exists() -> bool {
    Path::new(SHORTCUTS_PATH).exists()
}

/// Enables focus mode by running the specified shortcut.
///
/// This function executes a Shortcuts.app shortcut that should enable
/// focus mode on macOS. The execution is asynchronous with a timeout
/// to prevent blocking the timer.
///
/// # Arguments
///
/// * `shortcut_name` - The name of the shortcut to run (e.g., "Enable Work Focus")
///
/// # Errors
///
/// Returns an error if:
/// - Shortcuts.app is not found (`FocusModeError::ShortcutsNotFound`)
/// - The shortcut doesn't exist (`FocusModeError::ShortcutNotFound`)
/// - Execution times out (`FocusModeError::ExecutionTimeout`)
/// - Execution fails (`FocusModeError::ExecutionFailed`)
///
/// # Example
///
/// ```no_run
/// use pomodoro::focus::enable_focus;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// enable_focus("Enable Work Focus").await?;
/// println!("Focus mode enabled!");
/// # Ok(())
/// # }
/// ```
pub async fn enable_focus(shortcut_name: &str) -> Result<(), FocusModeError> {
    enable_focus_with_timeout(shortcut_name, DEFAULT_TIMEOUT_SECONDS).await
}

/// Enables focus mode with a custom timeout.
///
/// # Arguments
///
/// * `shortcut_name` - The name of the shortcut to run
/// * `timeout_seconds` - Maximum time to wait for execution
pub async fn enable_focus_with_timeout(
    shortcut_name: &str,
    timeout_seconds: u64,
) -> Result<(), FocusModeError> {
    info!("フォーカスモードを有効化します: {}", shortcut_name);

    // Check if Shortcuts.app is available
    if !shortcuts_exists() {
        warn!("Shortcuts.appが見つかりません。フォーカスモード連携をスキップします。");
        return Err(FocusModeError::ShortcutsNotFound);
    }

    // Execute shortcut with timeout
    let result = timeout(
        Duration::from_secs(timeout_seconds),
        execute_shortcut(shortcut_name),
    )
    .await;

    match result {
        Ok(Ok(())) => {
            info!("フォーカスモードを有効化しました");
            Ok(())
        }
        Ok(Err(e)) => {
            error!("フォーカスモード有効化に失敗: {}", e);
            Err(e)
        }
        Err(_) => {
            error!("フォーカスモード有効化がタイムアウトしました");
            Err(FocusModeError::ExecutionTimeout(
                shortcut_name.to_string(),
                timeout_seconds,
            ))
        }
    }
}

/// Disables focus mode by running the specified shortcut.
///
/// This function executes a Shortcuts.app shortcut that should disable
/// focus mode on macOS. The execution is asynchronous with a timeout
/// to prevent blocking the timer.
///
/// # Arguments
///
/// * `shortcut_name` - The name of the shortcut to run (e.g., "Disable Work Focus")
///
/// # Errors
///
/// Returns an error if:
/// - Shortcuts.app is not found (`FocusModeError::ShortcutsNotFound`)
/// - The shortcut doesn't exist (`FocusModeError::ShortcutNotFound`)
/// - Execution times out (`FocusModeError::ExecutionTimeout`)
/// - Execution fails (`FocusModeError::ExecutionFailed`)
///
/// # Example
///
/// ```no_run
/// use pomodoro::focus::disable_focus;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// disable_focus("Disable Work Focus").await?;
/// println!("Focus mode disabled!");
/// # Ok(())
/// # }
/// ```
pub async fn disable_focus(shortcut_name: &str) -> Result<(), FocusModeError> {
    disable_focus_with_timeout(shortcut_name, DEFAULT_TIMEOUT_SECONDS).await
}

/// Disables focus mode with a custom timeout.
///
/// # Arguments
///
/// * `shortcut_name` - The name of the shortcut to run
/// * `timeout_seconds` - Maximum time to wait for execution
pub async fn disable_focus_with_timeout(
    shortcut_name: &str,
    timeout_seconds: u64,
) -> Result<(), FocusModeError> {
    info!("フォーカスモードを無効化します: {}", shortcut_name);

    // Check if Shortcuts.app is available
    if !shortcuts_exists() {
        warn!("Shortcuts.appが見つかりません。フォーカスモード連携をスキップします。");
        return Err(FocusModeError::ShortcutsNotFound);
    }

    // Execute shortcut with timeout
    let result = timeout(
        Duration::from_secs(timeout_seconds),
        execute_shortcut(shortcut_name),
    )
    .await;

    match result {
        Ok(Ok(())) => {
            info!("フォーカスモードを無効化しました");
            Ok(())
        }
        Ok(Err(e)) => {
            error!("フォーカスモード無効化に失敗: {}", e);
            Err(e)
        }
        Err(_) => {
            error!("フォーカスモード無効化がタイムアウトしました");
            Err(FocusModeError::ExecutionTimeout(
                shortcut_name.to_string(),
                timeout_seconds,
            ))
        }
    }
}

/// Executes a shortcut using the shortcuts command-line tool.
///
/// This function spawns the shortcuts command in a blocking task to avoid
/// blocking the async runtime.
async fn execute_shortcut(shortcut_name: &str) -> Result<(), FocusModeError> {
    // Execute command in blocking task
    let output = tokio::task::spawn_blocking({
        let shortcut_name = shortcut_name.to_string();
        move || {
            Command::new(SHORTCUTS_PATH)
                .arg("run")
                .arg(&shortcut_name)
                .output()
        }
    })
    .await
    .map_err(|e| FocusModeError::Other(format!("タスク実行エラー: {}", e)))?
    .map_err(|e| FocusModeError::Other(format!("コマンド実行エラー: {}", e)))?;

    // Check exit code
    if output.status.success() {
        Ok(())
    } else {
        // Parse error output
        let stderr = String::from_utf8_lossy(&output.stderr);

        // Check for common error patterns
        if stderr.contains("not found")
            || stderr.contains("does not exist")
            || stderr.contains("Couldn't find")
            || stderr.contains("no such shortcut")
        {
            Err(FocusModeError::ShortcutNotFound(shortcut_name.to_string()))
        } else {
            Err(FocusModeError::ExecutionFailed(
                shortcut_name.to_string(),
                stderr.trim().to_string(),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shortcuts_exists() {
        // On macOS 12+, /usr/bin/shortcuts should exist
        // On other systems, it won't
        let exists = shortcuts_exists();

        #[cfg(target_os = "macos")]
        {
            // Note: This test may pass or fail depending on macOS version
            // We just verify the function returns a boolean without panicking
            let _ = exists;
        }

        #[cfg(not(target_os = "macos"))]
        {
            // On non-macOS, shortcuts should not exist
            assert!(!exists);
        }
    }

    #[test]
    fn test_shortcuts_path_constant() {
        assert_eq!(SHORTCUTS_PATH, "/usr/bin/shortcuts");
    }

    #[test]
    fn test_default_timeout_constant() {
        assert_eq!(DEFAULT_TIMEOUT_SECONDS, 5);
    }

    #[tokio::test]
    async fn test_enable_focus_shortcuts_not_found() {
        // Skip this test on macOS 12+ where shortcuts exists
        if shortcuts_exists() {
            return;
        }

        let result = enable_focus("Test Shortcut").await;
        assert!(result.is_err());

        match result {
            Err(FocusModeError::ShortcutsNotFound) => {}
            other => panic!("Expected ShortcutsNotFound, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_disable_focus_shortcuts_not_found() {
        // Skip this test on macOS 12+ where shortcuts exists
        if shortcuts_exists() {
            return;
        }

        let result = disable_focus("Test Shortcut").await;
        assert!(result.is_err());

        match result {
            Err(FocusModeError::ShortcutsNotFound) => {}
            other => panic!("Expected ShortcutsNotFound, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_enable_focus_with_custom_timeout() {
        // Skip this test on macOS 12+ where shortcuts exists
        if shortcuts_exists() {
            return;
        }

        let result = enable_focus_with_timeout("Test Shortcut", 10).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_disable_focus_with_custom_timeout() {
        // Skip this test on macOS 12+ where shortcuts exists
        if shortcuts_exists() {
            return;
        }

        let result = disable_focus_with_timeout("Test Shortcut", 10).await;
        assert!(result.is_err());
    }

    // Integration tests that require Shortcuts.app
    // These are marked #[ignore] because they require manual setup

    #[tokio::test]
    #[ignore = "Requires Shortcuts.app with 'Enable Work Focus' shortcut"]
    async fn test_enable_focus_integration() {
        let result = enable_focus("Enable Work Focus").await;
        assert!(result.is_ok(), "Failed to enable focus: {:?}", result);
    }

    #[tokio::test]
    #[ignore = "Requires Shortcuts.app with 'Disable Work Focus' shortcut"]
    async fn test_disable_focus_integration() {
        let result = disable_focus("Disable Work Focus").await;
        assert!(result.is_ok(), "Failed to disable focus: {:?}", result);
    }

    #[tokio::test]
    #[ignore = "Requires Shortcuts.app"]
    async fn test_shortcut_not_found() {
        // Skip if shortcuts doesn't exist
        if !shortcuts_exists() {
            return;
        }

        let result = enable_focus("NonExistentShortcutThatDefinitelyDoesNotExist12345").await;

        match result {
            Err(FocusModeError::ShortcutNotFound(name)) => {
                assert!(name.contains("NonExistentShortcut"));
            }
            Err(FocusModeError::ExecutionFailed(_, _)) => {
                // This is also acceptable - different macOS versions may return different errors
            }
            other => panic!(
                "Expected ShortcutNotFound or ExecutionFailed, got {:?}",
                other
            ),
        }
    }
}
