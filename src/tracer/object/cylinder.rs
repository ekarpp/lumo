use super::*;

#[cfg(test)]
mod cylinder_tests;

/// Cylinder aligned with the `y` axis with base at `y=0`
pub struct Cylinder {
    /// Radius of the cylinder
    radius: Float,
    /// Height of the cylinder
    height: Float,
    /// Material of the cylinder
    material: Material,
}

impl Cylinder {
    /// Cylinder constructor
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

impl Object for Cylinder {
    fn hit(&self, r: &Ray, t_min: Float, t_max: Float) -> Option<Hit> {
        let xo = r.origin;
        let wi = r.dir;

        // co-planar to cylinder
        if wi.x == 0.0 && wi.z == 0.0 {
            return None;
        }

        let dx = EFloat::from(wi.x); let dz = EFloat::from(wi.z);
        let ox = EFloat::from(xo.x); let oz = EFloat::from(xo.z);

        let radius2 = EFloat::from(self.radius) * EFloat::from(self.radius);

        let a = dx * dx + dz * dz;
        let b = EFloat::from(2.0) * (dx * ox + dz * oz);
        let c = ox * ox + oz * oz - radius2;

        let (t0, t1) = EFloat::quadratic(a, b, c)?;

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
        let xi = Point::new(
            xi.x * radius2.value / hit_radius2,
            xi.y,
            xi.z * radius2.value / hit_radius2,
        );

        let ni = Normal::new(xi.x, 0.0, xi.z) / self.radius;

        let u = ((-xi.z).atan2(xi.x) + crate::PI) / (2.0 * crate::PI);
        let v = xi.y / self.height;
        let uv = Vec2::new(u, v);

        let err = efloat::gamma(3) * Vec3::new(xi.x, 0.0, xi.z).abs();

        Hit::new(t.value, &self.material, r.dir, xi, err, ni, ni, uv)
    }

    fn hit_t(&self, r: &Ray, t_min: Float, t_max: Float) -> Float {
        let xo = r.origin;
        let wi = r.dir;

        // check coplanarity
        if wi.x == 0.0 && wi.z == 0.0 { return crate::INF; }

        let xo_xz = Vec2::new(xo.x, xo.z);
        let wi_xz = Vec2::new(wi.x, wi.z);

        let a = wi_xz.dot(wi_xz);
        let b = 2.0 * wi_xz.dot(xo_xz);
        let c = xo_xz.dot(xo_xz) - self.radius * self.radius;

        let Some((t0, t1)) = util::quadratic(a, b, c) else { return crate::INF; };

        if t0 >= t_max || t1 <= t_min { return crate::INF; }

        if t0 > t_min {
            t0
        } else if t1 >= t_max {
            crate::INF
        } else {
            t1
        }
    }
}
