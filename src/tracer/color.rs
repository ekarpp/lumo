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
    /// Red color
    pub const RED: Self = Self { rgb: Vec3::X };
    /// Green color
    pub const GREEN: Self = Self { rgb: Vec3::Y };
    /// Blue color
    pub const BLUE: Self = Self { rgb: Vec3::Z };


    /// Decodes 8-bit sRGB encoded `r`, `g`, and `b` channels to linear RGB.
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        let dec = |v: u8| -> Float {
            let u = v as Float / 255.0;
            if u <= 0.04045 {
                u / 12.92
            } else {
                ((u + 0.055) / 1.055).powf(2.4)
            }
        };

        Self {
            rgb: Vec3::new(dec(r), dec(g), dec(b))
        }
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
        let enc = |v: Float| -> u8 {
            let ev = if v <= 0.0031308 {
                12.92 * v
            } else {
                1.055 * v.powf(1.0 / 2.4) - 0.055
            };

            (ev * 255.0) as u8
        };

        (enc(self.rgb.x), enc(self.rgb.y), enc(self.rgb.z))
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

impl Sub<Color> for Float {
    type Output = Color;

    fn sub(self, rhs: Color) -> Self::Output {
        Color { rgb: self - rhs.rgb }
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

impl MulAssign<Float> for Color {
    fn mul_assign(&mut self, rhs: Float) {
        self.rgb *= rhs;
    }
}

impl Div for Color {
    type Output = Self;

    fn div(self, rhs: Color) -> Self::Output {
        Self { rgb: self.rgb / rhs.rgb }
    }
}
