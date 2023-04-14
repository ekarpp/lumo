use super::*;

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
    fn material(&self) -> &Material {
        &self.material
    }

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
            Hit::new(t, self, xi, self.normal, self.normal)
        }
    }
}

impl Sampleable for Disk {
    fn sample_on(&self, rand_sq: DVec2) -> DVec3 {
        let rand_disk = rand_utils::square_to_disk(rand_sq);

        self.origin
            + self.uvw.to_world(DVec3::new(
                rand_disk.x * self.radius,
                rand_disk.y * self.radius,
                0.0,
            ))
    }

    fn sample_towards(&self, xo: DVec3, rand_sq: DVec2) -> Ray {
        let xi = self.sample_on(rand_sq);
        let wi = xi - xo;
        Ray::new(xo, wi)
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
