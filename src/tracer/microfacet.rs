use crate::DVec3;
use std::f64::consts::PI;

#[derive(Copy, Clone)]
pub enum MfDistribution {
    Ggx(f64),
    Beckmann(f64),
}

impl MfDistribution {
    pub fn d(&self, wh: DVec3, no: DVec3) -> f64 {
        match self {
            Self::Ggx(roughness) => {
                let cos_theta2 = wh.dot(no).powi(2);
                let roughness2 = roughness * roughness;

                roughness2
                    / (PI * (cos_theta2 * (roughness2 - 1.0) + 1.0).powi(2))
            }
            Self::Beckmann(roughness) => {
                let roughness2 = roughness * roughness;
                let cos_theta2 = wh.dot(no).powi(2);
                let tan_theta2 = (1.0 - cos_theta2) / cos_theta2;

                (-tan_theta2 / roughness2).exp()
                    / (PI * roughness2 * cos_theta2.powi(2))
            }
        }
    }

    pub fn g(&self, wo: DVec3, wi: DVec3, no: DVec3) -> f64 {
        1.0 / (1.0 + self.lambda(wo, no) + self.lambda(wi, no))
    }

    pub fn f(&self, wo: DVec3, wh: DVec3, color: DVec3, metallic: f64, eta: f64)
             -> DVec3 {
        let f0 = (eta - 1.0) / (eta + 1.0);
        let f0 = DVec3::splat(f0 * f0).lerp(color, metallic);

        let wo_dot_wh = wo.dot(wh);
        f0 + (DVec3::ONE - f0) * (1.0 - wo_dot_wh).powi(5)
    }

    pub fn sample_theta(&self, rand_f: f64) -> f64 {
        match self {
            Self::Ggx(roughness) => {
                (roughness * (rand_f / (1.0 - rand_f)).sqrt()).atan()
            }
            Self::Beckmann(roughness) => {
                let roughness2 = roughness * roughness;
                (-roughness2 * (1.0 - rand_f).ln()).sqrt().atan()
            }
        }
    }

    fn lambda(&self, w: DVec3, no: DVec3) -> f64 {
        match self {
            Self::Ggx(roughness) => {
                let w_dot_no2 = w.dot(no).powi(2);
                let tan_w = (1.0 - w_dot_no2) / w_dot_no2;
                let roughness2 = roughness * roughness;

                ((1.0 + roughness2 * tan_w).sqrt() - 1.0) / 2.0
            }
            Self::Beckmann(roughness) => {
                let w_dot_no2 = w.dot(no).powi(2);
                let tan_w = ((1.0 - w_dot_no2) / w_dot_no2).abs();
                let a = 1.0 / (roughness * tan_w);

                if a >= 1.6 {
                    0.0
                } else {
                    (1.0 - 1.259 * a + 0.396 * a * a)
                        / (3.535 * a + 2.181 * a * a)
                }
            }
        }
    }
}
