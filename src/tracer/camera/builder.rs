use super::*;

/// Specifies the camera type
pub enum CameraType {
    /// "Traditional" pinhole camera
    Perspective,
    /// Camera that preserves angles
    Orthographic,
}

/// Temporary structure to hold data while building cameras
pub struct CameraBuilder {
    origin: Point,
    towards: Point,
    up: Direction,
    zoom: Float,
    lens_radius: Float,
    focal_length: Float,
    resolution: (u64, u64),
    camera_type: CameraType,
    vfov: Float,
}

impl Default for CameraBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl CameraBuilder {
    /// Initialize a new builder with the default configuration
    pub fn new() -> Self {
        Self {
            origin: Point::ZERO,
            towards: -Point::Z,
            up: Direction::Y,
            zoom: 1.0,
            lens_radius: 0.0,
            focal_length: 0.0,
            resolution: (1024, 768),
            camera_type: CameraType::Perspective,
            vfov: 90.0,
        }
    }

    /// Set `origin` of the camera
    pub fn origin(mut self, x: Float, y: Float, z: Float) -> Self {
        self.origin = Vec3::new(x, y, z);
        self
    }

    /// Set point `towards` which the camera is looking
    pub fn towards(mut self, x: Float, y: Float, z: Float) -> Self {
        self.towards = Vec3::new(x, y, z);
        self
    }

    /// Set `up` direction of the camera
    pub fn up(mut self, x: Float, y: Float, z: Float) -> Self {
        self.up = Vec3::new(x, y, z);
        self
    }

    /// Set `zoom` of the camera
    pub fn zoom(mut self, zoom: Float) -> Self {
        self.zoom = zoom;
        self
    }

    /// Set the `lens_radius` of the camera
    pub fn lens_radius(mut self, lens_radius: Float) -> Self {
        self.lens_radius = lens_radius;
        self
    }

    /// Set `focal_length` of the camera
    pub fn focal_length(mut self, focal_length: Float) -> Self {
        self.focal_length = focal_length;
        self
    }

    /// Set the `resolution` of the image
    pub fn resolution(mut self, resolution: (u64, u64)) -> Self {
        self.resolution = resolution;
        self
    }

    /// Set the `camera_type`
    pub fn camera_type(mut self, camera_type: CameraType) -> Self {
        self.camera_type = camera_type;
        self
    }

    /// Set the vertical field of view for perspective camera
    pub fn vfov(mut self, vfov: Float) -> Self {
        self.vfov = vfov;
        self
    }

    /// Build the camera
    pub fn build(&self) -> Camera {
        let cts = match self.camera_type {
            CameraType::Perspective => matrices::perspective_projection(self.vfov),
            CameraType::Orthographic => matrices::orthographic_projection(),
        };

        let wtc = matrices::world_to_camera(self.origin, self.towards, self.up);
        let sctr = matrices::screen_to_raster(self.resolution, self.zoom);

        let cfg = CameraConfig::new(
            self.lens_radius,
            self.focal_length,
            self.resolution,
            wtc,
            sctr,
            cts,
        );

        match self.camera_type {
            CameraType::Perspective => Camera::Perspective(cfg),
            CameraType::Orthographic => Camera::Orthographic(cfg),
        }
    }
}
