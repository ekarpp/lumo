use super::*;

/// Perspective
pub struct PerspectiveCamera {
    /// Eye position
    origin: DVec3,
    /// Basis of camera space
    camera_basis: Onb,
    /// Image resolution
    resolution: DVec2,
    /// Vertical field-of-view in radians
    vfov: f64,
}

impl PerspectiveCamera {
    /// Camera at origin pointing towards `-z` with `y` axis as up
    /// and resolution `1000 x 1000`
    pub fn default() -> Box<Self> {
        Self::new(DVec3::ZERO, DVec3::NEG_Z, DVec3::Y, 90.0, IVec2::ONE * 1000)
    }

    /// # Arguments
    /// * `origin` - Eye position in world space
    /// * `towards` - Point eye is looking at in world space
    /// * `up` - Up direction of the camera
    /// * `vfov` - Vertical field-of-view in  degrees
    /// * `resolution` - Resolution of the image
    pub fn new(
        origin: DVec3,
        towards: DVec3,
        up: DVec3,
        vfov: f64,
        resolution: IVec2,
    ) -> Box<Self> {
        assert!(vfov > 0.0 && vfov < 180.0);
        let camera_basis = _camera_basis(origin, towards, up);
        let vfov = vfov.to_radians();
        Box::new(Self {
            origin,
            camera_basis,
            vfov,
            resolution: resolution.as_dvec2(),
        })
    }
}

impl Camera for PerspectiveCamera {
    fn get_resolution(&self) -> IVec2 { self.resolution.as_ivec2() }

    fn generate_ray(&self, raster_xy: DVec2) -> Ray {
        let min_res = self.resolution.min_element() as f64;
        let image_xyz = DVec3::new(
            (2.0 * raster_xy.x - self.resolution.x) / min_res,
            (2.0 * raster_xy.y - self.resolution.y) / min_res,
            resolution.y / (min_res * (self.vfov / 2.0).tan())
        );
        Ray::new(self.origin, self.camera_basis.to_world(image_xyz))
    }
}
