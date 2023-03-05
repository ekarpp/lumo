use glam::f64::DVec3;
use crate::tracer::hit::Hit;
use crate::tracer::ray::Ray;
use crate::tracer::scene::Scene;

/**
 * spec_coeff: color of specular lobe
 * q: specular reflection exponent, smaller = more profound lobe
 */
pub fn phong_illum(
    color: DVec3,
    h: &Hit,
    spec_coeff: DVec3,
    q: f64,
    scene: &Scene
) -> DVec3 {
    /* shaded color */
    let mut shaded = color * scene.ambient;

    /* vector to light from hit point */
    let l = scene.light - h.p;

    let ray_to_light = Ray {
        origin: h.p,
        dir: l
    };

    if scene.hit_light(&ray_to_light) {
        /* unit length vector to light from hit point */
        let lu = l.normalize();

        /* l mirrored around sphere normal */
        /* l points in the "wrong direction" but later on, so does h.pp,
         * so they cancel out */
        let r = lu - 2.0 * h.n * lu.dot(h.n);

        shaded += (h.n.dot(lu).max(0.0) * color
                   /* compute the specular lobe */
                   + r.dot(h.p).max(0.0).powf(q) * spec_coeff)
        /* scale by reciprocal of squared distance to light */
            / l.length_squared()
    }

    shaded
}

pub fn reflect_ray(h: &Hit) -> Option<Ray> {
    Some(Ray {
        origin: h.p,
        dir: h.p - 2.0 * h.n * h.p.dot(h.n)
    })
}

pub fn refract_ray(h: &Hit, r: &Ray) -> Option<Ray> {

    /* TODO: CLEANUP AND VERIFY OK */
    const ETA: f64 = 1.5;
    let inside = h.n.dot(r.dir) > 0.0;
    let eta = if inside { ETA } else { 1.0 / ETA };
    let norm = if inside { -h.n } else { h.n };

    /* Snell-Descartes law */
    let up = r.dir.normalize();
    let cos_in = norm.dot(-up).min(1.0);
    let sin_out = (1.0 - cos_in*cos_in)*eta*eta;

    if sin_out > 1.0 {
        return reflect_ray(h);
    }

    let dir = eta*up + norm *
        (eta*cos_in - (1.0 - sin_out).sqrt());

    Some(Ray {
        origin: h.p,
        dir: dir
    })
}
