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

        let radius2 = self.radius * self.radius;

        let a = wo.x * wo.x + wo.z * wo.z;
        let b = 2.0 * (wo.x * xo.x + wo.z * xo.z);
        let c = xo.x * xo.x + xo.z * xo.z - radius2;

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

        // reproject x and z
        let hit_radius2 = xi.x * xi.x + xi.z * xi.z;
        let xi = DVec3::new(
            xi.x * radius2 / hit_radius2,
            xi.y,
            xi.z * radius2 / hit_radius2,
        );

        let ni = DVec3::new(xi.x, 0.0, xi.z) / self.radius;

        let u = ((-xi.z).atan2(xi.x) + PI) / (2.0 * PI);
        let v = xi.y / self.height;
        let uv = DVec2::new(u, v);

        let err = efloat::gamma(3) * DVec3::new(xi.x, 0.0, xi.z).abs();

        Hit::new(t, &self.material, xi, err, ni, ni, uv)
    }
}
