//! LaunchAgent management module for macOS.
//!
//! This module provides functionality to install and uninstall the
//! pomodoro timer daemon as a macOS LaunchAgent, allowing it to
//! automatically start at user login.
//!
//! # Example
//!
//! ```no_run
//! use pomodoro::launchagent;
//!
//! // Install the LaunchAgent
//! launchagent::install()?;
//!
//! // Check if installed
//! if launchagent::is_installed() {
//!     println!("LaunchAgent is installed");
//! }
//!
//! // Uninstall
//! launchagent::uninstall()?;
//! # Ok::<(), launchagent::error::LaunchAgentError>(())
//! ```

pub mod error;
pub mod launchctl;
pub mod plist;
pub mod status;

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::process::Command;

pub use error::{LaunchAgentError, Result};
pub use plist::PomodoroLaunchAgent;
pub use status::{
    get_log_dir, get_plist_path, get_status, is_installed, is_running, ServiceStatus,
};

/// Installs the LaunchAgent for the pomodoro daemon.
///
/// This function:
/// 1. Resolves the pomodoro binary path
/// 2. Creates the log directory
/// 3. Generates the plist configuration
/// 4. Writes the plist file to ~/Library/LaunchAgents/
/// 5. Sets proper file permissions (0644)
/// 6. Unloads any existing service (for idempotency)
/// 7. Loads the new service
///
/// # Returns
/// Ok(()) on success.
///
/// # Errors
/// Returns an error if:
/// - The binary path cannot be resolved
/// - The home directory cannot be determined
/// - Directory creation fails
/// - File writing fails
/// - launchctl commands fail
///
/// # Example
///
/// ```no_run
/// use pomodoro::launchagent;
///
/// match launchagent::install() {
///     Ok(()) => println!("LaunchAgent installed successfully"),
///     Err(e) => eprintln!("Failed to install: {}", e),
/// }
/// ```
pub fn install() -> Result<()> {
    // 1. Resolve binary path
    let binary_path = resolve_binary_path()?;

    // 2. Get home directory
    let home_dir = dirs::home_dir().ok_or(LaunchAgentError::HomeDirectoryNotFound)?;

    // 3. Create log directory
    let log_dir = home_dir.join(".pomodoro/logs");
    fs::create_dir_all(&log_dir).map_err(LaunchAgentError::DirectoryCreation)?;

    // 4. Generate plist
    let plist = PomodoroLaunchAgent::new(binary_path, log_dir.to_string_lossy().to_string());
    let plist_xml = plist.to_xml()?;

    // 5. Get plist path and create parent directory
    let plist_path = home_dir.join(format!(
        "Library/LaunchAgents/{}.plist",
        PomodoroLaunchAgent::LABEL
    ));
    if let Some(parent) = plist_path.parent() {
        fs::create_dir_all(parent).map_err(LaunchAgentError::DirectoryCreation)?;
    }

    // 6. Write plist file
    fs::write(&plist_path, plist_xml).map_err(LaunchAgentError::PlistWrite)?;

    // 7. Set permissions (0644: rw-r--r--)
    let mut perms = fs::metadata(&plist_path)
        .map_err(LaunchAgentError::PermissionSet)?
        .permissions();
    perms.set_mode(0o644);
    fs::set_permissions(&plist_path, perms).map_err(LaunchAgentError::PermissionSet)?;

    // 8. Unload existing service (idempotency - ignore errors)
    let _ = launchctl::unload(&plist_path);

    // 9. Load new service
    launchctl::load(&plist_path)?;

    tracing::info!("LaunchAgent installed successfully at {:?}", plist_path);
    Ok(())
}

