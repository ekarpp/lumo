use super::*;

/// Infinite plane defined by a single point and a normal
pub struct Plane {
    /// Unidirectional normal
    normal: Normal,
    /// Material of the plane
    material: Material,
    /// `p.dot(-norm)`, used for fast hit calculations
    d: EFloat,
    /// u for uv-basis
    u: Normal,
    /// v for uv-basis
    v: Normal,
}

impl Plane {
    /// Constructs an infinite plane given a point and a normal
    pub fn new(p: Point, n: Normal, material: Material) -> Box<Self> {
        assert!(n.dot(n) != 0.0);
        let normal = n.normalize();
        let onb = Onb::new(normal);
        let u = onb.u;
        let v = onb.v;

        let nx = EFloat::from(normal.x); let ny = EFloat::from(normal.y);
        let nz = EFloat::from(normal.z); let px = EFloat::from(p.x);
        let py = EFloat::from(p.y); let pz = EFloat::from(p.z);

        // p.dot(-normal)
        let d = px * (-nx) + py * (-ny) + pz * (-nz);

        Box::new(Self {
            normal,
            material,
            d, u, v
        })
    }
}

impl Object for Plane {
    fn hit(&self, r: &Ray, t_min: Float, t_max: Float) -> Option<Hit> {
        let xo = r.origin;
        let wi = r.dir;

        // co planar to plane
        if self.normal.dot(wi).abs() < crate::EPSILON {
            return None;
        }

        let dx = EFloat::from(wi.x); let dy = EFloat::from(wi.y);
        let dz = EFloat::from(wi.z); let ox = EFloat::from(xo.x);
        let oy = EFloat::from(xo.y); let oz = EFloat::from(xo.z);

        let nx = EFloat::from(self.normal.x);
        let ny = EFloat::from(self.normal.y);
        let nz = EFloat::from(self.normal.z);

        let t = -(self.d + nx * ox + ny * oy + nz * oz)
            / (nx * dx + ny * dy + nz * dz);

        if t.high >= t_max || t.low <= t_min {
            None
        } else {
            let xi = r.at(t.value);
            let err = Vec3::new(
                (ox + dx * t).abs_error(),
                (oy + dy * t).abs_error(),
                (oz + dz * t).abs_error(),
            );

            let map_uv = |n: Normal, p: Point| -> Float {
                let dt = n.dot(p).fract();
                if dt < 0.0 {
                    dt + 1.0
                } else {
                    dt
                }
            };

            let uv = Vec2::new(
                map_uv(self.u, xi),
                map_uv(self.v, xi),
            );

            Hit::new(
                t.value,
                &self.material,
                wi,
                xi,
                err,
                self.normal,
                self.normal,
                uv
            )
        }
    }

    fn hit_t(&self, r: &Ray, t_min: Float, t_max: Float) -> Float {
        let xo = r.origin;
        let wi = r.dir;

        // check coplanarity
        if self.normal.dot(wi).abs() < crate::EPSILON { return crate::INF; }

        let t = -(self.d.value + self.normal.dot(xo)) / self.normal.dot(wi);
        if t <= t_min || t >= t_max { crate::INF } else { t }
    }
}

#[cfg(test)]
mod plane_tests {
    use super::*;
    fn plane() -> Box<Plane> {
        Plane::new(Point::ZERO, Normal::Z, Material::Blank)
    }

    test_util::test_object!(plane());

    #[test]
    fn no_hit_parallel() {
        let p = plane();
        let r = Ray::new(Point::Z, Direction::X);
        assert!(p.hit(&r, 0.0, crate::INF).is_none());
    }
}
