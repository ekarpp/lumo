use super::*;

#[cfg(test)]
mod cylinder_tests;

/// Cylinder aligned with the `y` axis with base at `y=0`
pub struct Cylinder {
    /// Radius of the cylinder
    radius: f64,
    /// Height of the cylinder
    height: f64,
    /// Material of the cylinder
    material: Material,
}

impl Cylinder {
    /// Cylinder constructor
    pub fn new(height: f64, radius: f64, material: Material) -> Box<Self> {
        assert!(height > 0.0);
        assert!(radius > 0.0);

        Box::new(Self {
            height,
            radius,
            material,
        })
    }
}

impl Object for Cylinder {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<Hit> {
        let xo = r.origin;
        let wo = r.dir;

        // co-planar to cylinder
        if wo.x == 0.0 && wo.z == 0.0 {
            return None;
        }

        let a = wo.x * wo.x + wo.z * wo.z;
        let b = 2.0 * (wo.x * xo.x + wo.z * xo.z);
        let c = xo.x * xo.x + xo.z * xo.z - self.radius * self.radius;

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
        // what if the other t is fine?
        if xi.y < 0.0 || xi.y > self.height {
            return None;
        }

        let ni = DVec3::new(xi.x, 0.0, xi.z) / self.radius;

        let u = ((-xi.z).atan2(xi.x) + PI) / (2.0 * PI);
        let v = xi.y / self.height;
        let uv = DVec2::new(u, v);

        Hit::new(t, &self.material, xi, ni, ni, uv)
    }
}
