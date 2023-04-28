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
    d: f64,
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

        Box::new(Self {
            origin,
            material,
            radius,
            normal,
            d: origin.dot(-normal),
            uvw: Onb::new(normal),
        })
    }
}

impl Object for Disk {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<Hit> {
        let wo = r.dir;
        if self.normal.dot(wo).abs() < EPSILON {
            return None;
        }

        let t = -(self.d + self.normal.dot(r.origin)) / self.normal.dot(wo);
        if t < t_min + EPSILON || t > t_max {
            return None;
        }

        let xi = r.at(t);

        if xi.distance_squared(self.origin) > self.radius * self.radius {
            None
        } else {
            let xi_local = (xi - self.origin) / self.radius;
            let u = self.uvw.u.dot(xi_local);
            let v = self.uvw.v.dot(xi_local);
            let uv = (DVec2::new(u, v) + DVec2::ONE) / 2.0;
            let err = DVec3::ZERO;
            Hit::new(t, &self.material, xi, err, self.normal, self.normal, uv)
        }
    }
}

impl Sampleable for Disk {
    fn area(&self) -> f64 {
        PI * self.radius * self.radius
    }

    fn sample_on(&self, rand_sq: DVec2) -> (DVec3, DVec3) {
        let rand_disk = rand_utils::square_to_disk(rand_sq);

        let xo = self.origin + self.uvw.to_world(DVec3::new(
            rand_disk.x * self.radius,
            rand_disk.y * self.radius,
            0.0,
        ));

        (xo, self.normal)
    }

    fn sample_towards(&self, xo: DVec3, rand_sq: DVec2) -> DVec3 {
        let (xi, _) = self.sample_on(rand_sq);
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
