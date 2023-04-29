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
        let wi = r.dir;

        // co-planar to cylinder
        if wi.x == 0.0 && wi.z == 0.0 {
            return None;
        }

        let dx = EFloat64::from(wi.x); let dz = EFloat64::from(wi.z);
        let ox = EFloat64::from(xo.x); let oz = EFloat64::from(xo.z);

        let radius2 = EFloat64::from(self.radius) * EFloat64::from(self.radius);

        let a = dx * dx + dz * dz;
        let b = EFloat64::from(2.0) * (dx * ox + dz * oz);
        let c = ox * ox + oz * oz - radius2;

        let t0t1 = EFloat64::quadratic(a, b, c);
        if t0t1.is_none() {
            return None;
        }
        let (t0, t1) = t0t1.unwrap();

        // cylinder behind or too far
        if t0.high >= t_max || t1.low <= t_min {
            return None;
        }

        let mut t = if t0.low > t_min {
            t0
        } else {
            if t1.high >= t_max {
                return None;
            }
            t1
        };
        let mut xi = r.at(t.value);

        // check both hits against cylinder height
        if xi.y < 0.0 || xi.y > self.height {
            t = t1;
            xi = r.at(t.value);

            if t.high >= t_max || xi.y < 0.0 || xi.y > self.height {
                return None;
            }
        }

        // reproject x and z
        let hit_radius2 = xi.x * xi.x + xi.z * xi.z;
        let xi = DVec3::new(
            xi.x * radius2.value / hit_radius2,
            xi.y,
            xi.z * radius2.value / hit_radius2,
        );

        let ni = DVec3::new(xi.x, 0.0, xi.z) / self.radius;

        let u = ((-xi.z).atan2(xi.x) + PI) / (2.0 * PI);
        let v = xi.y / self.height;
        let uv = DVec2::new(u, v);

        let err = efloat::gamma(3) * DVec3::new(xi.x, 0.0, xi.z).abs();

        Hit::new(t.value, &self.material, xi, err, ni, ni, uv)
    }
}
