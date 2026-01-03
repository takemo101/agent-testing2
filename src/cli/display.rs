//! Display utilities for the Pomodoro Timer CLI.
//!
//! This module provides formatted output for:
//! - Success messages
//! - Error messages
//! - Status display
//! - Timer information

use crate::types::IpcResponse;

// ============================================================================
// Display
// ============================================================================

/// Display utilities for CLI output.
pub struct Display;

impl Display {
    /// Shows a success message for timer start.
    pub fn show_start_success(response: &IpcResponse) {
        println!("* タイマーを開始しました");

        if let Some(data) = &response.data {
            if let Some(task_name) = &data.task_name {
                println!("  タスク: {}", task_name);
            }
            if let Some(remaining) = data.remaining_seconds {
                let (minutes, seconds) = Self::format_time(remaining);
                println!("  残り時間: {}:{:02}", minutes, seconds);
            }
        }
    }

    /// Shows a success message for timer pause.
    pub fn show_pause_success(response: &IpcResponse) {
        println!("|| タイマーを一時停止しました");

        if let Some(data) = &response.data {
            if let Some(remaining) = data.remaining_seconds {
                let (minutes, seconds) = Self::format_time(remaining);
                println!("  残り時間: {}:{:02}", minutes, seconds);
            }
        }
    }

    /// Shows a success message for timer resume.
    pub fn show_resume_success(response: &IpcResponse) {
        println!("> タイマーを再開しました");

        if let Some(data) = &response.data {
            if let Some(remaining) = data.remaining_seconds {
                let (minutes, seconds) = Self::format_time(remaining);
                println!("  残り時間: {}:{:02}", minutes, seconds);
            }
        }
    }

    /// Shows a success message for timer stop.
    pub fn show_stop_success(_response: &IpcResponse) {
        println!("[] タイマーを停止しました");
    }

    /// Shows the current timer status.
    pub fn show_status(response: &IpcResponse) {
        println!("ポモドーロタイマー ステータス");
        println!("─────────────────────────────");

        if let Some(data) = &response.data {
            let state = data.state.as_deref().unwrap_or("unknown");
            let state_display = match state {
                "working" => "作業中",
                "breaking" => "休憩中",
                "long_breaking" => "長い休憩中",
                "paused" => "一時停止中",
                "stopped" => "停止中",
                _ => state,
            };
            println!("状態: {}", state_display);

            if state != "stopped" {
                if let Some(remaining) = data.remaining_seconds {
                    let (minutes, seconds) = Self::format_time(remaining);
                    println!("残り時間: {}:{:02}", minutes, seconds);
                }
                if let Some(count) = data.pomodoro_count {
                    println!("ポモドーロ: #{}", count);
                }
                if let Some(task) = &data.task_name {
                    println!("タスク: {}", task);
                }
            }
        } else {
            println!("タイマーは起動していません");
        }
    }

    /// Shows a success message for LaunchAgent installation.
    pub fn show_install_success() {
        println!("* LaunchAgentをインストールしました");
        println!("  次回ログイン時から自動的に起動します");
    }

    /// Shows a success message for LaunchAgent uninstallation.
    pub fn show_uninstall_success() {
        println!("* LaunchAgentをアンインストールしました");
        println!("  次回ログイン時から自動起動しなくなります");
    }

    /// Shows an error message.
    pub fn show_error(message: &str) {
        eprintln!("エラー: {}", message);
    }

