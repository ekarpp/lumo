use crate::DVec3;
use crate::tracer::ray::Ray;

/// Abstraction for a camera
pub struct Camera {
    /// Origin of the camera
    origin: DVec3,
    /// Vector pointing from BLC to the ULC
    horiz: DVec3,
    /// Vector pointing from BLC to BRC
    vert: DVec3,
    /// Bottom left corner of the image plane
    blc: DVec3
}

impl Camera {
    /// Returns a ray pointing towards a point on the image plane.
    ///
    /// # Arguments
    /// * `x` - fraction of the width of the desired point on the plane
    /// * `y` - fraction of the height of the desired point on the plane
    pub fn ray_at(&self, x: f64, y: f64) -> Ray {
        Ray::new(
            self.origin,
            self.blc + x*self.horiz + y*self.vert - self.origin,
        )
    }

    /// Returns a camera
    ///
    /// # Arguments
    ///
    /// * `aspect_ratio` - aspect ratio of the image plane
    /// * `vfov` - vertical field of view, in degrees
    /// * `from` - camera origin
    /// * `towards` - camera is looking at this point
    /// * `up` - defines up direction for the camera
    /// * `focal_length` - focal length of the camera
    pub fn new(
        aspect_ratio: f64,
        vfov: f64,
        from: DVec3,
        towards: DVec3,
        up: DVec3,
        focal_length: f64
    ) -> Self {
        assert!(from != towards);
        assert!(vfov != 0.0);

        let h = (vfov.to_radians() / 2.0).tan();
        /* viewport height */
        let vph = 2.0 * h;
        /* viewport width */
        let vpw = vph * aspect_ratio;

        let z = (from - towards).normalize();
        let x = up.cross(z).normalize();
        let y = z.cross(x);

        let horiz = x * vpw * focal_length;
        let vert = y * vph * focal_length;

        Self {
            origin: from,
            horiz: horiz,
            vert: vert,
            blc: from - (horiz + vert) / 2.0 - z*focal_length,
        }
    }
}
