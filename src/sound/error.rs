//! Sound system error types.
//!
//! This module defines the error types for the sound playback system.
//! All errors are designed to provide helpful messages for debugging
//! and graceful degradation when audio is unavailable.

use thiserror::Error;

/// Errors that can occur in the sound playback system.
#[derive(Debug, Error)]
pub enum SoundError {
    /// Audio device is not available (e.g., no speakers connected).
    #[error("オーディオデバイスが利用できません: {0}")]
    DeviceNotAvailable(String),

    /// Sound file was not found at the specified path.
    #[error("サウンドファイルが見つかりません: {0}")]
    FileNotFound(String),

    /// Failed to decode the audio file.
    #[error("サウンドファイルのデコードに失敗しました: {0}")]
    DecodeError(String),

    /// Failed to create the audio output stream.
    #[error("オーディオストリームの作成に失敗しました: {0}")]
    StreamError(String),

    /// Generic sound playback error.
    #[error("サウンド再生エラー: {0}")]
    PlaybackError(String),
}

impl SoundError {
    /// Returns true if this error is related to device availability.
    #[must_use]
    pub fn is_device_error(&self) -> bool {
        matches!(self, Self::DeviceNotAvailable(_) | Self::StreamError(_))
    }

    /// Returns true if this error is related to the audio file.
    #[must_use]
    pub fn is_file_error(&self) -> bool {
        matches!(self, Self::FileNotFound(_) | Self::DecodeError(_))
    }

    /// Returns true if playback should fall back to embedded sound.
    #[must_use]
    pub fn should_fallback_to_embedded(&self) -> bool {
        matches!(self, Self::FileNotFound(_))
    }

    /// Returns a user-friendly suggestion for resolving this error.
    #[must_use]
    pub fn suggestion(&self) -> &'static str {
        match self {
            Self::DeviceNotAvailable(_) => "オーディオデバイスを接続してください",
            Self::FileNotFound(_) => "埋め込みサウンドで再生を試みます",
            Self::DecodeError(_) => "サウンドファイルが破損している可能性があります",
            Self::StreamError(_) => "オーディオ設定を確認してください",
            Self::PlaybackError(_) => "アプリケーションを再起動してください",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = SoundError::DeviceNotAvailable("no device".to_string());
        assert!(err.to_string().contains("no device"));
        assert!(err.to_string().contains("オーディオデバイスが利用できません"));

        let err = SoundError::FileNotFound("/path/to/sound.aiff".to_string());
        assert!(err.to_string().contains("/path/to/sound.aiff"));

        let err = SoundError::DecodeError("invalid format".to_string());
        assert!(err.to_string().contains("invalid format"));

        let err = SoundError::StreamError("stream failed".to_string());
        assert!(err.to_string().contains("stream failed"));

        let err = SoundError::PlaybackError("unknown error".to_string());
        assert!(err.to_string().contains("unknown error"));
    }

    #[test]
    fn test_is_device_error() {
        assert!(SoundError::DeviceNotAvailable("x".into()).is_device_error());
        assert!(SoundError::StreamError("x".into()).is_device_error());
        assert!(!SoundError::FileNotFound("x".into()).is_device_error());
        assert!(!SoundError::DecodeError("x".into()).is_device_error());
        assert!(!SoundError::PlaybackError("x".into()).is_device_error());
    }

    #[test]
    fn test_is_file_error() {
        assert!(SoundError::FileNotFound("x".into()).is_file_error());
        assert!(SoundError::DecodeError("x".into()).is_file_error());
        assert!(!SoundError::DeviceNotAvailable("x".into()).is_file_error());
        assert!(!SoundError::StreamError("x".into()).is_file_error());
        assert!(!SoundError::PlaybackError("x".into()).is_file_error());
    }

    #[test]
    fn test_should_fallback_to_embedded() {
        assert!(SoundError::FileNotFound("x".into()).should_fallback_to_embedded());
        assert!(!SoundError::DecodeError("x".into()).should_fallback_to_embedded());
        assert!(!SoundError::DeviceNotAvailable("x".into()).should_fallback_to_embedded());
    }

    #[test]
    fn test_suggestion() {
        let err = SoundError::DeviceNotAvailable("x".into());
        assert!(err.suggestion().contains("オーディオデバイス"));

        let err = SoundError::FileNotFound("x".into());
        assert!(err.suggestion().contains("埋め込みサウンド"));

        let err = SoundError::DecodeError("x".into());
        assert!(err.suggestion().contains("破損"));

        let err = SoundError::StreamError("x".into());
        assert!(err.suggestion().contains("オーディオ設定"));

        let err = SoundError::PlaybackError("x".into());
        assert!(err.suggestion().contains("再起動"));
    }
}
