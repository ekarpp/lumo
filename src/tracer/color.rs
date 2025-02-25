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

mod space;
mod spectrum;
mod rgb;
mod xyz;
mod wavelength;

const LAMBDA_MIN: Float = 380.0;
const LAMBDA_MAX: Float = 700.0;
const SPECTRUM_SAMPLES: usize = 4;

#[derive(Clone, Copy)]
/// Abstraction for color using `SPECTRUM_SAMPLES` from a spectrum
pub struct Color {
    samples: [Float; SPECTRUM_SAMPLES],
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "RBG[{:?}]", self.samples)
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
    pub fn luminance(&self, lambda: &ColorWavelength) -> Float {
        let y = xyz::Y(lambda);
        let pdf = lambda.pdf();

        (y * *self).mean() / (pdf * xyz::CIE_Y)
    }

    /// `self` at `lambda` mapped to the XYZ color space
    pub fn xyz(&self, lambda: &ColorWavelength) -> Vec3 {
        let x = xyz::X(lambda);
        let y = xyz::Y(lambda);
        let z = xyz::Z(lambda);
        let pdf = lambda.pdf();

        Vec3::new((x * *self).mean(), (y * *self).mean(), (z * *self).mean())
            / (pdf * xyz::CIE_Y)
    }

    /// Does `self` have NaN values
    pub fn is_nan(&self) -> bool {
        self.samples.iter().any(|v| v.is_nan())
    }

    /// Does `self` have negative values
    pub fn is_neg(&self) -> bool {
        self.samples.iter().any(|v| v < &0.0)
    }

    /// Is the "maxiumum" of `self` less than `v`
    pub fn max(&self) -> Float {
        self.samples.iter().fold(crate::NEG_INF, |acc, v| acc.max(*v))
    }

    /// Is the "maximum" of `self` greater than `v`
    pub fn min(&self) -> Float {
        self.samples.iter().fold(crate::INF, |acc, v| acc.min(*v))
    }

    /// Is the color black?
    pub fn is_black(&self) -> bool {
        self.samples.iter().all(|v| v == &0.0)
    }

    /// Mean of the samples
    pub fn mean(&self) -> Float {
        self.samples.iter().sum::<Float>() / SPECTRUM_SAMPLES as Float
    }

    /// Clamp the samples to `[mi,mx]`
    pub fn clamp(&self, mi: Float, mx: Float) -> Self {
        let samples = self.samples.iter()
            .map(|v| v.clamp(mi, mx))
            .collect::<Vec<Float>>().try_into().unwrap();
        Self { samples }
    }

    /// Map samples of `self` to `exp(sample)`
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

impl Div<Float> for Color {
    type Output = Self;

    fn div(self, rhs: Float) -> Self::Output {
        let samples = self.samples.iter()
            .map(|v| v / rhs)
            .collect::<Vec<Float>>().try_into().unwrap();
        Self { samples }
    }
}

impl DivAssign<Float> for Color {
    fn div_assign(&mut self, rhs: Float) {
        for i in 0..SPECTRUM_SAMPLES {
            self.samples[i] /= rhs;
        }
    }
}

impl Mul<Float> for Color {
    type Output = Self;

    fn mul(self, rhs: Float) -> Self::Output {
        let samples = self.samples.iter()
            .map(|v| v * rhs)
            .collect::<Vec<Float>>().try_into().unwrap();
        Self { samples }
    }
}

impl Mul<Color> for Float {
    type Output = Color;

    fn mul(self, rhs: Color) -> Self::Output {
        let samples = rhs.samples.iter()
            .map(|v| v * self)
            .collect::<Vec<Float>>().try_into().unwrap();
        Color { samples }
    }
}

impl MulAssign<Float> for Color {
    fn mul_assign(&mut self, rhs: Float) {
        for i in 0..SPECTRUM_SAMPLES {
            self.samples[i] *= rhs;
        }
    }
}

impl Add<Float> for Color {
    type Output = Self;

    fn add(self, rhs: Float) -> Self::Output {
        let samples = self.samples.iter()
            .map(|v| v + rhs)
            .collect::<Vec<Float>>().try_into().unwrap();
        Self { samples }
    }
}

impl Add<Color> for Float {
    type Output = Color;

    fn add(self, rhs: Color) -> Self::Output {
        let samples = rhs.samples.iter()
            .map(|v| v + self)
            .collect::<Vec<Float>>().try_into().unwrap();
        Color { samples }
    }
}

impl Sub<Float> for Color {
    type Output = Self;

    fn sub(self, rhs: Float) -> Self::Output {
        let samples = self.samples.iter()
            .map(|v| v - rhs)
            .collect::<Vec<Float>>().try_into().unwrap();
        Self { samples }
    }
}

impl Sub<Color> for Float {
    type Output = Color;

    fn sub(self, rhs: Color) -> Self::Output {
        let samples = rhs.samples.iter()
            .map(|v| self - v)
            .collect::<Vec<Float>>().try_into().unwrap();
        Color { samples }
    }
}

impl Add for Color {
    type Output = Self;

    fn add(self, rhs: Color) -> Self::Output {
        let samples = self.samples.iter().zip(rhs.samples.iter())
            .map(|(lhs, rhs)| { lhs + rhs })
            .collect::<Vec<Float>>().try_into().unwrap();
        Self { samples }
    }
}

impl AddAssign for Color {
    fn add_assign(&mut self, rhs: Color) {
        for i in 0..SPECTRUM_SAMPLES {
            self.samples[i] += rhs.samples[i];
        }
    }
}

impl Sub for Color {
    type Output = Self;

    fn sub(self, rhs: Color) -> Self::Output {
        let samples = self.samples.iter().zip(rhs.samples.iter())
            .map(|(lhs, rhs)| { lhs - rhs })
            .collect::<Vec<Float>>().try_into().unwrap();
        Self { samples }
    }
}

impl SubAssign for Color {
    fn sub_assign(&mut self, rhs: Color) {
        for i in 0..SPECTRUM_SAMPLES {
            self.samples[i] -= rhs.samples[i];
        }
    }
}

impl Mul for Color {
    type Output = Self;

    fn mul(self, rhs: Color) -> Self::Output {
        let samples = self.samples.iter().zip(rhs.samples.iter())
            .map(|(lhs, rhs)| { lhs * rhs })
            .collect::<Vec<Float>>().try_into().unwrap();
        Self { samples }
    }
}

impl MulAssign for Color {
    fn mul_assign(&mut self, rhs: Color) {
        for i in 0..SPECTRUM_SAMPLES {
            self.samples[i] *= rhs.samples[i];
        }
    }
}

impl Div for Color {
    type Output = Self;

    fn div(self, rhs: Color) -> Self::Output {
        let samples = self.samples.iter().zip(rhs.samples.iter())
            .map(|(lhs, rhs)| { lhs / rhs })
            .collect::<Vec<Float>>().try_into().unwrap();
        Self { samples }
    }
}
