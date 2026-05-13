use nih_plug::prelude::*;
use nih_plug_egui::EguiState;
use std::sync::Arc;

#[derive(Params)]
pub struct MyFilterParams {
    /// Persistent editor state (window size, etc.)
    #[persist = "editor-state"]
    pub editor_state: Arc<EguiState>,

    /// Low-pass cutoff frequency (20Hz - 20kHz)
    #[id = "lp_freq"]
    pub lp_freq: FloatParam,

    /// High-pass cutoff frequency (20Hz - 20kHz)
    #[id = "hp_freq"]
    pub hp_freq: FloatParam,

    /// Filter Q factor (resonance)
    #[id = "q"]
    pub q: FloatParam,

    /// Master bypass switch
    #[id = "bypass"]
    pub bypass: BoolParam,
}

impl Default for MyFilterParams {
    fn default() -> Self {
        Self {
            editor_state: EguiState::from_size(500, 500),
            lp_freq: FloatParam::new(
                "LP Frequency",
                20000.0,
                FloatRange::Skewed {
                    min: 20.0,
                    max: 20000.0,
                    factor: FloatRange::skew_factor(0.25),
                },
            )
            .with_unit(" Hz")
            .with_smoother(SmoothingStyle::Logarithmic(50.0)),

            hp_freq: FloatParam::new(
                "HP Frequency",
                20.0,
                FloatRange::Skewed {
                    min: 20.0,
                    max: 20000.0,
                    factor: FloatRange::skew_factor(0.25),
                },
            )
            .with_unit(" Hz")
            .with_smoother(SmoothingStyle::Logarithmic(50.0)),

            q: FloatParam::new(
                "Q",
                10.0,
                FloatRange::Linear {
                    min: 0.1,
                    max: 10.0,
                },
            )
            .with_smoother(SmoothingStyle::Linear(50.0)),

            bypass: BoolParam::new("Bypass", false),
        }
    }
}
