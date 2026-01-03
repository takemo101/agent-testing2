//! Command definitions for the Pomodoro Timer CLI.
//!
//! Uses clap derive macro for argument parsing.

use clap::{Args, Parser, Subcommand};

// ============================================================================
// CLI Structure
// ============================================================================

/// Pomodoro Timer CLI - A macOS-native productivity tool
#[derive(Parser, Debug)]
#[command(
    name = "pomodoro",
    version,
    about = "macOS専用ポモドーロタイマーCLI",
    long_about = "ターミナル上で動作するシンプルなポモドーロタイマー。\n\
                  macOSのネイティブ機能（通知、メニューバー、フォーカスモード）と統合されています。",
    propagate_version = true
)]
pub struct Cli {
    /// Subcommand to execute
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Enable verbose output for debugging
    #[arg(short, long, global = true)]
    pub verbose: bool,
}

// ============================================================================
// Subcommands
// ============================================================================

/// Available subcommands
#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    /// Start a new pomodoro timer session
    Start(StartArgs),

    /// Pause the current timer
    Pause,

    /// Resume a paused timer
    Resume,

    /// Stop the current timer
    Stop,

    /// Show current timer status
    Status,

    /// Run as daemon (background service)
    #[command(hide = true)]
    Daemon,

    /// Install LaunchAgent for auto-start on login
    Install,

    /// Uninstall LaunchAgent
    Uninstall,

    /// Generate shell completion scripts
    Completions {
        /// Shell type for completion script
        #[arg(value_enum)]
        shell: clap_complete::Shell,
    },
}

// ============================================================================
// Start Command Arguments
// ============================================================================

/// Arguments for the start command
#[derive(Args, Debug, Clone)]
pub struct StartArgs {
    /// Work duration in minutes (1-120)
    #[arg(
        short,
        long,
        default_value = "25",
        value_parser = clap::value_parser!(u32).range(1..=120)
    )]
    pub work: u32,

    /// Short break duration in minutes (1-60)
    #[arg(
        short,
        long,
        default_value = "5",
        value_parser = clap::value_parser!(u32).range(1..=60)
    )]
    pub break_time: u32,

    /// Long break duration in minutes (1-60)
    #[arg(
        short,
        long,
        default_value = "15",
        value_parser = clap::value_parser!(u32).range(1..=60)
    )]
    pub long_break: u32,

    /// Task name for this session
    #[arg(short, long, value_parser = validate_task_name)]
    pub task: Option<String>,

    /// Enable auto-cycle (automatically start next work session after break)
    #[arg(short, long)]
    pub auto_cycle: bool,

    /// Enable Focus Mode integration (requires macOS Shortcuts.app)
    #[arg(short, long)]
    pub focus_mode: bool,

    /// Disable notification sounds
    #[arg(long)]
    pub no_sound: bool,
}

impl Default for StartArgs {
    fn default() -> Self {
        Self {
            work: 25,
            break_time: 5,
            long_break: 15,
            task: None,
            auto_cycle: false,
            focus_mode: false,
            no_sound: false,
        }
    }
}

// ============================================================================
// Validation Functions
// ============================================================================

