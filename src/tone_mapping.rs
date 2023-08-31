use crate::tracer::Color;

/// Enum for different tone mappers
pub enum ToneMap {
    /// Applies no tone mapping
    NoMap,
    /// Clamps values to \[0,1\]
    Clamp,
    /// Reinhard tone mapping to luminance `M(l) = l / (1 + l)`
    Reinhard,
    /// Hable tone map fron Uncharted 2
    HableFilmic,
    /// ACES (Academy Color Encoding System) approximation by Krzysztof Narkowicz
    ACES,
}

impl ToneMap {
    /// Tone maps the `rgb` sample with channels in `\[0,∞\]`
    pub fn map(&self, rgb: Color) -> Color {
        #[cfg(debug_assertions)]
        if rgb.rgb.is_nan() {
            println!("Found NaN during tone mapping.");
            return Color::new(0, 255, 0);
        }
        #[cfg(debug_assertions)]
        if rgb.rgb.is_negative_bitmask() > 0 {
            println!("Found negative value during tone mapping.");
            return Color::new(255, 0, 0);
        }
        match self {
            Self::NoMap => rgb,
            Self::Clamp => rgb.clamp(0.0, 1.0),
            Self::Reinhard => rgb / (1.0 + rgb.luminance()),
            Self::HableFilmic => {
                let exposure = 2.0;
                let curr = Self::hable_partial(rgb * exposure);
                let white = Color::splat(11.2);
                let white_scale = Color::WHITE / Self::hable_partial(white);
                curr * white_scale
            }
            Self::ACES => {
                let rgb = rgb * 0.6;
                let a = 2.51;
                let b = 0.03;
                let c = 2.43;
                let d = 0.59;
                let e = 0.14;
                ((rgb * (rgb * a + b)) / (rgb * (rgb * c + d) + e))
                    .clamp(0.0, 1.0)
            }
        }
    }

    fn hable_partial(rgb: Color) -> Color {
        let a = 0.15;
        let b = 0.50;
        let c = 0.10;
        let d = 0.20;
        let e = 0.02;
        let f = 0.30;

        (rgb * (rgb * a + c * b) + d * e) / (rgb * (rgb * a + b) + d * f) - e / f
    }
}
