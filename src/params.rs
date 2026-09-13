use nih_plug::prelude::*;
use nih_plug_egui::EguiState;
use std::sync::Arc;

/// Default GUI window width in pixels.
const DEFAULT_EDITOR_WIDTH: u32 = 370;

/// Default GUI window height in pixels.
const DEFAULT_EDITOR_HEIGHT: u32 = 420;

/// Minimum allowable filter cutoff frequency limit in Hz.
const MIN_FREQUENCY_HZ: f32 = 20.0;

/// Maximum allowable filter cutoff frequency limit in Hz.
const MAX_FREQUENCY_HZ: f32 = 20000.0;

/// Default low-pass cutoff frequency in Hz (fully open by default).
const DEFAULT_LP_FREQUENCY_HZ: f32 = 20000.0;

/// Default high-pass cutoff frequency in Hz (fully open by default).
const DEFAULT_HP_FREQUENCY_HZ: f32 = 20.0;

/// Logarithmic skew factor (`0.25`) for human perceptual frequency distribution.
///
/// **Why:** Musical pitch perception is logarithmic. A skew factor of `0.25` allocates
/// roughly half of the UI knob rotation to the musically critical range of 20 Hz to 2 kHz,
/// rather than compressing 90% of the audible spectrum into the first quarter turn.
const FREQUENCY_SKEW_FACTOR: f32 = 0.25;

/// Minimum resonance Q factor limit.
const MIN_Q_FACTOR: f32 = 0.1;

/// Maximum resonance Q factor limit.
const MAX_Q_FACTOR: f32 = 10.0;

/// Default Butterworth neutral Q factor (`0.50` provides flat passband response).
const DEFAULT_Q_FACTOR: f32 = 0.50;

/// Parameter automation smoothing duration in milliseconds.
///
/// **Why:** Smooths rapid DAW automation or MIDI CC changes over a 50ms window to prevent
/// audible zipper noise and abrupt parameter step discontinuities during real-time playback.
const SMOOTHING_TIME_MS: f32 = 50.0;

/// Converts string user input into a floating-point frequency value in Hertz.
///
/// Strips optional "Hz" or "hz" suffixes and isolates numeric characters, decimal points, and minus signs.
///
/// # Arguments
/// * `input_str` - Raw string entered into the UI text field (e.g., `"1000 Hz"`, `"440hz"`, `"20.5"`).
///
/// # Returns
/// `Some(f32)` with the parsed float value if parsing succeeds, or `None` if the input is invalid.
pub fn parse_frequency_string(input_str: &str) -> Option<f32> {
    let cleaned = input_str.replace("Hz", "").replace("hz", "");
    let digits: String = cleaned
        .trim()
        .chars()
        .filter(|character| character.is_ascii_digit() || *character == '.' || *character == '-')
        .collect();
    digits.parse::<f32>().ok()
}

/// Central parameter structure for the `lh_filter` plugin instance.
///
/// Exposes thread-safe parameter references to both the DAW host automation framework
/// and the egui graphic user interface thread.
#[derive(Params)]
pub struct MyFilterParams {
    /// Persistent state manager for GUI window dimensions and UI positioning.
    #[persist = "editor-state"]
    pub editor_state: Arc<EguiState>,

    /// Low-pass cutoff frequency parameter (20 Hz - 20 kHz, logarithmic skew).
    #[id = "lp_freq"]
    pub lp_freq: FloatParam,

    /// High-pass cutoff frequency parameter (20 Hz - 20 kHz, logarithmic skew).
    #[id = "hp_freq"]
    pub hp_freq: FloatParam,

    /// Resonance Q factor parameter (0.1 - 10.0, linear scale).
    #[id = "q"]
    pub q: FloatParam,

    /// Master hard bypass switch.
    #[id = "bypass"]
    pub bypass: BoolParam,
}

impl Default for MyFilterParams {
    fn default() -> Self {
        Self {
            editor_state: EguiState::from_size(DEFAULT_EDITOR_WIDTH, DEFAULT_EDITOR_HEIGHT),

            lp_freq: FloatParam::new(
                "LP Frequency",
                DEFAULT_LP_FREQUENCY_HZ,
                FloatRange::Skewed {
                    min: MIN_FREQUENCY_HZ,
                    max: MAX_FREQUENCY_HZ,
                    factor: FloatRange::skew_factor(FREQUENCY_SKEW_FACTOR),
                },
            )
            .with_unit(" Hz")
            .with_smoother(SmoothingStyle::Logarithmic(SMOOTHING_TIME_MS))
            .with_value_to_string(Arc::new(|val| format!("{:.0} Hz", val)))
            .with_string_to_value(Arc::new(parse_frequency_string)),

            hp_freq: FloatParam::new(
                "HP Frequency",
                DEFAULT_HP_FREQUENCY_HZ,
                FloatRange::Skewed {
                    min: MIN_FREQUENCY_HZ,
                    max: MAX_FREQUENCY_HZ,
                    factor: FloatRange::skew_factor(FREQUENCY_SKEW_FACTOR),
                },
            )
            .with_unit(" Hz")
            .with_smoother(SmoothingStyle::Logarithmic(SMOOTHING_TIME_MS))
            .with_value_to_string(Arc::new(|val| format!("{:.0} Hz", val)))
            .with_string_to_value(Arc::new(parse_frequency_string)),

            q: FloatParam::new(
                "Q",
                DEFAULT_Q_FACTOR,
                FloatRange::Linear {
                    min: MIN_Q_FACTOR,
                    max: MAX_Q_FACTOR,
                },
            )
            .with_smoother(SmoothingStyle::Linear(SMOOTHING_TIME_MS))
            .with_value_to_string(Arc::new(|val| format!("{:.2}", val)))
            .with_string_to_value(Arc::new(|input| input.trim().parse::<f32>().ok())),

            bypass: BoolParam::new("Bypass", false),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_frequency_string() {
        assert_eq!(parse_frequency_string("1000 Hz"), Some(1000.0));
        assert_eq!(parse_frequency_string("440hz"), Some(440.0));
        assert_eq!(parse_frequency_string("20.5"), Some(20.5));
        assert_eq!(parse_frequency_string("invalid"), None);
    }

    #[test]
    fn test_default_params_initialization() {
        let params = MyFilterParams::default();
        assert_eq!(params.lp_freq.value(), DEFAULT_LP_FREQUENCY_HZ);
        assert_eq!(params.hp_freq.value(), DEFAULT_HP_FREQUENCY_HZ);
        assert_eq!(params.q.value(), DEFAULT_Q_FACTOR);
        assert!(!params.bypass.value());
    }
}
