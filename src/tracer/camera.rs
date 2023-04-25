use crate::rand_utils;
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
    /// Focal length i.e. distance to focal point behind camera
    pub focal_length: f64,
    /// Radius of the camera lens
    pub lens_radius: f64,
}

impl CameraConfig {
    /// Creates a new config with the given arguments
    pub fn new(
        origin: DVec3,
        towards: DVec3,
        up: DVec3,
        lens_radius: f64,
        focal_length: f64,
        resolution: (i32, i32)
    ) -> Self {
        assert!(resolution.0 > 0 && resolution.1 > 0);
        assert!(lens_radius >= 0.0);
        assert!(origin != towards);
        assert!(up.length() != 0.0);

        let forward = (towards - origin).normalize();
        let right = forward.cross(up).normalize();
        let down = forward.cross(right);

        // x = right, y = down, z = towards
        let camera_basis = Onb::new_from_basis(right, down, forward);
        let (width, height) = resolution;

        Self {
            lens_radius,
            focal_length,
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
    /// Orthographic camera that preserves angles. All rays are cast in the same
    /// direction but from a plane instead of a single point
    ///
    /// # Arguments
    /// * `origin` - Camera position in world space
    /// * `towards` - Point in world space the camera is looking at
    /// * `up` - Up direction of the camera
    /// * `image_plane_scale` - Scale of the plane rays are cast from
    /// * `lens_radius` - Radius of the lens for depth of field. Bigger means more profound effect
    /// * `focal_length` - Distance to the plane of focus for depth of field
    /// * `width` - Width of the rendered image
    /// * `height` - Height of the rendered image
    pub fn orthographic(
        origin: DVec3,
        towards: DVec3,
        up: DVec3,
        image_plane_scale: f64,
        lens_radius: f64,
        focal_length: f64,
        width: i32,
        height: i32,
    ) -> Self {
        assert!(image_plane_scale > 0.0);

        Self::Orthographic(
            CameraConfig::new(
                origin,
                towards,
                up,
                lens_radius,
                focal_length,
                (width, height)
            ),
            image_plane_scale,
        )
    }

    /// Perspective camera where sense of depth is more profound. Rays are cast
    /// from a single point towards points on the image plane.
    ///
    /// # Arguments
    /// * `origin` - Camera position in world space
    /// * `towards` - Point in world space the camera is looking at
    /// * `up` - Up direction of the camera
    /// * `vfov` - Vertical field of view of the camera
    /// * `lens_radius` - Radius of the lens for depth of field. Bigger means more profound effect
    /// * `focal_length` - Distance to the plane of focus for depth of field
    /// * `width` - Width of the rendered image
    /// * `height` - Height of the rendered image
    pub fn perspective(
        origin: DVec3,
        towards: DVec3,
        up: DVec3,
        vfov: f64,
        lens_radius: f64,
        focal_length: f64,
        width: i32,
        height: i32,
    ) -> Self {
        assert!(vfov > 0.0 && vfov < 180.0);

        Self::Perspective(
            CameraConfig::new(
                origin,
                towards,
                up,
                lens_radius,
                focal_length,
                (width, height)
            ),
            vfov.to_radians() / 2.0,
        )
    }

    /// The "default" camera. Perspective camera at world space origin
    /// pointing towards `-z` with `y` as up and vfov at 90° with no DOF
    pub fn default(width: i32, height: i32) -> Self {
        Self::perspective(
            DVec3::ZERO,
            DVec3::NEG_Z,
            DVec3::Y,
            90.0,
            0.0,
            0.0,
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

    /// Adds depth of field to camera space ray and transform to world space ray
    fn add_dof(xo_local: DVec3, wi_local: DVec3, cfg: &CameraConfig) -> Ray {
        let (xo_local, wi_local) = if cfg.lens_radius == 0.0 {
            (xo_local, wi_local)
        } else {
            let lens_xy = cfg.lens_radius
                * rand_utils::square_to_disk(rand_utils::unit_square());
            let lens_xyz = lens_xy.extend(0.0);

            let focus_distance = cfg.focal_length / wi_local.z;

            let focus_xyz = focus_distance * wi_local;

            (lens_xyz, focus_xyz - lens_xyz)
        };
        // refactor camera basis to DAffine3
        let xo = cfg.origin + cfg.camera_basis.to_world(xo_local);
        let wi = cfg.camera_basis.to_world(wi_local);
        Ray::new(xo, wi)
    }

    /// Generates a ray given a point in raster space `\[0,width\] x \[0,height\]`
    pub fn generate_ray(&self, raster_xy: DVec2) -> Ray {
        let resolution = self.get_resolution().as_dvec2();
        let min_res = resolution.min_element();
        let image_xy = (2.0 * raster_xy - resolution) / min_res;

        match self {
            Self::Perspective(cfg, vfov_half) => {
                let wi_local = image_xy.extend(
                    resolution.y / (min_res * vfov_half.tan())
                ).normalize();

                Self::add_dof(DVec3::ZERO, wi_local, cfg)
            }
            Self::Orthographic(cfg, scale) => {
                let image_xyz = image_xy.extend(0.0);
                let xo_local = *scale * image_xyz;

                Self::add_dof(xo_local, DVec3::Z, cfg)
            }
        }

    }
}
