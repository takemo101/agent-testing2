//! Notification action and category definitions.

use objc2::rc::Retained;
use objc2_foundation::{NSArray, NSString};
use objc2_user_notifications::{
    UNNotificationAction, UNNotificationActionOptions, UNNotificationCategory,
    UNNotificationCategoryOptions,
};

pub mod action_ids {
    pub const PAUSE: &str = "PAUSE_ACTION";
    pub const STOP: &str = "STOP_ACTION";
}

pub mod category_ids {
    pub const WORK_COMPLETE: &str = "WORK_COMPLETE";
    pub const BREAK_COMPLETE: &str = "BREAK_COMPLETE";
    pub const LONG_BREAK_COMPLETE: &str = "LONG_BREAK_COMPLETE";
}

#[must_use]
pub fn create_pause_action() -> Retained<UNNotificationAction> {
    let identifier = NSString::from_str(action_ids::PAUSE);
    let title = NSString::from_str("一時停止");

    UNNotificationAction::actionWithIdentifier_title_options(
        &identifier,
        &title,
        UNNotificationActionOptions::Foreground,
    )
}

#[must_use]
pub fn create_stop_action() -> Retained<UNNotificationAction> {
    let identifier = NSString::from_str(action_ids::STOP);
    let title = NSString::from_str("停止");

    UNNotificationAction::actionWithIdentifier_title_options(
        &identifier,
        &title,
        UNNotificationActionOptions::Destructive,
    )
}

#[must_use]
pub fn create_actions() -> Vec<Retained<UNNotificationAction>> {
    vec![create_pause_action(), create_stop_action()]
}

fn create_category(
    identifier: &str,
    actions: &[Retained<UNNotificationAction>],
) -> Retained<UNNotificationCategory> {
    let identifier = NSString::from_str(identifier);

    let refs: Vec<&UNNotificationAction> = actions.iter().map(|a| a.as_ref()).collect();
    let actions_array: Retained<NSArray<UNNotificationAction>> = NSArray::from_slice(&refs);

    let intent_identifiers: Retained<NSArray<NSString>> = NSArray::from_slice(&[] as &[&NSString]);

    UNNotificationCategory::categoryWithIdentifier_actions_intentIdentifiers_options(
        &identifier,
        &actions_array,
        &intent_identifiers,
        UNNotificationCategoryOptions::empty(),
    )
}

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
}
