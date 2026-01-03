//! Notification action and category definitions.
//!
//! This module defines the notification actions (buttons) and categories
//! that are used by the Pomodoro timer notifications.

use objc2::rc::Retained;
use objc2_foundation::{NSArray, NSString};
use objc2_user_notifications::{
    UNNotificationAction, UNNotificationActionOptions, UNNotificationCategory,
    UNNotificationCategoryOptions,
};

/// Notification action identifiers.
pub mod action_ids {
    /// Action ID for pausing the timer.
    pub const PAUSE: &str = "PAUSE_ACTION";
    /// Action ID for stopping the timer.
    pub const STOP: &str = "STOP_ACTION";
}

/// Notification category identifiers.
pub mod category_ids {
    /// Category for work session complete notifications.
    pub const WORK_COMPLETE: &str = "WORK_COMPLETE";
    /// Category for break complete notifications.
    pub const BREAK_COMPLETE: &str = "BREAK_COMPLETE";
    /// Category for long break complete notifications.
    pub const LONG_BREAK_COMPLETE: &str = "LONG_BREAK_COMPLETE";
}

/// Creates the pause action for notifications.
///
/// This action allows users to pause the timer directly from the notification.
#[must_use]
pub fn create_pause_action() -> Retained<UNNotificationAction> {
    let identifier = NSString::from_str(action_ids::PAUSE);
    let title = NSString::from_str("一時停止");

    unsafe {
        UNNotificationAction::actionWithIdentifier_title_options(
            &identifier,
            &title,
            UNNotificationActionOptions::Foreground,
        )
    }
}

/// Creates the stop action for notifications.
///
/// This action allows users to stop the timer directly from the notification.
/// Marked as destructive to indicate it ends the current session.
#[must_use]
pub fn create_stop_action() -> Retained<UNNotificationAction> {
    let identifier = NSString::from_str(action_ids::STOP);
    let title = NSString::from_str("停止");

    unsafe {
        UNNotificationAction::actionWithIdentifier_title_options(
            &identifier,
            &title,
            UNNotificationActionOptions::Destructive,
        )
    }
}

/// Creates all notification actions.
///
/// Returns a vector of all actions that should be available on notifications.
#[must_use]
pub fn create_actions() -> Vec<Retained<UNNotificationAction>> {
    vec![create_pause_action(), create_stop_action()]
}

/// Creates a notification category with the given identifier and actions.
fn create_category(
    identifier: &str,
    actions: &[Retained<UNNotificationAction>],
) -> Retained<UNNotificationCategory> {
    let identifier = NSString::from_str(identifier);

    // Convert Vec<Retained<UNNotificationAction>> to NSArray
    let actions_array: Retained<NSArray<UNNotificationAction>> = unsafe {
        let refs: Vec<&UNNotificationAction> = actions.iter().map(|a| a.as_ref()).collect();
        NSArray::from_slice(&refs)
    };

    let intent_identifiers: Retained<NSArray<NSString>> =
        unsafe { NSArray::from_slice(&[] as &[&NSString]) };

    unsafe {
        UNNotificationCategory::categoryWithIdentifier_actions_intentIdentifiers_options(
            &identifier,
            &actions_array,
            &intent_identifiers,
            UNNotificationCategoryOptions::empty(),
        )
    }
}

/// Creates all notification categories.
///
/// Each category corresponds to a different notification type
/// (work complete, break complete, long break complete).
#[must_use]
pub fn create_categories() -> Vec<Retained<UNNotificationCategory>> {
    let actions = create_actions();

    vec![
        create_category(category_ids::WORK_COMPLETE, &actions),
        create_category(category_ids::BREAK_COMPLETE, &actions),
        create_category(category_ids::LONG_BREAK_COMPLETE, &actions),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_action_ids() {
        assert_eq!(action_ids::PAUSE, "PAUSE_ACTION");
        assert_eq!(action_ids::STOP, "STOP_ACTION");
    }

    #[test]
    fn test_category_ids() {
        assert_eq!(category_ids::WORK_COMPLETE, "WORK_COMPLETE");
        assert_eq!(category_ids::BREAK_COMPLETE, "BREAK_COMPLETE");
        assert_eq!(category_ids::LONG_BREAK_COMPLETE, "LONG_BREAK_COMPLETE");
    }

    // Note: Tests that create actual UNNotificationAction/UNNotificationCategory
    // objects can only run on macOS. They are tested in integration tests.
}