    /// Formats remaining seconds as (minutes, seconds).
    fn format_time(total_seconds: u32) -> (u32, u32) {
        let minutes = total_seconds / 60;
        let seconds = total_seconds % 60;
        (minutes, seconds)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ResponseData;

    // ------------------------------------------------------------------------
    // Format Time Tests
    // ------------------------------------------------------------------------

    mod format_time_tests {
        use super::*;

        #[test]
        fn test_format_time_zero() {
            let (minutes, seconds) = Display::format_time(0);
            assert_eq!(minutes, 0);
            assert_eq!(seconds, 0);
        }

        #[test]
        fn test_format_time_seconds_only() {
            let (minutes, seconds) = Display::format_time(45);
            assert_eq!(minutes, 0);
            assert_eq!(seconds, 45);
        }

        #[test]
        fn test_format_time_one_minute() {
            let (minutes, seconds) = Display::format_time(60);
            assert_eq!(minutes, 1);
            assert_eq!(seconds, 0);
        }

        #[test]
        fn test_format_time_mixed() {
            let (minutes, seconds) = Display::format_time(90);
            assert_eq!(minutes, 1);
            assert_eq!(seconds, 30);
        }

        #[test]
        fn test_format_time_25_minutes() {
            let (minutes, seconds) = Display::format_time(25 * 60);
            assert_eq!(minutes, 25);
            assert_eq!(seconds, 0);
        }

        #[test]
        fn test_format_time_large() {
            let (minutes, seconds) = Display::format_time(120 * 60 + 59);
            assert_eq!(minutes, 120);
            assert_eq!(seconds, 59);
        }
    }

    // ------------------------------------------------------------------------
    // Display Output Tests (using captured output patterns)
    // ------------------------------------------------------------------------

    mod display_tests {
        use super::*;

        fn create_working_response() -> IpcResponse {
            IpcResponse::success(
                "タイマーを開始しました",
                Some(ResponseData {
                    state: Some("working".to_string()),
                    remaining_seconds: Some(1500),
                    pomodoro_count: Some(1),
                    task_name: Some("Test Task".to_string()),
                }),
            )
        }

        fn create_paused_response() -> IpcResponse {
            IpcResponse::success(
                "タイマーを一時停止しました",
                Some(ResponseData {
                    state: Some("paused".to_string()),
                    remaining_seconds: Some(1200),
                    pomodoro_count: Some(1),
                    task_name: None,
                }),
            )
        }

        fn create_stopped_response() -> IpcResponse {
            IpcResponse::success(
                "タイマーを停止しました",
                Some(ResponseData {
                    state: Some("stopped".to_string()),
                    remaining_seconds: Some(0),
                    pomodoro_count: Some(0),
                    task_name: None,
                }),
            )
        }

        #[test]
        fn test_show_start_success() {
            // This test verifies the function doesn't panic
            let response = create_working_response();
            Display::show_start_success(&response);
        }

        #[test]
        fn test_show_pause_success() {
            let response = create_paused_response();
            Display::show_pause_success(&response);
        }

        #[test]
        fn test_show_resume_success() {
            let response = create_working_response();
            Display::show_resume_success(&response);
        }

        #[test]
        fn test_show_stop_success() {
            let response = create_stopped_response();
            Display::show_stop_success(&response);
        }

        #[test]
        fn test_show_status_working() {
            let response = create_working_response();
            Display::show_status(&response);
        }

        #[test]
        fn test_show_status_stopped() {
            let response = create_stopped_response();
            Display::show_status(&response);
        }

        #[test]
        fn test_show_status_no_data() {
            let response = IpcResponse::success("", None);
            Display::show_status(&response);
        }

        #[test]
        fn test_show_install_success() {
            Display::show_install_success();
        }

        #[test]
        fn test_show_uninstall_success() {
            Display::show_uninstall_success();
        }

        #[test]
        fn test_show_error() {
            Display::show_error("Test error message");
        }

        #[test]
        fn test_show_start_no_task() {
            let response = IpcResponse::success(
                "タイマーを開始しました",
                Some(ResponseData {
                    state: Some("working".to_string()),
                    remaining_seconds: Some(1500),
                    pomodoro_count: Some(0),
                    task_name: None,
                }),
            );
            Display::show_start_success(&response);
        }

        #[test]
        fn test_show_status_breaking() {
            let response = IpcResponse::success(
                "",
                Some(ResponseData {
                    state: Some("breaking".to_string()),
                    remaining_seconds: Some(300),
                    pomodoro_count: Some(1),
                    task_name: None,
                }),
            );
            Display::show_status(&response);
        }

        #[test]
        fn test_show_status_long_breaking() {
            let response = IpcResponse::success(
                "",
                Some(ResponseData {
                    state: Some("long_breaking".to_string()),
                    remaining_seconds: Some(900),
                    pomodoro_count: Some(4),
                    task_name: None,
                }),
            );
            Display::show_status(&response);
        }

        #[test]
        fn test_show_status_paused() {
            let response = create_paused_response();
            Display::show_status(&response);
        }

        #[test]
        fn test_show_status_unknown_state() {
            let response = IpcResponse::success(
                "",
                Some(ResponseData {
                    state: Some("unknown_state".to_string()),
                    remaining_seconds: Some(100),
                    pomodoro_count: Some(0),
                    task_name: None,
                }),
            );
            Display::show_status(&response);
        }
    }
}
