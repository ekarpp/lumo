use super::*;

#[derive(Clone)]
/// Represents a linear RGB value
pub struct RGB {
    rgb: Vec3,
}

impl From<Vec3> for RGB {
    fn from(rgb: Vec3) -> Self {
        Self { rgb }
    }
}

impl RGB {
    /// Constant black color
    pub const BLACK: Self = RGB { rgb: Vec3::ZERO };
    /// Constant white color
    pub const WHITE: Self = RGB { rgb: Vec3::ONE };

    /// Get the red channel
    pub fn r(&self) -> Float { self.rgb.x }

    /// Get the green channel
    pub fn g(&self) -> Float { self.rgb.y }

    /// Get the blue channel
    pub fn b(&self) -> Float { self.rgb.z }
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
