//! Focus mode error types.
//!
//! This module defines the error types for the Shortcuts.app-based
//! focus mode integration. All errors are designed to provide helpful
//! messages and allow graceful degradation when focus mode is unavailable.

use thiserror::Error;

/// Errors that can occur in the focus mode integration.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum FocusModeError {
    /// Shortcuts.app is not available (macOS 12+ required).
    #[error("Shortcuts.appが見つかりません。macOS 12以降が必要です。")]
    ShortcutsNotFound,

    /// The specified shortcut was not found in Shortcuts.app.
    #[error("ショートカット '{0}' が見つかりません。Shortcuts.appで作成してください。")]
    ShortcutNotFound(String),

    /// Shortcut execution timed out.
    #[error("ショートカット '{0}' の実行がタイムアウトしました（{1}秒）")]
    ExecutionTimeout(String, u64),

    /// Shortcut execution failed with an error.
    #[error("ショートカット '{0}' の実行に失敗しました: {1}")]
    ExecutionFailed(String, String),

    /// Generic focus mode error.
    #[error("フォーカスモード連携エラー: {0}")]
    Other(String),
}

impl FocusModeError {
    /// Returns true if this error is due to missing Shortcuts.app.
    #[must_use]
    pub fn is_shortcuts_not_found(&self) -> bool {
        matches!(self, Self::ShortcutsNotFound)
    }

    /// Returns true if this error is due to a missing shortcut.
    #[must_use]
    pub fn is_shortcut_not_found(&self) -> bool {
        matches!(self, Self::ShortcutNotFound(_))
    }

    /// Returns true if this error is a timeout.
    #[must_use]
    pub fn is_timeout(&self) -> bool {
        matches!(self, Self::ExecutionTimeout(_, _))
    }

    /// Returns true if the error is recoverable and timer should continue.
    ///
    /// All focus mode errors are recoverable - the timer should always
    /// continue even if focus mode fails.
    #[must_use]
    pub fn is_recoverable(&self) -> bool {
        true
    }

    /// Returns a user-friendly suggestion for resolving this error.
    #[must_use]
    pub fn suggestion(&self) -> &'static str {
        match self {
            Self::ShortcutsNotFound => {
                "macOS 12 (Monterey) 以降にアップグレードしてください"
            }
            Self::ShortcutNotFound(_) => {
                "Shortcuts.appで 'Enable Work Focus' と 'Disable Work Focus' ショートカットを作成してください"
            }
            Self::ExecutionTimeout(_, _) => {
                "ショートカットの処理を簡素化するか、タイムアウト時間を延長してください"
            }
            Self::ExecutionFailed(_, _) => {
                "Shortcuts.appを開いてショートカットが正しく動作するか確認してください"
            }
            Self::Other(_) => {
                "設定を確認し、アプリケーションを再起動してください"
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display_shortcuts_not_found() {
        let err = FocusModeError::ShortcutsNotFound;
        assert!(err.to_string().contains("Shortcuts.app"));
        assert!(err.to_string().contains("macOS 12"));
    }

    #[test]
    fn test_error_display_shortcut_not_found() {
        let err = FocusModeError::ShortcutNotFound("Enable Work Focus".to_string());
        assert!(err.to_string().contains("Enable Work Focus"));
        assert!(err.to_string().contains("Shortcuts.appで作成"));
    }

    #[test]
    fn test_error_display_execution_timeout() {
        let err = FocusModeError::ExecutionTimeout("Enable Work Focus".to_string(), 5);
        assert!(err.to_string().contains("Enable Work Focus"));
        assert!(err.to_string().contains("タイムアウト"));
        assert!(err.to_string().contains("5秒"));
    }

    #[test]
    fn test_error_display_execution_failed() {
        let err = FocusModeError::ExecutionFailed(
            "Enable Work Focus".to_string(),
            "unknown error".to_string(),
        );
        assert!(err.to_string().contains("Enable Work Focus"));
        assert!(err.to_string().contains("unknown error"));
    }

    #[test]
    fn test_error_display_other() {
        let err = FocusModeError::Other("unexpected condition".to_string());
        assert!(err.to_string().contains("unexpected condition"));
    }

    #[test]
    fn test_is_shortcuts_not_found() {
        assert!(FocusModeError::ShortcutsNotFound.is_shortcuts_not_found());
        assert!(!FocusModeError::ShortcutNotFound("x".into()).is_shortcuts_not_found());
        assert!(!FocusModeError::ExecutionTimeout("x".into(), 5).is_shortcuts_not_found());
        assert!(!FocusModeError::ExecutionFailed("x".into(), "y".into()).is_shortcuts_not_found());
        assert!(!FocusModeError::Other("x".into()).is_shortcuts_not_found());
    }

    #[test]
    fn test_is_shortcut_not_found() {
        assert!(!FocusModeError::ShortcutsNotFound.is_shortcut_not_found());
        assert!(FocusModeError::ShortcutNotFound("x".into()).is_shortcut_not_found());
        assert!(!FocusModeError::ExecutionTimeout("x".into(), 5).is_shortcut_not_found());
        assert!(!FocusModeError::ExecutionFailed("x".into(), "y".into()).is_shortcut_not_found());
        assert!(!FocusModeError::Other("x".into()).is_shortcut_not_found());
    }

    #[test]
    fn test_is_timeout() {
        assert!(!FocusModeError::ShortcutsNotFound.is_timeout());
        assert!(!FocusModeError::ShortcutNotFound("x".into()).is_timeout());
        assert!(FocusModeError::ExecutionTimeout("x".into(), 5).is_timeout());
        assert!(!FocusModeError::ExecutionFailed("x".into(), "y".into()).is_timeout());
        assert!(!FocusModeError::Other("x".into()).is_timeout());
    }

    #[test]
    fn test_is_recoverable() {
        // All focus mode errors are recoverable
        assert!(FocusModeError::ShortcutsNotFound.is_recoverable());
        assert!(FocusModeError::ShortcutNotFound("x".into()).is_recoverable());
        assert!(FocusModeError::ExecutionTimeout("x".into(), 5).is_recoverable());
        assert!(FocusModeError::ExecutionFailed("x".into(), "y".into()).is_recoverable());
        assert!(FocusModeError::Other("x".into()).is_recoverable());
    }

    #[test]
    fn test_suggestion() {
        assert!(FocusModeError::ShortcutsNotFound
            .suggestion()
            .contains("macOS 12"));

        assert!(FocusModeError::ShortcutNotFound("x".into())
            .suggestion()
            .contains("Enable Work Focus"));

        assert!(FocusModeError::ExecutionTimeout("x".into(), 5)
            .suggestion()
            .contains("タイムアウト"));

        assert!(FocusModeError::ExecutionFailed("x".into(), "y".into())
            .suggestion()
            .contains("Shortcuts.app"));

        assert!(FocusModeError::Other("x".into())
            .suggestion()
            .contains("再起動"));
    }

    #[test]
    fn test_error_clone() {
        let err = FocusModeError::ShortcutNotFound("test".to_string());
        let cloned = err.clone();
        assert_eq!(err, cloned);
    }

    #[test]
    fn test_error_eq() {
        let err1 = FocusModeError::ExecutionTimeout("test".to_string(), 5);
        let err2 = FocusModeError::ExecutionTimeout("test".to_string(), 5);
        let err3 = FocusModeError::ExecutionTimeout("other".to_string(), 5);

        assert_eq!(err1, err2);
        assert_ne!(err1, err3);
    }
}
