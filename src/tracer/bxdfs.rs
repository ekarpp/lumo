use crate::pdfs::{Pdf, CosPdf};
use crate::rand_utils;
use crate::consts::ETA;
use crate::tracer::hit::Hit;
use crate::tracer::ray::Ray;

/// Scattering function for diffuse material.
/// # Arguments
/// * `h` - The hit from which we scatter.
/// * `r` - Incoming ray to the hit point.
pub fn diffuse_bsdf(h: &Hit, _r: &Ray) -> Option<(Ray, f64)> {
    let pdf = CosPdf::new(h.norm);
    let dir = pdf.generate_dir(rand_utils::rand_unit_square());
    Some( (Ray::new(h.p, dir), pdf.pdf_val(dir, h)) )
}

/// Scattering function for mirror material. Perfect reflection.
pub fn mirror_bsdf(h: &Hit, _r: &Ray) -> Option<(Ray, f64)> {
    let dir = h.p - 2.0 * h.p.project_onto(h.norm);
    Some( (Ray::new(h.p, dir), 1.0) )
}

/// Scattering function for glass material.
/// Refracts according to Snell-Descartes law.
pub fn glass_bsdf(h: &Hit, r: &Ray) -> Option<(Ray, f64)> {
    let eta_ratio = if h.object.inside(r) { ETA } else { ETA.recip() };
    let norm = if h.object.inside(r) { -h.norm } else { h.norm };

    /* Snell-Descartes law */
    let wo = r.dir.normalize();
    let cos_in = norm.dot(-wo).min(1.0);
    let sin_out = (1.0 - cos_in * cos_in) * eta_ratio * eta_ratio;

    /* total reflection */
    if sin_out > 1.0 {
        return mirror_bsdf(h, r);
    }

    let dir = eta_ratio * wo + norm *
        (eta_ratio * cos_in - (1.0 - sin_out).sqrt());

    Some( (Ray::new(h.p, dir), 1.0) )
}
