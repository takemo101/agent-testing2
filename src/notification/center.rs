//! UNUserNotificationCenter wrapper.
//!
//! This module provides a Rust-friendly wrapper around the macOS
//! UNUserNotificationCenter API.

use std::cell::RefCell;
use std::ptr::NonNull;

use block2::RcBlock;
use objc2::rc::Retained;
use objc2::runtime::{Bool, ProtocolObject};
use objc2_foundation::{NSError, NSSet};
use objc2_user_notifications::{
    UNAuthorizationOptions, UNAuthorizationStatus, UNNotificationCategory, UNNotificationRequest,
    UNNotificationSettings, UNUserNotificationCenter, UNUserNotificationCenterDelegate,
};
use tokio::sync::oneshot;

use super::error::NotificationError;

/// Wrapper around `UNUserNotificationCenter` providing async Rust APIs.
pub struct NotificationCenter;

impl NotificationCenter {
    /// Gets the current notification center singleton.
    #[must_use]
    pub fn current() -> Retained<UNUserNotificationCenter> {
        unsafe { UNUserNotificationCenter::currentNotificationCenter() }
    }

    /// Requests notification authorization from the user.
    ///
    /// This will show a system dialog asking the user to allow notifications.
    ///
    /// # Returns
    /// `Ok(true)` if authorization was granted, `Ok(false)` if denied,
    /// or an error if the request failed.
    pub async fn request_authorization() -> Result<bool, NotificationError> {
        let (tx, rx) = oneshot::channel::<Result<bool, NotificationError>>();

        let options = UNAuthorizationOptions::Alert
            | UNAuthorizationOptions::Sound
            | UNAuthorizationOptions::Badge;

        let cb = RefCell::new(Some(tx));
        let block = RcBlock::new(move |granted: Bool, error: *mut NSError| {
            if let Some(sender) = cb.borrow_mut().take() {
                let result = if !error.is_null() {
                    let err_ref = unsafe { error.as_ref() }.unwrap();
                    let description = err_ref.localizedDescription();
                    Err(NotificationError::AuthorizationFailed(
                        description.to_string(),
                    ))
                } else {
                    Ok(granted.as_bool())
                };
                let _ = sender.send(result);
            }
        });

        unsafe {
            Self::current().requestAuthorizationWithOptions_completionHandler(options, &block);
        }

        rx.await
            .map_err(|_| NotificationError::InitializationFailed("Channel closed".to_string()))?
    }

    /// Gets the current authorization status.
    pub async fn get_authorization_status() -> Result<UNAuthorizationStatus, NotificationError> {
        let (tx, rx) = oneshot::channel::<UNAuthorizationStatus>();

        let cb = RefCell::new(Some(tx));
        let block = RcBlock::new(move |settings: NonNull<UNNotificationSettings>| {
            if let Some(sender) = cb.borrow_mut().take() {
                let status = unsafe { settings.as_ref().authorizationStatus() };
                let _ = sender.send(status);
            }
        });

        unsafe {
            Self::current().getNotificationSettingsWithCompletionHandler(&block);
        }

        rx.await
            .map_err(|_| NotificationError::InitializationFailed("Channel closed".to_string()))
    }

    /// Checks if notifications are authorized.
    pub async fn is_authorized() -> Result<bool, NotificationError> {
        let status = Self::get_authorization_status().await?;
        Ok(matches!(
            status,
            UNAuthorizationStatus::Authorized
                | UNAuthorizationStatus::Provisional
                | UNAuthorizationStatus::Ephemeral
        ))
    }

    /// Sets the notification categories (action buttons).
    ///
    /// # Arguments
    /// * `categories` - The categories to register
    pub fn set_notification_categories(categories: &[Retained<UNNotificationCategory>]) {
        let refs: Vec<&UNNotificationCategory> = categories.iter().map(|c| c.as_ref()).collect();
        let categories_set: Retained<NSSet<UNNotificationCategory>> = NSSet::from_slice(&refs);

        unsafe {
            Self::current().setNotificationCategories(&categories_set);
        }
    }

    /// Sets the notification center delegate.
    ///
    /// # Arguments
    /// * `delegate` - The delegate to handle notification events
    pub fn set_delegate(delegate: &ProtocolObject<dyn UNUserNotificationCenterDelegate>) {
        unsafe {
            Self::current().setDelegate(Some(delegate));
        }
    }

    /// Adds a notification request to the notification center.
    ///
    /// # Arguments
    /// * `request` - The notification request to add
    ///
    /// # Returns
    /// `Ok(())` if the notification was added successfully, or an error.
    pub async fn add_notification_request(
        request: &UNNotificationRequest,
    ) -> Result<(), NotificationError> {
        let (tx, rx) = oneshot::channel::<Result<(), NotificationError>>();

        let cb = RefCell::new(Some(tx));
        let block = RcBlock::new(move |error: *mut NSError| {
            if let Some(sender) = cb.borrow_mut().take() {
                let result = if !error.is_null() {
                    let err_ref = unsafe { error.as_ref() }.unwrap();
                    let description = err_ref.localizedDescription();
                    Err(NotificationError::SendFailed(description.to_string()))
                } else {
                    Ok(())
                };
                let _ = sender.send(result);
            }
        });

        unsafe {
            Self::current().addNotificationRequest_withCompletionHandler(request, Some(&block));
        }

        rx.await
            .map_err(|_| NotificationError::SendFailed("Channel closed".to_string()))?
    }

    /// Removes all pending notification requests.
    pub fn remove_all_pending_notifications() {
        unsafe {
            Self::current().removeAllPendingNotificationRequests();
        }
    }

    /// Removes all delivered notifications.
    pub fn remove_all_delivered_notifications() {
        unsafe {
            Self::current().removeAllDeliveredNotifications();
        }
    }
}

#[cfg(test)]
mod tests {
    // Note: NotificationCenter tests can only run on macOS
    // since they require the objc2 runtime and user interaction.
}
