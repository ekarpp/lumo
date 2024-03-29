use crate::{Vec3, Float};
use std::ops::{
    Add, AddAssign, Sub, SubAssign,
    Mul, MulAssign, Div, DivAssign
};

#[derive(Clone, Copy)]
/// Abstraction for color using linear RGB values
pub struct Color {
    /// The linear RGB values
    pub rgb: Vec3,
}

impl Color {
    /// Black color
    pub const BLACK: Self = Self { rgb: Vec3::ZERO };
    /// White color
    pub const WHITE: Self = Self { rgb: Vec3::ONE };

    /// Decodes 8-bit sRGB encoded `r`, `g`, and `b` channels to linear RGB.
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        let rgb = Vec3::new(
            (r as Float / 255.0).powf(2.2),
            (g as Float / 255.0).powf(2.2),
            (b as Float / 255.0).powf(2.2),
        );

        Self { rgb }
    }

    /// Splats `value` to each RGB channel
    pub fn splat(value: Float) -> Self {
        Self { rgb: Vec3::splat(value) }
    }

    /// Maps linear RGB value to luminance
    pub fn luminance(&self) -> Float {
        self.rgb.dot(Vec3::new(0.2126, 0.7152, 0.0722))
    }

    /// LERP `self` with `other` using `c` as the coefficient
    pub fn lerp(&self, other: Self, c: Float) -> Self {
        Self { rgb: self.rgb.lerp(other.rgb, c) }
    }

    /// Gamma encodes self
    pub fn gamma_enc(&self) -> (u8, u8, u8) {
        let enc = self.rgb.powf(1.0 / 2.2) * 255.0;

        (enc.x as u8, enc.y as u8, enc.z as u8)
    }

    /// Clamps RGB channels between `lb` and `ub`
    pub fn clamp(&self, lb: Float, ub: Float) -> Self {
        Self { rgb: self.rgb.clamp(Vec3::splat(lb), Vec3::splat(ub)) }
    }

    /// Is the color black?
    pub fn is_black(&self) -> bool {
        self.rgb.length_squared() == 0.0
    }

    /// Mean of the RGB channel values
    pub fn mean(&self) -> Float {
        self.rgb.dot(Vec3::ONE) / 3.0
    }
}

impl From<Vec3> for Color {
    fn from(value: Vec3) -> Self {
        Self { rgb: value }
    }
}

impl Div<Float> for Color {
    type Output = Self;

    fn div(self, rhs: Float) -> Self::Output {
        Self { rgb: self.rgb / rhs }
    }
}

impl DivAssign<Float> for Color {
    fn div_assign(&mut self, rhs: Float) {
        self.rgb /= rhs;
    }
}

impl Mul<Float> for Color {
    type Output = Self;

    fn mul(self, rhs: Float) -> Self::Output {
        Self { rgb: self.rgb * rhs }
    }
}

impl Mul<Color> for Float {
    type Output = Color;

    fn mul(self, rhs: Color) -> Self::Output {
        Color { rgb: rhs.rgb * self }
    }
}

impl Add<Float> for Color {
    type Output = Self;

    fn add(self, rhs: Float) -> Self::Output {
        Self { rgb: self.rgb + rhs }
    }
}

impl Add<Color> for Float {
    type Output = Color;

    fn add(self, rhs: Color) -> Self::Output {
        Color { rgb: rhs.rgb + self }
    }
}

impl Sub<Float> for Color {
    type Output = Self;

    fn sub(self, rhs: Float) -> Self::Output {
        Self { rgb: self.rgb - rhs }
    }
}

impl Add for Color {
    type Output = Self;

    fn add(self, rhs: Color) -> Self::Output {
        Self { rgb: self.rgb + rhs.rgb }
    }
}

impl AddAssign for Color {
    fn add_assign(&mut self, rhs: Color) {
        self.rgb += rhs.rgb;
    }
}

impl Sub for Color {
    type Output = Self;

    fn sub(self, rhs: Color) -> Self::Output {
        Self { rgb: self.rgb - rhs.rgb }
    }
}

impl SubAssign for Color {
    fn sub_assign(&mut self, rhs: Color) {
        self.rgb -= rhs.rgb;
    }
}

impl Mul for Color {
    type Output = Self;

    fn mul(self, rhs: Color) -> Self::Output {
        Self { rgb: self.rgb * rhs.rgb }
    }
}

impl MulAssign for Color {
    fn mul_assign(&mut self, rhs: Color) {
        self.rgb *= rhs.rgb;
    }
}

impl Div for Color {
    type Output = Self;

    fn div(self, rhs: Color) -> Self::Output {
        Self { rgb: self.rgb / rhs.rgb }
    }
}
