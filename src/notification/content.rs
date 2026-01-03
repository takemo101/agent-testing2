//! Notification content construction.
//!
//! This module provides a builder for creating notification content
//! with type-safe fluent API.

use objc2::rc::Retained;
use objc2_foundation::NSString;
use objc2_user_notifications::{UNMutableNotificationContent, UNNotificationSound};

use super::actions::category_ids;

/// Maximum length for task names in notifications.
const MAX_TASK_NAME_LENGTH: usize = 100;

/// Builder for constructing notification content.
///
/// Provides a fluent API for setting notification properties.
pub struct NotificationContentBuilder {
    content: Retained<UNMutableNotificationContent>,
}

impl NotificationContentBuilder {
    /// Creates a new notification content builder.
    #[must_use]
    pub fn new() -> Self {
        let content = unsafe { UNMutableNotificationContent::new() };
        Self { content }
    }

    /// Sets the notification title.
    #[must_use]
    pub fn title(self, title: &str) -> Self {
        let title = NSString::from_str(title);
        unsafe {
            self.content.setTitle(&title);
        }
        self
    }

    /// Sets the notification subtitle.
    #[must_use]
    pub fn subtitle(self, subtitle: &str) -> Self {
        let subtitle = NSString::from_str(subtitle);
        unsafe {
            self.content.setSubtitle(&subtitle);
        }
        self
    }

    /// Sets the notification body text.
    #[must_use]
    pub fn body(self, body: &str) -> Self {
        let body = NSString::from_str(body);
        unsafe {
            self.content.setBody(&body);
        }
        self
    }

    /// Sets the category identifier for action buttons.
    #[must_use]
    pub fn category_identifier(self, category_id: &str) -> Self {
        let category_id = NSString::from_str(category_id);
        unsafe {
            self.content.setCategoryIdentifier(&category_id);
        }
        self
    }

    /// Sets the notification sound.
    #[must_use]
    pub fn sound(self, sound: Retained<UNNotificationSound>) -> Self {
        unsafe {
            self.content.setSound(Some(&sound));
        }
        self
    }

    /// Sets the default system sound.
    #[must_use]
    pub fn default_sound(self) -> Self {
        let sound = unsafe { UNNotificationSound::defaultSound() };
        self.sound(sound)
    }

    /// Builds and returns the notification content.
    #[must_use]
    pub fn build(self) -> Retained<UNMutableNotificationContent> {
        self.content
    }
}

impl Default for NotificationContentBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Validates a task name for use in notifications.
///
/// Returns the sanitized task name or None if invalid.
pub fn validate_task_name(task_name: &str) -> Option<String> {
    // Truncate to max length
    let truncated: String = task_name.chars().take(MAX_TASK_NAME_LENGTH).collect();

    // Remove control characters
    let sanitized: String = truncated.chars().filter(|c| !c.is_control()).collect();

    if sanitized.is_empty() {
        None
    } else {
        Some(sanitized)
    }
}

/// Creates notification content for work session completion.
///
/// # Arguments
/// * `task_name` - Optional task name to display as subtitle
#[must_use]
pub fn create_work_complete_content(
    task_name: Option<&str>,
) -> Retained<UNMutableNotificationContent> {
    let mut builder = NotificationContentBuilder::new()
        .title("🍅 ポモドーロタイマー")
        .body("作業時間が終了しました。休憩してください。")
        .category_identifier(category_ids::WORK_COMPLETE)
        .default_sound();

    if let Some(task) = task_name.and_then(validate_task_name).as_deref() {
        builder = builder.subtitle(task);
    }

    builder.build()
}

/// Creates notification content for break completion.
///
/// # Arguments
/// * `task_name` - Optional task name to display as subtitle
#[must_use]
pub fn create_break_complete_content(
    task_name: Option<&str>,
) -> Retained<UNMutableNotificationContent> {
    let mut builder = NotificationContentBuilder::new()
        .title("☕ ポモドーロタイマー")
        .body("休憩時間が終了しました。作業を再開してください。")
        .category_identifier(category_ids::BREAK_COMPLETE)
        .default_sound();

    if let Some(task) = task_name.and_then(validate_task_name).as_deref() {
        builder = builder.subtitle(task);
    }

    builder.build()
}

/// Creates notification content for long break completion.
///
/// # Arguments
/// * `task_name` - Optional task name to display as subtitle
#[must_use]
pub fn create_long_break_complete_content(
    task_name: Option<&str>,
) -> Retained<UNMutableNotificationContent> {
    let mut builder = NotificationContentBuilder::new()
        .title("☕ ポモドーロタイマー")
        .body("長い休憩時間が終了しました。作業を再開してください。")
        .category_identifier(category_ids::LONG_BREAK_COMPLETE)
        .default_sound();

    if let Some(task) = task_name.and_then(validate_task_name).as_deref() {
        builder = builder.subtitle(task);
    }

    builder.build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_task_name_valid() {
        let result = validate_task_name("API実装");
        assert_eq!(result, Some("API実装".to_string()));
    }

    #[test]
    fn test_validate_task_name_truncates_long() {
        let long_name = "a".repeat(150);
        let result = validate_task_name(&long_name);
        assert!(result.is_some());
        assert_eq!(result.unwrap().len(), MAX_TASK_NAME_LENGTH);
    }

    #[test]
    fn test_validate_task_name_removes_control_chars() {
        let result = validate_task_name("test\n\r\ttask");
        assert_eq!(result, Some("testtask".to_string()));
    }

    #[test]
    fn test_validate_task_name_empty() {
        let result = validate_task_name("");
        assert!(result.is_none());
    }

    #[test]
    fn test_validate_task_name_only_control_chars() {
        let result = validate_task_name("\n\r\t");
        assert!(result.is_none());
    }
}
