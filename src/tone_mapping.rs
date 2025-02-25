use crate::tracer::Color;

use crate::Float;
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
    pub fn map(&self, rgb: Color) -> Color {
        #[cfg(debug_assertions)]
        if rgb.rgb.is_nan() {
            println!("Found NaN during tone mapping.");
            return Color::GREEN;
        }
        #[cfg(debug_assertions)]
        if rgb.rgb.is_negative_bitmask() > 0 {
            println!("Found negative value during tone mapping.");
            return Color::RED;
        }
        #[cfg(debug_assertions)]
        if rgb.rgb.max_element() > SUSPICIOUSLY_LARGE_VALUE {
            println!("Found suspiciously large value during tone mapping.");
            return Color::BLUE;
        }
        match self {
            Self::NoMap => rgb,
            Self::Clamp(mx) => rgb.clamp(0.0, *mx),
            Self::Reinhard => rgb / (1.0 + rgb.luminance()),
        }
    }
}
