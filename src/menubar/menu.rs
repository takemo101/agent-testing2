//! Menu building and item state management for the menu bar.
//!
//! This module handles:
//! - Menu item configuration and state
//! - Determining which menu items should be enabled/disabled based on timer state
//!
//! The configuration logic is platform-independent and fully testable.
//! Actual menu creation using tray-icon is done in the platform-specific code.

use crate::types::{TimerPhase, TimerState};

// ============================================================================
// MenuItemConfig
// ============================================================================

/// Configuration for a menu item.
#[derive(Debug, Clone)]
pub struct MenuItemConfig {
    /// Display text for the menu item
    pub text: String,
    /// Whether the menu item is enabled (clickable)
    pub enabled: bool,
}

impl MenuItemConfig {
    /// Creates a new menu item configuration.
    pub fn new(text: impl Into<String>, enabled: bool) -> Self {
        Self {
            text: text.into(),
            enabled,
        }
    }
}

// ============================================================================
// MenuConfig
// ============================================================================

/// Complete menu configuration based on current timer state.
///
/// This struct contains the configuration for all menu items,
/// which can then be used to build the actual menu on macOS.
#[derive(Debug, Clone)]
pub struct MenuConfig {
    /// Title item (always disabled, shows app name)
    pub title: MenuItemConfig,
    /// Status info items (always disabled, shows current state)
    pub status_items: Vec<MenuItemConfig>,
    /// Pause button
    pub pause: MenuItemConfig,
    /// Resume button
    pub resume: MenuItemConfig,
    /// Stop button
    pub stop: MenuItemConfig,
    /// Quit button (always enabled)
    pub quit: MenuItemConfig,
}

// ============================================================================
// MenuBuilder
// ============================================================================

/// Builds menu configuration based on timer state.
///
/// This struct handles the logic for determining which menu items
/// should be shown and whether they should be enabled or disabled.
#[derive(Debug, Default)]
pub struct MenuBuilder;

impl MenuBuilder {
    /// Creates a new MenuBuilder.
    pub fn new() -> Self {
        Self
    }

    /// Builds a complete menu configuration based on the current timer state.
    ///
    /// # Arguments
    ///
    /// * `state` - The current timer state
    ///
    /// # Returns
    ///
    /// A `MenuConfig` with all menu items configured appropriately.
    pub fn build(&self, state: &TimerState) -> MenuConfig {
        let status_items = self.build_status_items(state);

        MenuConfig {
            title: MenuItemConfig::new("ポモドーロタイマー", false),
            status_items,
            pause: self.build_pause_item(state),
            resume: self.build_resume_item(state),
            stop: self.build_stop_item(state),
            quit: MenuItemConfig::new("終了", true),
        }
    }

    /// Builds status display items.
    fn build_status_items(&self, state: &TimerState) -> Vec<MenuItemConfig> {
        let mut items = Vec::new();

        match state.phase {
            TimerPhase::Stopped => {
                items.push(MenuItemConfig::new("停止中", false));
            }
            TimerPhase::Working
            | TimerPhase::Breaking
            | TimerPhase::LongBreaking
            | TimerPhase::Paused => {
                // Task name (if any)
                if let Some(ref task_name) = state.task_name {
                    let phase_text = match state.phase {
                        TimerPhase::Working => "作業中",
                        TimerPhase::Breaking | TimerPhase::LongBreaking => "休憩中",
                        TimerPhase::Paused => "一時停止",
                        TimerPhase::Stopped => "停止中",
                    };
                    items.push(MenuItemConfig::new(
                        format!("{}: {}", phase_text, task_name),
                        false,
                    ));
                }

                // Remaining time
                let minutes = state.remaining_seconds / 60;
                let seconds = state.remaining_seconds % 60;
                items.push(MenuItemConfig::new(
                    format!("残り時間: {:02}:{:02}", minutes, seconds),
                    false,
                ));

                // Pomodoro count
                items.push(MenuItemConfig::new(
                    format!("ポモドーロ: #{}", state.pomodoro_count),
                    false,
                ));
            }
        }

        items
    }

    /// Builds the pause menu item.
    ///
    /// Enabled when: Working or Breaking/LongBreaking
    fn build_pause_item(&self, state: &TimerState) -> MenuItemConfig {
        let enabled = matches!(
            state.phase,
            TimerPhase::Working | TimerPhase::Breaking | TimerPhase::LongBreaking
        );
        MenuItemConfig::new("⏸ 一時停止", enabled)
    }

    /// Builds the resume menu item.
    ///
    /// Enabled when: Paused
    fn build_resume_item(&self, state: &TimerState) -> MenuItemConfig {
        let enabled = state.phase == TimerPhase::Paused;
        MenuItemConfig::new("▶ 再開", enabled)
    }

