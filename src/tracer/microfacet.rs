use crate::DVec3;
use std::f64::consts::PI;

#[derive(Copy, Clone)]
pub enum MfDistribution {
    Ggx(f64),
}

impl MfDistribution {
    pub fn get_roughness(&self) -> f64 {
        match self {
            Self::Ggx(roughness) => *roughness,
        }
    }

    pub fn d(&self, wh: DVec3, no: DVec3) -> f64 {
        match self {
            Self::Ggx(roughness) => {
                let wh_dot_no2 = wh.dot(no).powi(2);
                let tan_wh = (1.0 - wh_dot_no2) / wh_dot_no2;
                let roughness2 = roughness * roughness;

                roughness2
                    / (PI * (wh_dot_no2 * (roughness2 + tan_wh)).powi(2))
            }
        }
    }

    pub fn g(&self, wo: DVec3, no: DVec3) -> f64 {
        1.0 / (1.0 + self.lambda(wo, no))
    }

    fn lambda(&self, wo: DVec3, no: DVec3) -> f64 {
        match self {
            Self::Ggx(roughness) => {
                let wo_dot_no2 = wo.dot(no).powi(2);
                let tan_wo = (1.0 - wo_dot_no2) / wo_dot_no2;
                let roughness2 = roughness * roughness;

                ((1.0 + roughness2 * tan_wo).sqrt() - 1.0) / 2.0
            }
        }
    }
}
