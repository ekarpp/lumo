use super::*;

/// Cone aligned with the `y` axis and base disk at `y=0`
pub struct Cone {
    /// Height of the cone
    height: f64,
    /// Radius of the circle at the bottom of the cone
    radius: f64,
    /// Material of the cone
    material: Material,
}

impl Cone {
    /// Constructs a cone from the given `height` and `radius`
    pub fn new(height: f64, radius: f64, material: Material) -> Box<Self> {
        Box::new(Self {
            height,
            radius,
            material,
        })
    }
}

impl Object for Cone {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<Hit> {
        let xo = r.origin;
        let wi = r.dir;

        let tan_theta = self.radius / self.height;
        let tan2_theta = tan_theta * tan_theta;
        let xoy_height = xo.y - self.height;

        let a = wi.dot(DVec3::new(wi.x, -tan2_theta * wi.y, wi.z));
        let b = 2.0 * wi.dot(DVec3::new(xo.x, -tan2_theta * xoy_height, xo.z));
        let c = xo.x * xo.x - tan2_theta * xoy_height.powi(2) + xo.z * xo.z;

        let disc = b * b - 4.0 * a * c;

        if disc < 0.0 {
            return None;
        }

        let disc_root = disc.sqrt();
        let mut t = (-b - disc_root) / (2.0 * a);
        if t < t_min + EPSILON || t > t_max {
            t = (-b + disc_root) / (2.0 * a);
            if t < t_min + EPSILON || t > t_max {
                return None;
            }
        }

        let xi = r.at(t);

        if xi.y < 0.0 || xi.y > self.height {
            return None;
        }

        let u = xi.x.atan2(xi.z) / 2.0 * PI;
        let v = xi.y / self.height;
        let uv = DVec2::new(u, v);

        let radius = (xi.x * xi.x + xi.z * xi.z).sqrt();
        let ni = DVec3::new(xi.x, radius * tan_theta, xi.z);
        let ni = ni.normalize();

        Hit::new(t, &self.material, xi, ni, ni, uv)
    }
}
