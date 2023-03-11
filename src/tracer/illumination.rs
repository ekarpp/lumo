use crate::DVec3;
use crate::consts::{SHADOW_RAYS, ETA};
use crate::tracer::hit::Hit;
use crate::tracer::ray::Ray;
use crate::tracer::scene::Scene;
use crate::tracer::texture::Texture;

pub fn illuminate(texture: &Texture, h: &Hit, scene: &Scene) -> DVec3 {
    texture.albedo_at(h.p)
        * (scene.ambient +
           scene.sample_lights_from(h).iter().map(|lh: &Hit| {
               let r = Ray::new(
                   h.p,
                   lh.p - h.p,
                   0,
               );
               h.object.scatter_pdf(&r, h)
                   / lh.object.pdf(&r, lh)
           }).fold(DVec3::ZERO, |acc, c| acc + c) / SHADOW_RAYS as f64)
}

pub fn reflect_ray(h: &Hit, r: &Ray) -> Ray {
    Ray::new(
        h.p,
        h.p - 2.0 * h.norm * h.p.dot(h.norm),
        r.depth,
    )
}

pub fn refract_ray(h: &Hit, r: &Ray) -> Ray {
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
