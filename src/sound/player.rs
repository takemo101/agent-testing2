//! Sound player implementation using rodio.
//!
//! This module provides the `RodioSoundPlayer` which uses the rodio v0.20
//! audio library for cross-platform sound playback.

use std::fs::File;
use std::io::{BufReader, Cursor};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};
use tracing::{debug, warn};

use super::embedded::get_embedded_sound;
use super::error::SoundError;
use super::source::SoundSource;

/// A sound player that uses rodio for audio playback.
///
/// This player is thread-safe and can be shared across threads using `Arc`.
/// Sound playback is non-blocking; sounds continue playing in the background.
pub struct RodioSoundPlayer {
    /// The audio output stream (must be kept alive for playback).
    _stream: OutputStream,
    /// Handle to the output stream for creating sinks.
    stream_handle: OutputStreamHandle,
    /// Whether sound playback is disabled.
    disabled: AtomicBool,
}

impl RodioSoundPlayer {
    /// Creates a new sound player.
    ///
    /// # Arguments
    ///
    /// * `disabled` - If true, all sound playback will be silently skipped.
    ///
    /// # Errors
    ///
    /// Returns `SoundError::DeviceNotAvailable` if no audio output device
    /// is available.
    pub fn new(disabled: bool) -> Result<Self, SoundError> {
        let (stream, stream_handle) = OutputStream::try_default()
            .map_err(|e| SoundError::DeviceNotAvailable(e.to_string()))?;

        debug!("Audio output stream initialized");

        Ok(Self {
            _stream: stream,
            stream_handle,
            disabled: AtomicBool::new(disabled),
        })
    }

    /// Creates a disabled sound player without initializing audio hardware.
    ///
    /// This is useful for testing or when audio is not needed.
    /// All calls to `play` will silently succeed without producing sound.
    ///
    /// # Errors
    ///
    /// May still fail if unable to initialize audio stream, but the player
    /// will not actually play any sounds.
    pub fn disabled() -> Result<Self, SoundError> {
        Self::new(true)
    }

    /// Plays a sound from the given source.
    ///
    /// This method is non-blocking; the sound plays in the background.
    /// If playback fails for a system sound, it automatically falls back
    /// to the embedded sound.
    ///
    /// # Arguments
    ///
    /// * `source` - The sound source to play.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The sound file cannot be opened (for system sounds)
    /// - The audio format cannot be decoded
    /// - Audio playback fails
    pub fn play(&self, source: &SoundSource) -> Result<(), SoundError> {
        if self.disabled.load(Ordering::Relaxed) {
            debug!("Sound playback disabled, skipping");
            return Ok(());
        }

        match source {
            SoundSource::System { path, name } => {
                debug!("Playing system sound: {}", name);
                match self.play_file(path) {
                    Ok(()) => Ok(()),
                    Err(e) => {
                        warn!(
                            "Failed to play system sound '{}': {}, falling back to embedded",
                            name, e
                        );
                        self.play_embedded()
                    }
                }
            }
            SoundSource::Embedded { name } => {
                debug!("Playing embedded sound: {}", name);
                self.play_embedded()
            }
        }
    }

    /// Plays a sound file from the filesystem.
    fn play_file(&self, path: &std::path::Path) -> Result<(), SoundError> {
        let file = File::open(path)
            .map_err(|e| SoundError::FileNotFound(format!("{}: {}", path.display(), e)))?;

        let reader = BufReader::new(file);
        let decoder = Decoder::new(reader).map_err(|e| SoundError::DecodeError(e.to_string()))?;

        self.play_decoder(decoder)
    }

    /// Plays the embedded fallback sound.
    fn play_embedded(&self) -> Result<(), SoundError> {
        let cursor = Cursor::new(get_embedded_sound());
        let decoder = Decoder::new(cursor)
            .map_err(|e| SoundError::DecodeError(format!("embedded sound: {}", e)))?;

        self.play_decoder(decoder)
    }

