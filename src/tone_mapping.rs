use crate::tracer::{Color, ColorWavelength};
use crate::Float;

#[cfg(debug_assertions)]
use crate::tracer::Spectrum;

#[cfg(debug_assertions)]
const SUSPICIOUSLY_LARGE_VALUE: Float = 1_000.0;

/// Enum for different tone mappers
pub enum ToneMap {
    /// Applies no tone mapping
    NoMap,
    /// Clamps values to \[0,`arg`\]
    Clamp(Float),
    /// Reinhard tone mapping to luminance `M(l) = l / (1 + l)`
    Reinhard,
}

impl ToneMap {
    /// Tone maps the `rgb` sample with channels in `\[0,∞\]`
    pub fn map(&self, col: Color, lambda: &ColorWavelength) -> Color {
        #[cfg(debug_assertions)]
        if col.is_nan() {
            println!("Found NaN during tone mapping.");
            return Spectrum::GREEN.sample(lambda);
        }
        #[cfg(debug_assertions)]
        if col.is_neg() {
            println!("Found negative value during tone mapping.");
            return Spectrum::RED.sample(lambda);
        }
        #[cfg(debug_assertions)]
        if col.max() > SUSPICIOUSLY_LARGE_VALUE {
            println!("Found suspiciously large value during tone mapping.");
            return Spectrum::BLUE.sample(lambda);
        }
        match self {
            Self::NoMap => col,
            Self::Clamp(mx) => col.clamp(0.0, *mx),
            Self::Reinhard => col / (1.0 + col.luminance(lambda)),
        }
    }
}
