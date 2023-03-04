use glam::f64::DVec3;
use crate::tracer::hit::Hit;
use crate::tracer::ray::Ray;

/**
 * spec_coeff: color of specular lobe
 * q: specular reflection exponent, smaller = more profound lobe
 */
pub fn phong_illum(color: DVec3, h: &Hit, spec_coeff: DVec3, q: f64) -> DVec3 {
    /* unit length vector to light from hit point */
    let l = h.l.normalize();

    /* l mirrored around sphere normal */
    /* l points in the "wrong direction" but later on, so does h.pp,
     * so they cancel out */
    let r = l - 2.0 * h.n * l.dot(h.n);


    (h.n.dot(l).max(0.0) * color
     /* compute the specular lobe */
     + r.dot(h.p).max(0.0).powf(q) * spec_coeff)
    /* scale by reciprocal of squared distance to light */
        / h.l.length_squared()
}

pub fn reflect_ray(h: &Hit) -> Option<Ray> {
    Some(Ray {
        origin: h.p,
        dir: h.p - 2.0 * h.n * h.p.dot(h.n)
    })
}

pub fn refract_ray(h: &Hit, r: &Ray) -> Option<Ray> {
    const ETA: f64 = 1.5;
    let eta = if h.inside { ETA } else { 1.0 / ETA };

    /* Snell-Descartes law */
    let up = r.dir.normalize();
    let cos_in = h.n.dot(-up).min(1.0);
    let sin_out = (1.0 - cos_in*cos_in)*eta*eta;

    if sin_out > 1.0 {
        return reflect_ray(h);
    }

    let dir = eta*up + h.n *
        (eta*cos_in - (1.0 - sin_out).sqrt());
    Some(Ray {
        origin: h.p,
        dir: dir
    })
}
