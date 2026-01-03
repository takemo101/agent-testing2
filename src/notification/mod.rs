//! macOS notification system integration.
//!
//! This module provides native macOS notification support using
//! `objc2-user-notifications`. It includes:
//!
//! - Notification authorization handling
//! - Action buttons (pause/stop) on notifications
//! - Delegate-based event handling
//! - Async-friendly APIs
//!
//! # Example
//!
//! ```rust,ignore
//! use pomodoro::notification::{NotificationManager, NotificationActionEvent};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Initialize the notification manager
//!     let manager = NotificationManager::new().await?;
//!
//!     // Send a work complete notification
//!     manager.send_work_complete_notification(Some("API実装")).await?;
//!
//!     // Handle action events
//!     while let Some(event) = manager.try_recv_action() {
//!         match event {
//!             NotificationActionEvent::Pause => println!("Pausing timer"),
//!             NotificationActionEvent::Stop => println!("Stopping timer"),
//!             _ => {}
//!         }
//!     }
//!
//!     Ok(())
//! }
//! ```
//!
//! # Requirements
//!
//! - macOS 10.14+
//! - The binary must be code-signed for notifications to work properly
//!
//! # Code Signing
//!
//! For development, use ad-hoc signing:
//! ```bash
//! codesign --force --deep --sign - target/release/pomodoro
//! ```

mod actions;
mod center;
mod content;
mod delegate;
pub mod error;
mod request;

use std::sync::mpsc::{self, Receiver, TryRecvError};

use objc2::rc::Retained;
use objc2::MainThreadMarker;

pub use self::actions::{action_ids, category_ids};
pub use self::content::{
    create_break_complete_content, create_long_break_complete_content,
    create_work_complete_content, validate_task_name, NotificationContentBuilder,
};
pub use self::delegate::{NotificationActionEvent, NotificationDelegate};
pub use self::error::NotificationError;

use self::actions::create_categories;
use self::center::NotificationCenter;
use self::request::create_notification_request;

/// Maximum retry attempts for sending notifications.
const MAX_RETRIES: u32 = 3;

/// Delay between retry attempts in milliseconds.
const RETRY_DELAY_MS: u64 = 1000;

/// Manages the notification system.
///
/// This is the main entry point for sending notifications and receiving
/// action events.
pub struct NotificationManager {
    /// Receiver for action events from the delegate.
    action_receiver: Receiver<NotificationActionEvent>,
    /// Retained delegate to keep it alive.
    _delegate: Retained<NotificationDelegate>,
}

impl NotificationManager {
    /// Creates a new notification manager.
    ///
    /// This will:
    /// 1. Request notification authorization from the user
    /// 2. Set up the notification delegate
    /// 3. Register notification categories (action buttons)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Authorization is denied
    /// - Not running on the main thread
    /// - System notification center is unavailable
    pub async fn new() -> Result<Self, NotificationError> {
        // Verify we're on the main thread
        let mtm = MainThreadMarker::new().ok_or_else(|| {
            NotificationError::InitializationFailed(
                "通知システムはメインスレッドで初期化する必要があります".to_string(),
            )
        })?;

        // Request authorization
        let granted = NotificationCenter::request_authorization().await?;
        if !granted {
            return Err(NotificationError::PermissionDenied);
        }

        // Create action channel
        let (sender, receiver) = mpsc::channel();

        // Create and set delegate
        let delegate = NotificationDelegate::new(mtm, sender);
        NotificationCenter::set_delegate(&NotificationDelegate::as_protocol(&delegate));

        // Register notification categories
        let categories = create_categories();
        NotificationCenter::set_notification_categories(&categories);

        Ok(Self {
            action_receiver: receiver,
            _delegate: delegate,
        })
    }

    /// Creates a notification manager with fallback behavior.
    ///
    /// Returns `None` if initialization fails (with error logged),
    /// allowing the application to continue without notifications.
    pub async fn new_with_fallback() -> Option<Self> {
        match Self::new().await {
            Ok(manager) => Some(manager),
            Err(NotificationError::UnsignedBinary) => {
                tracing::warn!("⚠️  バイナリが署名されていません。通知機能は無効です。");
                tracing::info!(
                    "署名するには: codesign --force --deep --sign - target/release/pomodoro"
                );
                None
            }
            Err(NotificationError::PermissionDenied) => {
                tracing::warn!("⚠️  通知許可が拒否されています。");
                tracing::info!("システム環境設定 > 通知 で許可してください。");
                None
            }
            Err(e) => {
                tracing::warn!("⚠️  通知システムの初期化に失敗しました: {}", e);
                None
            }
        }
    }

    /// Checks if notifications are currently authorized.
    pub async fn is_authorized() -> Result<bool, NotificationError> {
        NotificationCenter::is_authorized().await
    }

