//! Launchctl command execution wrapper.
//!
//! Provides functions to interact with macOS `launchctl` command
//! for managing LaunchAgent services.

use std::path::Path;
use std::process::Command;

use super::error::{LaunchAgentError, Result};

/// Loads a LaunchAgent service.
///
/// Executes `launchctl load <plist_path>` to register the service with launchd.
///
/// # Arguments
/// * `plist_path` - Absolute path to the plist file
///
/// # Returns
/// Ok(()) on success, or an error if the command fails.
///
/// # Errors
/// - If the launchctl command fails to execute
/// - If the service fails to load
pub fn load(plist_path: &Path) -> Result<()> {
    let output = Command::new("launchctl")
        .arg("load")
        .arg(plist_path)
        .output()
        .map_err(|e| LaunchAgentError::LaunchctlExecution(e.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(LaunchAgentError::ServiceLoad(stderr.to_string()));
    }

    tracing::debug!("launchctl load succeeded for {:?}", plist_path);
    Ok(())
}

/// Unloads a LaunchAgent service.
///
/// Executes `launchctl unload <plist_path>` to unregister the service from launchd.
///
/// # Arguments
/// * `plist_path` - Absolute path to the plist file
///
/// # Returns
/// Ok(()) on success, or an error if the command fails.
///
/// # Note
/// If the service is already unloaded, this function may return an error.
/// Callers should typically ignore errors from this function when used
/// for ensuring a clean state before loading.
pub fn unload(plist_path: &Path) -> Result<()> {
    let output = Command::new("launchctl")
        .arg("unload")
        .arg(plist_path)
        .output()
        .map_err(|e| LaunchAgentError::LaunchctlExecution(e.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        tracing::warn!(
            "launchctl unload failed (may be already unloaded): {}",
            stderr
        );
        return Err(LaunchAgentError::ServiceUnload(stderr.to_string()));
    }

    tracing::debug!("launchctl unload succeeded for {:?}", plist_path);
    Ok(())
}

/// Bootstraps a LaunchAgent service using the new API (macOS 10.10+).
///
/// Executes `launchctl bootstrap <domain> <plist_path>` to register the service.
///
/// # Arguments
/// * `domain` - The launchd domain (e.g., "gui/501")
/// * `plist_path` - Absolute path to the plist file
///
/// # Returns
/// Ok(()) on success, or an error if the command fails.
#[allow(dead_code)]
pub fn bootstrap(domain: &str, plist_path: &Path) -> Result<()> {
    let output = Command::new("launchctl")
        .arg("bootstrap")
        .arg(domain)
        .arg(plist_path)
        .output()
        .map_err(|e| LaunchAgentError::LaunchctlExecution(e.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(LaunchAgentError::ServiceLoad(stderr.to_string()));
    }

    tracing::debug!("launchctl bootstrap succeeded for {:?}", plist_path);
    Ok(())
}

/// Boots out a LaunchAgent service using the new API (macOS 10.10+).
///
/// Executes `launchctl bootout <domain>/<label>` to unregister the service.
///
/// # Arguments
/// * `domain` - The launchd domain (e.g., "gui/501")
/// * `label` - The service label (e.g., "com.example.pomodoro")
///
/// # Returns
/// Ok(()) on success, or an error if the command fails.
#[allow(dead_code)]
pub fn bootout(domain: &str, label: &str) -> Result<()> {
    let output = Command::new("launchctl")
        .arg("bootout")
        .arg(format!("{}/{}", domain, label))
        .output()
        .map_err(|e| LaunchAgentError::LaunchctlExecution(e.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        tracing::warn!(
            "launchctl bootout failed (may be already stopped): {}",
            stderr
        );
        return Err(LaunchAgentError::ServiceUnload(stderr.to_string()));
    }

    tracing::debug!("launchctl bootout succeeded for {}/{}", domain, label);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_load_returns_result() {
        let path = PathBuf::from("/nonexistent/path/to/plist");
        let _result = load(&path);
    }

    #[test]
    fn test_unload_returns_result() {
        let path = PathBuf::from("/nonexistent/path/to/plist");
        let _result = unload(&path);
    }

    #[test]
    fn test_bootstrap_invalid_domain() {
        let path = PathBuf::from("/nonexistent/path/to/plist");
        let result = bootstrap("invalid_domain", &path);
        assert!(result.is_err());
    }

    #[test]
    fn test_bootout_invalid_domain() {
        let result = bootout("invalid_domain", "com.example.test");
        assert!(result.is_err());
    }
}
