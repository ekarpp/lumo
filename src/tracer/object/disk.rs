use super::*;

#[cfg(test)]
mod disk_tests;

/// A two dimensional disk
pub struct Disk {
    /// Origin of the disk
    origin: DVec3,
    /// Normal direction of the disk
    normal: DVec3,
    /// Radius of the disk
    radius: f64,
    /// `p.dot(-norm)`, used to determine if ray hits the plane of the disk
    d: EFloat64,
    /// Material of the disk
    material: Material,
    /// ONB for normal, used for sampling points on the disk
    uvw: Onb,
}

impl Disk {
    /// Creates a disk of `radius` at `origin` with normal towards `normal_dir`
    pub fn new(origin: DVec3, normal_dir: DVec3, radius: f64, material: Material) -> Box<Self> {
        assert!(normal_dir.dot(normal_dir) != 0.0);
        let normal = normal_dir.normalize();
        let nx = EFloat64::from(normal.x); let ny = EFloat64::from(normal.y);
        let nz = EFloat64::from(normal.z); let ox = EFloat64::from(origin.x);
        let oy = EFloat64::from(origin.y); let oz = EFloat64::from(origin.z);

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
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<Hit> {
        let xo = r.origin;
        let wi = r.dir;

        // co planar to disk
        if self.normal.dot(wi).abs() < EPSILON {
            return None;
        }

        let dx = EFloat64::from(wi.x); let dy = EFloat64::from(wi.y);
        let dz = EFloat64::from(wi.z); let ox = EFloat64::from(xo.x);
        let oy = EFloat64::from(xo.y); let oz = EFloat64::from(xo.z);

        let nx = EFloat64::from(self.normal.x);
        let ny = EFloat64::from(self.normal.y);
        let nz = EFloat64::from(self.normal.z);

        let t = -(self.d + nx * ox + ny * oy + nz * oz)
            / (nx * dx + ny * dy + nz * dz);

        if t.high >= t_max || t.low <= t_min {
            return None;
        }

        let xi = r.at(t.value);

        if xi.distance_squared(self.origin) > self.radius * self.radius {
            None
        } else {
            let err = DVec3::new(
                (ox + dx * t).abs_error(),
                (oy + dy * t).abs_error(),
                (oz + dz * t).abs_error(),
            );

            let xi_local = (xi - self.origin) / self.radius;
            let u = self.uvw.u.dot(xi_local);
            let v = self.uvw.v.dot(xi_local);
            let uv = (DVec2::new(u, v) + DVec2::ONE) / 2.0;
            Hit::new(
                t.value,
                &self.material,
                r.backface(self.normal),
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
    fn area(&self) -> f64 {
        PI * self.radius * self.radius
    }

    fn sample_on(&self, rand_sq: DVec2) -> Hit {
        let rand_disk = rand_utils::square_to_disk(rand_sq);

        let xo = self.origin + self.uvw.to_world(DVec3::new(
            rand_disk.x * self.radius,
            rand_disk.y * self.radius,
            0.0,
        ));

        Hit::new(
            0.0,
            &self.material,
            false,
            xo,
            DVec3::ZERO,
            self.normal,
            self.normal,
            DVec2::ZERO,
        ).unwrap()
    }

    fn sample_towards(&self, xo: DVec3, rand_sq: DVec2) -> DVec3 {
        let xi = self.sample_on(rand_sq).p;
        xi - xo
    }

    fn sample_towards_pdf(&self, ri: &Ray) -> (f64, Option<Hit>) {
        match self.hit(ri, 0.0, INFINITY) {
            None => (0.0, None),
            Some(hi) => {
                let p = 1.0 / (PI * self.radius * self.radius);

                (p, Some(hi))
            }
        }
    }
}
