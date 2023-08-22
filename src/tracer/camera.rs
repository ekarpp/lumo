use crate::{Point, Direction, Float, Vec2, rand_utils};
use crate::tracer::film::FilmSample;
use crate::tracer::ray::Ray;
use crate::tracer::onb::Onb;
use crate::tracer::Color;
use glam::IVec2;

/// Common configuration for cameras
pub struct CameraConfig {
    /// Camera position in world space
    pub origin: Point,
    /// Basis of camera space
    pub camera_basis: Onb,
    /// Image resolution
    pub resolution: IVec2,
    /// Focal length i.e. distance to focal point behind camera
    pub focal_length: Float,
    /// Radius of the camera lens
    pub lens_radius: Float,
}

impl CameraConfig {
    /// Creates a new config with the given arguments
    pub fn new(
        origin: Point,
        towards: Point,
        up: Direction,
        lens_radius: Float,
        focal_length: Float,
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
    Perspective(CameraConfig, Float),
    /// Orthographic camera that preserves angles with configurable image plane scale
    Orthographic(CameraConfig, Float),
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
    #[allow(clippy::too_many_arguments)]
    pub fn orthographic(
        origin: Point,
        towards: Point,
        up: Direction,
        image_plane_scale: Float,
        lens_radius: Float,
        focal_length: Float,
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
    #[allow(clippy::too_many_arguments)]
    pub fn perspective(
        origin: Point,
        towards: Point,
        up: Direction,
        vfov: Float,
        lens_radius: Float,
        focal_length: Float,
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
            Point::ZERO,
            Point::NEG_Z,
            Direction::Y,
            90.0,
            0.0,
            0.0,
            width,
            height,
        )
    }

    fn get_cfg(&self) -> &CameraConfig {
        match self {
            Self::Orthographic(cfg, _) | Self::Perspective(cfg, _) => cfg,
        }
    }

    /// Returns the resolution of the image
    pub fn get_resolution(&self) -> IVec2 {
        self.get_cfg().resolution
    }

    /// Adds depth of field to camera space ray and transform to world space ray
    fn add_dof(xo_local: Point, wi_local: Direction, cfg: &CameraConfig) -> Ray {
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
    pub fn generate_ray(&self, raster_xy: Vec2) -> Ray {
        let resolution = self.get_resolution();
        let resolution = Vec2::new(
            resolution.x as Float,
            resolution.y as Float,
        );
        let min_res = resolution.min_element();
        // raster to screen here
        let screen_xy = (2.0 * raster_xy - resolution) / min_res;

        match self {
            Self::Perspective(cfg, vfov_half) => {
                // is this correct ???
                let wi_local = screen_xy.extend(
                    resolution.y / (min_res * vfov_half.tan())
                ).normalize();

                Self::add_dof(Point::ZERO, wi_local, cfg)
            }
            Self::Orthographic(cfg, scale) => {
                let screen_xyz = screen_xy.extend(0.0);
                let xo_local = *scale * screen_xyz;

                Self::add_dof(xo_local, Direction::Z, cfg)
            }
        }
    }

    /// Samples a ray leaving from the lens of the camera towards `xi`
    pub fn sample_towards(&self, xi: Point, rand_sq: Vec2) -> Ray {
        let cfg = self.get_cfg();
        let xo_local = rand_utils::square_to_disk(rand_sq).extend(0.0)
            * cfg.lens_radius;
        let xo = cfg.origin + cfg.camera_basis.to_world(xo_local);

        let wi = (xi - xo).normalize();

        Ray::new(xo, wi)
    }

    /// Probability that `ro` towards `xi` got sampled
    pub fn sample_towards_pdf(&self, ro: &Ray, xi: Point) -> Float {
        let cfg = self.get_cfg();
        let xo = ro.origin;
        let wi = ro.dir;
        let ng = cfg.camera_basis.to_world(Direction::Z);

        let lens_area = if cfg.lens_radius == 0.0 {
            1.0
        } else {
            cfg.lens_radius * cfg.lens_radius * crate::PI
        };

        let pdf = xi.distance_squared(xo) / (ng.dot(wi) * lens_area);
        pdf.max(0.0)
    }

    /// PDF for `wi` direction.
    pub fn pdf(&self, wi: Direction) -> Float {
        let cfg = self.get_cfg();
        let wi_local = cfg.camera_basis.to_local(wi);
        let cos_theta = wi_local.z;

        if cos_theta <= 0.0 {
            0.0
        } else {
            let area_coeff = {
                let res = self.get_resolution();
                let res = Vec2::new(
                    res.x as Float,
                    res.y as Float,
                );
                let min_res = res.min_element();
                let screen_bounds = res / min_res;
                screen_bounds.x * screen_bounds.y
            };

            1.0 / (area_coeff * cos_theta * cos_theta * cos_theta)
        }
    }

    /// Incident importance for the ray `ro` starting from the camera lens
    pub fn importance_sample(&self, ro: &Ray) -> FilmSample {
        match self {
            Self::Orthographic(..) => unimplemented!(),
            Self::Perspective(cfg, _) => {
                let wi = ro.dir;
                let wi_local = cfg.camera_basis.to_local(wi);
                let cos_theta = wi_local.z;
                if cos_theta < 0.0 {
                    return FilmSample::default();
                }

                let pdf = self.pdf(wi);

                let color = Color::splat(pdf);

                let fl = if cfg.lens_radius == 0.0 {
                    1.0 / cos_theta
                } else {
                    cfg.focal_length / cos_theta
                };

                let resolution = self.get_resolution();
                let resolution = Vec2::new(
                    resolution.x as Float,
                    resolution.y as Float,
                );
                let min_res = resolution.min_element();

                let focus = ro.at(fl);
                let focus_local = cfg.camera_basis.to_local(focus) + cfg.origin;
                let raster_xy = (focus_local.truncate() * min_res + resolution) / 2.0;

                FilmSample::new(color, raster_xy, true)
            }
        }
    }
}
