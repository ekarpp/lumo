use crate::{
    Point, Direction, Float, Vec2, Transform, Normal,
    Mat3, Mat4, Vec4, Vec3, rand_utils, spherical_utils
};
use glam::IVec2;
use crate::tracer::{ Color, ray::Ray };

mod matrices;
mod builder;

pub use builder::CameraBuilder;
pub use builder::CameraType;


/// Common configuration for cameras
pub struct CameraConfig {
    /// Image resolution
    pub resolution: IVec2,
    /// Focal length i.e. distance to focal point behind camera
    pub focal_length: Float,
    /// Radius of the camera lens
    pub lens_radius: Float,
    /// Screen space to camera space transformation
    pub screen_to_camera: Mat4,
    /// Raster space to screen space transformation
    pub raster_to_screen: Transform,
    /// Camera space to world space transformation
    camera_to_world: Transform,
    /// Image plane area in camera space
    pub image_plane_area: Float,
}

impl CameraConfig {
    /// Creates a new config with the given arguments
    pub fn new(
        lens_radius: Float,
        focal_length: Float,
        resolution: (i32, i32),
        camera_to_world: Transform,
        raster_to_screen: Transform,
        screen_to_camera: Mat4,
    ) -> Self {
        assert!(lens_radius >= 0.0);

        let (width, height) = resolution;
        let resolution = IVec2::new(width, height);
        // in screen space
        let p_min = raster_to_screen
            .transform_point3(Vec3::ZERO);
        let p_max = raster_to_screen
            .transform_point3(Vec3::new(width as Float, height as Float, 0.0));

        // in camera space
        let p_min = screen_to_camera.project_point3(p_min);
        let p_max = screen_to_camera.project_point3(p_max);

        let p_min = p_min.truncate() / (if p_min.z == 0.0 { 1.0 } else { p_min.z });
        let p_max = p_max.truncate() / (if p_max.z == 0.0 { 1.0 } else { p_max.z });

        let p_delta = p_max - p_min;
        let image_plane_area = (p_delta.x * p_delta.y).abs();

        Self {
            lens_radius,
            focal_length,
            screen_to_camera,
            camera_to_world,
            raster_to_screen,
            resolution,
            image_plane_area,
        }
    }

    pub fn point_to_local(&self, xo: Point) -> Point {
        self.camera_to_world.inverse().transform_point3(xo)
    }

    pub fn point_to_world(&self, xo_local: Point) -> Point {
        self.camera_to_world.transform_point3(xo_local)
    }

    pub fn direction_to_local(&self, wi: Direction) -> Direction {
        self.camera_to_world.inverse().transform_vector3(wi)
    }

    pub fn direction_to_world(&self, wi_local: Direction) -> Direction {
        self.camera_to_world.transform_vector3(wi_local)
    }

    pub fn normal_to_local(&self, no: Normal) -> Normal {
        let m = self.camera_to_world.matrix3.transpose();
        no.x * m.x_axis + no.y * m.y_axis + no.z * m.z_axis
    }

    pub fn normal_to_world(&self, no: Normal) -> Normal {
        let m = self.camera_to_world.matrix3.inverse().transpose();
        no.x * m.x_axis + no.y * m.y_axis + no.z * m.z_axis
    }

    pub fn raster_to_camera(&self, raster_xy: Vec2) -> Point {
        let raster_xyz = raster_xy.extend(0.0);
        let screen_xyz = self.raster_to_screen.transform_point3(raster_xyz);
        self.screen_to_camera.project_point3(screen_xyz)
    }

    pub fn camera_to_raster(&self, xo_local: Point) -> Vec2 {
        let screen_xyz = self.screen_to_camera.inverse().project_point3(xo_local);
        let raster_xyz = self.raster_to_screen.inverse().transform_point3(screen_xyz);
        raster_xyz.truncate()
    }
}

/// Camera abstraction
pub enum Camera {
    /// Perspective camera with configurable vertical field-of-view
    Perspective(CameraConfig),
    /// Orthographic camera that preserves angles
    Orthographic(CameraConfig),
}

impl Camera {
    /// Return a new `CameraBuilder`
    pub fn builder() -> CameraBuilder {
        CameraBuilder::default()
    }

    fn get_cfg(&self) -> &CameraConfig {
        match self {
            Self::Orthographic(cfg) | Self::Perspective(cfg) => cfg,
        }
    }

    fn bounds(&self, raster_xy: Vec2) -> bool {
        let res = self.get_resolution();
        raster_xy.x >= 0.0 && raster_xy.x < res.x as Float
            && raster_xy.y >= 0.0 && raster_xy.y < res.y as Float
    }

    fn raster_xy(&self, ri: &Ray) -> Option<Vec2> {
        match self {
            Self::Orthographic(cfg) => {
                if cfg.lens_radius != 0.0 { unimplemented!() }
                let xo = ri.origin;
                let xo_local = cfg.point_to_local(xo);
                let raster_xy = cfg.camera_to_raster(xo_local);

                if self.bounds(raster_xy) {
                    Some( raster_xy )
                } else {
                    None
                }
            }
            Self::Perspective(cfg) => {
                let wi = ri.dir;
                let wi_local = cfg.direction_to_local(wi);
                let cos_theta = spherical_utils::cos_theta(wi_local);
                if cos_theta <= 0.0 {
                    return None;
                }

                let fl = if cfg.lens_radius == 0.0 {
                    1.0 / cos_theta
                } else {
                    cfg.focal_length / cos_theta
                };
                let xo = ri.origin;
                let xo_local = cfg.point_to_local(xo);
                let focus_local = xo_local + wi_local * fl;
                let raster_xy = cfg.camera_to_raster(focus_local);

                if self.bounds(raster_xy) {
                    Some( raster_xy )
                } else {
                    None
                }
            }
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

            (xo_local + lens_xyz, focus_xyz - lens_xyz)
        };

        let xo = cfg.point_to_world(xo_local);
        let wi = cfg.direction_to_world(wi_local);
        Ray::new(xo, wi)
    }

