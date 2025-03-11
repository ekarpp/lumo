use super::*;
use std::str::FromStr;

mod tables;

#[cfg(test)]
mod spectrum_tests;

type TexFloat = f32;

#[derive(Clone)]
/// Represents a color spectrum at `BINS` bins of uniform width
/// between `LAMBDA_MIN` and `LAMBDA_MAX` wavelengths.
pub struct Spectrum {
    c0: TexFloat,
    c1: TexFloat,
    c2: TexFloat,
    scale: TexFloat,
}

impl Spectrum {
    /// White spectrum
    pub const WHITE: Self = Self::_from_rgb(1.0, 1.0, 1.0);
    /// Black spectrum
    pub const BLACK: Self = Self { c0: 0.0, c1: 0.0, c2: 0.0, scale: 0.0 };
    /// Red spectrum
    pub const RED: Self = Self::_from_rgb(1.0, 0.0, 0.0);
    /// Green spectrum
    pub const GREEN: Self = Self::_from_rgb(0.0, 1.0, 0.0);
    /// Blue spectrum
    pub const BLUE: Self = Self::_from_rgb(0.0, 0.0, 1.0);
    /// Red spectrum
    pub const YELLOW: Self = Self::_from_rgb(1.0, 1.0, 0.0);
    /// Blue spectrum
    pub const MAGENTA: Self = Self::_from_rgb(1.0, 0.0, 1.0);
    /// Green spectrum
    pub const CYAN: Self = Self::_from_rgb(0.0, 1.0, 1.0);

    /// Create spectrum value from sRGB channels
    pub fn from_srgb(r: u8, g: u8, b: u8) -> Self {
        let rgb = RGB::from_srgb(r, g, b);
        Self::from_rgb(rgb)
    }

    const fn _from_rgb(r: Float, g: Float, b: Float) -> Self {
        let rgb = RGB::new(r, g, b);
        Self::from_rgb(rgb)
    }

    /// Create spectrum value from linear RGB value
    // Jakob & Hanika 2019
    pub const fn from_rgb(rgb: RGB) -> Self {
        let maxc = if rgb.r() > rgb.g() { 0 } else { 1 };
        let maxc = if rgb.c(maxc) > rgb.b() { maxc } else { 2 };
        if rgb.c(maxc) == 0.0 || rgb.is_black() {
            return Spectrum::BLACK;
        }

        let scale = if rgb.c(maxc) > 1.0 {
            2.0 * rgb.c(maxc) as TexFloat
        } else {
            1.0
        };

        let mx = rgb.c(maxc) as TexFloat;
        let (c0, c1, c2) = tables::srgb::eval(
            maxc,
            rgb.c(maxc + 1) as TexFloat / mx,
            rgb.c(maxc + 2) as TexFloat / mx,
            mx / scale,
        );
        Self { c0, c1, c2, scale, }
    }

    const fn from_XYZ(xyz: XYZ) -> Self {
        Self::from_rgb(ColorSpace::sRGB.from_XYZ(xyz))
    }

    /// Parse string of format `(<wavelength>:<intensity> )*` to a spectrum
    pub fn from_pts(pts: &str) -> Self {
        let mut pairs: Vec<(Float, Float)> = pts.split_whitespace()
            .filter_map(|pt| {
                let (lambda, i) = pt.split_once(":")?;
                let lambda = Float::from_str(lambda).ok()?;
                let i = Float::from_str(i).ok()?;
                Some( (lambda, i) )
            })
            .collect();

        pairs.sort_by(|l, r| l.0.total_cmp(&r.0));

        let dense_spec = DenseSpectrum::from_points(pairs);

        Self::from_XYZ(dense_spec.to_xyz())
    }

    /// Samples `self` at `lambda` wavelengths
    pub fn sample(&self, lambda: &ColorWavelength) -> Color {
        let samples: [Float; SPECTRUM_SAMPLES] = lambda.iter()
            .map(|wl| self.sample_one(*wl))
            .collect::<Vec<Float>>().try_into().unwrap();
        Color::from(samples)
    }

    /// Samples self at a single wavelength `lambda`
    #[inline]
    pub fn sample_one(&self, lambda: Float) -> Float {
        let lambda = lambda as TexFloat;
        (self.scale
         * self.sigmoid(self.c0 * lambda * lambda + self.c1 * lambda + self.c2))
            as Float
    }

    #[inline]
    fn sigmoid(&self, x: TexFloat) -> TexFloat {
        0.5 + x / (2.0 * (1.0 + x.powi(2)).sqrt())
    }

    /// Are we a constant black spectrum?
    #[inline]
    pub fn is_black(&self) -> bool {
        self.scale == 0.0
    }
}

impl Mul<Float> for Spectrum {
    type Output = Self;

    fn mul(self, rhs: Float) -> Self::Output {
        Self {
            c0: self.c0,
            c1: self.c1,
            c2: self.c2,
            scale: self.scale * rhs as TexFloat,
        }
    }
}

impl Mul<Spectrum> for Float {
    type Output = Spectrum;

    fn mul(self, rhs: Spectrum) -> Self::Output {
        Spectrum {
            c0: rhs.c0,
            c1: rhs.c1,
            c2: rhs.c2,
            scale: self as TexFloat * rhs.scale,
        }
    }
}
