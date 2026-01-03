//! Service status checking functionality.
//!
//! Provides functions to check the status of the LaunchAgent service.

use std::process::Command;

use super::error::{LaunchAgentError, Result};
use super::plist::PomodoroLaunchAgent;

/// Represents the status of a LaunchAgent service.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceStatus {
    /// Whether the service is currently running
    pub running: bool,
    /// The process ID if running
    pub pid: Option<u32>,
    /// The last exit status code
    pub status_code: Option<i32>,
}

impl ServiceStatus {
    /// Creates a new stopped service status.
    pub fn stopped() -> Self {
        Self {
            running: false,
            pid: None,
            status_code: None,
        }
    }

    /// Creates a new running service status.
    pub fn running(pid: u32) -> Self {
        Self {
            running: true,
            pid: Some(pid),
            status_code: Some(0),
        }
    }
}

impl Default for ServiceStatus {
    fn default() -> Self {
        Self::stopped()
    }
}

/// Checks if the LaunchAgent plist file is installed.
///
/// # Returns
/// `true` if the plist file exists, `false` otherwise.
pub fn is_installed() -> bool {
    let home_dir = match dirs::home_dir() {
        Some(dir) => dir,
        None => return false,
    };

    let plist_path = home_dir.join(format!(
        "Library/LaunchAgents/{}.plist",
        PomodoroLaunchAgent::LABEL
    ));

    plist_path.exists()
}

/// Checks if the LaunchAgent service is running.
///
/// Uses `launchctl list <label>` to determine if the service is registered
/// and running.
///
/// # Returns
/// `true` if the service is running, `false` otherwise.
///
/// # Errors
/// Returns an error if the launchctl command fails to execute.
pub fn is_running() -> Result<bool> {
    let output = Command::new("launchctl")
        .arg("list")
        .arg(PomodoroLaunchAgent::LABEL)
        .output()
        .map_err(|e| LaunchAgentError::LaunchctlExecution(e.to_string()))?;

    // Exit code 0 means the service exists in launchd
    Ok(output.status.success())
}

/// Gets the detailed status of the LaunchAgent service.
///
/// Uses `launchctl list <label>` to get service information including
/// PID and last exit status.
///
/// # Returns
/// A `ServiceStatus` struct with the service information.
///
/// # Errors
/// Returns an error if the launchctl command fails to execute.
pub fn get_status() -> Result<ServiceStatus> {
    let output = Command::new("launchctl")
        .arg("list")
        .arg(PomodoroLaunchAgent::LABEL)
        .output()
        .map_err(|e| LaunchAgentError::LaunchctlExecution(e.to_string()))?;

    if !output.status.success() {
        return Ok(ServiceStatus::stopped());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_launchctl_output(&stdout)
}

/// Parses the output of `launchctl list <label>`.
///
/// The output format is typically:
/// ```text
/// {
///     "Label" = "com.example.pomodoro";
///     "LimitLoadToSessionType" = "Aqua";
///     "OnDemand" = false;
///     "LastExitStatus" = 0;
///     "PID" = 12345;
///     "Program" = "/usr/local/bin/pomodoro";
///     ...
/// };
/// ```
fn parse_launchctl_output(output: &str) -> Result<ServiceStatus> {
    let pid = extract_field(output, "PID").and_then(|s| s.parse::<u32>().ok());

    let status_code = extract_field(output, "LastExitStatus").and_then(|s| s.parse::<i32>().ok());

    Ok(ServiceStatus {
        running: pid.is_some(),
        pid,
        status_code,
    })
}

/// Extracts a field value from launchctl output.
fn extract_field<'a>(output: &'a str, field: &str) -> Option<&'a str> {
    let pattern = format!("\"{}\" = ", field);
    output.lines().find_map(|line| {
        if line.contains(&pattern) {
            line.split('=')
                .nth(1)
                .map(|s| s.trim().trim_matches(';').trim_matches('"').trim())
        } else {
            None
        }
    })
}

/// Returns the plist file path.
///
/// # Returns
/// The full path to the plist file, or None if home directory cannot be determined.
pub fn get_plist_path() -> Option<std::path::PathBuf> {
    dirs::home_dir().map(|home| {
        home.join(format!(
            "Library/LaunchAgents/{}.plist",
            PomodoroLaunchAgent::LABEL
        ))
    })
}

/// Returns the log directory path.
///
/// # Returns
/// The full path to the log directory, or None if home directory cannot be determined.
pub fn get_log_dir() -> Option<std::path::PathBuf> {
    dirs::home_dir().map(|home| home.join(".pomodoro/logs"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_status_stopped() {
        let status = ServiceStatus::stopped();
        assert!(!status.running);
        assert!(status.pid.is_none());
        assert!(status.status_code.is_none());
    }

    #[test]
    fn test_service_status_running() {
        let status = ServiceStatus::running(12345);
        assert!(status.running);
        assert_eq!(status.pid, Some(12345));
        assert_eq!(status.status_code, Some(0));
    }

    #[test]
    fn test_service_status_default() {
        let status = ServiceStatus::default();
        assert!(!status.running);
        assert!(status.pid.is_none());
        assert!(status.status_code.is_none());
    }

    #[test]
    fn test_parse_launchctl_output_running() {
        let output = r#"{
    "Label" = "com.example.pomodoro";
    "LastExitStatus" = 0;
    "PID" = 12345;
    "Program" = "/usr/local/bin/pomodoro";
};"#;

        let status = parse_launchctl_output(output).expect("Failed to parse");
        assert!(status.running);
        assert_eq!(status.pid, Some(12345));
        assert_eq!(status.status_code, Some(0));
    }

    #[test]
    fn test_parse_launchctl_output_stopped() {
        let output = r#"{
    "Label" = "com.example.pomodoro";
    "LastExitStatus" = 1;
    "Program" = "/usr/local/bin/pomodoro";
};"#;

        let status = parse_launchctl_output(output).expect("Failed to parse");
        assert!(!status.running);
        assert!(status.pid.is_none());
        assert_eq!(status.status_code, Some(1));
    }

    #[test]
    fn test_extract_field_pid() {
        let output = r#"    "PID" = 12345;"#;
        let pid = extract_field(output, "PID");
        assert_eq!(pid, Some("12345"));
    }

    #[test]
    fn test_extract_field_last_exit_status() {
        let output = r#"    "LastExitStatus" = 0;"#;
        let status = extract_field(output, "LastExitStatus");
        assert_eq!(status, Some("0"));
    }

    #[test]
    fn test_extract_field_not_found() {
        let output = r#"    "Label" = "test";"#;
        let pid = extract_field(output, "PID");
        assert!(pid.is_none());
    }

    #[test]
    fn test_get_plist_path() {
        let path = get_plist_path();
        // Should return Some if home directory exists
        if dirs::home_dir().is_some() {
            assert!(path.is_some());
            let path = path.unwrap();
            assert!(path.to_string_lossy().contains("LaunchAgents"));
            assert!(path
                .to_string_lossy()
                .contains("com.example.pomodoro.plist"));
        }
    }

    #[test]
    fn test_get_log_dir() {
        let path = get_log_dir();
        if dirs::home_dir().is_some() {
            assert!(path.is_some());
            let path = path.unwrap();
            assert!(path.to_string_lossy().contains(".pomodoro/logs"));
        }
    }

    // is_installed test depends on the actual file system state
    // is_running test requires launchctl to be available
}
