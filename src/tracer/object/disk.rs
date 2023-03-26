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
}

impl Disk {
    /// Creates a disk of `radius` at `origin` with normal towards `normal_dir`
    pub fn new(
        origin: DVec3,
        normal_dir: DVec3,
        radius: f64,
        material: Material,
    ) -> Box<Self> {
        assert!(normal_dir.dot(normal_dir) != 0.0);
        let normal = normal_dir.normalize();

        Box::new(Self {
            origin,
            material,
            radius,
            normal,
            d: origin.dot(-normal),
        })
    }
}

impl Object for Disk {
    fn material(&self) -> &Material { &self.material }

    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<Hit> {
        let wo = r.dir;
        if self.normal.dot(wo).abs() < EPSILON {
            return None;
        }

        let t = -(self.d + self.normal.dot(r.origin)) / self.normal.dot(wo);
        if t < t_min + EPSILON || t > t_max - EPSILON {
            return None;
        }

        let xi = r.at(t);

        if xi.distance_squared(self.origin) > self.radius * self.radius {
            None
        } else {
            Hit::new(
                t,
                self,
                xi,
                self.normal
            )
        }
    }

    fn sample_towards_pdf(&self, _ri: &Ray) -> f64 { todo!() }
    fn sample_on(&self, _rand_sq: DVec2) -> DVec3 { todo!() }
    fn sample_towards(&self, _xo: DVec3, _rand_sq: DVec2) -> Ray { todo!() }
}
