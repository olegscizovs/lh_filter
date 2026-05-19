use std::f32::consts::PI;

/// The filter type for our biquad implementation.
#[derive(Clone, Copy, PartialEq)]
pub enum FilterType {
    Lowpass,
    Highpass,
}

/// Standard RBJ (Robert Bristow-Johnson) Biquad filter implementation.
/// State is kept in x1, x2 (inputs) and y1, y2 (outputs).
pub struct Biquad {
    a1: f32,
    a2: f32,
    b0: f32,
    b1: f32,
    b2: f32,
    x1: f32,
    x2: f32,
    y1: f32,
    y2: f32,
}

impl Biquad {
    pub fn new() -> Self {
        Self {
            a1: 0.0, a2: 0.0,
            b0: 0.0, b1: 0.0, b2: 0.0,
            x1: 0.0, x2: 0.0,
            y1: 0.0, y2: 0.0,
        }
    }

    /// Resets all filter state to zero. Must be called from Plugin::reset().
    pub fn reset(&mut self) {
        self.x1 = 0.0;
        self.x2 = 0.0;
        self.y1 = 0.0;
        self.y2 = 0.0;
    }

    /// Calculates coefficients based on RBJ formulas.
    /// Frequency is clamped to 20Hz-Nyquist to avoid instability.
    pub fn update_coefficients(&mut self, f_type: FilterType, sample_rate: f32, frequency: f32, q: f32) {
        let frequency = frequency.clamp(20.0, sample_rate * 0.49); // Nyquist safety
        let w0 = 2.0 * PI * frequency / sample_rate;
        let sin_w0 = w0.sin();
        let cos_w0 = w0.cos();
        let alpha = sin_w0 / (2.0 * q.max(0.01));

        let (b0, b1, b2, a0, a1, a2) = match f_type {
            FilterType::Lowpass => {
                let b0 = (1.0 - cos_w0) / 2.0;
                let b1 = 1.0 - cos_w0;
                let b2 = (1.0 - cos_w0) / 2.0;
                let a0 = 1.0 + alpha;
                let a1 = -2.0 * cos_w0;
                let a2 = 1.0 - alpha;
                (b0, b1, b2, a0, a1, a2)
            }
            FilterType::Highpass => {
                let b0 = (1.0 + cos_w0) / 2.0;
                let b1 = -(1.0 + cos_w0);
                let b2 = (1.0 + cos_w0) / 2.0;
                let a0 = 1.0 + alpha;
                let a1 = -2.0 * cos_w0;
                let a2 = 1.0 - alpha;
                (b0, b1, b2, a0, a1, a2)
            }
        };

        // Normalize by a0
        self.b0 = b0 / a0;
        self.b1 = b1 / a0;
        self.b2 = b2 / a0;
        self.a1 = a1 / a0;
        self.a2 = a2 / a0;
    }

    /// Process a single sample through the filter.
    /// Inlined for performance in the process loop.
    #[inline(always)]
    pub fn process(&mut self, sample: f32) -> f32 {
        let out = self.b0 * sample + self.b1 * self.x1 + self.b2 * self.x2 - self.a1 * self.y1 - self.a2 * self.y2;

        // Shift history
        self.x2 = self.x1;
        self.x1 = sample;
        self.y2 = self.y1;
        self.y1 = out;

        out
    }
}

/// Soft limiter using tanh saturation to prevent clipping from resonance peaks.
/// Zero-allocation, stateless per-sample operation — safe for the audio thread.
///
/// The ceiling is set to 0 dBFS (1.0). Signals below the threshold pass through
/// with minimal coloration. Signals above are smoothly compressed via hyperbolic
/// tangent, which provides musically pleasant saturation rather than hard digital
/// clipping.
pub struct SoftLimiter {
    /// Release envelope state for smooth gain recovery
    envelope: f32,
}

impl SoftLimiter {
    pub fn new() -> Self {
        Self { envelope: 0.0 }
    }

    pub fn reset(&mut self) {
        self.envelope = 0.0;
    }

    /// Process a single sample through the soft limiter.
    /// Uses tanh saturation with a 0 dBFS ceiling.
    #[inline(always)]
    pub fn process(&mut self, sample: f32) -> f32 {
        let abs_sample = sample.abs();

        // Track the envelope with fast attack, slow release
        if abs_sample > self.envelope {
            // Instant attack — catch transients immediately
            self.envelope = abs_sample;
        } else {
            // Slow release (~5ms at 44.1kHz) for smooth recovery
            self.envelope = self.envelope * 0.9997 + abs_sample * 0.0003;
        }

        // Only engage saturation when signal exceeds ~-3 dBFS threshold
        if self.envelope > 0.707 {
            // tanh soft-clip: maps (-inf, +inf) → (-1.0, +1.0) smoothly
            sample.tanh()
        } else {
            sample
        }
    }
}
