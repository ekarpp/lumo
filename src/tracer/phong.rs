use glam::f32::Vec3;
use crate::tracer::hit::Hit;

/**
 * spec_coeff: color of specular lobe
 * q: specular reflection exponent, smaller = more profound lobe
 */
pub fn phong_shading(h: &Hit, spec_coeff: Vec3, q: f32) -> Vec3 {
    /* unit length vector to light from hit point */
    let l = h.l.normalize();

    /* l mirrored around sphere normal */
    let r = l - 2.0 * h.n * l.dot(h.n).max(0.0);


    (h.n.dot(l).max(0.0) * h.sphere.color
     /* compute the specular lobe */
     + r.dot(h.p).max(0.0).powf(q) * spec_coeff)
        /* scale by reciprocal of squared distance to light */
        / h.l.length_squared()
}
