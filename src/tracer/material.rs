use glam::f64::DVec3;
use crate::tracer::hit::Hit;
use crate::tracer::ray::Ray;

pub trait Material {
    fn shade(&self, h: &Hit) -> DVec3;
    fn reflect(&self, h: &Hit) -> Option<Ray>;
    fn refract(&self, h: &Hit) -> Option<Ray>;
}

pub struct Default {}

impl Material for Default {
    fn shade(&self, h: &Hit) -> DVec3 {
        crate::tracer::phong::phong_shading(
            h,
            DVec3::splat(0.9),
            3.0
        )
    }

    fn reflect(&self, _h: &Hit) -> Option<Ray> {
        None
    }

    fn refract(&self, _h: &Hit) -> Option<Ray> {
        None
    }
}

pub struct Mirror {}

impl Material for Mirror {
    fn shade(&self, _h: &Hit) -> DVec3 {
        DVec3::ZERO
    }

    fn reflect(&self, h: &Hit) -> Option<Ray> {
        Some(Ray {
            origin: h.p + crate::EPSILON * h.n,
            dir: h.p - 2.0 * h.n * h.p.dot(h.n)
        })
    }

    fn refract(&self, _h: &Hit) -> Option<Ray> {
        None
    }
}

pub struct Glass {}

impl Material for Glass {
    fn shade(&self, _h: &Hit) -> DVec3 {
        DVec3::ZERO
    }

    fn reflect(&self, _h: &Hit) -> Option<Ray> {
        None
    }

    fn refract(&self, h: &Hit) -> Option<Ray> {
        const ETA: f64 = 1.5;
        let eta = if h.inside { ETA } else { 1.0 / ETA };

        /* Snell-Descartes law */
        let up = h.p.normalize();
        let cos_in = h.n.dot(-up);
        let sin_in = (1.0 - cos_in*cos_in).sqrt();
        if sin_in < 1.0 / eta {
            // REFLECT
            return Some(Ray {
                origin: h.p + crate::EPSILON * h.n,
                dir: h.p - 2.0 * h.n * h.p.dot(h.n)
            });
        }

        let dir = eta*up + h.n*
            (eta*cos_in - (1.0 - eta*eta*sin_in*sin_in).sqrt());
        Some(Ray {
            origin: h.p + if h.inside {
                -crate::EPSILON*h.n
            } else {
                crate::EPSILON*h.n
            },
            dir: dir
        })
    }
}
