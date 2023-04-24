use crate::tracer::ray::Ray;
use crate::tracer::onb::Onb;
use glam::{DVec2, DVec3, IVec2};

/// Common configuration for cameras
pub struct CameraConfig {
    /// Camera position in world space
    pub origin: DVec3,
    /// Basis of camera space
    pub camera_basis: Onb,
    /// Image resolution
    pub resolution: IVec2,
}

impl CameraConfig {
    /// Creates a new config with the given arguments
    pub fn new(origin: DVec3, towards: DVec3, up: DVec3, resolution: (i32, i32)) -> Self {
        assert!(origin != towards);

        let forward = (towards - origin).normalize();
        let right = forward.cross(up).normalize();
        let down = forward.cross(right);

        // x = right, y = down, z = towards
        let camera_basis = Onb::new_from_basis(right, down, forward);
        let (width, height) = resolution;

        Self {
            origin,
            camera_basis,
            resolution: IVec2::new(width, height),
        }
    }
}

/// Camera abstraction
pub enum Camera {
    /// Perspective camera with configurable vertical field-of-view
    Perspective(CameraConfig, f64),
    /// Orthographic camera that preserves angles with configurable image plane scale
    Orthographic(CameraConfig, f64),
}

impl Camera {
    /// TODO
    pub fn orthographic(
        origin: DVec3,
        towards: DVec3,
        up: DVec3,
        image_plane_scale: f64,
        width: i32,
        height: i32,
    ) -> Self {
        Self::Orthographic(
            CameraConfig::new(origin, towards, up, (width, height)),
            image_plane_scale,
        )
    }

    /// TODO
    pub fn perspective(
        origin: DVec3,
        towards: DVec3,
        up: DVec3,
        vfov: f64,
        width: i32,
        height: i32,
    ) -> Self {
        Self::Perspective(
            CameraConfig::new(origin, towards, up, (width, height)),
            vfov.to_radians() / 2.0,
        )
    }

    /// The "default" camera. Perspective camera at world space origin
    /// pointing towards `-z` with `y` as up and vfov of 90°
    pub fn default(width: i32, height: i32) -> Self {
        Self::perspective(
            DVec3::ZERO,
            DVec3::NEG_Z,
            DVec3::Y,
            90.0,
            width,
            height,
        )
    }

    /// Returns the resolution of the image
    pub fn get_resolution(&self) -> IVec2 {
        match self {
            Self::Orthographic(cfg, _) | Self::Perspective(cfg, _) => cfg.resolution,
        }
    }

    /// Generates a ray given a point in raster space `\[0,width\] x \[0,height\]`
    pub fn generate_ray(&self, raster_xy: DVec2) -> Ray {
        let resolution = self.get_resolution().as_dvec2();
        let min_res = resolution.min_element();
        let image_xy = (2.0 * raster_xy - resolution) / min_res;

        match self {
            Self::Perspective(cfg, vfov_half) => {
                let image_xyz = image_xy.extend(
                    resolution.y / (min_res * vfov_half.tan())
                );

                Ray::new(cfg.origin, cfg.camera_basis.to_world(image_xyz))
            }
            Self::Orthographic(cfg, scale) => {
                let image_xyz = image_xy.extend(0.0);

                Ray::new(
                    cfg.origin + *scale * cfg.camera_basis.to_world(image_xyz),
                    cfg.camera_basis.to_world(DVec3::Z)
                )
            }
        }
    }
}
