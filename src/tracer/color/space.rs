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
    XYZ_to_RGB: &'static Mat3,
    W: &'static XYZ,
    trc: TransferFunction,
    name: &'static str,
}

impl fmt::Display for ColorSpace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl ColorSpace {
    /// Return reference to the default color space DCI-P3
    #[allow(clippy::should_implement_trait)]
    pub fn default() -> &'static Self {
        &Self::DCI_P3
    }

    /// White point in XYZ of sRGB color space
    pub const sRGB_W: XYZ = xyz::from_xyY(xyz::w_D65, 1.0);

    /// White point in XYZ of DCI-P3 color space
    pub const DCI_P3_W: XYZ = xyz::from_xyY(xyz::w_D65, 1.0);

    /// White point in XYZ of Rec. 2020  color space
    pub const Rec_2020_W: XYZ = xyz::from_xyY(xyz::w_D65, 1.0);

    /// XYZ to RGB conversion matrix for the sRGB color space
    pub const sRGB_XYZ_to_RGB: Mat3 = Self::xyz_to_rgb(
        Vec2::new(0.64, 0.33),
        Vec2::new(0.3, 0.6),
        Vec2::new(0.15, 0.06),
        Self::sRGB_W,
    );

    /// XYZ to RGB conversion matrix for the DCI-P3 color space
    pub const DCI_P3_XYZ_to_RGB: Mat3 = Self::xyz_to_rgb(
        Vec2::new(0.68, 0.32),
        Vec2::new(0.265, 0.69),
        Vec2::new(0.15, 0.06),
        Self::DCI_P3_W,
    );

    /// XYZ to RGB conversion matrix for the Rec. 2020 color space
    pub const Rec_2020_XYZ_to_RGB: Mat3 = Self::xyz_to_rgb(
        Vec2::new(0.708, 0.292),
        Vec2::new(0.170, 0.797),
        Vec2::new(0.131, 0.046),
        Self::Rec_2020_W,
    );

    /// Conversion matrix from XYZ to the LMS color space, Stockamn & Sharpe 2000
    pub const XYZ_to_LMS: Mat3 = Mat3::new(
        Vec3::new( 0.210576, 0.855098, -0.0396983),
        Vec3::new(-0.417076, 1.177260,  0.0786283),
        Vec3::new( 0.0,      0.0,       0.5168350),
    );

    /// Conversion matrix from the LMS color space to XYZ
    pub const LMS_to_XYZ: Mat3 = Self::XYZ_to_LMS.inv();

    /// DCI-P3 color space
    pub const sRGB: Self = Self::new(
        &Self::sRGB_XYZ_to_RGB,
        &Self::sRGB_W,
        TransferFunction::sRGB,
        "sRGB",
    );

    /// DCI-P3 color space
    pub const DCI_P3: Self = Self::new(
        &Self::DCI_P3_XYZ_to_RGB,
        &Self::DCI_P3_W,
        TransferFunction::sRGB,
        "DCI-P3",
    );

    /// Rec. 2020 color space
    pub const Rec_2020: Self = Self::new(
        &Self::Rec_2020_XYZ_to_RGB,
        &Self::Rec_2020_W,
        TransferFunction::rec_2020,
        "Rec. 2020",
    );

    /// Transform `color` sampled at `lambda` to the colorspace of `self`
    #[inline]
    pub fn from_color(&self, color: &Color, lambda: &ColorWavelength, wb: &Mat3) -> RGB {
        self.from_XYZ(wb.mul_vec3(color.xyz(lambda)))
    }

    /// Transform XYZ vector to the color space
    pub const fn from_XYZ(&self, xyz: Vec3) -> RGB {
        RGB::from(self.XYZ_to_RGB.mul_vec3(xyz))
    }

    /// Apply the tone reproduction curve of `self` to the linear `rgb`
    pub fn encode(&self, rgb: RGB) -> (u8, u8, u8) {
        (
            self.trc.apply(rgb.r()),
            self.trc.apply(rgb.g()),
            self.trc.apply(rgb.b()),
        )
    }

    /// Von Kries transformation matrix for the color space and `illuminant`
    pub fn wb_matrix(&self, illuminant: &'static DenseSpectrum) -> Mat3 {
        let illum_xy = xyz::to_xyY(illuminant.to_xyz());

        let diagonal = Self::XYZ_to_LMS.mul_vec3(*self.W)
            / Self::XYZ_to_LMS.mul_vec3(xyz::from_xyY(illum_xy, 1.0));

        Self::LMS_to_XYZ * Mat3::diag(diagonal) * Self::XYZ_to_LMS
    }

    const fn new(
        XYZ_to_RGB: &'static Mat3,
        W: &'static Vec3,
        trc: TransferFunction,
        name: &'static str
    ) -> Self {
        Self { XYZ_to_RGB, W, trc, name }
    }

    const fn xyz_to_rgb(
        r: Vec2,
        g: Vec2,
        b: Vec2,
        W: XYZ,
    ) -> Mat3 {
        let R = xyz::from_xyY(r, 1.0);
        let G = xyz::from_xyY(g, 1.0);
        let B = xyz::from_xyY(b, 1.0);

        let RGB_c = Mat3::new(R, G, B).transpose();
        let C = RGB_c.inv().mul_vec3(W);

        let RGB_to_XYZ = RGB_c.mul_mat3(Mat3::diag(C));
        RGB_to_XYZ.inv()
    }
}
