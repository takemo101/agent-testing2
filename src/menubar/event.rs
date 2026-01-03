//! Event handling for menu bar interactions.
//!
//! This module handles:
//! - Menu event processing
//! - Command dispatching based on menu item clicks
//!
//! The event types and command mapping are platform-independent.
//! Actual event handling with tray-icon is done in the platform-specific code.

use std::fmt;

// ============================================================================
// MenuAction
// ============================================================================

/// Actions that can be triggered from the menu bar.
///
/// These actions are platform-independent and represent what the user
/// wants to do. The actual IPC communication happens elsewhere.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuAction {
    /// Pause the current timer
    Pause,
    /// Resume a paused timer
    Resume,
    /// Stop the timer completely
    Stop,
    /// Quit the daemon
    Quit,
}

impl fmt::Display for MenuAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MenuAction::Pause => write!(f, "pause"),
            MenuAction::Resume => write!(f, "resume"),
            MenuAction::Stop => write!(f, "stop"),
            MenuAction::Quit => write!(f, "quit"),
        }
    }
}

impl MenuAction {
    /// Returns the IPC command name for this action.
    pub fn as_command(&self) -> &'static str {
        match self {
            MenuAction::Pause => "pause",
            MenuAction::Resume => "resume",
            MenuAction::Stop => "stop",
            MenuAction::Quit => "quit",
        }
    }

    /// Returns a human-readable description of this action.
    pub fn description(&self) -> &'static str {
        match self {
            MenuAction::Pause => "一時停止",
            MenuAction::Resume => "再開",
            MenuAction::Stop => "停止",
            MenuAction::Quit => "終了",
        }
    }
}

// ============================================================================
// MenuItemId
// ============================================================================

/// Identifiers for menu items.
///
/// On macOS, these map to actual menu item IDs from tray-icon.
/// For testing, we use our own enum-based identification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MenuItemId {
    /// Pause menu item
    Pause,
    /// Resume menu item
    Resume,
    /// Stop menu item
    Stop,
    /// Quit menu item
    Quit,
    /// Unknown or unhandled menu item
    Unknown,
}

impl MenuItemId {
    /// Converts a menu item ID to the corresponding action.
    ///
    /// Returns `Some(action)` if the item corresponds to an action,
    /// or `None` for items like title or status that don't trigger actions.
    pub fn to_action(&self) -> Option<MenuAction> {
        match self {
            MenuItemId::Pause => Some(MenuAction::Pause),
            MenuItemId::Resume => Some(MenuAction::Resume),
            MenuItemId::Stop => Some(MenuAction::Stop),
            MenuItemId::Quit => Some(MenuAction::Quit),
            MenuItemId::Unknown => None,
        }
    }
}

// ============================================================================
// EventHandler
// ============================================================================

/// Handles menu events and converts them to actions.
///
/// This struct provides the logic for processing menu events.
/// It's designed to be used with the actual tray-icon event handling
/// on macOS, but the core logic is platform-independent.
#[derive(Debug, Default)]
pub struct EventHandler;

impl EventHandler {
    /// Creates a new EventHandler.
    pub fn new() -> Self {
        Self
    }

    /// Processes a menu item click and returns the corresponding action.
    ///
    /// # Arguments
    ///
    /// * `item_id` - The ID of the clicked menu item
    ///
    /// # Returns
    ///
    /// The action to perform, or `None` if the item doesn't trigger an action.
    pub fn handle_click(&self, item_id: MenuItemId) -> Option<MenuAction> {
        let action = item_id.to_action();
        
        if let Some(ref action) = action {
            tracing::info!(
                action = %action,
                "メニューバーからアクションを受信"
            );
        }
        
        action
    }

    /// Logs the result of an action execution.
    pub fn log_action_result(&self, action: &MenuAction, success: bool) {
        if success {
            tracing::info!(
                action = %action,
                "アクション実行成功"
            );
        } else {
            tracing::error!(
                action = %action,
                "アクション実行失敗"
            );
        }
    }
}

// ============================================================================
// TrayUpdate
// ============================================================================

/// Updates that can be sent to the tray icon from other parts of the system.
///
/// This enum is used with crossbeam-channel to send updates from the
/// timer engine (running in tokio) to the tray icon (running on the main thread).
#[derive(Debug, Clone)]
pub enum TrayUpdate {
    /// Update the title text displayed in the menu bar
    SetTitle(String),
    /// Update the menu items (rebuild the menu)
    RebuildMenu,
    /// Shutdown the tray icon
    Shutdown,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ------------------------------------------------------------------------
    // MenuAction Tests
    // ------------------------------------------------------------------------

    mod menu_action_tests {
        use super::*;

        #[test]
        fn test_display() {
            assert_eq!(format!("{}", MenuAction::Pause), "pause");
            assert_eq!(format!("{}", MenuAction::Resume), "resume");
            assert_eq!(format!("{}", MenuAction::Stop), "stop");
            assert_eq!(format!("{}", MenuAction::Quit), "quit");
        }

