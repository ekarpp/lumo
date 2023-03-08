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

    /* shaded color, just ambient for now */
    let mut shaded = DVec3::ZERO;

    if scene.in_light(h.p) {
        /* vector to light from hit point */
        let l = scene.to_light(h.p);
        /* unit length vector to light from hit point */
        let lu = l.normalize();

        /* diffuse term */
        shaded += h.norm.dot(lu).max(0.0) * color;

        /*
        /* l mirrored around sphere normal. pointing from light to hit point */
        let r = 2.0 * h.n * lu.dot(h.n) - lu;
        shaded += -r.dot(h.p.normalize()).max(0.0).powf(q) * spec_coeff;
         */

        /* halfway vector between camera and light. (Blinn-Phong model) */
	/* h.p points in "wrong direction". do these need normalization? */
        let halfway = (l - h.p).normalize();
        shaded += h.norm.dot(halfway).max(0.0).powf(q) * spec_coeff;

        /* scale diffuse and specular by squared distance to light */
        shaded /= l.length_squared();
    }

    shaded + color * scene.ambient
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
