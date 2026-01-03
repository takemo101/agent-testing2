//! Menu bar UI module for the Pomodoro Timer.
//!
//! This module provides:
//! - Tray icon management for macOS menu bar
//! - Real-time timer display (e.g., "🍅 15:30")
//! - Dropdown menu with pause/resume/stop actions
//! - Event handling for menu interactions
//!
//! # Architecture
//!
//! The module is split into platform-independent and platform-specific parts:
//!
//! - `icon.rs`: Title text generation (platform-independent, fully testable)
//! - `menu.rs`: Menu configuration (platform-independent, fully testable)
//! - `event.rs`: Event types and handling (platform-independent, fully testable)
//! - `mod.rs`: TrayIconManager (platform-specific on macOS)
//!
//! # Usage
//!
//! The tray icon is created and managed by the daemon. Updates are sent via
//! a crossbeam channel from the timer engine (running in tokio) to the tray
//! icon (running on the main thread).
//!
//! ```ignore
//! use pomodoro::menubar::{TrayIconManager, TrayUpdate};
//! use crossbeam_channel::unbounded;
//!
//! // Create channel for updates
//! let (tx, rx) = unbounded();
//!
//! // Create manager (on main thread, macOS only)
//! let mut manager = TrayIconManager::new(rx)?;
//! manager.initialize()?;
//!
//! // From timer engine (tokio task)
//! tx.send(TrayUpdate::SetTitle("🍅 15:30".to_string()))?;
//! ```

pub mod event;
pub mod icon;
pub mod menu;

// Re-export main types
pub use event::{EventHandler, MenuAction, MenuItemId, TrayUpdate};
pub use icon::IconManager;
pub use menu::{MenuBuilder, MenuConfig, MenuItemConfig};

use crate::types::TimerState;
use crossbeam_channel::Receiver;
use std::sync::{Arc, RwLock};

// ============================================================================
// TrayIconManager
// ============================================================================

/// Manages the tray icon and menu bar UI.
///
/// This struct coordinates between:
/// - IconManager: Generates display text
/// - MenuBuilder: Configures menu items
/// - EventHandler: Processes menu clicks
///
/// On macOS, it also manages the actual tray-icon instance.
/// On other platforms, it operates in a no-op mode.
pub struct TrayIconManager {
    /// Icon manager for title generation
    icon_manager: IconManager,
    /// Menu builder for menu configuration
    menu_builder: MenuBuilder,
    /// Event handler for menu clicks
    event_handler: EventHandler,
    /// Current timer state (shared with daemon)
    current_state: Arc<RwLock<TimerState>>,
    /// Channel for receiving updates from timer engine
    update_rx: Receiver<TrayUpdate>,
    /// Whether the manager is initialized
    initialized: bool,
    /// Platform-specific tray icon instance (macOS only)
    #[cfg(target_os = "macos")]
    tray_icon: Option<tray_icon::TrayIcon>,
}

impl TrayIconManager {
    /// Creates a new TrayIconManager.
    ///
    /// # Arguments
    ///
    /// * `initial_state` - Initial timer state
    /// * `update_rx` - Channel for receiving updates from timer engine
    ///
    /// # Note
    ///
    /// On macOS, the actual tray icon is not created until `initialize()` is called.
    /// This is because macOS requires the event loop to be running before creating
    /// the tray icon.
    pub fn new(initial_state: TimerState, update_rx: Receiver<TrayUpdate>) -> Self {
        Self {
            icon_manager: IconManager::new(),
            menu_builder: MenuBuilder::new(),
            event_handler: EventHandler::new(),
            current_state: Arc::new(RwLock::new(initial_state)),
            update_rx,
            initialized: false,
            #[cfg(target_os = "macos")]
            tray_icon: None,
        }
    }

    /// Returns a reference to the current state.
    pub fn current_state(&self) -> Arc<RwLock<TimerState>> {
        Arc::clone(&self.current_state)
    }

    /// Returns whether the manager is initialized.
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    /// Returns a reference to the icon manager.
    pub fn icon_manager(&self) -> &IconManager {
        &self.icon_manager
    }

    /// Returns a mutable reference to the icon manager.
    pub fn icon_manager_mut(&mut self) -> &mut IconManager {
        &mut self.icon_manager
    }

    /// Returns a reference to the menu builder.
    pub fn menu_builder(&self) -> &MenuBuilder {
        &self.menu_builder
    }

    /// Returns a reference to the event handler.
    pub fn event_handler(&self) -> &EventHandler {
        &self.event_handler
    }

    /// Generates the current title for the menu bar.
    pub fn generate_title(&self) -> String {
        let state = self.current_state.read().unwrap();
        self.icon_manager.generate_title(&state)
    }

    /// Generates the current menu configuration.
    pub fn generate_menu_config(&self) -> MenuConfig {
        let state = self.current_state.read().unwrap();
        self.menu_builder.build(&state)
    }

