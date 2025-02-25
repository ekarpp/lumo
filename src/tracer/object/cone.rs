use super::*;

#[cfg(test)]
mod cone_tests;

/// Cone aligned with the `y` axis and base disk at `y=0`
pub struct Cone {
    /// Height of the cone
    height: Float,
    /// Radius of the circle at the bottom of the cone
    radius: Float,
    /// Material of the cone
    material: Material,
}

impl Cone {
    /// Constructs a cone from the given `height` and `radius`
    pub fn new(height: Float, radius: Float, material: Material) -> Box<Self> {
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
    fn hit(&self, r: &Ray, t_min: Float, t_max: Float) -> Option<Hit> {
        let xo = r.origin;
        let wi = r.dir;

        let dx = EFloat::from(wi.x); let dy = EFloat::from(wi.y);
        let dz = EFloat::from(wi.z); let ox = EFloat::from(xo.x);
        let oy = EFloat::from(xo.y); let oz = EFloat::from(xo.z);

        let tan_theta = EFloat::from(self.radius) / EFloat::from(self.height);
        let tan2_theta = tan_theta * tan_theta;
        let oy_height = oy - EFloat::from(self.height);

        let a = dx * dx - tan2_theta * dy * dy + dz * dz;
        let b = EFloat::from(2.0) * (dx * ox - tan2_theta * dy * oy_height + dz * oz);
        let c = ox * ox - tan2_theta * oy_height * oy_height + oz * oz;

        let (t0, t1) = EFloat::quadratic(a, b, c)?;

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

        let radius = (xi.x * xi.x + xi.z * xi.z).sqrt();
        let ni = Normal::new(xi.x, radius * tan_theta.value, xi.z);
        let ni = ni.normalize();

        if ni.dot(wi) >= 0.0 /* && !r.is_light_ray() */ {
            // backface, make it see through
            // TODO: handle shadows
            return None;
        }

        let err = Vec3::new(
            (ox + dx * t).abs_error(),
            (oy + dy * t).abs_error(),
            (oz + dz * t).abs_error(),
        );

        let u = ((-xi.z).atan2(xi.x) + crate::PI) / (2.0 * crate::PI);
        let v = xi.y / self.height;
        let uv = Vec2::new(u, v);

        Hit::new(t.value, &self.material, wi, xi, err, ni, ni, uv)
    }
}
