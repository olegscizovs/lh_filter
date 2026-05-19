use nih_plug::prelude::*;
use std::sync::Arc;

mod dsp;
mod editor;
mod params;

use dsp::{Biquad, FilterType, SoftLimiter};
use params::MyFilterParams;

pub struct MyFilter {
    params: Arc<MyFilterParams>,
    lp_filter_l: Biquad,
    lp_filter_r: Biquad,
    hp_filter_l: Biquad,
    hp_filter_r: Biquad,
    limiter_l: SoftLimiter,
    limiter_r: SoftLimiter,
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
        }
    }
}

impl Plugin for MyFilter {
    const NAME: &'static str = "lh_filter V1";
    const VENDOR: &'static str = "Creator";
    const URL: &'static str = "";
    const EMAIL: &'static str = "jaqueole@gmail.com";

    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[
        AudioIOLayout {
            main_input_channels: std::num::NonZeroU32::new(2),
            main_output_channels: std::num::NonZeroU32::new(2),
            ..AudioIOLayout::const_default()
        },
    ];

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

    fn reset(&mut self) {
        // Reset all DSP component states to avoid pops
        self.lp_filter_l.reset();
        self.lp_filter_r.reset();
        self.hp_filter_l.reset();
        self.hp_filter_r.reset();
        self.limiter_l.reset();
        self.limiter_r.reset();
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        // SECURITY: Check bypass first. Process block must perform direct buffer copy (implicit here).
        if self.params.bypass.value() {
            return ProcessStatus::Normal;
        }

        let sample_rate = context.transport().sample_rate;
        let num_samples = buffer.samples();
        let num_channels = buffer.channels();

        // BEST PRACTICE: No allocations in this loop.
        for i in 0..num_samples {
            // SMOOTHING: next() advances the parameter smoother
            let lp_freq = self.params.lp_freq.smoothed.next();
            let hp_freq = self.params.hp_freq.smoothed.next();
            let q = self.params.q.smoothed.next();

            // DSP: Update coefficients once per sample to follow smoothing
            self.lp_filter_l.update_coefficients(FilterType::Lowpass, sample_rate, lp_freq, q);
            self.lp_filter_r.update_coefficients(FilterType::Lowpass, sample_rate, lp_freq, q);
            self.hp_filter_l.update_coefficients(FilterType::Highpass, sample_rate, hp_freq, q);
            self.hp_filter_r.update_coefficients(FilterType::Highpass, sample_rate, hp_freq, q);

            for channel_idx in 0..num_channels {
                let sample = &mut buffer.as_slice()[channel_idx][i];
                if channel_idx == 0 {
                    *sample = self.lp_filter_l.process(*sample);
                    *sample = self.hp_filter_l.process(*sample);
                    *sample = self.limiter_l.process(*sample);
                } else if channel_idx == 1 {
                    *sample = self.lp_filter_r.process(*sample);
                    *sample = self.hp_filter_r.process(*sample);
                    *sample = self.limiter_r.process(*sample);
                }
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
