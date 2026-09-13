use std::f32::consts::PI;

/// Safety floor for cutoff frequency in Hertz.
///
/// **Why:** Frequencies at or below 0 Hz result in undefined log-space calculations
/// and infinite coefficient values in trigonometric formulas.
const MIN_FREQUENCY_HZ: f32 = 20.0;

/// Safety ceiling multiplier relative to sample rate (`0.49 * sample_rate`).
///
/// **Why:** Keeping the cutoff frequency slightly below the Nyquist limit (`0.50 * sample_rate`)
/// prevents mathematical divergence and severe phase distortion near the digital Nyquist boundary.
const MAX_NYQUIST_FACTOR: f32 = 0.49;

/// Minimum Q factor limit for resonance calculation.
///
/// **Why:** A Q factor of zero causes a division-by-zero error when computing the filter
/// bandwidth multiplier (`alpha = sin(w0) / (2 * Q)`).
const MIN_Q_FACTOR: f32 = 0.01;

/// Threshold amplitude (~-3 dBFS = 0.707) where soft limiter compression begins.
const SOFT_LIMITER_THRESHOLD: f32 = 0.707;

/// Available headroom margin above threshold (`1.0 - 0.707 = 0.293`).
const SOFT_LIMITER_MARGIN: f32 = 0.293;

/// Threshold Q factor above which resonance gain compensation is dynamically engaged.
const REASONABLE_Q_THRESHOLD: f32 = 0.707;

/// Anti-denormal flushing threshold to prevent CPU spikes from subnormal floating-point values.
const DENORMAL_FLUSH_THRESHOLD: f32 = 1e-15;

/// Filter topology selection for biquad frequency processing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FilterType {
    /// Low-pass filter (attenuates frequencies above the cutoff point).
    Lowpass,
    /// High-pass filter (attenuates frequencies below the cutoff point).
    Highpass,
}

/// Second-Order Transposed Direct Form II (TDF2) Biquad Filter.
///
/// **Why TDF2:** Transposed Direct Form II stores delay line state in energy variables (`s1, s2`)
/// rather than raw past samples (`x1, x2, y1, y2`). When coefficients change sample-by-sample
/// during parameter automation or knob movement, TDF2 maintains continuous internal state
/// without producing audible clicks or step discontinuities.
pub struct Biquad {
    a1: f32,
    a2: f32,
    b0: f32,
    b1: f32,
    b2: f32,
    s1: f32,
    s2: f32,
}

impl Biquad {
    /// Instantiates a new biquad filter with zeroed state and clear coefficients.
    pub fn new() -> Self {
        Self {
            a1: 0.0,
            a2: 0.0,
            b0: 0.0,
            b1: 0.0,
            b2: 0.0,
            s1: 0.0,
            s2: 0.0,
        }
    }

    /// Clears internal state variables (`s1`, `s2`) to zero.
    ///
    /// **Why:** Prevents residual energy from past audio buffers from leaking into new playback sessions as clicks.
    pub fn reset(&mut self) {
        self.s1 = 0.0;
        self.s2 = 0.0;
    }

    /// Recalculates normalized filter coefficients (`b0, b1, b2, a1, a2`) based on RBJ Cookbook formulas,
    /// applying Q gain compensation to prevent signal clipping at high Q settings.
    ///
    /// # Arguments
    /// * `filter_type` - Type of filter topology (`Lowpass` or `Highpass`).
    /// * `sample_rate` - Current DAW audio engine sample rate in Hz (e.g. 44100.0 or 48000.0).
    /// * `frequency_hz` - Desired cutoff frequency in Hz. Clamped to `[20.0, 0.49 * sample_rate]`.
    /// * `q_factor` - Resonance quality factor.
    pub fn update_coefficients(
        &mut self,
        filter_type: FilterType,
        sample_rate: f32,
        frequency_hz: f32,
        q_factor: f32,
    ) {
        let safe_frequency = frequency_hz.clamp(MIN_FREQUENCY_HZ, sample_rate * MAX_NYQUIST_FACTOR);
        let angular_freq = 2.0 * PI * safe_frequency / sample_rate;
        let sin_w0 = angular_freq.sin();
        let cos_w0 = angular_freq.cos();
        let safe_q = q_factor.max(MIN_Q_FACTOR);
        let alpha = sin_w0 / (2.0 * safe_q);

        let (b0_raw, b1_raw, b2_raw, a0_raw, a1_raw, a2_raw) = match filter_type {
            FilterType::Lowpass => {
                let half_one_minus_cos = (1.0 - cos_w0) * 0.5;
                (
                    half_one_minus_cos,
                    1.0 - cos_w0,
                    half_one_minus_cos,
                    1.0 + alpha,
                    -2.0 * cos_w0,
                    1.0 - alpha,
                )
            }
            FilterType::Highpass => {
                let half_one_plus_cos = (1.0 + cos_w0) * 0.5;
                (
                    half_one_plus_cos,
                    -(1.0 + cos_w0),
                    half_one_plus_cos,
                    1.0 + alpha,
                    -2.0 * cos_w0,
                    1.0 - alpha,
                )
            }
        };

        // Resonance Gain Compensation:
        // High Q values boost the cutoff peak by a factor proportional to Q (+20 dB at Q=10).
        // Scaling numerator coefficients by 1/sqrt(Q / 0.707) preserves high Q selectivity while preventing
        // massive volume spikes that cause digital clipping on low-frequency transients.
        let q_gain_comp = if safe_q > REASONABLE_Q_THRESHOLD {
            1.0 / (safe_q / REASONABLE_Q_THRESHOLD).sqrt()
        } else {
            1.0
        };

        self.b0 = (b0_raw / a0_raw) * q_gain_comp;
        self.b1 = (b1_raw / a0_raw) * q_gain_comp;
        self.b2 = (b2_raw / a0_raw) * q_gain_comp;
        self.a1 = a1_raw / a0_raw;
        self.a2 = a2_raw / a0_raw;
    }