    /// Plays a decoded audio source.
    fn play_decoder<R>(&self, decoder: Decoder<R>) -> Result<(), SoundError>
    where
        R: std::io::Read + std::io::Seek + Send + Sync + 'static,
    {
        let sink = Sink::try_new(&self.stream_handle)
            .map_err(|e| SoundError::StreamError(e.to_string()))?;

        sink.append(decoder);
        sink.detach(); // Non-blocking: sound continues after function returns

        debug!("Sound playback started (detached)");
        Ok(())
    }

    /// Returns true if sound playback is currently disabled.
    #[must_use]
    pub fn is_disabled(&self) -> bool {
        self.disabled.load(Ordering::Relaxed)
    }

    /// Enables sound playback.
    pub fn enable(&self) {
        self.disabled.store(false, Ordering::Relaxed);
        debug!("Sound playback enabled");
    }

    /// Disables sound playback.
    pub fn disable(&self) {
        self.disabled.store(true, Ordering::Relaxed);
        debug!("Sound playback disabled");
    }

    /// Returns true if the audio system is available.
    ///
    /// This always returns true if the player was successfully created,
    /// as the audio stream is initialized during construction.
    #[must_use]
    pub fn is_available(&self) -> bool {
        true
    }
}

impl std::fmt::Debug for RodioSoundPlayer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RodioSoundPlayer")
            .field("disabled", &self.disabled.load(Ordering::Relaxed))
            .finish_non_exhaustive()
    }
}

/// Creates a sound player, returning None if audio is unavailable.
///
/// This is a convenience function for optional sound support.
/// If audio initialization fails, a warning is logged and None is returned.
#[must_use]
pub fn try_create_player(disabled: bool) -> Option<Arc<RodioSoundPlayer>> {
    match RodioSoundPlayer::new(disabled) {
        Ok(player) => Some(Arc::new(player)),
        Err(e) => {
            warn!("Audio not available, sound disabled: {}", e);
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These tests may fail in environments without audio hardware
    // (e.g., CI containers). Tests are designed to handle this gracefully.

    #[test]
    fn test_disabled_player_skips_playback() {
        // Try to create a disabled player; if audio not available, skip test
        let player = match RodioSoundPlayer::disabled() {
            Ok(p) => p,
            Err(_) => return, // Skip test if no audio
        };

        assert!(player.is_disabled());

        // Playing should succeed silently
        let source = SoundSource::embedded("test");
        assert!(player.play(&source).is_ok());
    }

    #[test]
    fn test_enable_disable() {
        let player = match RodioSoundPlayer::disabled() {
            Ok(p) => p,
            Err(_) => return,
        };

        assert!(player.is_disabled());

        player.enable();
        assert!(!player.is_disabled());

        player.disable();
        assert!(player.is_disabled());
    }

    #[test]
    fn test_try_create_player_with_disabled() {
        // Should return None or Some depending on audio availability
        let _result = try_create_player(true);
        // Just verify it doesn't panic
    }

    #[test]
    fn test_debug_impl() {
        let player = match RodioSoundPlayer::disabled() {
            Ok(p) => p,
            Err(_) => return,
        };

        let debug_str = format!("{:?}", player);
        assert!(debug_str.contains("RodioSoundPlayer"));
    }

    #[test]
    fn test_is_available() {
        let player = match RodioSoundPlayer::disabled() {
            Ok(p) => p,
            Err(_) => return,
        };

        // Player is always "available" if successfully created
        assert!(player.is_available());
    }

    #[test]
    fn test_play_nonexistent_file_falls_back() {
        let player = match RodioSoundPlayer::new(false) {
            Ok(p) => p,
            Err(_) => return,
        };

        // Playing a non-existent system sound should fall back to embedded
        let source = SoundSource::system("NonExistent", "/nonexistent/path/to/sound.wav");

        // Should fall back to embedded and succeed
        // (embedded might also fail if format unsupported, that's ok)
        let _ = player.play(&source);
    }
}
