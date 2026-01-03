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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notification_type() {
        let nt = NotificationType::WorkComplete;
        assert_eq!(nt, NotificationType::WorkComplete);
    }
}
