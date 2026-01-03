//! Pomodoro Timer CLI - A macOS-native productivity tool
//!
//! This tool helps you stay focused using the Pomodoro Technique:
//! - 25 minutes of focused work
//! - 5 minutes of short break
//! - 15-30 minutes of long break after 4 pomodoros

use clap::Parser;

pub mod types;

/// Pomodoro Timer CLI - Stay focused, stay productive
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    /// Subcommand to execute
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(clap::Subcommand, Debug)]
enum Commands {
    /// Start a new pomodoro session
    Start {
        /// Work duration in minutes (default: 25)
        #[arg(short, long, default_value = "25")]
        work: u32,

        /// Break duration in minutes (default: 5)
        #[arg(short, long, default_value = "5")]
        r#break: u32,

        /// Long break duration in minutes (default: 15)
        #[arg(short, long, default_value = "15")]
        long_break: u32,

        /// Task name for this session
        #[arg(short, long)]
        task: Option<String>,
    },
    /// Pause the current timer
    Pause,
    /// Resume a paused timer
    Resume,
    /// Stop the current timer
    Stop,
    /// Show current timer status
    Status,
    /// Run as daemon (background service)
    Daemon,
    /// Install LaunchAgent for auto-start
    Install,
    /// Uninstall LaunchAgent
    Uninstall,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Start {
            work,
            r#break,
            long_break,
            task,
        }) => {
            println!(
                "Starting pomodoro: work={}min, break={}min, long_break={}min",
                work, r#break, long_break
            );
            if let Some(task_name) = task {
                println!("Task: {}", task_name);
            }
        }
        Some(Commands::Pause) => println!("Pausing timer..."),
        Some(Commands::Resume) => println!("Resuming timer..."),
        Some(Commands::Stop) => println!("Stopping timer..."),
        Some(Commands::Status) => println!("Status: No active timer"),
        Some(Commands::Daemon) => println!("Starting daemon mode..."),
        Some(Commands::Install) => println!("Installing LaunchAgent..."),
        Some(Commands::Uninstall) => println!("Uninstalling LaunchAgent..."),
        None => println!("Use --help to see available commands"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_parse() {
        // Test that CLI can be parsed without arguments
        let cli = Cli::parse_from(["pomodoro"]);
        assert!(cli.command.is_none());
    }

    #[test]
    fn test_start_command() {
        let cli = Cli::parse_from(["pomodoro", "start", "--work", "30", "--task", "coding"]);
        match cli.command {
            Some(Commands::Start { work, task, .. }) => {
                assert_eq!(work, 30);
                assert_eq!(task, Some("coding".to_string()));
            }
            _ => panic!("Expected Start command"),
        }
    }

    #[test]
    fn test_status_command() {
        let cli = Cli::parse_from(["pomodoro", "status"]);
        assert!(matches!(cli.command, Some(Commands::Status)));
    }
}