    /// Sends a work session complete notification.
    ///
    /// # Arguments
    /// * `task_name` - Optional task name to display in the notification
    pub async fn send_work_complete_notification(
        &self,
        task_name: Option<&str>,
    ) -> Result<(), NotificationError> {
        let content = create_work_complete_content(task_name);
        let request = create_notification_request(&content);
        NotificationCenter::add_notification_request(&request).await
    }

    /// Sends a break complete notification.
    ///
    /// # Arguments
    /// * `task_name` - Optional task name to display in the notification
    pub async fn send_break_complete_notification(
        &self,
        task_name: Option<&str>,
    ) -> Result<(), NotificationError> {
        let content = create_break_complete_content(task_name);
        let request = create_notification_request(&content);
        NotificationCenter::add_notification_request(&request).await
    }

    /// Sends a long break complete notification.
    ///
    /// # Arguments
    /// * `task_name` - Optional task name to display in the notification
    pub async fn send_long_break_complete_notification(
        &self,
        task_name: Option<&str>,
    ) -> Result<(), NotificationError> {
        let content = create_long_break_complete_content(task_name);
        let request = create_notification_request(&content);
        NotificationCenter::add_notification_request(&request).await
    }

    /// Sends a notification with automatic retry on failure.
    ///
    /// # Arguments
    /// * `task_name` - Optional task name
    /// * `notification_type` - Type of notification to send
    pub async fn send_notification_with_retry(
        &self,
        task_name: Option<&str>,
        notification_type: NotificationType,
    ) -> Result<(), NotificationError> {
        let content = match notification_type {
            NotificationType::WorkComplete => create_work_complete_content(task_name),
            NotificationType::BreakComplete => create_break_complete_content(task_name),
            NotificationType::LongBreakComplete => create_long_break_complete_content(task_name),
        };

        let request = create_notification_request(&content);
        let mut retries = 0;

        loop {
            match NotificationCenter::add_notification_request(&request).await {
                Ok(()) => return Ok(()),
                Err(e) if retries < MAX_RETRIES => {
                    retries += 1;
                    tracing::warn!(
                        "通知送信失敗（リトライ {}/{}）: {}",
                        retries,
                        MAX_RETRIES,
                        e
                    );
                    tokio::time::sleep(tokio::time::Duration::from_millis(RETRY_DELAY_MS)).await;
                }
                Err(e) => return Err(e),
            }
        }
    }

    /// Tries to receive an action event without blocking.
    ///
    /// Returns `None` if no event is available.
    #[must_use]
    pub fn try_recv_action(&self) -> Option<NotificationActionEvent> {
        match self.action_receiver.try_recv() {
            Ok(event) => Some(event),
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) => None,
        }
    }

    /// Receives an action event, blocking until one is available.
    ///
    /// # Errors
    ///
    /// Returns an error if the channel is disconnected.
    pub fn recv_action(&self) -> Result<NotificationActionEvent, mpsc::RecvError> {
        self.action_receiver.recv()
    }

    /// Removes all pending and delivered notifications.
    pub fn clear_all_notifications(&self) {
        NotificationCenter::remove_all_pending_notifications();
        NotificationCenter::remove_all_delivered_notifications();
    }
}

/// Types of notifications that can be sent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationType {
    /// Work session completed.
    WorkComplete,
    /// Short break completed.
    BreakComplete,
    /// Long break completed.
    LongBreakComplete,
}

#[allow(async_fn_in_trait)]
pub trait NotificationSender {
    async fn send_work_complete(&self, task_name: Option<&str>) -> Result<(), NotificationError>;
    async fn send_break_complete(&self, task_name: Option<&str>) -> Result<(), NotificationError>;
    async fn send_long_break_complete(
        &self,
        task_name: Option<&str>,
    ) -> Result<(), NotificationError>;
    fn try_recv_action(&self) -> Option<NotificationActionEvent>;
    fn is_available(&self) -> bool;
    fn clear_all(&self);
}

// Implement NotificationSender for NotificationManager
#[cfg(target_os = "macos")]
impl NotificationSender for NotificationManager {
    async fn send_work_complete(&self, task_name: Option<&str>) -> Result<(), NotificationError> {
        self.send_work_complete_notification(task_name).await
    }

    async fn send_break_complete(&self, task_name: Option<&str>) -> Result<(), NotificationError> {
        self.send_break_complete_notification(task_name).await
    }

    async fn send_long_break_complete(
        &self,
        task_name: Option<&str>,
    ) -> Result<(), NotificationError> {
        self.send_long_break_complete_notification(task_name).await
    }

    fn try_recv_action(&self) -> Option<NotificationActionEvent> {
        NotificationManager::try_recv_action(self)
    }

    fn is_available(&self) -> bool {
        true
    }

    fn clear_all(&self) {
        self.clear_all_notifications()
    }
}

