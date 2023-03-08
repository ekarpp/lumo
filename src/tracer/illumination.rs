use crate::DVec3;
use crate::tracer::hit::Hit;
use crate::tracer::ray::Ray;
use crate::tracer::scene::Scene;
use crate::tracer::texture::Texture;

/**
 * spec_coeff: color of specular lobe
 * q: specular reflection exponent, smaller = more profound lobe
 */
pub fn phong_illum(
    texture: &Texture,
    h: &Hit,
    spec_coeff: DVec3,
    q: f64,
    scene: &Scene
) -> DVec3 {
    let color = texture.color_at(h.p);

    match scene.ratio_in_light(h.p) {
        None => color * scene.ambient,
        Some(r) => color * scene.ambient
            + r * _diffuse_specular(color, h, scene.to_light(h.p), spec_coeff, q),
    }
}

fn _diffuse_specular(color: DVec3, h: &Hit, l: DVec3, spec_coeff: DVec3, q: f64)
                     -> DVec3 {
    /* unit length vector to light from hit point */
    let lu = l.normalize();

    /*
    /* l mirrored around sphere normal. pointing from light to hit point */
    let r = 2.0 * h.n * lu.dot(h.n) - lu;
    shaded += -r.dot(h.p.normalize()).max(0.0).powf(q) * spec_coeff;
     */

    /* halfway vector between camera and light. (Blinn-Phong model) */
    /* h.p points in "wrong direction". do these need normalization? */
    let halfway = (l - h.p).normalize();

    /* specular term */
    (h.norm.dot(halfway).max(0.0).powf(q) * spec_coeff
       /* diffuse term */
     + h.norm.dot(lu).max(0.0) * color)
        /* scale by squared distance to light */
        / l.length_squared()
}

pub fn reflect_ray(h: &Hit, r: &Ray) -> Ray {
    Ray::new(
        h.p,
        h.p - 2.0 * h.norm * h.p.dot(h.norm),
        r.depth,
    )
}

pub fn refract_ray(h: &Hit, r: &Ray) -> Ray {
    const ETA: f64 = 1.5;
    let eta_ratio = if h.object.inside(r) { ETA } else { 1.0 / ETA };

    /* Snell-Descartes law */
    let up = r.dir.normalize();
    let cos_in = h.norm.dot(-up).min(1.0);
    let sin_out = (1.0 - cos_in*cos_in)*eta_ratio*eta_ratio;

    if sin_out > 1.0 {
        return reflect_ray(h, r);
    }

    let dir = eta_ratio*up + h.norm *
        (eta_ratio*cos_in - (1.0 - sin_out).sqrt());

    Ray::new(
        h.p,
        dir,
        r.depth,
    )
}
