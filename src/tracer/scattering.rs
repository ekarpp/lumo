use crate::onb;
use crate::rand_utils;
use crate::consts::ETA;
use crate::tracer::hit::Hit;
use crate::tracer::ray::Ray;

pub fn diffuse_scatter_ray(h: &Hit, r: &Ray) -> Option<Ray> {
    let (u, v) = onb::uvw_basis(h.norm);
    Some(Ray::new(
        h.p,
        onb::to_uvw_basis(rand_utils::rand_unit_hemisphere(), u, v, h.norm),
        r.depth
    ))
}

pub fn reflect_ray(h: &Hit, r: &Ray) -> Option<Ray> {
    Some(Ray::new(
        h.p,
        h.p - 2.0 * h.norm * h.p.dot(h.norm),
        r.depth,
    ))
}

pub fn refract_ray(h: &Hit, r: &Ray) -> Option<Ray> {
    let eta_ratio = if h.object.inside(r) { ETA } else { ETA.recip() };

    /* Snell-Descartes law */
    let up = r.dir.normalize();
    let cos_in = h.norm.dot(-up).min(1.0);
    let sin_out = (1.0 - cos_in * cos_in) * eta_ratio * eta_ratio;

    if sin_out > 1.0 {
        return reflect_ray(h, r);
    }

    let dir = eta_ratio * up + h.norm *
        (eta_ratio * cos_in - (1.0 - sin_out).sqrt());

    Some(Ray::new(
        h.p,
        dir,
        r.depth,
    ))
}