    /// Updates the current state.
    pub fn update_state(&self, state: TimerState) {
        let mut current = self.current_state.write().unwrap();
        *current = state;
    }

    /// Processes a pending update from the channel.
    ///
    /// Returns `true` if an update was processed, `false` if the channel was empty.
    pub fn process_pending_update(&mut self) -> bool {
        match self.update_rx.try_recv() {
            Ok(update) => {
                self.handle_update(update);
                true
            }
            Err(crossbeam_channel::TryRecvError::Empty) => false,
            Err(crossbeam_channel::TryRecvError::Disconnected) => {
                tracing::warn!("メニューバー更新チャネルが切断されました");
                false
            }
        }
    }

    /// Handles an update from the timer engine.
    fn handle_update(&mut self, update: TrayUpdate) {
        match update {
            TrayUpdate::SetTitle(title) => {
                tracing::debug!(title = %title, "メニューバータイトル更新");
                #[cfg(target_os = "macos")]
                if let Some(ref tray_icon) = self.tray_icon {
                    tray_icon.set_title(Some(&title));
                }
            }
            TrayUpdate::RebuildMenu => {
                tracing::debug!("メニュー再構築");
                // Menu rebuilding is handled by the platform-specific code
            }
            TrayUpdate::Shutdown => {
                tracing::info!("メニューバーをシャットダウン");
                self.shutdown();
            }
        }
    }

    /// Shuts down the tray icon.
    pub fn shutdown(&mut self) {
        self.initialized = false;
        #[cfg(target_os = "macos")]
        {
            self.tray_icon = None;
        }
    }

    /// Initializes the tray icon (macOS only).
    ///
    /// This must be called from the main thread after the event loop is running.
    ///
    /// # Errors
    ///
    /// Returns an error if the tray icon cannot be created.
    #[cfg(target_os = "macos")]
    pub fn initialize(&mut self) -> anyhow::Result<()> {
        use anyhow::Context;
        use tray_icon::TrayIconBuilder;

        let state = self.current_state.read().unwrap();
        let title = self.icon_manager.generate_title(&state);
        let menu_config = self.menu_builder.build(&state);
        drop(state);

        // Build the menu
        let menu = self.build_native_menu(&menu_config)?;

        // Create the tray icon
        let tray_icon = TrayIconBuilder::new()
            .with_title(&title)
            .with_menu(Box::new(menu))
            .with_tooltip("ポモドーロタイマー")
            .build()
            .context("トレイアイコンの作成に失敗しました")?;

        self.tray_icon = Some(tray_icon);
        self.initialized = true;

        tracing::info!("メニューバーアイコンを初期化しました");
        Ok(())
    }

    /// Initializes the tray icon (non-macOS, no-op).
    #[cfg(not(target_os = "macos"))]
    pub fn initialize(&mut self) -> anyhow::Result<()> {
        tracing::warn!("メニューバーはmacOSでのみサポートされています");
        self.initialized = true;
        Ok(())
    }

    /// Builds a native menu from the configuration (macOS only).
    #[cfg(target_os = "macos")]
    fn build_native_menu(
        &self,
        config: &MenuConfig,
    ) -> anyhow::Result<tray_icon::menu::Menu> {
        use tray_icon::menu::{Menu, MenuItem, PredefinedMenuItem};

        let menu = Menu::new();

        // Title (disabled)
        let title_item = MenuItem::new(&config.title.text, false, None);
        menu.append(&title_item)?;

        menu.append(&PredefinedMenuItem::separator())?;

        // Status items (all disabled)
        for item in &config.status_items {
            let status_item = MenuItem::new(&item.text, false, None);
            menu.append(&status_item)?;
        }

        menu.append(&PredefinedMenuItem::separator())?;

        // Action items
        let pause_item = MenuItem::new(&config.pause.text, config.pause.enabled, None);
        menu.append(&pause_item)?;

        let resume_item = MenuItem::new(&config.resume.text, config.resume.enabled, None);
        menu.append(&resume_item)?;

        let stop_item = MenuItem::new(&config.stop.text, config.stop.enabled, None);
        menu.append(&stop_item)?;

        menu.append(&PredefinedMenuItem::separator())?;

        // Quit item
        let quit_item = MenuItem::new(&config.quit.text, config.quit.enabled, None);
        menu.append(&quit_item)?;

        Ok(menu)
    }
}