    /// Builds the stop menu item.
    ///
    /// Enabled when: Not Stopped
    fn build_stop_item(&self, state: &TimerState) -> MenuItemConfig {
        let enabled = state.phase != TimerPhase::Stopped;
        MenuItemConfig::new("⏹ 停止", enabled)
    }

    /// Checks if a menu item should be enabled for pause action.
    pub fn is_pause_enabled(phase: &TimerPhase) -> bool {
        matches!(
            phase,
            TimerPhase::Working | TimerPhase::Breaking | TimerPhase::LongBreaking
        )
    }

    /// Checks if a menu item should be enabled for resume action.
    pub fn is_resume_enabled(phase: &TimerPhase) -> bool {
        *phase == TimerPhase::Paused
    }

    /// Checks if a menu item should be enabled for stop action.
    pub fn is_stop_enabled(phase: &TimerPhase) -> bool {
        *phase != TimerPhase::Stopped
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
    // MenuItemConfig Tests
    // ------------------------------------------------------------------------

    mod menu_item_config_tests {
        use super::*;

        #[test]
        fn test_new() {
            let item = MenuItemConfig::new("Test", true);
            assert_eq!(item.text, "Test");
            assert!(item.enabled);
        }

        #[test]
        fn test_new_disabled() {
            let item = MenuItemConfig::new("Disabled", false);
            assert_eq!(item.text, "Disabled");
            assert!(!item.enabled);
        }

        #[test]
        fn test_clone() {
            let item = MenuItemConfig::new("Original", true);
            let cloned = item.clone();
            assert_eq!(cloned.text, "Original");
            assert!(cloned.enabled);
        }
    }

    // ------------------------------------------------------------------------
    // MenuBuilder Tests
    // ------------------------------------------------------------------------

    mod menu_builder_tests {
        use super::*;

        #[test]
        fn test_new() {
            let builder = MenuBuilder::new();
            assert!(std::mem::size_of_val(&builder) == 0); // Zero-sized type
        }

        #[test]
        fn test_default() {
            let _builder = MenuBuilder::default();
        }
    }

    // ------------------------------------------------------------------------
    // Menu Build Tests - Stopped State
    // ------------------------------------------------------------------------

    mod stopped_state_tests {
        use super::*;

        #[test]
        fn test_build_stopped_state() {
            let builder = MenuBuilder::new();
            let state = TimerState::new(PomodoroConfig::default());

            let config = builder.build(&state);

            // Title is always disabled
            assert_eq!(config.title.text, "ポモドーロタイマー");
            assert!(!config.title.enabled);

            // Status shows "停止中"
            assert_eq!(config.status_items.len(), 1);
            assert_eq!(config.status_items[0].text, "停止中");
            assert!(!config.status_items[0].enabled);

            // Pause disabled
            assert!(!config.pause.enabled);

            // Resume disabled
            assert!(!config.resume.enabled);

            // Stop disabled (already stopped)
            assert!(!config.stop.enabled);

            // Quit always enabled
            assert!(config.quit.enabled);
        }
    }

    // ------------------------------------------------------------------------
    // Menu Build Tests - Working State
    // ------------------------------------------------------------------------

    mod working_state_tests {
        use super::*;

        #[test]
        fn test_build_working_state_with_task() {
            let builder = MenuBuilder::new();
            let mut state = TimerState::new(PomodoroConfig::default());
            state.start_working(Some("API実装".to_string()));
            state.remaining_seconds = 930; // 15:30
            state.pomodoro_count = 2;

            let config = builder.build(&state);

            // Status items
            assert_eq!(config.status_items.len(), 3);
            assert_eq!(config.status_items[0].text, "作業中: API実装");
            assert_eq!(config.status_items[1].text, "残り時間: 15:30");
            assert_eq!(config.status_items[2].text, "ポモドーロ: #2");

            // Pause enabled
            assert!(config.pause.enabled);

            // Resume disabled
            assert!(!config.resume.enabled);

            // Stop enabled
            assert!(config.stop.enabled);
        }

        #[test]
        fn test_build_working_state_without_task() {
            let builder = MenuBuilder::new();
            let mut state = TimerState::new(PomodoroConfig::default());
            state.start_working(None);
            state.remaining_seconds = 1500; // 25:00

            let config = builder.build(&state);

            // Status items (no task name)
            assert_eq!(config.status_items.len(), 2);
            assert_eq!(config.status_items[0].text, "残り時間: 25:00");
            assert_eq!(config.status_items[1].text, "ポモドーロ: #0");
        }
    }

    // ------------------------------------------------------------------------
    // Menu Build Tests - Breaking State
    // ------------------------------------------------------------------------

    mod breaking_state_tests {
        use super::*;

        #[test]
        fn test_build_short_break_state() {
            let builder = MenuBuilder::new();
            let mut state = TimerState::new(PomodoroConfig::default());
            state.pomodoro_count = 1;
            state.start_breaking();
            state.remaining_seconds = 270; // 4:30

            let config = builder.build(&state);

            // Status items
            assert_eq!(config.status_items.len(), 2);
            assert_eq!(config.status_items[0].text, "残り時間: 04:30");
            assert_eq!(config.status_items[1].text, "ポモドーロ: #1");

            // Pause enabled (can pause during break)
            assert!(config.pause.enabled);

            // Resume disabled
            assert!(!config.resume.enabled);

            // Stop enabled
            assert!(config.stop.enabled);
        }

        #[test]
        fn test_build_long_break_state() {
            let builder = MenuBuilder::new();
            let mut state = TimerState::new(PomodoroConfig::default());
            state.pomodoro_count = 4;
            state.start_breaking(); // Will be LongBreaking
            state.remaining_seconds = 900; // 15:00

            let config = builder.build(&state);

            // Pause enabled
            assert!(config.pause.enabled);

            // Resume disabled
            assert!(!config.resume.enabled);

            // Stop enabled
            assert!(config.stop.enabled);
        }

        #[test]
        fn test_build_break_with_task() {
            let builder = MenuBuilder::new();
            let mut state = TimerState::new(PomodoroConfig::default());
            state.start_working(Some("テスト作業".to_string()));
            state.pomodoro_count = 1;
            // Manually set to breaking while keeping task name
            state.start_breaking();
            // Task name is cleared on start_breaking, so we set it again
            state.task_name = Some("テスト作業".to_string());

            let config = builder.build(&state);

            // Should show task name
            assert!(config.status_items[0].text.contains("テスト作業"));
        }
    }

    // ------------------------------------------------------------------------
    // Menu Build Tests - Paused State
    // ------------------------------------------------------------------------

    mod paused_state_tests {
        use super::*;

        #[test]
        fn test_build_paused_state() {
            let builder = MenuBuilder::new();
            let mut state = TimerState::new(PomodoroConfig::default());
            state.start_working(Some("作業中".to_string()));
            state.remaining_seconds = 500;
            state.pomodoro_count = 1;
            state.pause();

            let config = builder.build(&state);

            // Status items
            assert_eq!(config.status_items.len(), 3);
            assert!(config.status_items[0].text.contains("一時停止"));
            assert_eq!(config.status_items[1].text, "残り時間: 08:20");
            assert_eq!(config.status_items[2].text, "ポモドーロ: #1");

            // Pause disabled
            assert!(!config.pause.enabled);

            // Resume enabled
            assert!(config.resume.enabled);

            // Stop enabled
            assert!(config.stop.enabled);
        }
    }

    // ------------------------------------------------------------------------
    // Static Enable Check Tests
    // ------------------------------------------------------------------------

    mod enable_check_tests {
        use super::*;

        #[test]
        fn test_is_pause_enabled() {
            assert!(!MenuBuilder::is_pause_enabled(&TimerPhase::Stopped));
            assert!(MenuBuilder::is_pause_enabled(&TimerPhase::Working));
            assert!(MenuBuilder::is_pause_enabled(&TimerPhase::Breaking));
            assert!(MenuBuilder::is_pause_enabled(&TimerPhase::LongBreaking));
            assert!(!MenuBuilder::is_pause_enabled(&TimerPhase::Paused));
        }

        #[test]
        fn test_is_resume_enabled() {
            assert!(!MenuBuilder::is_resume_enabled(&TimerPhase::Stopped));
            assert!(!MenuBuilder::is_resume_enabled(&TimerPhase::Working));
            assert!(!MenuBuilder::is_resume_enabled(&TimerPhase::Breaking));
            assert!(!MenuBuilder::is_resume_enabled(&TimerPhase::LongBreaking));
            assert!(MenuBuilder::is_resume_enabled(&TimerPhase::Paused));
        }

        #[test]
        fn test_is_stop_enabled() {
            assert!(!MenuBuilder::is_stop_enabled(&TimerPhase::Stopped));
            assert!(MenuBuilder::is_stop_enabled(&TimerPhase::Working));
            assert!(MenuBuilder::is_stop_enabled(&TimerPhase::Breaking));
            assert!(MenuBuilder::is_stop_enabled(&TimerPhase::LongBreaking));
            assert!(MenuBuilder::is_stop_enabled(&TimerPhase::Paused));
        }
    }
}
