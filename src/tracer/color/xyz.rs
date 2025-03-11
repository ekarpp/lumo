use super::*;

pub type XYZ = Vec3;

pub const W_D65: Vec3 = illuminants::D65.to_xyz();
/// xy-coordinates of the D65 illuminant
pub const w_D65: Vec2 = to_xyY(W_D65);

/// XYZ values from xyY values
pub const fn from_xyY(xy: Vec2, Y: Float) -> XYZ {
    if xy.y == 0.0 {
        Vec3::ZERO
    } else {
        Vec3::new(
            xy.x * Y / xy.y,
            Y,
            (1.0 - xy.x - xy.y) * Y / xy.y,
        )
    }
}

/// xyY values from XYZ values
pub const fn to_xyY(xyz: XYZ) -> Vec2 {
    Vec2::new(
        xyz.x / (xyz.x + xyz.y + xyz.z),
        xyz.y / (xyz.x + xyz.y + xyz.z),
    )
}

pub mod cie1931 {
    use super::*;

    /// Integral of the XYZ 1931 Y-curve
    pub const Y_INTEGRAL: Float = 106.856895;

    /// XYZ 1931 X-curve from 360nm to 830nm at 5nm interval
    pub const X: DenseSpectrum = DenseSpectrum::new(samples::cie1931::X::SAMPLES);

    /// XYZ 1931 Y-curve from 360nm to 830nm at 5nm interval
    pub const Y: DenseSpectrum = DenseSpectrum::new(samples::cie1931::Y::SAMPLES);

    /// XYZ 1931 Z-curve from 360nm to 830nm at 5nm interval
    pub const Z: DenseSpectrum = DenseSpectrum::new(samples::cie1931::Z::SAMPLES);

}
