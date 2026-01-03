//! Pomodoro Timer CLI - A macOS-native productivity tool
//!
//! This tool helps you stay focused using the Pomodoro Technique:
//! - 25 minutes of focused work
//! - 5 minutes of short break
//! - 15-30 minutes of long break after 4 pomodoros

use anyhow::Result;
use clap::{CommandFactory, Parser};

pub mod cli;
pub mod daemon;
pub mod types;

use cli::{Cli, Commands, Display, IpcClient};

/// Main entry point
#[tokio::main(flavor = "current_thread")]
async fn main() {
    // Initialize logging
    init_tracing();

    // Parse command line arguments
    let cli = Cli::parse();

    // Execute command
    if let Err(e) = execute(cli).await {
        Display::show_error(&e.to_string());
        std::process::exit(1);
    }
}

/// Initializes the tracing subscriber for logging.
fn init_tracing() {
    use tracing_subscriber::{fmt, EnvFilter};

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("warn"));

    fmt()
        .with_env_filter(filter)
        .with_target(false)
        .without_time()
        .init();
}

/// Executes the CLI command.
async fn execute(cli: Cli) -> Result<()> {
    // Set verbose logging if requested
    if cli.verbose {
        tracing::info!("Verbose mode enabled");
    }

    match cli.command {
        Some(Commands::Start(args)) => {
            let client = IpcClient::new()?;
            let response = client.start(&args).await?;
            Display::show_start_success(&response);
        }
        Some(Commands::Pause) => {
            let client = IpcClient::new()?;
            let response = client.pause().await?;
            Display::show_pause_success(&response);
        }
        Some(Commands::Resume) => {
            let client = IpcClient::new()?;
            let response = client.resume().await?;
            Display::show_resume_success(&response);
        }
        Some(Commands::Stop) => {
            let client = IpcClient::new()?;
            let response = client.stop().await?;
            Display::show_stop_success(&response);
        }
        Some(Commands::Status) => {
            let client = IpcClient::new()?;
            let response = client.status().await?;
            Display::show_status(&response);
        }
        Some(Commands::Daemon) => {
            // Daemon mode will be implemented in a future issue
            eprintln!("Daemonモードはまだ実装されていません");
            eprintln!("今後のリリースで対応予定です");
            std::process::exit(1);
        }
        Some(Commands::Install) => {
            // LaunchAgent installation will be implemented in Issue #10
            Display::show_install_success();
            eprintln!("注意: LaunchAgentのインストールは今後のリリースで対応予定です");
        }
        Some(Commands::Uninstall) => {
            // LaunchAgent uninstallation will be implemented in Issue #10
            Display::show_uninstall_success();
            eprintln!("注意: LaunchAgentのアンインストールは今後のリリースで対応予定です");
        }
        Some(Commands::Completions { shell }) => {
            generate_completions(shell);
        }
        None => {
            // No command provided, show help
            Cli::command().print_help()?;
        }
    }

    Ok(())
}

/// Generates shell completion scripts.
fn generate_completions(shell: clap_complete::Shell) {
    use clap_complete::generate;
    use std::io;

    let mut cmd = Cli::command();
    let bin_name = cmd.get_name().to_string();
    generate(shell, &mut cmd, bin_name, &mut io::stdout());
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_parse_no_args() {
        let cli = Cli::parse_from(["pomodoro"]);
        assert!(cli.command.is_none());
    }

    #[test]
    fn test_cli_parse_status() {
        let cli = Cli::parse_from(["pomodoro", "status"]);
        assert!(matches!(cli.command, Some(Commands::Status)));
    }

    #[test]
    fn test_cli_parse_start() {
        let cli = Cli::parse_from(["pomodoro", "start"]);
        assert!(matches!(cli.command, Some(Commands::Start(_))));
    }

    #[test]
    fn test_cli_parse_start_with_options() {
        let cli = Cli::parse_from([
            "pomodoro",
            "start",
            "--work",
            "30",
            "--task",
            "Test",
        ]);
        match cli.command {
            Some(Commands::Start(args)) => {
                assert_eq!(args.work, 30);
                assert_eq!(args.task, Some("Test".to_string()));
            }
            _ => panic!("Expected Start command"),
        }
    }

    #[test]
    fn test_cli_parse_verbose() {
        let cli = Cli::parse_from(["pomodoro", "--verbose", "status"]);
        assert!(cli.verbose);
    }
}