    /// Area of the camera lens
    pub fn lens_area(&self) -> Float {
        let cfg = self.get_cfg();

        if cfg.lens_radius == 0.0 {
            1.0
        } else {
            crate::PI * cfg.lens_radius.powi(2)
        }
    }

    /// Generates a ray given a point in raster space `\[0,width\] x \[0,height\]`
    pub fn generate_ray(&self, raster_xy: Vec2) -> Ray {
        match self {
            Self::Perspective(cfg) => {
                let wi_local = cfg.raster_to_camera(raster_xy).normalize();
                Self::add_dof(Point::ZERO, wi_local, cfg)
            }
            Self::Orthographic(cfg) => {
                let xo_local = cfg.raster_to_camera(raster_xy);
                Self::add_dof(xo_local, Direction::Z, cfg)
            }
        }
    }

    /// Samples a ray leaving from camera lens to `xi` in world.
    pub fn sample_towards(&self, xi: Point, rand_sq: Vec2) -> Option<Ray> {
        let ri = match self {
            Self::Orthographic(cfg) => {
                let lens_xyz = cfg.lens_radius
                    * rand_utils::square_to_disk(rand_sq).extend(0.0);
                let xi_local = cfg.point_to_local(xi);
                let xo_local = xi_local * Vec3::new(1.0, 1.0, 0.0);
                let xo = cfg.point_to_world(xo_local + lens_xyz);
                let wi = (xi - xo).normalize();

                Ray::new(xo, wi)
            }
            Self::Perspective(cfg) => {
                let xo_local = cfg.lens_radius
                    * rand_utils::square_to_disk(rand_sq).extend(0.0);
                let xi_local = cfg.point_to_local(xi);
                let wi_local = (xi_local - xo_local).normalize();
                let xo = cfg.point_to_world(xo_local);
                let wi = cfg.direction_to_world(wi_local);
                Ray::new(xo, wi)
            }
        };

        if self.raster_xy(&ri).is_some() { Some( ri ) } else { None }
    }

    /// PDF for origin of `ri` on lens w.r.t. area measure
    pub fn pdf_xo(&self, ri: &Ray) -> Float {
        match self {
            Self::Orthographic(cfg) => {
                if self.raster_xy(ri).is_some() {
                    1.0 / cfg.image_plane_area
                } else {
                    0.0
                }
            }
            Self::Perspective(cfg) => {
                let xo = ri.origin;
                let xo_local = cfg.point_to_local(xo);
                let r2 = (cfg.lens_radius + crate::EPSILON).powi(2);
                if xo_local.distance_squared(Vec3::ZERO) < r2 {
                    1.0 / self.lens_area()
                } else {
                    0.0
                }
            }
        }
    }

    /// PDF for direction of `ri` w.r.t. SA measure
    pub fn pdf_wi(&self, ri: &Ray) -> Float {
        match self {
            Self::Orthographic(cfg) => {
                if cfg.lens_radius != 0.0 {
                    unimplemented!();
                }
                let wi = ri.dir;
                let wi_local = cfg.direction_to_local(wi);
                let cos_theta = spherical_utils::cos_theta(wi_local);
                if 1.0 - cos_theta < crate::EPSILON {
                    1.0
                } else {
                    0.0
                }
            }
            Self::Perspective(cfg) => {
                if self.raster_xy(ri).is_none() {
                    return 0.0;
                }
                let wi = ri.dir;
                let wi_local = cfg.direction_to_local(wi);
                let cos_theta = spherical_utils::cos_theta(wi_local);
                1.0 / (cfg.image_plane_area * cos_theta.powi(3))
            }
        }
    }

    /// PDF for importance arriving at `xi` w.r.t. SA
    pub fn pdf_importance(&self, ri: &Ray, xi: Point) -> Float {
        match self {
            Self::Orthographic(..) => unimplemented!(),
            Self::Perspective(cfg) => {
                if self.raster_xy(ri).is_none() {
                    return 0.0;
                }

                let xo = ri.origin;
                let wi = ri.dir;
                let ng = cfg.normal_to_world(Normal::Z);

                let pdf = xi.distance_squared(xo)
                    / (ng.dot(wi).abs() * self.lens_area());
                pdf.max(0.0)
            }
        }
    }

    /// Incident importance for a ray `ri` starting from camera lens
    pub fn sample_importance(&self, ri: &Ray) -> Option<(Color, Vec2)> {
        let raster_xy = self.raster_xy(ri)?;
        match self {
            Self::Orthographic(cfg) => {
                let imp = 1.0 / cfg.image_plane_area;
                Some( (imp * Color::WHITE, raster_xy) )
            }
            Self::Perspective(cfg) => {
                let wi = ri.dir;
                let wi_local = cfg.direction_to_local(wi);
                let cos_theta = spherical_utils::cos_theta(wi_local);

                let denom = cfg.image_plane_area
                    * cos_theta.powi(4)
                    * self.lens_area();
                let imp = 1.0 / denom;
                Some( (imp * Color::WHITE, raster_xy) )
            }
        }
    }
}
