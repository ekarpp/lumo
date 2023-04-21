use super::*;

/// Orthographic camera that Preserves angles
pub struct OrthographicCamera {
    /// Camera position
    origin: DVec3,
    /// Basis of camera space
    camera_basis: Onb,
    /// Image resolution
    resolution: DVec2,
}

impl OrthographicCamera {
    /// Camera at origin pointing towards `-z` with `y` axis as up
    /// and resolution `1000 x 1000`
    pub fn default() -> Box<Self> {
        Self::new(DVec3::ZERO, DVec3::NEG_Z, DVec3::Y, IVec2::ONE * 1000)
    }

    /// # Arguments
    /// * `origin` - Eye position in world space
    /// * `towards` - Point eye is looking at in world space
    /// * `up` - Up direction of the camera
    /// * `resolution` - Resolution of the image
    pub fn new(
        origin: DVec3,
        towards: DVec3,
        up: DVec3,
        resolution: IVec2,
    ) -> Box<Self> {
        let camera_basis = _camera_basis(origin, towards, up);

        Box::new(Self {
            origin,
            camera_basis,
            resolution: resolution.as_dvec2(),
        })
    }
}

impl Camera for OrthographicCamera {
    fn get_resolution(&self) -> IVec2 { self.resolution.as_ivec2() }

    fn generate_ray(&self, raster_xy: DVec2) -> Ray {
        let image_xyz = DVec2::new(
            2.0 * raster_xy.x / self.resolution.x - 1.0,
            2.0 * raster_xy.y * self.resolution.x / self.resolution.y.powi(2) - 1.0
        ).extend(0.0);

        Ray::new(
            self.origin + self.camera_basis.to_world(image_xyz),
            self.camera_basis.to_world(DVec3::Z)
        )
    }
}
