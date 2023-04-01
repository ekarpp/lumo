use super::*;

/// Cylinder aligned with the y axis defined by its radius + maximum
/// and minimum y coordinates.
pub struct Cylinder {
    /// Radius of the cylinder
    radius: f64,
    /// Minimum y-point of the cylinder
    y_min: f64,
    /// Maximum y-point of the cylinder
    y_max: f64,
    /// Material of the cylinder
    material: Material,
}

impl Cylinder {
    /// Cylinder constructor
    pub fn new(y_min: f64, y_max: f64, radius: f64, material: Material) -> Box<Self> {
        assert!(y_max > y_min);

        Box::new(Self {
            y_min,
            y_max,
            radius,
            material,
        })
    }
}

impl Object for Cylinder {
    fn material(&self) -> &Material {
        &self.material
    }

    fn sample_on(&self, _rand_sq: DVec2) -> DVec3 {
        todo!()
    }
    fn sample_towards(&self, _xo: DVec3, _rand_sq: DVec2) -> Ray {
        todo!()
    }
    fn sample_towards_pdf(&self, _ri: &Ray) -> f64 {
        todo!()
    }

    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<Hit> {
        let xo = r.origin;
        let wo = r.dir;

        let a = wo.x * wo.x + wo.z * wo.z;
        let b = 2.0 * (wo.x * xo.x + wo.z * xo.z);
        let c = xo.x * xo.x + xo.z * xo.z - self.radius * self.radius;

        let disc = b * b - 4.0 * a * c;

        if disc < 0.0 {
            return None;
        }

        let disc_root = disc.sqrt();
        let mut t = (-b - disc_root) / (2.0 * a);
        if t < t_min + EPSILON || t > t_max {
            t = (-b + disc_root) / (2.0 * a);
            if t < t_min + EPSILON || t > t_max {
                return None;
            }
        }

        let xi = r.at(t);
        // what if the other t is fine?
        if xi.y < self.y_min || xi.y > self.y_max {
            return None;
        }

        let ni = DVec3::new(xi.x, 0.0, xi.z) / self.radius;

        Hit::new(t, self, xi, ni)
    }
}