/// Validates the task name.
///
/// - Must not be empty
/// - Must not exceed 100 characters
fn validate_task_name(s: &str) -> Result<String, String> {
    if s.is_empty() {
        return Err("タスク名は空にできません".to_string());
    }
    if s.len() > 100 {
        return Err("タスク名は100文字以内にしてください".to_string());
    }
    Ok(s.to_string())
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ------------------------------------------------------------------------
    // Cli Tests
    // ------------------------------------------------------------------------

    mod cli_tests {
        use super::*;

        #[test]
        fn test_parse_no_args() {
            let cli = Cli::parse_from(["pomodoro"]);
            assert!(cli.command.is_none());
            assert!(!cli.verbose);
        }

        #[test]
        fn test_parse_verbose_flag() {
            let cli = Cli::parse_from(["pomodoro", "--verbose"]);
            assert!(cli.verbose);
        }

        #[test]
        fn test_parse_short_verbose_flag() {
            let cli = Cli::parse_from(["pomodoro", "-v"]);
            assert!(cli.verbose);
        }

        #[test]
        fn test_parse_status_command() {
            let cli = Cli::parse_from(["pomodoro", "status"]);
            assert!(matches!(cli.command, Some(Commands::Status)));
        }

        #[test]
        fn test_parse_pause_command() {
            let cli = Cli::parse_from(["pomodoro", "pause"]);
            assert!(matches!(cli.command, Some(Commands::Pause)));
        }

        #[test]
        fn test_parse_resume_command() {
            let cli = Cli::parse_from(["pomodoro", "resume"]);
            assert!(matches!(cli.command, Some(Commands::Resume)));
        }

        #[test]
        fn test_parse_stop_command() {
            let cli = Cli::parse_from(["pomodoro", "stop"]);
            assert!(matches!(cli.command, Some(Commands::Stop)));
        }

        #[test]
        fn test_parse_daemon_command() {
            let cli = Cli::parse_from(["pomodoro", "daemon"]);
            assert!(matches!(cli.command, Some(Commands::Daemon)));
        }

        #[test]
        fn test_parse_install_command() {
            let cli = Cli::parse_from(["pomodoro", "install"]);
            assert!(matches!(cli.command, Some(Commands::Install)));
        }

        #[test]
        fn test_parse_uninstall_command() {
            let cli = Cli::parse_from(["pomodoro", "uninstall"]);
            assert!(matches!(cli.command, Some(Commands::Uninstall)));
        }

        #[test]
        fn test_parse_completions_bash() {
            let cli = Cli::parse_from(["pomodoro", "completions", "bash"]);
            match cli.command {
                Some(Commands::Completions { shell }) => {
                    assert_eq!(shell, clap_complete::Shell::Bash);
                }
                _ => panic!("Expected Completions command"),
            }
        }

        #[test]
        fn test_parse_completions_zsh() {
            let cli = Cli::parse_from(["pomodoro", "completions", "zsh"]);
            match cli.command {
                Some(Commands::Completions { shell }) => {
                    assert_eq!(shell, clap_complete::Shell::Zsh);
                }
                _ => panic!("Expected Completions command"),
            }
        }

        #[test]
        fn test_parse_completions_fish() {
            let cli = Cli::parse_from(["pomodoro", "completions", "fish"]);
            match cli.command {
                Some(Commands::Completions { shell }) => {
                    assert_eq!(shell, clap_complete::Shell::Fish);
                }
                _ => panic!("Expected Completions command"),
            }
        }
    }

    // ------------------------------------------------------------------------
    // Start Command Tests
    // ------------------------------------------------------------------------

    mod start_args_tests {
        use super::*;

        #[test]
        fn test_parse_start_defaults() {
            let cli = Cli::parse_from(["pomodoro", "start"]);
            match cli.command {
                Some(Commands::Start(args)) => {
                    assert_eq!(args.work, 25);
                    assert_eq!(args.break_time, 5);
                    assert_eq!(args.long_break, 15);
                    assert!(args.task.is_none());
                    assert!(!args.auto_cycle);
                    assert!(!args.focus_mode);
                    assert!(!args.no_sound);
                }
                _ => panic!("Expected Start command"),
            }
        }

        #[test]
        fn test_parse_start_work() {
            let cli = Cli::parse_from(["pomodoro", "start", "--work", "30"]);
            match cli.command {
                Some(Commands::Start(args)) => {
                    assert_eq!(args.work, 30);
                }
                _ => panic!("Expected Start command"),
            }
        }

        #[test]
        fn test_parse_start_work_short() {
            let cli = Cli::parse_from(["pomodoro", "start", "-w", "45"]);
            match cli.command {
                Some(Commands::Start(args)) => {
                    assert_eq!(args.work, 45);
                }
                _ => panic!("Expected Start command"),
            }
        }

        #[test]
        fn test_parse_start_break_time() {
            let cli = Cli::parse_from(["pomodoro", "start", "--break-time", "10"]);
            match cli.command {
                Some(Commands::Start(args)) => {
                    assert_eq!(args.break_time, 10);
                }
                _ => panic!("Expected Start command"),
            }
        }

        #[test]
        fn test_parse_start_long_break() {
            let cli = Cli::parse_from(["pomodoro", "start", "--long-break", "20"]);
            match cli.command {
                Some(Commands::Start(args)) => {
                    assert_eq!(args.long_break, 20);
                }
                _ => panic!("Expected Start command"),
            }
        }

        #[test]
        fn test_parse_start_task() {
            let cli = Cli::parse_from(["pomodoro", "start", "--task", "Write code"]);
            match cli.command {
                Some(Commands::Start(args)) => {
                    assert_eq!(args.task, Some("Write code".to_string()));
                }
                _ => panic!("Expected Start command"),
            }
        }

        #[test]
        fn test_parse_start_task_short() {
            let cli = Cli::parse_from(["pomodoro", "start", "-t", "Review PR"]);
            match cli.command {
                Some(Commands::Start(args)) => {
                    assert_eq!(args.task, Some("Review PR".to_string()));
                }
                _ => panic!("Expected Start command"),
            }
        }

        #[test]
        fn test_parse_start_auto_cycle() {
            let cli = Cli::parse_from(["pomodoro", "start", "--auto-cycle"]);
            match cli.command {
                Some(Commands::Start(args)) => {
                    assert!(args.auto_cycle);
                }
                _ => panic!("Expected Start command"),
            }
        }

        #[test]
        fn test_parse_start_focus_mode() {
            let cli = Cli::parse_from(["pomodoro", "start", "--focus-mode"]);
            match cli.command {
                Some(Commands::Start(args)) => {
                    assert!(args.focus_mode);
                }
                _ => panic!("Expected Start command"),
            }
        }

        #[test]
        fn test_parse_start_no_sound() {
            let cli = Cli::parse_from(["pomodoro", "start", "--no-sound"]);
            match cli.command {
                Some(Commands::Start(args)) => {
                    assert!(args.no_sound);
                }
                _ => panic!("Expected Start command"),
            }
        }

        #[test]
        fn test_parse_start_all_options() {
            let cli = Cli::parse_from([
                "pomodoro",
                "start",
                "--work",
                "50",
                "--break-time",
                "10",
                "--long-break",
                "30",
                "--task",
                "Deep work",
                "--auto-cycle",
                "--focus-mode",
                "--no-sound",
            ]);
            match cli.command {
                Some(Commands::Start(args)) => {
                    assert_eq!(args.work, 50);
                    assert_eq!(args.break_time, 10);
                    assert_eq!(args.long_break, 30);
                    assert_eq!(args.task, Some("Deep work".to_string()));
                    assert!(args.auto_cycle);
                    assert!(args.focus_mode);
                    assert!(args.no_sound);
                }
                _ => panic!("Expected Start command"),
            }
        }

        #[test]
        fn test_parse_start_boundary_work_min() {
            let cli = Cli::parse_from(["pomodoro", "start", "--work", "1"]);
            match cli.command {
                Some(Commands::Start(args)) => {
                    assert_eq!(args.work, 1);
                }
                _ => panic!("Expected Start command"),
            }
        }

        #[test]
        fn test_parse_start_boundary_work_max() {
            let cli = Cli::parse_from(["pomodoro", "start", "--work", "120"]);
            match cli.command {
                Some(Commands::Start(args)) => {
                    assert_eq!(args.work, 120);
                }
                _ => panic!("Expected Start command"),
            }
        }

        #[test]
        fn test_parse_start_boundary_break_min() {
            let cli = Cli::parse_from(["pomodoro", "start", "--break-time", "1"]);
            match cli.command {
                Some(Commands::Start(args)) => {
                    assert_eq!(args.break_time, 1);
                }
                _ => panic!("Expected Start command"),
            }
        }

        #[test]
        fn test_parse_start_boundary_break_max() {
            let cli = Cli::parse_from(["pomodoro", "start", "--break-time", "60"]);
            match cli.command {
                Some(Commands::Start(args)) => {
                    assert_eq!(args.break_time, 60);
                }
                _ => panic!("Expected Start command"),
            }
        }

        #[test]
        fn test_start_args_default() {
            let args = StartArgs::default();
            assert_eq!(args.work, 25);
            assert_eq!(args.break_time, 5);
            assert_eq!(args.long_break, 15);
            assert!(args.task.is_none());
            assert!(!args.auto_cycle);
            assert!(!args.focus_mode);
            assert!(!args.no_sound);
        }
    }

    // ------------------------------------------------------------------------
    // Validation Tests
    // ------------------------------------------------------------------------

    mod validation_tests {
        use super::*;

        #[test]
        fn test_validate_task_name_valid() {
            let result = validate_task_name("Valid task name");
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), "Valid task name");
        }

        #[test]
        fn test_validate_task_name_japanese() {
            let result = validate_task_name("タスク名テスト");
            assert!(result.is_ok());
        }

        #[test]
        fn test_validate_task_name_empty() {
            let result = validate_task_name("");
            assert!(result.is_err());
            assert!(result.unwrap_err().contains("空"));
        }

        #[test]
        fn test_validate_task_name_too_long() {
            let long_name = "a".repeat(101);
            let result = validate_task_name(&long_name);
            assert!(result.is_err());
            assert!(result.unwrap_err().contains("100"));
        }

        #[test]
        fn test_validate_task_name_exactly_100() {
            let name = "a".repeat(100);
            let result = validate_task_name(&name);
            assert!(result.is_ok());
        }

        #[test]
        fn test_validate_task_name_single_char() {
            let result = validate_task_name("a");
            assert!(result.is_ok());
        }
    }

    // ------------------------------------------------------------------------
    // Error Case Tests (using try_parse)
    // ------------------------------------------------------------------------

    mod error_tests {
        use super::*;

        #[test]
        fn test_parse_start_work_too_low() {
            let result = Cli::try_parse_from(["pomodoro", "start", "--work", "0"]);
            assert!(result.is_err());
        }

        #[test]
        fn test_parse_start_work_too_high() {
            let result = Cli::try_parse_from(["pomodoro", "start", "--work", "121"]);
            assert!(result.is_err());
        }

        #[test]
        fn test_parse_start_break_too_low() {
            let result = Cli::try_parse_from(["pomodoro", "start", "--break-time", "0"]);
            assert!(result.is_err());
        }

        #[test]
        fn test_parse_start_break_too_high() {
            let result = Cli::try_parse_from(["pomodoro", "start", "--break-time", "61"]);
            assert!(result.is_err());
        }

        #[test]
        fn test_parse_start_long_break_too_low() {
            let result = Cli::try_parse_from(["pomodoro", "start", "--long-break", "0"]);
            assert!(result.is_err());
        }

        #[test]
        fn test_parse_start_long_break_too_high() {
            let result = Cli::try_parse_from(["pomodoro", "start", "--long-break", "61"]);
            assert!(result.is_err());
        }

        #[test]
        fn test_parse_start_work_not_number() {
            let result = Cli::try_parse_from(["pomodoro", "start", "--work", "abc"]);
            assert!(result.is_err());
        }

        #[test]
        fn test_parse_start_work_negative() {
            let result = Cli::try_parse_from(["pomodoro", "start", "--work", "-5"]);
            assert!(result.is_err());
        }

        #[test]
        fn test_parse_unknown_command() {
            let result = Cli::try_parse_from(["pomodoro", "unknown"]);
            assert!(result.is_err());
        }

        #[test]
        fn test_parse_completions_invalid_shell() {
            let result = Cli::try_parse_from(["pomodoro", "completions", "invalid"]);
            assert!(result.is_err());
        }
    }
}
