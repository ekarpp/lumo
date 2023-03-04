use glam::f64::DVec3;
use crate::tracer::hit::Hit;
use crate::tracer::ray::Ray;

pub trait Material {
    fn shade(&self, h: &Hit) -> DVec3;
    fn reflect(&self, h: &Hit) -> Option<Ray>;
    fn refract(&self, h: &Hit, r: &Ray) -> Option<Ray>;
    fn is_translucent(&self) -> bool;
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

    fn refract(&self, _h: &Hit, _r: &Ray) -> Option<Ray> {
        None
    }

    fn is_translucent(&self) -> bool { false }
}

pub struct Mirror {}

impl Material for Mirror {
    fn shade(&self, _h: &Hit) -> DVec3 {
        DVec3::ZERO
    }

    fn reflect(&self, h: &Hit) -> Option<Ray> {
        Some(Ray {
            origin: h.p,
            dir: h.p - 2.0 * h.n * h.p.dot(h.n)
        })
    }

    fn refract(&self, _h: &Hit, _r: &Ray) -> Option<Ray> {
        None
    }

    fn is_translucent(&self) -> bool { false }
}

pub struct Glass {}

impl Material for Glass {
    fn shade(&self, _h: &Hit) -> DVec3 {
        DVec3::ZERO
    }

    fn reflect(&self, _h: &Hit) -> Option<Ray> {
        None
    }

    fn refract(&self, h: &Hit, r: &Ray) -> Option<Ray> {
        const ETA: f64 = 1.5;
        let eta = if h.inside { ETA } else { 1.0 / ETA };

        /* Snell-Descartes law */
        let up = r.dir.normalize();
        let cos_in = h.n.dot(-up).min(1.0);
        let sin_out = (1.0 - cos_in*cos_in)*eta*eta;

        if sin_out > 1.0 {
            // REFLECT
            return Some(Ray {
                origin: h.p,
                dir: h.p - 2.0 * h.n * h.p.dot(h.n)
            });
        }

        let dir = eta*up + h.n *
            (eta*cos_in - (1.0 - sin_out).sqrt());
        Some(Ray {
            origin: h.p,
            dir: dir
        })
    }

    fn is_translucent(&self) -> bool { true }
}
