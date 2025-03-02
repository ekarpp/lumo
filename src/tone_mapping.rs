use crate::tracer::FilmSample;
use crate::Float;
use std::fmt;

#[cfg(debug_assertions)]
use crate::tracer::Spectrum;

#[cfg(debug_assertions)]
const SUSPICIOUSLY_LARGE_VALUE: Float = 1_000.0;

/// Enum for different tone mappers
#[derive(Clone)]
pub enum ToneMap {
    /// Applies no tone mapping
    NoMap,
    /// Clamps values to \[0,`arg`\]
    Clamp(Float),
    /// Reinhard tone mapping to luminance `M(l) = l / (1 + l)`
    Reinhard,
}

impl fmt::Display for ToneMap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoMap => write!(f, "no map"),
            Self::Clamp(arg) => write!(f, "clamp[{}]", arg),
            Self::Reinhard => write!(f, "Reinhard"),
        }
    }
}

impl Default for ToneMap {
    fn default() -> Self { Self::NoMap }
}

impl ToneMap {
    /// Tone maps the `rgb` sample with channels in `\[0,∞\]`
    pub fn map(&self, sample: &mut FilmSample) {
        let color = sample.color;
        let lambda = &sample.lambda;
        #[cfg(debug_assertions)]
        if color.is_nan() {
            println!("Found NaN during tone mapping.");
            sample.color = Spectrum::GREEN.sample(lambda);
        }
        #[cfg(debug_assertions)]
        if color.is_neg() {
            println!("Found negative value during tone mapping.");
            sample.color = Spectrum::RED.sample(lambda);
        }
        #[cfg(debug_assertions)]
        if color.max() > SUSPICIOUSLY_LARGE_VALUE {
            println!("Found suspiciously large value during tone mapping.");
            sample.color = Spectrum::BLUE.sample(lambda);
        }
        sample.color = match self {
            Self::NoMap => color,
            Self::Clamp(mx) => color.clamp(0.0, *mx),
            Self::Reinhard => color / (1.0 + color.luminance(lambda)),
        }
    }
}
