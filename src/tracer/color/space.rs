#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
use super::*;

enum TransferFunction {
    sRGB, rec_2020
}

impl TransferFunction {
    pub fn apply(&self, c: Float) -> u8 {
        match self {
            Self::sRGB => {
                let ec = if c <= 0.0031308 {
                    12.92 * c
                } else {
                    1.055 * c.powf(1.0 / 2.4) - 0.055
                };

                (ec * 255.0) as u8
            }
            Self::rec_2020 => {
                let beta = 0.018053968510807;
                let alpha = 1.0 + 5.5 * beta;
                let ec = if c <= beta {
                    4.5 * c
                } else {
                    alpha * c.powf(0.45) - (alpha - 1.0)
                };

                // TODO: 10 or 12 bit depth is specified
                (ec * 255.0) as u8
            }
        }
    }
}

/// Represents a color space
pub struct ColorSpace {
    XYZ_to_RGB: Mat3,
    trc: TransferFunction,
    name: String,
}

impl Default for ColorSpace {
    fn default() -> Self {
        Self::dci_p3()
    }
}

impl fmt::Display for ColorSpace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl ColorSpace {
    /// Transform `color` sampled at `lambda` to the colorspace of `self`
    pub fn from_color(&self, color: &Color, lambda: &ColorWavelength) -> RGB {
        let xyz = color.xyz(lambda);
        let rgb = self.XYZ_to_RGB.mul_vec3(xyz);

        RGB::from(rgb)
    }

    /// Apply the tone reproduction curve of `self` to the linear `rgb`
    pub fn encode(&self, rgb: RGB) -> (u8, u8, u8) {
        (
            self.trc.apply(rgb.r()),
            self.trc.apply(rgb.g()),
            self.trc.apply(rgb.b()),
        )
    }

    fn init(
        r: Vec2,
        g: Vec2,
        b: Vec2,
        w: Vec2,
        trc: TransferFunction,
        name: String
    ) -> Self {
        let R = xyz::from_xyY(r, 1.0);
        let G = xyz::from_xyY(g, 1.0);
        let B = xyz::from_xyY(b, 1.0);
        let W = xyz::from_xyY(w, 1.0);

        let RGB_c = Mat3::new(R, G, B).transpose();
        let C = RGB_c.inv().mul_vec3(W);

        let RGB_to_XYZ = RGB_c * Mat3::diag(C);
        let XYZ_to_RGB = RGB_to_XYZ.inv();

        Self { XYZ_to_RGB, trc, name }
    }

    /// sRGB color space creator
    pub fn sRGB() -> Self {
        let r = Vec2::new(0.64, 0.33);
        let g = Vec2::new(0.3, 0.6);
        let b = Vec2::new(0.15, 0.06);
        let w = xyz::w_D65;

        Self::init(r, g, b, w, TransferFunction::sRGB, "sRGB".to_string())
    }

    /// DCI-P3 color space creator
    pub fn dci_p3() -> Self {
        let r = Vec2::new(0.68, 0.32);
        let g = Vec2::new(0.265, 0.69);
        let b = Vec2::new(0.15, 0.06);
        let w = xyz::w_D65;

        Self::init(r, g, b, w, TransferFunction::sRGB, "DCI-P3".to_string())
    }

    /// Rec. 2020 color space creator
    pub fn rec_2020() -> Self {
        let r = Vec2::new(0.708, 0.292);
        let g = Vec2::new(0.170, 0.797);
        let b = Vec2::new(0.131, 0.046);
        let w = xyz::w_D65;

        Self::init(r, g, b, w, TransferFunction::rec_2020, "Rec. 2020".to_string())
    }
}
