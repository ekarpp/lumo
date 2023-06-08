use super::*;

#[cfg(test)]
mod cone_tests;

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
        assert!(height > 0.0);
        assert!(radius > 0.0);

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

        let dx = EFloat64::from(wi.x); let dy = EFloat64::from(wi.y);
        let dz = EFloat64::from(wi.z); let ox = EFloat64::from(xo.x);
        let oy = EFloat64::from(xo.y); let oz = EFloat64::from(xo.z);

        let tan_theta = EFloat64::from(self.radius) / EFloat64::from(self.height);
        let tan2_theta = tan_theta * tan_theta;
        let oy_height = oy - EFloat64::from(self.height);

        let a = dx * dx - tan2_theta * dy * dy + dz * dz;
        let b = EFloat64::from(2.0) * (dx * ox - tan2_theta * dy * oy_height + dz * oz);
        let c = ox * ox - tan2_theta * oy_height * oy_height + oz * oz;

        let t0t1 = EFloat64::quadratic(a, b, c);
        if t0t1.is_none() {
            return None;
        }
        let (t0, t1) = t0t1.unwrap();

        // cone behind or too far
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

        // check both hits against cone height
        if xi.y < 0.0 || xi.y > self.height {
            t = t1;
            xi = r.at(t.value);

            if t.high >= t_max || xi.y < 0.0 || xi.y > self.height {
                return None;
            }
        }

        // TODO: propagate errors from transformation
        let err = DVec3::new(
            50.0 * (ox + dx * t).abs_error(),
            50.0 * (oy + dy * t).abs_error(),
            50.0 * (oz + dz * t).abs_error(),
        );

        let u = ((-xi.z).atan2(xi.x) + PI) / (2.0 * PI);
        let v = xi.y / self.height;
        let uv = DVec2::new(u, v);

        let radius = (xi.x * xi.x + xi.z * xi.z).sqrt();
        let ni = DVec3::new(xi.x, radius * tan_theta.value, xi.z);
        let ni = ni.normalize();

        Hit::new(t.value, &self.material, r.dir, xi, err, ni, ni, uv)
    }
}
