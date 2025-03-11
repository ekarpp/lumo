use crate::{
    Point, Direction, Float, Vec2, Transform, Normal,
    Mat3, Vec3, rng, math::spherical_utils
};
use crate::math::vec2::UVec2;
use crate::tracer::{
    Color, ColorSpace, color::{DenseSpectrum, illuminants},
    Film, PixelFilter, ray::Ray
};

mod matrices;
mod builder;

pub use builder::CameraBuilder;
pub use builder::CameraType;


/// Common configuration for cameras
pub struct CameraConfig {
    /// Image resolution
    pub resolution: UVec2,
    /// Focal length i.e. distance to focal point behind camera
    pub focal_length: Float,
    /// Radius of the camera lens
    pub lens_radius: Float,
    /// Screen space to camera space (or vice-versa) transformation
    pub camera_to_screen: Transform,
    /// Raster space to screen space (or vice-versa) transformation
    pub screen_to_raster: Transform,
    /// Camera space to world space (or vice-versa) transformation
    world_to_camera: Transform,
    /// Image plane area in camera space
    pub image_plane_area: Float,
    pixel_filter: PixelFilter,
    color_space: &'static ColorSpace,
    illuminant: &'static DenseSpectrum,
}

impl CameraConfig {
    /// Creates a new config with the given arguments
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        lens_radius: Float,
        focal_length: Float,
        resolution: (u64, u64),
        color_space: &'static ColorSpace,
        pixel_filter: PixelFilter,
        illuminant: &'static DenseSpectrum,
        world_to_camera: Transform,
        screen_to_raster: Transform,
        camera_to_screen: Transform,
    ) -> Self {
        assert!(lens_radius >= 0.0);

        let (width, height) = resolution;
        let resolution = UVec2::new(width, height);
        // in screen space
        let p_min = screen_to_raster
            .transform_pt_inv(Vec3::ZERO);
        let p_max = screen_to_raster
            .transform_pt_inv(Vec3::new(width as Float, height as Float, 0.0));

        // in camera space
        let p_min = camera_to_screen.transform_pt_inv(p_min);
        let p_max = camera_to_screen.transform_pt_inv(p_max);

        let p_min = p_min.truncate() / (if p_min.z == 0.0 { 1.0 } else { p_min.z });
        let p_max = p_max.truncate() / (if p_max.z == 0.0 { 1.0 } else { p_max.z });

        let p_delta = p_max - p_min;
        let image_plane_area = (p_delta.x * p_delta.y).abs();

        Self {
            lens_radius,
            focal_length,
            camera_to_screen,
            world_to_camera,
            screen_to_raster,
            resolution,
            image_plane_area,
            pixel_filter,
            color_space,
            illuminant,
        }
    }

    pub fn point_to_local(&self, xo: Point) -> Point {
        self.world_to_camera.transform_pt(xo)
    }

    pub fn point_to_world(&self, xo_local: Point) -> Point {
        self.world_to_camera.transform_pt_inv(xo_local)
    }

    pub fn direction_to_local(&self, wi: Direction) -> Direction {
        self.world_to_camera.transform_dir(wi)
    }

    pub fn direction_to_world(&self, wi_local: Direction) -> Direction {
        self.world_to_camera.transform_dir_inv(wi_local)
    }

    pub fn normal_to_local(&self, no: Normal) -> Normal {
        self.world_to_camera.to_normal().mul_vec3(no)
    }

    pub fn normal_to_world(&self, no: Normal) -> Normal {
        self.world_to_camera.to_normal_inv().mul_vec3(no)
    }

    pub fn raster_to_camera(&self, raster_xy: Vec2) -> Point {
        let raster_xyz = raster_xy.extend(0.0);
        let screen_xyz = self.screen_to_raster.transform_pt_inv(raster_xyz);
        self.camera_to_screen.transform_pt_inv(screen_xyz)
    }

    pub fn camera_to_raster(&self, xo_local: Point) -> Vec2 {
        let screen_xyz = self.camera_to_screen.transform_pt(xo_local);
        let raster_xyz = self.screen_to_raster.transform_pt(screen_xyz);
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

    /// Camera for the ported Cornell box scene
    pub fn cornell_box() -> Camera {
        CameraBuilder::new()
            .origin(278.0, 273.0, -800.0)
            .towards(278.0, 273.0, 0.0)
            .zoom(2.8)
            .focal_length(0.035)
            .resolution((512, 512))
            .illuminant(illuminants::CORNELL)
            .build()
    }

    /// Create a film for the camera
    pub fn create_film(&self, samples: u64) -> Film {
        let cfg = self.get_cfg();
        Film::new(
            cfg.resolution,
            samples,
            cfg.color_space,
            cfg.pixel_filter,
            cfg.illuminant,
        )
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
    pub fn get_resolution(&self) -> UVec2 {
        self.get_cfg().resolution
    }

    /// Adds depth of field to camera space ray and transform to world space ray
    fn add_dof(
        xo_local: Point,
        wi_local: Direction,
        cfg: &CameraConfig,
        rand_sq: Vec2
    ) -> Ray {
        let (xo_local, wi_local) = if cfg.lens_radius == 0.0 {
            (xo_local, wi_local)
        } else {
            let lens_xy = cfg.lens_radius * rng::maps::square_to_disk(rand_sq);
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
    pub fn generate_ray(&self, raster_xy: Vec2, rand_sq: Vec2) -> Ray {
        match self {
            Self::Perspective(cfg) => {
                let wi_local = cfg.raster_to_camera(raster_xy).normalize();
                Self::add_dof(Point::ZERO, wi_local, cfg, rand_sq)
            }
            Self::Orthographic(cfg) => {
                let xo_local = cfg.raster_to_camera(raster_xy);
                Self::add_dof(xo_local, Direction::Z, cfg, rand_sq)
            }
        }
    }

    /// Samples a ray leaving from camera lens to `xi` in world.
    pub fn sample_towards(&self, xi: Point, rand_sq: Vec2) -> Option<Ray> {
        let ri = match self {
            Self::Orthographic(cfg) => {
                let lens_xyz = cfg.lens_radius
                    * rng::maps::square_to_disk(rand_sq).extend(0.0);
                let xi_local = cfg.point_to_local(xi);
                let xo_local = xi_local * Vec3::new(1.0, 1.0, 0.0);
                let xo = cfg.point_to_world(xo_local + lens_xyz);
                let wi = (xi - xo).normalize();

                Ray::new(xo, wi)
            }
            Self::Perspective(cfg) => {
                let xo_local = cfg.lens_radius
                    * rng::maps::square_to_disk(rand_sq).extend(0.0);
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
