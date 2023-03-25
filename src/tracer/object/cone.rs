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

}

impl Object for Cone {
    fn material(&self) -> &Material { &self.material }

    fn sample_on(&self, _rand_sq: DVec2) -> DVec3 { todo!() }
    fn sample_towards(&self, _xo: DVec3, _rand_sq: DVec2) -> Ray { todo!() }
    fn sample_towards_pdf(&self, _ri: &Ray) -> f64 { todo!() }

    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<Hit> {
        // cos of angle at tip of the cone
        let cos_theta = self.height
            / (self.height * self.height + self.radius * self.radius).sqrt();

        let cos2_theta = cos_theta * cos_theta;

        let xo = r.origin;
        let xo_to_tip = self.tip - xo;

        let wo = r.dir.normalize();
        let wo_dot_axis = wo.dot(self.axis);
        let xt_dot_axis = xo_to_tip.dot(self.axis);
        let wo_dot_xt = xo_to_tip.dot(wo);
        let xt_dot_xt = xo_to_tip.dot(xo_to_tip);

        let a = wo_dot_axis * wo_dot_axis - cos2_theta;
        let b = 2.0 * (wo_dot_axis * xt_dot_axis - wo_dot_xt * cos2_theta);
        let c = xt_dot_axis * xt_dot_axis - xt_dot_xt * cos2_theta;

        let disc = b * b - 4.0 * a * c;

        if disc < 0.0 {
            return None;
        }

        let disc_root = disc.sqrt();
        let mut t = (-b - disc_root) / (2.0 * a);
        if t < t_min + EPSILON || t > t_max - EPSILON {
            t = (-b + disc_root) / (2.0 * a);
            if t < t_min + EPSILON || t > t_max - EPSILON {
                return None;
            }
        }

        let xi = r.at(t);
        let tip_to_xi = xi - self.tip;
        let ni = self.axis.cross(tip_to_xi).cross(tip_to_xi).normalize();

        Hit::new(
            t,
            self,
            xi,
            ni,
        )
    }
}
