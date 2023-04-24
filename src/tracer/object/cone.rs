use super::*;

/// Cone defined by its tip, height, axis and radius
pub struct Cone {
    /// Tip of the cone
    tip: DVec3,
    /// Axis or direction of the cone. Normalized in constructor.
    axis: DVec3,
    /// Height of the cone
    height: f64,
    /// Radius of the circle at the bottom of the cone
    radius: f64,
    /// Material of the cone
    material: Material,
}

impl Cone {
    /// Constructs a cone from the given parameters
    pub fn new(tip: DVec3, axis: DVec3, height: f64, radius: f64, material: Material) -> Box<Self> {
        Box::new(Self {
            tip,
            axis: axis.normalize(),
            height,
            radius,
            material,
        })
    }
}

impl Object for Cone {
    fn material(&self) -> &Material {
        &self.material
    }

    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<Hit> {
        let xo = r.origin;
        let tip_to_xo = xo - self.tip;

        let wo = r.dir;
        let wo_dot_wo = wo.dot(wo);
        let wo_dot_axis = wo.dot(self.axis);
        let wo_dot_txo = wo.dot(tip_to_xo);
        let txo_dot_axis = tip_to_xo.dot(self.axis);
        let txo_dot_txo = tip_to_xo.dot(tip_to_xo);

        let tmp = 1.0 + self.radius * self.radius / (self.height * self.height);

        let a = wo_dot_wo - wo_dot_axis.powi(2) * tmp;
        let b = 2.0 * (wo_dot_txo - wo_dot_axis * txo_dot_axis * tmp);
        let c = txo_dot_txo - txo_dot_axis.powi(2) * tmp;

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
        let tip_to_xi = xi - self.tip;
        let txi_dot_axis = tip_to_xi.dot(self.axis);

        if txi_dot_axis < 0.0 || txi_dot_axis > self.height {
            return None;
        }

        let cos_theta = self.height
            / (self.height * self.height + self.radius * self.radius).sqrt();

        let a = self.tip + self.axis * tip_to_xi.length() / cos_theta;
        let ni = (xi - a).normalize();

        let (u, _) = self.axis.any_orthonormal_pair();
        let uv = DVec2::new(u.dot(xi), txi_dot_axis / self.height);

        Hit::new(t, self, xi, ni, ni, uv)
    }
}
