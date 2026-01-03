//! Icon and title management for menu bar display.
//!
//! This module handles:
//! - Generating display text for the menu bar (e.g., "🍅 15:30")
//! - Managing icon state based on timer phase
//!
//! The text generation logic is platform-independent and fully testable.

use crate::types::{TimerPhase, TimerState};

// ============================================================================
// Constants
// ============================================================================

/// Emoji for work session
const WORKING_EMOJI: &str = "🍅";

/// Emoji for break session
const BREAK_EMOJI: &str = "☕";

/// Emoji for paused/stopped state
const STOPPED_EMOJI: &str = "⏸";

// ============================================================================
// IconManager
// ============================================================================

/// Manages icon and title generation for the menu bar.
///
/// This struct handles the logic for generating display text based on
/// the current timer state. The actual icon images are loaded separately
/// on macOS where they're needed.
#[derive(Debug, Default)]
pub struct IconManager {
    /// Last known timer phase (for optimization)
    last_phase: Option<TimerPhase>,
}

impl IconManager {
    /// Creates a new IconManager.
    pub fn new() -> Self {
        Self { last_phase: None }
    }

    /// Generates the title text for display in the menu bar.
    ///
    /// Format:
    /// - Working: "🍅 MM:SS"
    /// - Breaking/LongBreaking: "☕ MM:SS"
    /// - Paused: "⏸ 一時停止"
    /// - Stopped: "⏸ 停止中"
    ///
    /// # Examples
    ///
    /// ```
    /// use pomodoro::menubar::icon::IconManager;
    /// use pomodoro::types::{TimerState, TimerPhase, PomodoroConfig};
    ///
    /// let manager = IconManager::new();
    /// let mut state = TimerState::new(PomodoroConfig::default());
    /// state.start_working(None);
    ///
    /// // After 9 minutes 30 seconds have elapsed (15:30 remaining from 25:00)
    /// // Simulate: remaining_seconds would be 930
    /// ```
    pub fn generate_title(&self, state: &TimerState) -> String {
        match state.phase {
            TimerPhase::Working => {
                let minutes = state.remaining_seconds / 60;
                let seconds = state.remaining_seconds % 60;
                format!("{} {:02}:{:02}", WORKING_EMOJI, minutes, seconds)
            }
            TimerPhase::Breaking | TimerPhase::LongBreaking => {
                let minutes = state.remaining_seconds / 60;
                let seconds = state.remaining_seconds % 60;
                format!("{} {:02}:{:02}", BREAK_EMOJI, minutes, seconds)
            }
            TimerPhase::Paused => {
                format!("{} 一時停止", STOPPED_EMOJI)
            }
            TimerPhase::Stopped => {
                format!("{} 停止中", STOPPED_EMOJI)
            }
        }
    }