    /// Evaluates the Transposed Direct Form II biquad filter equations for a single audio sample.
    ///
    /// # Arguments
    /// * `sample` - Incoming floating-point audio sample value.
    ///
    /// # Returns
    /// Filtered floating-point audio sample.
    #[inline]
    pub fn process(&mut self, sample: f32) -> f32 {
        let output = self.b0 * sample + self.s1;

        self.s1 = self.b1 * sample - self.a1 * output + self.s2;
        self.s2 = self.b2 * sample - self.a2 * output;

        // Anti-denormal flushing: prevents subnormal float values from causing CPU spikes
        if self.s1.abs() < DENORMAL_FLUSH_THRESHOLD {
            self.s1 = 0.0;
        }
        if self.s2.abs() < DENORMAL_FLUSH_THRESHOLD {
            self.s2 = 0.0;
        }

        output
    }
}

impl Default for Biquad {
    fn default() -> Self {
        Self::new()
    }
}

/// Zero-allocation C1-continuous Soft Limiter for output peak protection.
///
/// Uses a continuous soft knee curve above `0.707` (-3 dBFS) with a strict `1.0` (0 dBFS) ceiling.
/// Guarantees zero step discontinuities (no clicking) and smooth harmonic saturation (no harsh square-wave clipping).
pub struct SoftLimiter {
    _unused: (),
}

impl SoftLimiter {
    /// Creates a new soft limiter instance.
    pub fn new() -> Self {
        Self { _unused: () }
    }

    /// Resets the soft limiter state.
    ///
    /// **Why `&mut self`:** Maintains API symmetry with `Biquad::reset()` so both
    /// DSP blocks can be reset identically in `MyFilter::reset()`.
    #[allow(clippy::unused_self)]
    pub fn reset(&mut self) {}

    /// Processes a single audio sample through a continuous soft-knee saturation curve.
    ///
    /// # Arguments
    /// * `sample` - Input audio sample.
    ///
    /// # Returns
    /// Output audio sample, smoothly compressed if exceeding -3 dBFS threshold, strictly capped at 0 dBFS.
    /// **Why `&mut self`:** Maintains API symmetry with `Biquad::process()` so the
    /// `process_channel_sample()` pipeline can call both identically.
    #[inline]
    #[allow(clippy::unused_self)]
    pub fn process(&mut self, sample: f32) -> f32 {
        let abs_sample = sample.abs();

        if abs_sample <= SOFT_LIMITER_THRESHOLD {
            sample
        } else {
            // C1-continuous soft knee curve:
            // f(x) = 0.707 + 0.293 * tanh((x - 0.707) / 0.293)
            // Ensures both value and derivative are 100% continuous at 0.707 boundary (no click/discontinuity).
            let excess = abs_sample - SOFT_LIMITER_THRESHOLD;
            let compressed = SOFT_LIMITER_THRESHOLD
                + SOFT_LIMITER_MARGIN * (excess / SOFT_LIMITER_MARGIN).tanh();
            sample.signum() * compressed
        }
    }
}

impl Default for SoftLimiter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_biquad_initialization_and_reset() {
        let mut biquad = Biquad::new();
        biquad.update_coefficients(FilterType::Lowpass, 44100.0, 1000.0, 0.707);

        let output = biquad.process(1.0);
        assert!(!output.is_nan());
        assert!(!output.is_infinite());

        biquad.reset();
        assert_eq!(biquad.s1, 0.0);
        assert_eq!(biquad.s2, 0.0);
    }

    #[test]
    fn test_biquad_frequency_clamping() {
        let mut biquad = Biquad::new();
        biquad.update_coefficients(FilterType::Lowpass, 44100.0, 100000.0, 0.707);
        let output = biquad.process(0.5);
        assert!(!output.is_nan());

        biquad.update_coefficients(FilterType::Highpass, 44100.0, 1.0, 0.707);
        let output_low = biquad.process(0.5);
        assert!(!output_low.is_nan());
    }

    #[test]
    fn test_soft_limiter_passthrough_and_clipping() {
        let mut limiter = SoftLimiter::new();

        let small_sample = 0.2;
        assert_eq!(limiter.process(small_sample), small_sample);

        let large_sample = 2.0;
        let limited = limiter.process(large_sample);
        assert!(limited < large_sample);
        assert!(limited <= 1.0);
        assert!(limited >= 0.707);
    }
}
