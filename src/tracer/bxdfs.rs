use crate::pdfs::{Pdf, CosPdf, UnitPdf};
use crate::rand_utils;
use crate::consts::ETA;
use crate::tracer::hit::Hit;
use crate::tracer::ray::{Ray, ScatterRay};

/* */
pub fn diffuse_bsdf(h: &Hit, _r: &Ray) -> Option<ScatterRay> {
    let pdf = CosPdf::new(h.norm);

    ScatterRay::new(
        Ray::new(
            h.p,
            pdf.generate_dir(rand_utils::rand_unit_square()),
        ),
        Box::new(pdf),
    )
}

/* perfect reflection */
pub fn mirror_bsdf(h: &Hit, _r: &Ray) -> Option<ScatterRay> {
    ScatterRay::new(
        Ray::new(
            h.p,
            h.p - 2.0 * h.norm * h.p.dot(h.norm),
        ),
        Box::new(UnitPdf::new()),
    )
}

pub fn glass_bsdf(h: &Hit, r: &Ray) -> Option<ScatterRay> {
    let eta_ratio = if h.object.inside(r) { ETA } else { ETA.recip() };

    /* Snell-Descartes law */
    let up = r.dir.normalize();
    let cos_in = h.norm.dot(-up).min(1.0);
    let sin_out = (1.0 - cos_in * cos_in) * eta_ratio * eta_ratio;

    /* total reflection */
    if sin_out > 1.0 {
        return mirror_bsdf(h, r);
    }

    let dir = eta_ratio * up + h.norm *
        (eta_ratio * cos_in - (1.0 - sin_out).sqrt());

    ScatterRay::new(
        Ray::new(
            h.p,
            dir,
        ),
        Box::new(UnitPdf::new()),
    )
}
