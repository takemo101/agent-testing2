//! Notification system error types.
//!
//! This module defines the error types for the notification system.
//! All errors are designed to provide helpful messages for debugging
//! and user-facing error handling.

use thiserror::Error;

/// Errors that can occur in the notification system.
#[derive(Debug, Error)]
pub enum NotificationError {
    /// Failed to request notification authorization from the system.
    #[error("通知許可の取得に失敗しました: {0}")]
    AuthorizationFailed(String),

    /// Failed to send a notification.
    #[error("通知の送信に失敗しました: {0}")]
    SendFailed(String),

    /// Notification permission was denied by the user.
    #[error("通知許可が拒否されています")]
    PermissionDenied,

    /// The binary is not code-signed (required for notifications on macOS).
    #[error("バイナリが署名されていません。codesignで署名してください")]
    UnsignedBinary,

    /// Failed to initialize the notification system.
    #[error("通知システムの初期化に失敗しました: {0}")]
    InitializationFailed(String),

    /// Invalid input provided to the notification system.
    #[error("無効な入力: {0}")]
    InvalidInput(String),

    /// The notification center is not available.
    #[error("通知センターが利用できません")]
    NotAvailable,
}

impl NotificationError {
    /// Returns true if this error is related to permissions.
    #[must_use]
    pub fn is_permission_error(&self) -> bool {
        matches!(
            self,
            Self::PermissionDenied | Self::AuthorizationFailed(_)
        )
    }

    /// Returns true if this error might be resolved by code signing.
    #[must_use]
    pub fn requires_code_signing(&self) -> bool {
        matches!(self, Self::UnsignedBinary)
    }

    /// Returns a user-friendly suggestion for resolving this error.
    #[must_use]
    pub fn suggestion(&self) -> &'static str {
        match self {
            Self::AuthorizationFailed(_) => {
                "システム環境設定 > 通知 でアプリの通知を許可してください"
            }
            Self::PermissionDenied => {
                "システム環境設定 > 通知 でアプリの通知を許可してください"
            }
            Self::UnsignedBinary => {
                "codesign --force --deep --sign - target/release/pomodoro"
            }
            Self::SendFailed(_) => "通知センターを確認してください",
            Self::InitializationFailed(_) => "アプリケーションを再起動してください",
            Self::InvalidInput(_) => "入力値を確認してください",
            Self::NotAvailable => "macOSで実行してください",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = NotificationError::PermissionDenied;
        assert_eq!(err.to_string(), "通知許可が拒否されています");

        let err = NotificationError::AuthorizationFailed("test".to_string());
        assert!(err.to_string().contains("test"));
    }

    #[test]
    fn test_is_permission_error() {
        assert!(NotificationError::PermissionDenied.is_permission_error());
        assert!(NotificationError::AuthorizationFailed("x".into()).is_permission_error());
        assert!(!NotificationError::UnsignedBinary.is_permission_error());
    }

    #[test]
    fn test_requires_code_signing() {
        assert!(NotificationError::UnsignedBinary.requires_code_signing());
        assert!(!NotificationError::PermissionDenied.requires_code_signing());
    }

    #[test]
    fn test_suggestion() {
        let err = NotificationError::UnsignedBinary;
        assert!(err.suggestion().contains("codesign"));
    }
}
