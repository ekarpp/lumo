use super::*;

#[cfg(test)]
mod disk_tests;

/// A two dimensional disk
pub struct Disk {
    /// Origin of the disk
    origin: Point,
    /// Normal direction of the disk
    normal: Normal,
    /// Radius of the disk
    radius: Float,
    /// `p.dot(-norm)`, used to determine if ray hits the plane of the disk
    d: EFloat,
    /// Material of the disk
    material: Material,
    /// ONB for normal, used for sampling points on the disk
    uvw: Onb,
}

impl Disk {
    /// Creates a disk of `radius` at `origin` with normal towards `normal_dir`
    pub fn new(
        origin: Point,
        normal_dir: Direction,
        radius: Float,
        material: Material
    ) -> Box<Self> {
        assert!(normal_dir.dot(normal_dir) != 0.0);
        let normal = normal_dir.normalize();
        let nx = EFloat::from(normal.x); let ny = EFloat::from(normal.y);
        let nz = EFloat::from(normal.z); let ox = EFloat::from(origin.x);
        let oy = EFloat::from(origin.y); let oz = EFloat::from(origin.z);

        // origin.dot(-normal)
        let d = ox * (-nx) + oy * (-ny) + oz * (-nz);

        Box::new(Self {
            origin,
            material,
            radius,
            normal,
            d,
            uvw: Onb::new(normal),
        })
    }
}

impl Object for Disk {
    fn hit(&self, r: &Ray, t_min: Float, t_max: Float) -> Option<Hit> {
        let xo = r.origin;
        let wi = r.dir;

        // co planar to disk
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
            return None;
        }

        let xi = r.at(t.value);

        if xi.distance_squared(self.origin) > self.radius * self.radius {
            None
        } else {
            let err = Vec3::new(
                (ox + dx * t).abs_error(),
                (oy + dy * t).abs_error(),
                (oz + dz * t).abs_error(),
            );

            let xi_local = (xi - self.origin) / self.radius;
            let u = self.uvw.u.dot(xi_local);
            let v = self.uvw.v.dot(xi_local);
            let uv = (Vec2::new(u, v) + Vec2::ONE) / 2.0;
            Hit::new(
                t.value,
                &self.material,
                r.dir,
                xi,
                err,
                self.normal,
                self.normal,
                uv
            )
        }
    }
}

impl Sampleable for Disk {
    fn area(&self) -> Float {
        crate::PI * self.radius * self.radius
    }

    fn sample_on(&self, rand_sq: Vec2) -> Hit {
        let rand_disk = rand_utils::square_to_disk(rand_sq);

        let xo = self.origin + self.uvw.to_world(Point::new(
            rand_disk.x * self.radius,
            rand_disk.y * self.radius,
            0.0,
        ));

        Hit::new(
            0.0,
            &self.material,
            -self.normal,
            xo,
            Vec3::ZERO,
            self.normal,
            self.normal,
            Vec2::ZERO,
        ).unwrap()
    }

    fn sample_towards(&self, xo: Point, rand_sq: Vec2) -> Direction {
        let xi = self.sample_on(rand_sq).p;
        xi - xo
    }

    fn sample_towards_pdf(&self, ri: &Ray) -> (Float, Option<Hit>) {
        match self.hit(ri, 0.0, crate::INF) {
            None => (0.0, None),
            Some(hi) => {
                let p = 1.0 / (crate::PI * self.radius * self.radius);

                (p, Some(hi))
            }
        }
    }
}
