use crate::DVec3;
use std::f64::consts::PI;
use crate::onb;
use crate::rand_utils;

pub trait Pdf {
    fn generate_dir(&self) -> DVec3;
    fn pdf_val(&self, dir: DVec3) -> f64;
}

/* cosine weighed samples on hemisphere pointing towards w */
pub struct CosPdf {
    u: DVec3,
    v: DVec3,
    w: DVec3,
}

impl CosPdf {
    fn new(norm: DVec3) -> Self {
        let (u, v) = onb::uvw_basis(norm);
        Self {
            u: u,
            v: v,
            w: norm,
        }
    }
}

impl Pdf for CosPdf {
    fn generate_dir(&self) -> DVec3 {
        onb::to_uvw_basis(
            rand_utils::rand_cos_unit_hemisphere(),
            self.u,
            self.v,
            self.w,
        )
    }

    fn pdf_val(&self, dir: DVec3) -> f64 {
        dir.dot(self.w) / PI
    }
}
