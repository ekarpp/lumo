use crate::DVec3;
use crate::tracer::Ray;

/// Abstraction for a camera, with the image plane modeled as a \[-1,1\]^2 square
pub struct Camera {
    /// Origin of the camera
    origin: DVec3,
    /// Unit vector pointing to the right in the image plane
    right: DVec3,
    /// Unit vector pointing up in the image plane
    up: DVec3,
    /// Vector from camera origin to middle of image plane
    forward: DVec3,
}

impl Default for Camera {
    fn default() -> Self {
        Self::new(
            DVec3::ZERO,
            DVec3::new(0.0, 0.0, -1.0),
            DVec3::new(0.0, 1.0, 0.0),
            1.0,
        )
    }
}


impl Camera {
    /// Returns a ray pointing towards a point on the image plane.
    ///
    /// # Arguments
    /// * `x` - x coordinate in a normalized \[-1,1\]^2 square
    /// * `y` - y coordinate in a normalized \[-1,1\]^2 square
    pub fn ray_at(&self, x: f64, y: f64) -> Ray {
        Ray::new(
            self.origin,
            self.forward + x * self.right + y * self.up
        )
    }

    /// Returns a camera
    ///
    /// # Arguments
    ///
    /// * `origin` - Camera origin
    /// * `towards` - Camera is looking at this point
    /// * `up_dir` - Defines up direction for the camera
    /// * `focal_length` - Focal length of the camera
    fn new(
        origin: DVec3,
        towards: DVec3,
        up_dir: DVec3,
        focal_length: f64
    ) -> Self {
        assert!(origin != towards);

        let forward = (towards - origin).normalize() * focal_length;
        let right = forward.cross(up_dir).normalize();
        let up = right.cross(forward);

        Self {
            origin,
            forward,
            right,
            up,
        }
    }
}
