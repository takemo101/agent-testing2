//! Sound playback system for the Pomodoro Timer.
//!
//! This module provides audio notification capabilities, including:
//!
//! - System sound discovery and playback
//! - Embedded fallback sounds
//! - Non-blocking audio playback
//! - Graceful degradation when audio is unavailable
//!
//! # Architecture
//!
//! ```text
//! ┌──────────────────┐
//! │   SoundPlayer    │ ← Main interface
//! └────────┬─────────┘
//!          │
//!          ▼
//! ┌──────────────────┐     ┌──────────────────┐
//! │   SoundSource    │────▶│  System Sounds   │
//! │                  │     │  (/System/...)   │
//! │                  │     ├──────────────────┤
//! │                  │────▶│ Embedded Sounds  │
//! └──────────────────┘     │  (fallback)      │
//!                          └──────────────────┘
//! ```
//!
//! # Usage
//!
//! ```rust,no_run
//! use pomodoro::sound::{RodioSoundPlayer, SoundSource, get_default_sound};
//!
//! // Create a player (may fail if no audio device)
//! let player = RodioSoundPlayer::new(false).expect("audio init");
//!
//! // Play the default notification sound
//! let source = get_default_sound();
//! player.play(&source).expect("playback failed");
//!
//! // Or play a specific system sound
//! let source = SoundSource::system("Glass", "/System/Library/Sounds/Glass.aiff");
//! player.play(&source).expect("playback failed");
//! ```
//!
//! # Feature Flags
//!
//! This module requires the `rodio` dependency with the `symphonia-all`
//! feature for full format support including AIFF (used by macOS system sounds).

mod embedded;
mod error;
mod player;
mod source;

// Re-export public types
pub use embedded::{get_embedded_sound, get_embedded_sound_format, DEFAULT_SOUND_DATA};
pub use error::SoundError;
pub use player::{try_create_player, RodioSoundPlayer};
pub use source::{discover_system_sounds, find_system_sound, get_default_sound, SoundSource};

/// Trait for sound playback implementations.
///
/// This trait abstracts the sound playback functionality, allowing for
/// different implementations (e.g., rodio-based, mock for testing).
pub trait SoundPlayer {
    /// Plays a sound from the given source.
    ///
    /// This method should be non-blocking; the sound plays in the background.
    ///
    /// # Errors
    ///
    /// Returns an error if playback fails.
    fn play(&self, source: &SoundSource) -> Result<(), SoundError>;

    /// Returns true if the audio system is available.
    fn is_available(&self) -> bool;

    /// Returns true if sound playback is disabled.
    fn is_disabled(&self) -> bool;

    /// Enables sound playback.
    fn enable(&self);

    /// Disables sound playback.
    fn disable(&self);
}

impl SoundPlayer for RodioSoundPlayer {
    fn play(&self, source: &SoundSource) -> Result<(), SoundError> {
        RodioSoundPlayer::play(self, source)
    }

    fn is_available(&self) -> bool {
        RodioSoundPlayer::is_available(self)
    }

    fn is_disabled(&self) -> bool {
        RodioSoundPlayer::is_disabled(self)
    }

    fn enable(&self) {
        RodioSoundPlayer::enable(self)
    }

    fn disable(&self) {
        RodioSoundPlayer::disable(self)
    }
}

/// Plays the default notification sound.
///
/// This is a convenience function that creates a temporary player and plays
/// the default sound. For repeated playback, prefer creating a `RodioSoundPlayer`
/// and reusing it.
///
/// # Errors
///
/// Returns an error if audio initialization or playback fails.
///
/// # Example
///
/// ```rust,no_run
/// use pomodoro::sound::play_notification_sound;
///
/// if let Err(e) = play_notification_sound() {
///     eprintln!("Could not play sound: {}", e);
/// }
/// ```
pub fn play_notification_sound() -> Result<(), SoundError> {
    let player = RodioSoundPlayer::new(false)?;
    let source = get_default_sound();
    player.play(&source)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_exports() {
        // Verify all public types are accessible
        let _: fn(bool) -> Result<RodioSoundPlayer, SoundError> = RodioSoundPlayer::new;
        let _: fn() -> SoundSource = get_default_sound;
        let _: fn() -> Vec<SoundSource> = discover_system_sounds;
        let _: fn(&str) -> Result<SoundSource, SoundError> = find_system_sound;
        let _: fn() -> &'static [u8] = get_embedded_sound;
    }

    #[test]
    fn test_sound_source_constructors() {
        let sys = SoundSource::system("Glass", "/path/to/Glass.aiff");
        assert!(sys.is_system());
        assert_eq!(sys.name(), "Glass");

        let emb = SoundSource::embedded("default");
        assert!(emb.is_embedded());
        assert_eq!(emb.name(), "default");
    }

    #[test]
    fn test_get_default_sound() {
        let source = get_default_sound();
        // Should always return a valid source
        assert!(!source.name().is_empty());
    }

    #[test]
    fn test_embedded_sound_data() {
        let data = get_embedded_sound();
        assert!(!data.is_empty());
        // Verify WAV header
        assert_eq!(&data[0..4], b"RIFF");
    }

    #[test]
    fn test_discover_system_sounds_no_panic() {
        // Should not panic even in container environments
        let _ = discover_system_sounds();
    }

    #[test]
    fn test_play_notification_sound_graceful_failure() {
        // May fail in container without audio, that's expected
        let _ = play_notification_sound();
    }
}
