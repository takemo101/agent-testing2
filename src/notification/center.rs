//! UNUserNotificationCenter wrapper.

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

pub struct NotificationCenter;

impl NotificationCenter {
    #[must_use]
    pub fn current() -> Retained<UNUserNotificationCenter> {
        UNUserNotificationCenter::currentNotificationCenter()
    }

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

        Self::current().requestAuthorizationWithOptions_completionHandler(options, &block);

        rx.await
            .map_err(|_| NotificationError::InitializationFailed("Channel closed".to_string()))?
    }

    pub async fn get_authorization_status() -> Result<UNAuthorizationStatus, NotificationError> {
        let (tx, rx) = oneshot::channel::<UNAuthorizationStatus>();

        let cb = RefCell::new(Some(tx));
        let block = RcBlock::new(move |settings: NonNull<UNNotificationSettings>| {
            if let Some(sender) = cb.borrow_mut().take() {
                let status = unsafe { settings.as_ref().authorizationStatus() };
                let _ = sender.send(status);
            }
        });

        Self::current().getNotificationSettingsWithCompletionHandler(&block);

        rx.await
            .map_err(|_| NotificationError::InitializationFailed("Channel closed".to_string()))
    }

    pub async fn is_authorized() -> Result<bool, NotificationError> {
        let status = Self::get_authorization_status().await?;
        Ok(matches!(
            status,
            UNAuthorizationStatus::Authorized
                | UNAuthorizationStatus::Provisional
                | UNAuthorizationStatus::Ephemeral
        ))
    }

    pub fn set_notification_categories(categories: &[Retained<UNNotificationCategory>]) {
        let refs: Vec<&UNNotificationCategory> = categories.iter().map(|c| c.as_ref()).collect();
        let categories_set: Retained<NSSet<UNNotificationCategory>> = NSSet::from_slice(&refs);

        Self::current().setNotificationCategories(&categories_set);
    }

    pub fn set_delegate(delegate: &ProtocolObject<dyn UNUserNotificationCenterDelegate>) {
        Self::current().setDelegate(Some(delegate));
    }

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

        Self::current().addNotificationRequest_withCompletionHandler(request, Some(&block));

        rx.await
            .map_err(|_| NotificationError::SendFailed("Channel closed".to_string()))?
    }

    pub fn remove_all_pending_notifications() {
        Self::current().removeAllPendingNotificationRequests();
    }

    pub fn remove_all_delivered_notifications() {
        Self::current().removeAllDeliveredNotifications();
    }
}

#[cfg(test)]
mod tests {}