        #[test]
        fn test_as_command() {
            assert_eq!(MenuAction::Pause.as_command(), "pause");
            assert_eq!(MenuAction::Resume.as_command(), "resume");
            assert_eq!(MenuAction::Stop.as_command(), "stop");
            assert_eq!(MenuAction::Quit.as_command(), "quit");
        }

        #[test]
        fn test_description() {
            assert_eq!(MenuAction::Pause.description(), "一時停止");
            assert_eq!(MenuAction::Resume.description(), "再開");
            assert_eq!(MenuAction::Stop.description(), "停止");
            assert_eq!(MenuAction::Quit.description(), "終了");
        }

        #[test]
        fn test_clone() {
            let action = MenuAction::Pause;
            let cloned = action;
            assert_eq!(action, cloned);
        }

        #[test]
        fn test_copy() {
            let action = MenuAction::Resume;
            let copied = action;
            assert_eq!(action, copied);
        }

        #[test]
        fn test_eq() {
            assert_eq!(MenuAction::Pause, MenuAction::Pause);
            assert_ne!(MenuAction::Pause, MenuAction::Resume);
        }
    }

    // ------------------------------------------------------------------------
    // MenuItemId Tests
    // ------------------------------------------------------------------------

    mod menu_item_id_tests {
        use super::*;

        #[test]
        fn test_to_action_pause() {
            assert_eq!(MenuItemId::Pause.to_action(), Some(MenuAction::Pause));
        }

        #[test]
        fn test_to_action_resume() {
            assert_eq!(MenuItemId::Resume.to_action(), Some(MenuAction::Resume));
        }

        #[test]
        fn test_to_action_stop() {
            assert_eq!(MenuItemId::Stop.to_action(), Some(MenuAction::Stop));
        }

        #[test]
        fn test_to_action_quit() {
            assert_eq!(MenuItemId::Quit.to_action(), Some(MenuAction::Quit));
        }

        #[test]
        fn test_to_action_unknown() {
            assert_eq!(MenuItemId::Unknown.to_action(), None);
        }

        #[test]
        fn test_hash() {
            use std::collections::HashSet;
            
            let mut set = HashSet::new();
            set.insert(MenuItemId::Pause);
            set.insert(MenuItemId::Resume);
            set.insert(MenuItemId::Stop);
            set.insert(MenuItemId::Quit);
            set.insert(MenuItemId::Unknown);
            
            assert_eq!(set.len(), 5);
            assert!(set.contains(&MenuItemId::Pause));
        }
    }

    // ------------------------------------------------------------------------
    // EventHandler Tests
    // ------------------------------------------------------------------------

    mod event_handler_tests {
        use super::*;

        #[test]
        fn test_new() {
            let _handler = EventHandler::new();
        }

        #[test]
        fn test_default() {
            let _handler = EventHandler::default();
        }

        #[test]
        fn test_handle_click_pause() {
            let handler = EventHandler::new();
            let action = handler.handle_click(MenuItemId::Pause);
            assert_eq!(action, Some(MenuAction::Pause));
        }

        #[test]
        fn test_handle_click_resume() {
            let handler = EventHandler::new();
            let action = handler.handle_click(MenuItemId::Resume);
            assert_eq!(action, Some(MenuAction::Resume));
        }

        #[test]
        fn test_handle_click_stop() {
            let handler = EventHandler::new();
            let action = handler.handle_click(MenuItemId::Stop);
            assert_eq!(action, Some(MenuAction::Stop));
        }

        #[test]
        fn test_handle_click_quit() {
            let handler = EventHandler::new();
            let action = handler.handle_click(MenuItemId::Quit);
            assert_eq!(action, Some(MenuAction::Quit));
        }

        #[test]
        fn test_handle_click_unknown() {
            let handler = EventHandler::new();
            let action = handler.handle_click(MenuItemId::Unknown);
            assert_eq!(action, None);
        }
    }

    // ------------------------------------------------------------------------
    // TrayUpdate Tests
    // ------------------------------------------------------------------------

    mod tray_update_tests {
        use super::*;

        #[test]
        fn test_set_title() {
            let update = TrayUpdate::SetTitle("🍅 15:30".to_string());
            match update {
                TrayUpdate::SetTitle(title) => assert_eq!(title, "🍅 15:30"),
                _ => panic!("Expected SetTitle"),
            }
        }

        #[test]
        fn test_rebuild_menu() {
            let update = TrayUpdate::RebuildMenu;
            assert!(matches!(update, TrayUpdate::RebuildMenu));
        }

        #[test]
        fn test_shutdown() {
            let update = TrayUpdate::Shutdown;
            assert!(matches!(update, TrayUpdate::Shutdown));
        }

        #[test]
        fn test_clone() {
            let update = TrayUpdate::SetTitle("test".to_string());
            let cloned = update.clone();
            match cloned {
                TrayUpdate::SetTitle(title) => assert_eq!(title, "test"),
                _ => panic!("Expected SetTitle"),
            }
        }
    }
}