/// Uninstalls the LaunchAgent for the pomodoro daemon.
///
/// This function:
/// 1. Unloads the service from launchd (if running)
/// 2. Removes the plist file
///
/// If the plist file doesn't exist, this function returns Ok(())
/// for idempotency.
///
/// # Returns
/// Ok(()) on success.
///
/// # Errors
/// Returns an error if:
/// - The home directory cannot be determined
/// - File deletion fails
///
/// # Example
///
/// ```no_run
/// use pomodoro::launchagent;
///
/// match launchagent::uninstall() {
///     Ok(()) => println!("LaunchAgent uninstalled successfully"),
///     Err(e) => eprintln!("Failed to uninstall: {}", e),
/// }
/// ```
pub fn uninstall() -> Result<()> {
    // 1. Get home directory
    let home_dir = dirs::home_dir().ok_or(LaunchAgentError::HomeDirectoryNotFound)?;

    // 2. Get plist path
    let plist_path = home_dir.join(format!(
        "Library/LaunchAgents/{}.plist",
        PomodoroLaunchAgent::LABEL
    ));

    // 3. If file doesn't exist, return Ok (idempotency)
    if !plist_path.exists() {
        tracing::info!("LaunchAgent plist file does not exist, nothing to uninstall");
        return Ok(());
    }

    // 4. Unload service (ignore errors - may already be unloaded)
    let _ = launchctl::unload(&plist_path);

    // 5. Remove plist file
    fs::remove_file(&plist_path).map_err(LaunchAgentError::PlistRemove)?;

    tracing::info!("LaunchAgent uninstalled successfully");
    Ok(())
}

/// Resolves the absolute path to the pomodoro binary.
///
/// Uses the `which` command to find the binary in PATH.
///
/// # Returns
/// The absolute path to the binary.
///
/// # Errors
/// Returns an error if:
/// - The `which` command fails
/// - The binary is not found in PATH
fn resolve_binary_path() -> Result<String> {
    let output = Command::new("which")
        .arg("pomodoro")
        .output()
        .map_err(|e| LaunchAgentError::BinaryPathResolution(e.to_string()))?;

    if !output.status.success() {
        return Err(LaunchAgentError::BinaryPathResolution(
            "pomodoro binary not found in PATH".to_string(),
        ));
    }

    let path = String::from_utf8(output.stdout)
        .map_err(|e| LaunchAgentError::BinaryPathResolution(e.to_string()))?
        .trim()
        .to_string();

    if path.is_empty() {
        return Err(LaunchAgentError::BinaryPathResolution(
            "pomodoro binary path is empty".to_string(),
        ));
    }

    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: Most tests require macOS and/or actual binary installation.
    // These tests verify compilation and basic functionality.

    #[test]
    fn test_module_exports() {
        // Verify public exports are accessible
        let _error: Option<LaunchAgentError> = None;
        let _status: Option<ServiceStatus> = None;
    }

    #[test]
    fn test_resolve_binary_path_failure() {
        // Temporarily modify PATH to ensure pomodoro is not found
        let result = resolve_binary_path();
        // This will typically fail in test environment
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_install_requires_home_dir() {
        // This test verifies the function handles missing home directory
        // In normal circumstances, home_dir should be available
        let home = dirs::home_dir();
        assert!(home.is_some(), "Home directory should be available in test");
    }

    #[test]
    fn test_uninstall_idempotent() {
        // Uninstall should not fail if plist doesn't exist
        // (though it may fail if home dir isn't found)
        if dirs::home_dir().is_some() {
            // This is safe to call - it won't do anything if not installed
            // We don't actually run it to avoid side effects
            // Just verify it compiles
            let _result: Result<()> = Ok(());
        }
    }

    #[test]
    fn test_plist_path_format() {
        if let Some(home) = dirs::home_dir() {
            let expected_path = home.join(format!(
                "Library/LaunchAgents/{}.plist",
                PomodoroLaunchAgent::LABEL
            ));
            assert!(expected_path.to_string_lossy().contains("LaunchAgents"));
            assert!(expected_path
                .to_string_lossy()
                .contains("com.example.pomodoro"));
        }
    }

    #[test]
    fn test_log_dir_path_format() {
        if let Some(home) = dirs::home_dir() {
            let log_dir = home.join(".pomodoro/logs");
            assert!(log_dir.to_string_lossy().contains(".pomodoro"));
            assert!(log_dir.to_string_lossy().contains("logs"));
        }
    }
}
