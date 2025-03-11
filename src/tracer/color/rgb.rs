use super::*;

#[derive(Clone)]
/// Represents a linear RGB value
pub struct RGB {
    rgb: Vec3,
}

impl RGB {
    /// Linear rgb values from vec3
    pub const fn from(rgb: Vec3) -> Self {
        Self { rgb }
    }

    /// Constant black color
    pub const BLACK: Self = RGB { rgb: Vec3::ZERO };
    /// Constant white color
    pub const WHITE: Self = RGB { rgb: Vec3::ONE };

    /// New `selfs` from channels
    #[inline]
    pub const fn new(r: Float, g: Float, b: Float) -> Self {
        Self {
            rgb: Vec3::new(r, g, b)
        }
    }

    /// Get the red channel
    #[inline]
    pub const fn r(&self) -> Float { self.rgb.x }

    /// Get the green channel
    #[inline]
    pub const fn g(&self) -> Float { self.rgb.y }

    /// Get the blue channel
    #[inline]
    pub const fn b(&self) -> Float { self.rgb.z }

    /// Get value for channel `c`
    #[inline]
    pub const fn c(&self, c: usize) -> Float {
        match c % 3 {
            0 => self.rgb.x,
            1 => self.rgb.y,
            2 => self.rgb.z,
            _ => unreachable!(),
        }
    }

    /// Decode gamma from a srgb value
    #[inline]
    pub fn srgb_decode(v: u8) -> Float {
        let u = v as Float / 255.0;
        if u <= 0.04045 {
            u / 12.92
        } else {
            ((u + 0.055) / 1.055).powf(2.4)
        }
    }

    /// Linear rgb value from srgb channels
    pub fn from_srgb(r: u8, g: u8, b: u8) -> Self {
        let rgb = Vec3::new(
            Self::srgb_decode(r),
            Self::srgb_decode(g),
            Self::srgb_decode(b),
        );
        Self { rgb }
    }

    /// Linear rgb value from rgbe bytes
    pub fn from_rgbe(r: u8, g: u8, b: u8, e: u8) -> Self {
        if e == 0 {
            Self::BLACK
        } else {
            let v = Float::powi(2.0, e as i32 - 128) / 256.0;
            let rgb = 0.5 + Vec3::new(
                v * r as Float,
                v * g as Float,
                v * b as Float,
            );

            Self { rgb }
        }
    }

    /// Are we black?
    #[inline]
    pub const fn is_black(&self) -> bool {
        self.rgb.x == 0.0
            && self.rgb.y == 0.0
            && self.rgb.z == 0.0
    }
}

impl AddAssign<&RGB> for RGB {
    fn add_assign(&mut self, rhs: &RGB) {
        self.rgb += rhs.rgb;
    }
}

impl Add for RGB {
    type Output = RGB;

    fn add(self, rhs: RGB) -> RGB {
        RGB { rgb: self.rgb + rhs.rgb }
    }
}

impl Mul<Float> for RGB {
    type Output = RGB;

    fn mul(self, rhs: Float) -> RGB {
        RGB { rgb: self.rgb * rhs }
    }
}

impl Mul<RGB> for Float {
    type Output = RGB;

    fn mul(self, rhs: RGB) -> RGB {
        RGB { rgb: self * rhs.rgb }
    }
}

impl Div<Float> for RGB {
    type Output = RGB;

    fn div(self, rhs: Float) -> RGB {
        RGB { rgb: self.rgb / rhs }
    }
}
