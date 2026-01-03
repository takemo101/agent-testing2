//! Notification delegate implementation.
//!
//! This module implements the `UNUserNotificationCenterDelegate` protocol
//! to handle notification events such as action button clicks.

use std::sync::mpsc::Sender;

use block2::Block;
use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2::{define_class, msg_send, DefinedClass, MainThreadMarker, MainThreadOnly};
use objc2_foundation::{NSObject, NSObjectProtocol};
use objc2_user_notifications::{
    UNNotification, UNNotificationPresentationOptions, UNNotificationResponse,
    UNUserNotificationCenter, UNUserNotificationCenterDelegate,
};

use super::actions::action_ids;

/// Events triggered by notification actions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotificationActionEvent {
    /// User clicked the pause action button.
    Pause,
    /// User clicked the stop action button.
    Stop,
    /// User clicked the notification itself (default action).
    Default,
    /// User dismissed the notification.
    Dismiss,
}

/// Instance variables for the notification delegate.
#[derive(Clone)]
pub struct NotificationDelegateIvars {
    /// Channel sender for notification action events.
    pub action_sender: Option<Sender<NotificationActionEvent>>,
}

define_class!(
    /// Delegate that handles notification events.
    ///
    /// This class implements `UNUserNotificationCenterDelegate` to receive
    /// callbacks when the user interacts with notifications.
    // SAFETY:
    // - The superclass NSObject does not have any subclassing requirements.
    // - `NotificationDelegate` does not implement `Drop`.
    #[unsafe(super(NSObject))]
    #[ivars = NotificationDelegateIvars]
    #[name = "PomodoroNotificationDelegate"]
    #[thread_kind = MainThreadOnly]
    pub struct NotificationDelegate;

    impl NotificationDelegate {}

    unsafe impl NSObjectProtocol for NotificationDelegate {}

    unsafe impl UNUserNotificationCenterDelegate for NotificationDelegate {
        /// Called when a notification is about to be presented while the app is in foreground.
        #[unsafe(method(userNotificationCenter:willPresentNotification:withCompletionHandler:))]
        fn will_present_notification(
            &self,
            _center: &UNUserNotificationCenter,
            _notification: &UNNotification,
            completion_handler: &Block<dyn Fn(UNNotificationPresentationOptions)>,
        ) {
            // Show notification even when app is in foreground
            let options = UNNotificationPresentationOptions::Banner
                | UNNotificationPresentationOptions::Sound
                | UNNotificationPresentationOptions::Badge;

            completion_handler.call((options,));
        }

        /// Called when the user interacts with a notification.
        #[unsafe(method(userNotificationCenter:didReceiveNotificationResponse:withCompletionHandler:))]
        fn did_receive_notification_response(
            &self,
            _center: &UNUserNotificationCenter,
            response: &UNNotificationResponse,
            completion_handler: &Block<dyn Fn()>,
        ) {
            let action_identifier = response.actionIdentifier();
            let action_str = action_identifier.to_string();

            // Map action identifier to event
            let event = match action_str.as_str() {
                id if id == action_ids::PAUSE => Some(NotificationActionEvent::Pause),
                id if id == action_ids::STOP => Some(NotificationActionEvent::Stop),
                "com.apple.UNNotificationDefaultActionIdentifier" => {
                    Some(NotificationActionEvent::Default)
                }
                "com.apple.UNNotificationDismissActionIdentifier" => {
                    Some(NotificationActionEvent::Dismiss)
                }
                _ => None,
            };

            // Send event through channel if we have a sender
            if let Some(event) = event {
                if let Some(sender) = &self.ivars().action_sender {
                    let _ = sender.send(event);
                }
            }

            // Must call completion handler
            completion_handler.call(());
        }
    }
);

impl NotificationDelegate {
    /// Creates a new notification delegate.
    ///
    /// # Arguments
    /// * `mtm` - Main thread marker to ensure we're on the main thread
    /// * `action_sender` - Channel sender for action events
    ///
    /// # Returns
    /// A retained pointer to the new delegate.
    #[must_use]
    pub fn new(
        mtm: MainThreadMarker,
        action_sender: Sender<NotificationActionEvent>,
    ) -> Retained<Self> {
        let ivars = NotificationDelegateIvars {
            action_sender: Some(action_sender),
        };
        let this = Self::alloc(mtm).set_ivars(ivars);
        unsafe { msg_send![super(this), init] }
    }

    /// Creates a notification delegate without an action sender.
    ///
    /// This is useful for receiving notifications without handling actions.
    #[must_use]
    pub fn new_without_sender(mtm: MainThreadMarker) -> Retained<Self> {
        let ivars = NotificationDelegateIvars {
            action_sender: None,
        };
        let this = Self::alloc(mtm).set_ivars(ivars);
        unsafe { msg_send![super(this), init] }
    }

    /// Converts a retained delegate to a protocol object.
    #[must_use]
    pub fn as_protocol(
        delegate: &Retained<Self>,
    ) -> Retained<ProtocolObject<dyn UNUserNotificationCenterDelegate>> {
        ProtocolObject::from_retained(delegate.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notification_action_event_equality() {
        assert_eq!(
            NotificationActionEvent::Pause,
            NotificationActionEvent::Pause
        );
        assert_ne!(
            NotificationActionEvent::Pause,
            NotificationActionEvent::Stop
        );
    }

    #[test]
    fn test_notification_action_event_debug() {
        let event = NotificationActionEvent::Pause;
        assert_eq!(format!("{:?}", event), "Pause");
    }
}
