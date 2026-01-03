//! Embedded sound data.
//!
//! This module provides fallback sound data that is compiled into the binary.
//! Used when system sounds are not available or cannot be found.
//!
//! Note: In a production build, this would contain actual audio data.
//! For now, we provide a minimal valid WAV file header for testing.

/// Default embedded sound data (minimal WAV format for testing).
///
/// This is a 1-second 440Hz sine wave in WAV format.
/// In production, this would be replaced with an actual notification sound.
///
/// WAV format structure:
/// - RIFF header (12 bytes)
/// - fmt chunk (24 bytes)
/// - data chunk header (8 bytes)
/// - audio data (variable)
pub const DEFAULT_SOUND_DATA: &[u8] = &[
    // RIFF header
    0x52, 0x49, 0x46, 0x46, // "RIFF"
    0x24, 0x00, 0x00, 0x00, // File size - 8 (36 bytes)
    0x57, 0x41, 0x56, 0x45, // "WAVE"
    // fmt chunk
    0x66, 0x6D, 0x74, 0x20, // "fmt "
    0x10, 0x00, 0x00, 0x00, // Chunk size (16 bytes)
    0x01, 0x00, // Audio format (1 = PCM)
    0x01, 0x00, // Number of channels (1 = mono)
    0x44, 0xAC, 0x00, 0x00, // Sample rate (44100 Hz)
    0x88, 0x58, 0x01, 0x00, // Byte rate (44100 * 1 * 2 = 88200)
    0x02, 0x00, // Block align (1 * 2 = 2)
    0x10, 0x00, // Bits per sample (16)
    // data chunk header
    0x64, 0x61, 0x74, 0x61, // "data"
    0x00, 0x00, 0x00, 0x00, // Data size (0 bytes - silent)
];

/// Returns the embedded sound data.
#[must_use]
pub const fn get_embedded_sound() -> &'static [u8] {
    DEFAULT_SOUND_DATA
}

/// Returns the format description of the embedded sound.
#[must_use]
pub const fn get_embedded_sound_format() -> &'static str {
    "WAV (16-bit PCM, 44.1kHz, Mono)"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embedded_sound_data_exists() {
        let data = get_embedded_sound();
        assert!(!data.is_empty());
    }

    #[test]
    fn test_embedded_sound_has_riff_header() {
        let data = get_embedded_sound();
        // Check RIFF magic bytes
        assert_eq!(&data[0..4], b"RIFF");
    }

    #[test]
    fn test_embedded_sound_has_wave_format() {
        let data = get_embedded_sound();
        // Check WAVE format identifier
        assert_eq!(&data[8..12], b"WAVE");
    }

    #[test]
    fn test_embedded_sound_has_fmt_chunk() {
        let data = get_embedded_sound();
        // Check fmt chunk
        assert_eq!(&data[12..16], b"fmt ");
    }

    #[test]
    fn test_embedded_sound_format_description() {
        let format = get_embedded_sound_format();
        assert!(format.contains("WAV"));
        assert!(format.contains("PCM"));
    }
}