    /// Returns the appropriate emoji for the current phase.
    ///
    /// This is useful for generating status messages or menu items.
    pub fn get_emoji(&self, phase: &TimerPhase) -> &'static str {
        match phase {
            TimerPhase::Working => WORKING_EMOJI,
            TimerPhase::Breaking | TimerPhase::LongBreaking => BREAK_EMOJI,
            TimerPhase::Paused | TimerPhase::Stopped => STOPPED_EMOJI,
        }
    }

    /// Checks if the phase has changed since last update.
    ///
    /// This can be used to optimize icon updates - only update the icon
    /// image when the phase actually changes.
    pub fn phase_changed(&mut self, phase: &TimerPhase) -> bool {
        let changed = self.last_phase.as_ref() != Some(phase);
        if changed {
            self.last_phase = Some(*phase);
        }
        changed
    }

    /// Formats remaining time as MM:SS string.
    ///
    /// This is useful for menu items that only need the time without emoji.
    pub fn format_time(remaining_seconds: u32) -> String {
        let minutes = remaining_seconds / 60;
        let seconds = remaining_seconds % 60;
        format!("{:02}:{:02}", minutes, seconds)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::PomodoroConfig;

    // ------------------------------------------------------------------------
    // IconManager Tests
    // ------------------------------------------------------------------------

    mod icon_manager_tests {
        use super::*;

        #[test]
        fn test_new() {
            let manager = IconManager::new();
            assert!(manager.last_phase.is_none());
        }

        #[test]
        fn test_default() {
            let manager = IconManager::default();
            assert!(manager.last_phase.is_none());
        }
    }

    // ------------------------------------------------------------------------
    // Title Generation Tests
    // ------------------------------------------------------------------------

    mod title_generation_tests {
        use super::*;

        #[test]
        fn test_working_title_25_minutes() {
            let manager = IconManager::new();
            let mut state = TimerState::new(PomodoroConfig::default());
            state.start_working(None);
            // 25 minutes = 1500 seconds

            let title = manager.generate_title(&state);
            assert_eq!(title, "🍅 25:00");
        }

        #[test]
        fn test_working_title_15_30() {
            let manager = IconManager::new();
            let config = PomodoroConfig::default();
            let mut state = TimerState::new(config);
            state.start_working(None);
            // Simulate 9:30 elapsed, 15:30 remaining
            state.remaining_seconds = 930;

            let title = manager.generate_title(&state);
            assert_eq!(title, "🍅 15:30");
        }

        #[test]
        fn test_working_title_single_digit_seconds() {
            let manager = IconManager::new();
            let config = PomodoroConfig::default();
            let mut state = TimerState::new(config);
            state.start_working(None);
            state.remaining_seconds = 65; // 1:05

            let title = manager.generate_title(&state);
            assert_eq!(title, "🍅 01:05");
        }

        #[test]
        fn test_working_title_zero() {
            let manager = IconManager::new();
            let config = PomodoroConfig::default();
            let mut state = TimerState::new(config);
            state.start_working(None);
            state.remaining_seconds = 0;

            let title = manager.generate_title(&state);
            assert_eq!(title, "🍅 00:00");
        }

        #[test]
        fn test_breaking_title() {
            let manager = IconManager::new();
            let config = PomodoroConfig::default();
            let mut state = TimerState::new(config);
            state.pomodoro_count = 1;
            state.start_breaking();
            // 5 minutes = 300 seconds

            let title = manager.generate_title(&state);
            assert_eq!(title, "☕ 05:00");
        }

        #[test]
        fn test_breaking_title_4_30() {
            let manager = IconManager::new();
            let config = PomodoroConfig::default();
            let mut state = TimerState::new(config);
            state.pomodoro_count = 1;
            state.start_breaking();
            state.remaining_seconds = 270; // 4:30

            let title = manager.generate_title(&state);
            assert_eq!(title, "☕ 04:30");
        }

        #[test]
        fn test_long_breaking_title() {
            let manager = IconManager::new();
            let config = PomodoroConfig::default();
            let mut state = TimerState::new(config);
            state.pomodoro_count = 4; // After 4 pomodoros, get long break
            state.start_breaking();
            // 15 minutes = 900 seconds

            let title = manager.generate_title(&state);
            assert_eq!(title, "☕ 15:00");
        }

        #[test]
        fn test_paused_title() {
            let manager = IconManager::new();
            let config = PomodoroConfig::default();
            let mut state = TimerState::new(config);
            state.start_working(None);
            state.remaining_seconds = 500;
            state.pause();

            let title = manager.generate_title(&state);
            assert_eq!(title, "⏸ 一時停止");
        }

        #[test]
        fn test_stopped_title() {
            let manager = IconManager::new();
            let state = TimerState::new(PomodoroConfig::default());
            // Default state is Stopped

            let title = manager.generate_title(&state);
            assert_eq!(title, "⏸ 停止中");
        }
    }

    // ------------------------------------------------------------------------
    // Emoji Tests
    // ------------------------------------------------------------------------

    mod emoji_tests {
        use super::*;

        #[test]
        fn test_get_emoji_working() {
            let manager = IconManager::new();
            assert_eq!(manager.get_emoji(&TimerPhase::Working), "🍅");
        }

        #[test]
        fn test_get_emoji_breaking() {
            let manager = IconManager::new();
            assert_eq!(manager.get_emoji(&TimerPhase::Breaking), "☕");
        }

        #[test]
        fn test_get_emoji_long_breaking() {
            let manager = IconManager::new();
            assert_eq!(manager.get_emoji(&TimerPhase::LongBreaking), "☕");
        }

        #[test]
        fn test_get_emoji_paused() {
            let manager = IconManager::new();
            assert_eq!(manager.get_emoji(&TimerPhase::Paused), "⏸");
        }

        #[test]
        fn test_get_emoji_stopped() {
            let manager = IconManager::new();
            assert_eq!(manager.get_emoji(&TimerPhase::Stopped), "⏸");
        }
    }

    // ------------------------------------------------------------------------
    // Phase Change Tests
    // ------------------------------------------------------------------------

    mod phase_change_tests {
        use super::*;

        #[test]
        fn test_phase_changed_first_call() {
            let mut manager = IconManager::new();
            assert!(manager.phase_changed(&TimerPhase::Working));
        }

        #[test]
        fn test_phase_changed_same_phase() {
            let mut manager = IconManager::new();
            manager.phase_changed(&TimerPhase::Working);
            assert!(!manager.phase_changed(&TimerPhase::Working));
        }

        #[test]
        fn test_phase_changed_different_phase() {
            let mut manager = IconManager::new();
            manager.phase_changed(&TimerPhase::Working);
            assert!(manager.phase_changed(&TimerPhase::Breaking));
        }

        #[test]
        fn test_phase_changed_sequence() {
            let mut manager = IconManager::new();

            // First call
            assert!(manager.phase_changed(&TimerPhase::Stopped));

            // Same phase
            assert!(!manager.phase_changed(&TimerPhase::Stopped));

            // Different phase
            assert!(manager.phase_changed(&TimerPhase::Working));

            // Same phase again
            assert!(!manager.phase_changed(&TimerPhase::Working));

            // Another different phase
            assert!(manager.phase_changed(&TimerPhase::Paused));
        }
    }

    // ------------------------------------------------------------------------
    // Format Time Tests
    // ------------------------------------------------------------------------

    mod format_time_tests {
        use super::*;

        #[test]
        fn test_format_time_25_minutes() {
            assert_eq!(IconManager::format_time(1500), "25:00");
        }

        #[test]
        fn test_format_time_15_30() {
            assert_eq!(IconManager::format_time(930), "15:30");
        }

        #[test]
        fn test_format_time_5_minutes() {
            assert_eq!(IconManager::format_time(300), "05:00");
        }

        #[test]
        fn test_format_time_1_05() {
            assert_eq!(IconManager::format_time(65), "01:05");
        }

        #[test]
        fn test_format_time_zero() {
            assert_eq!(IconManager::format_time(0), "00:00");
        }

        #[test]
        fn test_format_time_59_59() {
            assert_eq!(IconManager::format_time(3599), "59:59");
        }

        #[test]
        fn test_format_time_over_60_minutes() {
            // 120 minutes = 7200 seconds
            assert_eq!(IconManager::format_time(7200), "120:00");
        }
    }
}
