//! Sound source management.
//!
//! This module provides sound source detection and management for the
//! Pomodoro timer. It supports both macOS system sounds and embedded
//! fallback sounds.

use std::path::{Path, PathBuf};

use super::error::SoundError;

/// Represents the source of a sound to be played.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SoundSource {
    /// A macOS system sound from `/System/Library/Sounds/` or similar.
    System {
        /// The name of the sound (e.g., "Glass").
        name: String,
        /// The full path to the sound file.
        path: PathBuf,
    },
    /// An embedded sound compiled into the binary.
    Embedded {
        /// The name of the embedded sound (e.g., "default").
        name: String,
    },
}

impl SoundSource {
    /// Creates a new system sound source.
    ///
    /// # Note
    ///
    /// This constructor does not validate the path. Use `system_validated` for
    /// path validation against allowed directories.
    #[must_use]
    pub fn system(name: impl Into<String>, path: impl Into<PathBuf>) -> Self {
        Self::System {
            name: name.into(),
            path: path.into(),
        }
    }

    /// Creates a new system sound source with path validation.
    ///
    /// Only allows paths within the system sound directories:
    /// - `/System/Library/Sounds`
    /// - `/Library/Sounds`
    ///
    /// # Errors
    ///
    /// Returns `SoundError::InvalidPath` if the path is outside allowed directories.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pomodoro::sound::SoundSource;
    ///
    /// // Valid system sound path
    /// let source = SoundSource::system_validated("Glass", "/System/Library/Sounds/Glass.aiff");
    /// assert!(source.is_ok());
    ///
    /// // Invalid path (outside allowed directories)
    /// let source = SoundSource::system_validated("evil", "/tmp/evil.wav");
    /// assert!(source.is_err());
    /// ```
    pub fn system_validated(
        name: impl Into<String>,
        path: impl Into<PathBuf>,
    ) -> Result<Self, SoundError> {
        let path = path.into();
        validate_system_sound_path(&path)?;
        Ok(Self::System {
            name: name.into(),
            path,
        })
    }

    /// Creates a new embedded sound source.
    #[must_use]
    pub fn embedded(name: impl Into<String>) -> Self {
        Self::Embedded { name: name.into() }
    }

    /// Returns the name of the sound source.
    #[must_use]
    pub fn name(&self) -> &str {
        match self {
            Self::System { name, .. } | Self::Embedded { name } => name,
        }
    }

    /// Returns true if this is a system sound.
    #[must_use]
    pub fn is_system(&self) -> bool {
        matches!(self, Self::System { .. })
    }

    /// Returns true if this is an embedded sound.
    #[must_use]
    pub fn is_embedded(&self) -> bool {
        matches!(self, Self::Embedded { .. })
    }

    /// Returns the file path if this is a system sound.
    #[must_use]
    pub fn path(&self) -> Option<&PathBuf> {
        match self {
            Self::System { path, .. } => Some(path),
            Self::Embedded { .. } => None,
        }
    }
}

/// Directories to search for system sounds, in order of priority.
const SYSTEM_SOUND_DIRS: &[&str] = &["/System/Library/Sounds", "/Library/Sounds"];

fn validate_system_sound_path(path: &Path) -> Result<(), SoundError> {
    let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());

    for allowed_dir in SYSTEM_SOUND_DIRS {
        let allowed_path = PathBuf::from(allowed_dir);
        if let Ok(canonical_allowed) = allowed_path.canonicalize() {
            if canonical.starts_with(&canonical_allowed) {
                return Ok(());
            }
        }
        if canonical.starts_with(allowed_dir) {
            return Ok(());
        }
    }

    Err(SoundError::InvalidPath(format!(
        "Path '{}' is not within allowed system sound directories",
        path.display()
    )))
}

/// Supported audio file extensions.
const SUPPORTED_EXTENSIONS: &[&str] = &["aiff", "wav", "mp3", "m4a", "flac"];

/// Default sound names to try, in order of preference.
const DEFAULT_SOUND_NAMES: &[&str] = &["Glass", "Ping", "Pop", "Blow"];

/// Discovers available system sounds.
///
/// Scans the system sound directories and returns a list of available sounds.
/// Returns an empty vector if no sounds are found.
#[must_use]
pub fn discover_system_sounds() -> Vec<SoundSource> {
    let mut sounds = Vec::new();

    for dir in SYSTEM_SOUND_DIRS {
        let path = PathBuf::from(dir);
        if !path.exists() {
            continue;
        }

        if let Ok(entries) = std::fs::read_dir(&path) {
            for entry in entries.flatten() {
                let file_path = entry.path();
                if let Some(ext) = file_path.extension() {
                    let ext_str = ext.to_string_lossy().to_lowercase();
                    if SUPPORTED_EXTENSIONS.contains(&ext_str.as_str()) {
                        if let Some(stem) = file_path.file_stem() {
                            sounds.push(SoundSource::System {
                                name: stem.to_string_lossy().into_owned(),
                                path: file_path,
                            });
                        }
                    }
                }
            }
        }
    }

    // Sort by name for consistent ordering
    sounds.sort_by(|a, b| a.name().cmp(b.name()));
    sounds
}

