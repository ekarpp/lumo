use crate::{Vec3, Float, Mat3, Vec2};
use std::ops::{
    Add, AddAssign, Sub, SubAssign, Neg,
    Mul, MulAssign, Div, DivAssign
};
use std::fmt;

pub use wavelength::ColorWavelength;
pub use spectrum::Spectrum;
pub use rgb::RGB;
pub use space::ColorSpace;
pub use dense_spectrum::DenseSpectrum;
use xyz::XYZ;

pub mod illuminants;
mod samples;
mod space;
mod spectrum;
mod dense_spectrum;
mod rgb;
mod xyz;
mod wavelength;

const LAMBDA_MIN: Float = 360.0;
const LAMBDA_MAX: Float = 830.0;
const SPECTRUM_SAMPLES: usize = 4;

#[derive(Clone, Copy)]
/// Abstraction for color using `SPECTRUM_SAMPLES` from a spectrum
pub struct Color {
    samples: [Float; SPECTRUM_SAMPLES],
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Samples[{:?}]", self.samples)
    }
}

impl From<[Float; SPECTRUM_SAMPLES]> for Color {
    fn from(samples: [Float; SPECTRUM_SAMPLES]) -> Self {
        Self { samples }
    }
}

impl Color {
    /// Color sampled from the constant 1.0 spectrum
    pub const WHITE: Self = Self { samples: [1.0; SPECTRUM_SAMPLES] };
    /// Color sampled from the constant 0.0 spectrum
    pub const BLACK: Self = Self { samples: [0.0; SPECTRUM_SAMPLES] };

    /// Maps linear RGB value to luminance
    #[inline]
    pub fn luminance(&self, lambda: &ColorWavelength) -> Float {
        let pdf = lambda.pdf();
        (xyz::cie1931::Y.sample(lambda) * *self / pdf).mean() / xyz::cie1931::Y_INTEGRAL
    }

    /// `self` at `lambda` mapped to the XYZ color space
    pub fn xyz(&self, lambda: &ColorWavelength) -> XYZ {
        let pdf = lambda.pdf();

        XYZ::new(
            (xyz::cie1931::X.sample(lambda) * *self / pdf).mean(),
            (xyz::cie1931::Y.sample(lambda) * *self / pdf).mean(),
            (xyz::cie1931::Z.sample(lambda) * *self / pdf).mean()
        ) / xyz::cie1931::Y_INTEGRAL
    }

    /// Does `self` have NaN values
    #[inline]
    pub fn is_nan(&self) -> bool {
        self.samples.iter().any(|v| v.is_nan())
    }

    /// Does `self` have negative values
    #[inline]
    pub fn is_neg(&self) -> bool {
        self.samples.iter().any(|v| v < &0.0)
    }

    /// Is the "maxiumum" of `self` less than `v`
    #[inline]
    pub fn max(&self) -> Float {
        self.samples.iter().fold(crate::NEG_INF, |acc, v| acc.max(*v))
    }

    /// Is the "maximum" of `self` greater than `v`
    #[inline]
    pub fn min(&self) -> Float {
        self.samples.iter().fold(crate::INF, |acc, v| acc.min(*v))
    }

    /// Is the color black?
    #[inline]
    pub fn is_black(&self) -> bool {
        self.samples.iter().all(|v| v == &0.0)
    }

    /// Mean of the samples
    #[inline]
    pub fn mean(&self) -> Float {
        self.samples.iter().sum::<Float>() / SPECTRUM_SAMPLES as Float
    }

    /// Clamp the samples to `[mi,mx]`
    #[inline]
    pub fn clamp(&self, mi: Float, mx: Float) -> Self {
        let samples = self.samples.iter()
            .map(|v| v.clamp(mi, mx))
            .collect::<Vec<Float>>().try_into().unwrap();
        Self { samples }
    }

    /// Map samples of `self` to `exp(sample)`
    #[inline]
    pub fn exp(mut self) -> Self {
        for i in 0..SPECTRUM_SAMPLES {
            self.samples[i] = self.samples[i].exp();
        }
        self
    }
}

impl Neg for Color {
    type Output = Self;

    fn neg(self) -> Self::Output {
        let samples = self.samples.iter()
            .map(|v| -v)
            .collect::<Vec<Float>>().try_into().unwrap();
        Self { samples }
    }
}

macro_rules! impl_op {
    ( $( $op_trait:ident, $op_fn:ident, $op:tt, $an_trait:ident, $an_fn:ident, $an:tt),* ) => {
        $(
            impl $an_trait for Color {
                #[inline]
                fn $an_fn(&mut self, rhs: Color) {
                    for i in 0..SPECTRUM_SAMPLES {
                        self.samples[i] $an rhs.samples[i];
                    }
                }
            }

            impl $an_trait<Float> for Color {
                #[inline]
                fn $an_fn(&mut self, rhs: Float) {
                    for i in 0..SPECTRUM_SAMPLES {
                        self.samples[i] $an rhs;
                    }
                }
            }

            impl $op_trait for Color {
                type Output = Self;
                #[inline]
                fn $op_fn(self, rhs: Self) -> Self::Output {
                    let samples: [Float; SPECTRUM_SAMPLES] = self.samples.iter()
                        .zip(rhs.samples.iter())
                        .map(|(lhs, rhs)| lhs $op rhs)
                        .collect::<Vec<Float>>().try_into().unwrap();
                    Self { samples }

                }
            }

            impl $op_trait<Float> for Color {
                type Output = Self;
                #[inline]
                fn $op_fn(self, rhs: Float) -> Self::Output {
                    let samples: [Float; SPECTRUM_SAMPLES] = self.samples.iter()
                        .map(|v| v $op rhs)
                        .collect::<Vec<Float>>().try_into().unwrap();
                    Self { samples }

                }
            }

            impl $op_trait<Color> for Float {
                type Output = Color;
                #[inline]
                fn $op_fn(self, rhs: Color) -> Self::Output {
                    let samples: [Float; SPECTRUM_SAMPLES] = rhs.samples.iter()
                        .map(|v| self $op v)
                        .collect::<Vec<Float>>().try_into().unwrap();
                    Color { samples }
                }
            }
        )*
    }
}

impl_op! {
    Add, add, +, AddAssign, add_assign, +=,
    Sub, sub, -, SubAssign, sub_assign, -=,
    Mul, mul, *, MulAssign, mul_assign, *=
}

impl Div for Color {
    type Output = Self;

    fn div(self, rhs: Color) -> Self::Output {
        let samples: [Float; SPECTRUM_SAMPLES] = self.samples.iter()
            .zip(rhs.samples.iter())
            .map(|(lhs, rhs)| { if *rhs == 0.0 { 0.0 } else { lhs / rhs }})
            .collect::<Vec<Float>>().try_into().unwrap();
        Self { samples }
    }
}

impl Div<Float> for Color {
    type Output = Self;

    fn div(self, rhs: Float) -> Self::Output {
        let samples: [Float; SPECTRUM_SAMPLES] = self.samples.iter()
            .map(|v| v / rhs)
            .collect::<Vec<Float>>().try_into().unwrap();
        Self { samples }
    }
}

impl DivAssign<Float> for Color {
    fn div_assign(&mut self, rhs: Float) {
        for i in 0..SPECTRUM_SAMPLES {
            if rhs == 0.0 {
                self.samples[i] = 0.0;
            } else {
                self.samples[i] /= rhs;
            }
        }
    }
}