#[derive(Debug, Default)]
pub struct MockNotificationSender {
    notifications: std::sync::Mutex<Vec<(NotificationType, Option<String>)>>,
    action_events: std::sync::Mutex<Vec<NotificationActionEvent>>,
    available: std::sync::atomic::AtomicBool,
    should_fail: std::sync::atomic::AtomicBool,
}

impl MockNotificationSender {
    #[must_use]
    pub fn new() -> Self {
        Self {
            notifications: std::sync::Mutex::new(Vec::new()),
            action_events: std::sync::Mutex::new(Vec::new()),
            available: std::sync::atomic::AtomicBool::new(true),
            should_fail: std::sync::atomic::AtomicBool::new(false),
        }
    }

    pub fn set_available(&self, available: bool) {
        self.available
            .store(available, std::sync::atomic::Ordering::SeqCst);
    }

    pub fn set_should_fail(&self, should_fail: bool) {
        self.should_fail
            .store(should_fail, std::sync::atomic::Ordering::SeqCst);
    }

    pub fn inject_action_event(&self, event: NotificationActionEvent) {
        self.action_events.lock().unwrap().push(event);
    }

    #[must_use]
    pub fn get_notifications(&self) -> Vec<(NotificationType, Option<String>)> {
        self.notifications.lock().unwrap().clone()
    }

    #[must_use]
    pub fn notification_count(&self) -> usize {
        self.notifications.lock().unwrap().len()
    }

    pub fn clear_recorded(&self) {
        self.notifications.lock().unwrap().clear();
    }
}

impl NotificationSender for MockNotificationSender {
    async fn send_work_complete(&self, task_name: Option<&str>) -> Result<(), NotificationError> {
        if self.should_fail.load(std::sync::atomic::Ordering::SeqCst) {
            return Err(NotificationError::SendFailed("Mock failure".to_string()));
        }
        self.notifications
            .lock()
            .unwrap()
            .push((NotificationType::WorkComplete, task_name.map(String::from)));
        Ok(())
    }

    async fn send_break_complete(&self, task_name: Option<&str>) -> Result<(), NotificationError> {
        if self.should_fail.load(std::sync::atomic::Ordering::SeqCst) {
            return Err(NotificationError::SendFailed("Mock failure".to_string()));
        }
        self.notifications
            .lock()
            .unwrap()
            .push((NotificationType::BreakComplete, task_name.map(String::from)));
        Ok(())
    }

    async fn send_long_break_complete(
        &self,
        task_name: Option<&str>,
    ) -> Result<(), NotificationError> {
        if self.should_fail.load(std::sync::atomic::Ordering::SeqCst) {
            return Err(NotificationError::SendFailed("Mock failure".to_string()));
        }
        self.notifications.lock().unwrap().push((
            NotificationType::LongBreakComplete,
            task_name.map(String::from),
        ));
        Ok(())
    }

    fn try_recv_action(&self) -> Option<NotificationActionEvent> {
        let mut events = self.action_events.lock().unwrap();
        if events.is_empty() {
            None
        } else {
            Some(events.remove(0))
        }
    }

    fn is_available(&self) -> bool {
        self.available.load(std::sync::atomic::Ordering::SeqCst)
    }

    fn clear_all(&self) {
        // No-op for mock
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notification_type() {
        let nt = NotificationType::WorkComplete;
        assert_eq!(nt, NotificationType::WorkComplete);
    }

    #[tokio::test]
    async fn test_mock_notification_sender_basic() {
        let mock = MockNotificationSender::new();

        // Send notifications
        mock.send_work_complete(Some("Test Task")).await.unwrap();
        mock.send_break_complete(None).await.unwrap();

        // Verify
        let notifications = mock.get_notifications();
        assert_eq!(notifications.len(), 2);
        assert_eq!(
            notifications[0],
            (
                NotificationType::WorkComplete,
                Some("Test Task".to_string())
            )
        );
        assert_eq!(notifications[1], (NotificationType::BreakComplete, None));
    }

    #[tokio::test]
    async fn test_mock_notification_sender_failure() {
        let mock = MockNotificationSender::new();
        mock.set_should_fail(true);

        let result = mock.send_work_complete(None).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_mock_notification_sender_action_events() {
        let mock = MockNotificationSender::new();

        // Initially empty
        assert!(mock.try_recv_action().is_none());

        // Inject events
        mock.inject_action_event(NotificationActionEvent::Pause);
        mock.inject_action_event(NotificationActionEvent::Stop);

        // Receive in order
        assert_eq!(mock.try_recv_action(), Some(NotificationActionEvent::Pause));
        assert_eq!(mock.try_recv_action(), Some(NotificationActionEvent::Stop));
        assert!(mock.try_recv_action().is_none());
    }

    #[test]
    fn test_mock_notification_sender_availability() {
        let mock = MockNotificationSender::new();
        assert!(mock.is_available());

        mock.set_available(false);
        assert!(!mock.is_available());
    }
}
