//! Notification request creation.
//!
//! This module provides utilities for creating notification requests
//! that can be sent to the notification center.

use objc2::rc::Retained;
use objc2_foundation::NSString;
use objc2_user_notifications::{UNMutableNotificationContent, UNNotificationRequest};
use uuid::Uuid;

/// Creates a notification request with a unique identifier.
///
/// The request is configured for immediate delivery (no trigger).
///
/// # Arguments
/// * `content` - The notification content to display
///
/// # Returns
/// A notification request ready to be added to the notification center.
#[must_use]
pub fn create_notification_request(
    content: &UNMutableNotificationContent,
) -> Retained<UNNotificationRequest> {
    let identifier = NSString::from_str(&Uuid::new_v4().to_string());

    unsafe {
        UNNotificationRequest::requestWithIdentifier_content_trigger(
            &identifier,
            content,
            None, // No trigger = immediate delivery
        )
    }
}

/// Creates a notification request with a specific identifier.
///
/// This is useful when you want to update or remove a specific notification.
///
/// # Arguments
/// * `identifier` - The unique identifier for this notification
/// * `content` - The notification content to display
///
/// # Returns
/// A notification request ready to be added to the notification center.
#[must_use]
pub fn create_notification_request_with_id(
    identifier: &str,
    content: &UNMutableNotificationContent,
) -> Retained<UNNotificationRequest> {
    let identifier = NSString::from_str(identifier);

    unsafe {
        UNNotificationRequest::requestWithIdentifier_content_trigger(&identifier, content, None)
    }
}

#[cfg(test)]
mod tests {
    // Note: Request creation tests can only run on macOS
    // since they require the objc2 runtime.
}
