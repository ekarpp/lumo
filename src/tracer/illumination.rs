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

    /* vector to light from hit point */
    let l = scene.light - h.p;

    let ray_to_light = Ray {
        origin: h.p,
        dir: l
    };

    if scene.hit_light(&ray_to_light) {
        /* unit length vector to light from hit point */
        let lu = l.normalize();

        /* diffuse term */
        shaded += h.n.dot(lu).max(0.0) * color;


        /*
        /* l mirrored around sphere normal. pointing from light to hit point */
        let r = 2.0 * h.n * lu.dot(h.n) - lu;
        shaded += -r.dot(h.p.normalize()).max(0.0).powf(q) * spec_coeff;
         */

        /* halfway vector between camera and light. (Blinn-Phong model) */
        let halfway = (lu - h.p.normalize()) / (lu - h.p.normalize()).length();
        shaded += h.n.dot(halfway).max(0.0).powf(q) * spec_coeff;

        /* scale diffuse and specular by squared distance to light */
        shaded /= l.length_squared();
    }

    /* add ambient term */
    shaded + color * scene.ambient
}

pub fn reflect_ray(h: &Hit) -> Option<Ray> {
    Some(Ray {
        origin: h.p,
        dir: h.p - 2.0 * h.n * h.p.dot(h.n)
    })
}

pub fn refract_ray(h: &Hit, r: &Ray) -> Option<Ray> {
    const ETA: f64 = 1.5;

    let inside = h.n.dot(r.dir) > 0.0;
    let eta_ratio = if inside { ETA } else { 1.0 / ETA };
    let norm = if inside { -h.n } else { h.n };

    /* Snell-Descartes law */
    let up = r.dir.normalize();
    let cos_in = norm.dot(-up).min(1.0);
    let sin_out = (1.0 - cos_in*cos_in)*eta_ratio*eta_ratio;

    if sin_out > 1.0 {
        return reflect_ray(h);
    }

    let dir = eta_ratio*up + norm *
        (eta_ratio*cos_in - (1.0 - sin_out).sqrt());

    Some(Ray {
        origin: h.p,
        dir: dir
    })
}
