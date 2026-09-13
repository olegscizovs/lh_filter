use nih_plug::prelude::*;
use std::sync::Arc;

mod dsp;
mod editor;
mod params;

use dsp::{Biquad, FilterType, SoftLimiter};
use params::MyFilterParams;

/// Standard stereo channel configuration count (2 channels: Left and Right).
const STEREO_CHANNEL_COUNT: u32 = 2;

/// Left audio channel buffer index.
const LEFT_CHANNEL_INDEX: usize = 0;

/// Right audio channel buffer index.
const RIGHT_CHANNEL_INDEX: usize = 1;

/// Crossfade smoothing duration in milliseconds for bypass toggling.
///
/// **Why:** Ramping between dry (unfiltered) and wet (filtered) audio over a 10ms window
/// eliminates instantaneous step discontinuities in the waveform, completely preventing clicks when toggling bypass.
const BYPASS_SMOOTHING_TIME_MS: f32 = 10.0;

/// Core plugin processor instance for `lh_filter`.
///
/// Encapsulates parameter state, stereo DSP filter blocks, and smooth bypass crossfading.
pub struct MyFilter {
    params: Arc<MyFilterParams>,
    lp_filter_l: Biquad,
    lp_filter_r: Biquad,
    hp_filter_l: Biquad,
    hp_filter_r: Biquad,
    limiter_l: SoftLimiter,
    limiter_r: SoftLimiter,
    bypass_smoother: Smoother<f32>,
}

impl Default for MyFilter {
    fn default() -> Self {
        Self {
            params: Arc::new(MyFilterParams::default()),
            lp_filter_l: Biquad::new(),
            lp_filter_r: Biquad::new(),
            hp_filter_l: Biquad::new(),
            hp_filter_r: Biquad::new(),
            limiter_l: SoftLimiter::new(),
            limiter_r: SoftLimiter::new(),
            bypass_smoother: Smoother::new(SmoothingStyle::Linear(BYPASS_SMOOTHING_TIME_MS)),
        }
    }
}

/// Cascades a single audio sample sequentially through low-pass filtering, high-pass filtering, and soft-limiting.
///
/// # Arguments
/// * `sample` - Raw incoming input sample value.
/// * `lp` - Mutable reference to the channel's low-pass biquad filter.
/// * `hp` - Mutable reference to the channel's high-pass biquad filter.
/// * `limiter` - Mutable reference to the channel's soft limiter peak saturation block.
///
/// # Returns
/// Processed floating-point audio sample.
#[inline(always)]
fn process_channel_sample(sample: f32, lp: &mut Biquad, hp: &mut Biquad, limiter: &mut SoftLimiter) -> f32 {
    let lowpassed = lp.process(sample);
    let highpassed = hp.process(lowpassed);
    limiter.process(highpassed)
}

impl Plugin for MyFilter {
    const NAME: &'static str = "lh_filter V1";
    const VENDOR: &'static str = "Creator";
    const URL: &'static str = "";
    const EMAIL: &'static str = "jaqueole@gmail.com";

    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: std::num::NonZeroU32::new(STEREO_CHANNEL_COUNT),
        main_output_channels: std::num::NonZeroU32::new(STEREO_CHANNEL_COUNT),
        ..AudioIOLayout::const_default()
    }];

    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        editor::create_editor(self.params.clone(), self.params.editor_state.clone())
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        _buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        self.reset();
        true
    }

    /// Resets all internal DSP filter memory registers and bypass crossfade state to zero.
    ///
    /// **Why:** Called by the DAW host whenever playback resets or sample rates change to prevent
    /// past audio state from leaking into new playback sessions as loud clicks.
    fn reset(&mut self) {
        self.lp_filter_l.reset();
        self.lp_filter_r.reset();
        self.hp_filter_l.reset();
        self.hp_filter_r.reset();
        self.limiter_l.reset();
        self.limiter_r.reset();

        let initial_bypass_val = if self.params.bypass.value() { 1.0 } else { 0.0 };
        self.bypass_smoother.reset(initial_bypass_val);
    }

    /// Real-time audio processing loop.
    ///
    /// Evaluates sample-accurate parameter smoothing once per sample step, updates DSP biquad coefficients,
    /// and performs smooth dry/wet crossfading during bypass transitions.
    ///
    /// **Thread-Safety Note:** Performs zero dynamic heap allocations or system locks inside this loop
    /// to guarantee real-time audio thread execution safety.
    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        let sample_rate = context.transport().sample_rate;
        let is_bypassed = self.params.bypass.value();
        let target_bypass = if is_bypassed { 1.0 } else { 0.0 };
        self.bypass_smoother.set_target(sample_rate, target_bypass);

        // Fast path: if fully bypassed and crossfade smoothing is complete
        if is_bypassed && !self.bypass_smoother.is_smoothing() {
            return ProcessStatus::Normal;
        }

        let num_samples = buffer.samples();
        let num_channels = buffer.channels();

        for sample_index in 0..num_samples {
            let bypass_fade = self.bypass_smoother.next();
            let lp_freq = self.params.lp_freq.smoothed.next();
            let hp_freq = self.params.hp_freq.smoothed.next();
            let q_factor = self.params.q.smoothed.next();

            self.lp_filter_l.update_coefficients(FilterType::Lowpass, sample_rate, lp_freq, q_factor);
            self.lp_filter_r.update_coefficients(FilterType::Lowpass, sample_rate, lp_freq, q_factor);
            self.hp_filter_l.update_coefficients(FilterType::Highpass, sample_rate, hp_freq, q_factor);
            self.hp_filter_r.update_coefficients(FilterType::Highpass, sample_rate, hp_freq, q_factor);

            for channel_idx in 0..num_channels {
                let sample = &mut buffer.as_slice()[channel_idx][sample_index];
                let dry_sample = *sample;

                let wet_sample = if channel_idx == LEFT_CHANNEL_INDEX {
                    process_channel_sample(dry_sample, &mut self.lp_filter_l, &mut self.hp_filter_l, &mut self.limiter_l)
                } else if channel_idx == RIGHT_CHANNEL_INDEX {
                    process_channel_sample(dry_sample, &mut self.lp_filter_r, &mut self.hp_filter_r, &mut self.limiter_r)
                } else {
                    dry_sample
                };

                // Equal-gain linear crossfade: 1.0 = 100% dry (bypassed), 0.0 = 100% wet (filtered)
                *sample = dry_sample * bypass_fade + wet_sample * (1.0 - bypass_fade);
            }
        }

        ProcessStatus::Normal
    }
}

impl Vst3Plugin for MyFilter {
    const VST3_CLASS_ID: [u8; 16] = *b"AntigravityFiltr";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] = &[Vst3SubCategory::Filter, Vst3SubCategory::Fx];
}

nih_export_vst3!(MyFilter);