impl std::fmt::Debug for TrayIconManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TrayIconManager")
            .field("initialized", &self.initialized)
            .field("icon_manager", &self.icon_manager)
            .field("menu_builder", &self.menu_builder)
            .field("event_handler", &self.event_handler)
            .finish_non_exhaustive()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::PomodoroConfig;
    use crossbeam_channel::unbounded;

    // ------------------------------------------------------------------------
    // TrayIconManager Tests
    // ------------------------------------------------------------------------

    mod manager_tests {
        use super::*;

        #[test]
        fn test_new() {
            let (_, rx) = unbounded();
            let state = TimerState::new(PomodoroConfig::default());
            let manager = TrayIconManager::new(state, rx);

            assert!(!manager.is_initialized());
        }

        #[test]
        fn test_generate_title_stopped() {
            let (_, rx) = unbounded();
            let state = TimerState::new(PomodoroConfig::default());
            let manager = TrayIconManager::new(state, rx);

            let title = manager.generate_title();
            assert_eq!(title, "⏸ 停止中");
        }

        #[test]
        fn test_generate_title_working() {
            let (_, rx) = unbounded();
            let mut state = TimerState::new(PomodoroConfig::default());
            state.start_working(None);
            state.remaining_seconds = 930; // 15:30
            let manager = TrayIconManager::new(state, rx);

            let title = manager.generate_title();
            assert_eq!(title, "🍅 15:30");
        }

        #[test]
        fn test_generate_menu_config() {
            let (_, rx) = unbounded();
            let state = TimerState::new(PomodoroConfig::default());
            let manager = TrayIconManager::new(state, rx);

            let config = manager.generate_menu_config();
            assert_eq!(config.title.text, "ポモドーロタイマー");
            assert!(!config.pause.enabled);
            assert!(!config.resume.enabled);
            assert!(!config.stop.enabled);
            assert!(config.quit.enabled);
        }

        #[test]
        fn test_update_state() {
            let (_, rx) = unbounded();
            let state = TimerState::new(PomodoroConfig::default());
            let manager = TrayIconManager::new(state, rx);

            // Initial state is stopped
            assert_eq!(manager.generate_title(), "⏸ 停止中");

            // Update to working state
            let mut new_state = TimerState::new(PomodoroConfig::default());
            new_state.start_working(None);
            new_state.remaining_seconds = 600;
            manager.update_state(new_state);

            assert_eq!(manager.generate_title(), "🍅 10:00");
        }

        #[test]
        fn test_process_pending_update_empty() {
            let (_, rx) = unbounded();
            let state = TimerState::new(PomodoroConfig::default());
            let mut manager = TrayIconManager::new(state, rx);

            // No updates pending
            assert!(!manager.process_pending_update());
        }

        #[test]
        fn test_process_pending_update_set_title() {
            let (tx, rx) = unbounded();
            let state = TimerState::new(PomodoroConfig::default());
            let mut manager = TrayIconManager::new(state, rx);

            // Send an update
            tx.send(TrayUpdate::SetTitle("🍅 15:30".to_string())).unwrap();

            // Process it
            assert!(manager.process_pending_update());

            // No more updates
            assert!(!manager.process_pending_update());
        }

        #[test]
        fn test_process_pending_update_shutdown() {
            let (tx, rx) = unbounded();
            let state = TimerState::new(PomodoroConfig::default());
            let mut manager = TrayIconManager::new(state, rx);
            
            // Mark as initialized first
            manager.initialized = true;
            assert!(manager.is_initialized());

            // Send shutdown
            tx.send(TrayUpdate::Shutdown).unwrap();
            manager.process_pending_update();

            assert!(!manager.is_initialized());
        }

        #[test]
        fn test_current_state_arc() {
            let (_, rx) = unbounded();
            let state = TimerState::new(PomodoroConfig::default());
            let manager = TrayIconManager::new(state, rx);

            let state_arc1 = manager.current_state();
            let state_arc2 = manager.current_state();

            // Both should point to the same state
            assert!(Arc::ptr_eq(&state_arc1, &state_arc2));
        }

        #[test]
        fn test_accessors() {
            let (_, rx) = unbounded();
            let state = TimerState::new(PomodoroConfig::default());
            let mut manager = TrayIconManager::new(state, rx);

            // Test that accessors work
            let _ = manager.icon_manager();
            let _ = manager.icon_manager_mut();
            let _ = manager.menu_builder();
            let _ = manager.event_handler();
        }

        #[test]
        fn test_debug() {
            let (_, rx) = unbounded();
            let state = TimerState::new(PomodoroConfig::default());
            let manager = TrayIconManager::new(state, rx);

            let debug = format!("{:?}", manager);
            assert!(debug.contains("TrayIconManager"));
            assert!(debug.contains("initialized"));
        }
    }

    // ------------------------------------------------------------------------
    // Non-macOS Initialize Tests
    // ------------------------------------------------------------------------

    #[cfg(not(target_os = "macos"))]
    mod non_macos_tests {
        use super::*;

        #[test]
        fn test_initialize_non_macos() {
            let (_, rx) = unbounded();
            let state = TimerState::new(PomodoroConfig::default());
            let mut manager = TrayIconManager::new(state, rx);

            // Should succeed on non-macOS (no-op)
            let result = manager.initialize();
            assert!(result.is_ok());
            assert!(manager.is_initialized());
        }
    }
}
