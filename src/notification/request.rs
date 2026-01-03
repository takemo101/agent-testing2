//! Notification request creation.

use objc2::rc::Retained;
use objc2_foundation::NSString;
use objc2_user_notifications::{UNMutableNotificationContent, UNNotificationRequest};
use uuid::Uuid;

#[must_use]
pub fn create_notification_request(
    content: &UNMutableNotificationContent,
) -> Retained<UNNotificationRequest> {
    let identifier = NSString::from_str(&Uuid::new_v4().to_string());

    UNNotificationRequest::requestWithIdentifier_content_trigger(&identifier, content, None)
}

#[must_use]
pub fn create_notification_request_with_id(
    identifier: &str,
    content: &UNMutableNotificationContent,
) -> Retained<UNNotificationRequest> {
    let identifier = NSString::from_str(identifier);

    UNNotificationRequest::requestWithIdentifier_content_trigger(&identifier, content, None)
}

#[cfg(test)]
mod tests {}
