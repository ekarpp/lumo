use crate::{Float, Vec2};
use std::fmt;

#[cfg(test)]
mod filter_tests;

/// Filters used to construct a pixel from samples
#[derive(Clone, Copy)]
pub enum PixelFilter {
    /// Constant square (a.k.a box) filter
    Square(Float),
    /// Triangle filter
    Triangle(Float),
    /// Gaussian filter
    Gaussian(Float, Float),
    /// Mitchell-Netravali filter
    Mitchell(Float, Float),
}

impl Default for PixelFilter {
    fn default() -> Self {
        Self::gaussian(1.5, 1.5 / 4.0)
    }
}

impl fmt::Display for PixelFilter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Square(r) => write!(f, "Square[r={:.3}]", r),
            Self::Triangle(r) => write!(f, "Triangle[r={:.3}]", r),
            Self::Gaussian(r, sigma) => write!(f, "Gaussian[r={:.3}, sigma={:.3}]", r, sigma),
            Self::Mitchell(r, b) => {
                let c = (1.0 - b) / 2.0;
                write!(f, "Mitchell[r={:.3}, b={:.3}, c={:.3}]", r, b, c)
            }
        }
    }
}

impl PixelFilter {
    /// Square (a.k.a box) filter with radius `r`
    pub fn square(r: Float) -> Self {
        assert!(r > 0.0);
        Self::Square(r)
    }

    /// Triangle filter with radius `r`
    pub fn triangle(r: Float) -> Self {
        assert!(r > 0.0);
        Self::Triangle(r)
    }

    /// Gaussian filter with radius `r` and std dev `sigma`
    pub fn gaussian(r: Float, sigma: Float) -> Self {
        assert!(r > 0.0);
        assert!(sigma > 0.0);
        Self::Gaussian(r, sigma)
    }

    /// Mitchell-Netravali filter with radius `r`, `b=b` and `c = (1.0 - b) / 2.0`
    pub fn mitchell(r: Float, b: Float) -> Self {
        assert!(r > 0.0);
        Self::Mitchell(r, b)
    }

    /// Returns the discretized radius of the filter
    /// i.e. how many pixels in each direction we should look for
    #[inline]
    pub fn r_disc(&self) -> u64 {
        match self {
            Self::Square(r)
                | Self::Triangle(r)
                | Self::Gaussian(r, ..)
                | Self::Mitchell(r, ..) => {
                    (*r - 0.5).ceil() as u64
            }
        }
    }

    /// Evaluate the filter at `px`
    #[inline]
    pub fn eval(&self, px: Vec2) -> Float {
        match self {
            Self::Square(r) => if px.x.abs() < *r && px.y.abs() < *r { 1.0 } else { 0.0 },
            Self::Triangle(r) => {
                let offset = (*r - px.abs()).max(Vec2::ZERO);
                offset.x * offset.y
            }
            Self::Gaussian(r, sigma) => {
                let gx = Self::gauss(px.x, sigma);
                let gy = Self::gauss(px.y, sigma);
                let gr = Self::gauss(*r, sigma);

                (gx - gr).max(0.0) * (gy - gr).max(0.0)
            }
            Self::Mitchell(r, b) => {
                let c = (1.0 - b) / 2.0;
                Self::mitch(2.0 * px.x / r, b, c) * Self::mitch(2.0 * px.y / r, b, c)
            }
        }
    }

    /// Integral of the filter over the whole space
    #[inline]
    pub fn integral(&self) -> Float {
        match self {
            Self::Square(r) => 2.0 * r * 2.0 * r,
            Self::Triangle(r) => r * r * r * r,
            Self::Mitchell(r, _) => r * r * 0.25,
            Self::Gaussian(r, sigma) => {
                let ig = Self::gauss_ig(-r, r, sigma);
                let gr = Self::gauss(*r, sigma);
                (ig - 2.0 * r * gr).powi(2)
            }
        }
    }

    #[inline]
    fn gauss(x: Float, sigma: &Float) -> Float {
        #[cfg(debug_assertions)]
        assert!(sigma > &0.0);

        (-x.powi(2) / (2.0 * sigma * sigma)).exp()
            / (2.0 * crate::PI * sigma * sigma).max(0.0).sqrt()
    }

    #[inline]
    fn gauss_ig(x0: Float, x1: &Float, sigma: &Float) -> Float {
        let denom = sigma * Float::sqrt(2.0);

        0.5 * (libm::erf((-x0 / denom) as f64) - libm::erf((-x1 / denom) as f64)) as Float
    }

    #[inline]
    fn mitch(x: Float, b: &Float, c: Float) -> Float {
        let x = x.abs();
        let p = if x < 1.0 {
            (12.0 - 9.0 * b - 6.0 * c) * x.powi(3)
                + (-18.0 + 12.0 * b + 6.0 * c) * x.powi(2)
                + (6.0 - 2.0 * b)
        } else if x < 2.0 {
            (-b - 6.0 * c) * x.powi(3)
                + (6.0 * b + 30.0 * c) * x.powi(2)
                + (-12.0 * b - 48.0 * c) * x
                + (8.0 * b + 24.0 * c)
        } else {
            0.0
        };

        p / 6.0
    }
}