/// Gets the default sound source for timer notifications.
///
/// Attempts to find a suitable system sound, falling back to embedded sound
/// if no system sounds are available.
#[must_use]
pub fn get_default_sound() -> SoundSource {
    let system_sounds = discover_system_sounds();

    // Try to find one of the preferred sounds
    for preferred_name in DEFAULT_SOUND_NAMES {
        if let Some(sound) = system_sounds.iter().find(|s| s.name() == *preferred_name) {
            return sound.clone();
        }
    }

    // Fall back to the first available system sound
    if let Some(first) = system_sounds.into_iter().next() {
        return first;
    }

    // Ultimate fallback: embedded sound
    SoundSource::embedded("default")
}

/// Finds a system sound by name.
///
/// # Errors
///
/// Returns `SoundError::FileNotFound` if no sound with the given name exists.
pub fn find_system_sound(name: &str) -> Result<SoundSource, SoundError> {
    let sounds = discover_system_sounds();
    sounds
        .into_iter()
        .find(|s| s.name().eq_ignore_ascii_case(name))
        .ok_or_else(|| SoundError::FileNotFound(format!("System sound '{}' not found", name)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sound_source_system() {
        let source = SoundSource::system("Glass", "/System/Library/Sounds/Glass.aiff");
        assert!(source.is_system());
        assert!(!source.is_embedded());
        assert_eq!(source.name(), "Glass");
        assert!(source.path().is_some());
    }

    #[test]
    fn test_sound_source_embedded() {
        let source = SoundSource::embedded("default");
        assert!(source.is_embedded());
        assert!(!source.is_system());
        assert_eq!(source.name(), "default");
        assert!(source.path().is_none());
    }

    #[test]
    fn test_sound_source_equality() {
        let s1 = SoundSource::system("Glass", "/path/Glass.aiff");
        let s2 = SoundSource::system("Glass", "/path/Glass.aiff");
        let s3 = SoundSource::system("Ping", "/path/Ping.aiff");

        assert_eq!(s1, s2);
        assert_ne!(s1, s3);
    }

    #[test]
    fn test_sound_source_clone() {
        let source = SoundSource::system("Glass", "/System/Library/Sounds/Glass.aiff");
        let cloned = source.clone();
        assert_eq!(source, cloned);
    }

    #[test]
    fn test_discover_system_sounds_returns_vec() {
        // This test verifies the function runs without panic
        // Actual results depend on the system
        let sounds = discover_system_sounds();
        // The result is a vector (may be empty in container environment)
        // Just verify it doesn't panic and returns a valid vec
        let _ = sounds.len();
    }

    #[test]
    fn test_get_default_sound_returns_source() {
        // Should always return a valid source (embedded if no system sounds)
        let source = get_default_sound();
        assert!(!source.name().is_empty());
    }

    #[test]
    fn test_find_system_sound_not_found() {
        let result = find_system_sound("NonExistentSound12345");
        assert!(result.is_err());
        if let Err(SoundError::FileNotFound(msg)) = result {
            assert!(msg.contains("NonExistentSound12345"));
        } else {
            panic!("Expected FileNotFound error");
        }
    }

    #[test]
    fn test_supported_extensions() {
        // Verify our extension list is reasonable
        assert!(SUPPORTED_EXTENSIONS.contains(&"aiff"));
        assert!(SUPPORTED_EXTENSIONS.contains(&"wav"));
        assert!(SUPPORTED_EXTENSIONS.contains(&"mp3"));
    }

    #[test]
    fn test_default_sound_names() {
        assert!(DEFAULT_SOUND_NAMES.contains(&"Glass"));
        assert!(DEFAULT_SOUND_NAMES.contains(&"Ping"));
    }

    #[test]
    fn test_system_validated_accepts_valid_paths() {
        let result = SoundSource::system_validated("Glass", "/System/Library/Sounds/Glass.aiff");
        assert!(result.is_ok());

        let result = SoundSource::system_validated("Custom", "/Library/Sounds/Custom.wav");
        assert!(result.is_ok());
    }

    #[test]
    fn test_system_validated_rejects_invalid_paths() {
        let result = SoundSource::system_validated("evil", "/tmp/evil.wav");
        assert!(result.is_err());
        if let Err(SoundError::InvalidPath(msg)) = result {
            assert!(msg.contains("/tmp/evil.wav"));
        } else {
            panic!("Expected InvalidPath error");
        }

        let result = SoundSource::system_validated("home", "/Users/test/sound.wav");
        assert!(result.is_err());

        let result = SoundSource::system_validated("etc", "/etc/passwd");
        assert!(result.is_err());
    }

    #[test]
    fn test_system_validated_rejects_path_traversal() {
        let result = SoundSource::system_validated(
            "traversal",
            "/System/Library/Sounds/../../../etc/passwd",
        );
        assert!(result.is_err());
    }
}
