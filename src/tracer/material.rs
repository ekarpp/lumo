use glam::f64::DVec3;
use crate::tracer::hit::Hit;
use crate::tracer::ray::Ray;

pub enum Material {
    Default(Default),
    Mirror(Mirror),
    Glass(Glass)
}

impl Material {
    pub fn shade(&self, h: &Hit) -> DVec3 {
        match self {
            Material::Default(d) => d.shade(h),
            _ => DVec3::ZERO,
        }
    }

    pub fn reflect(&self, h: &Hit) -> Option<Ray> {
        match self {
            Material::Mirror(m) => m.reflect(h),
            _ => None,
        }
    }

    pub fn refract(&self, h: &Hit, r: &Ray) -> Option<Ray> {
        match self {
            Material::Glass(g) => g.refract(h, r),
            _ => None,
        }
    }

    pub fn is_translucent(&self) -> bool {
        match self {
            Material::Glass(_) => true,
            _ => false,
        }
    }

    pub fn color(&self) -> DVec3 {
        match self {
            Material::Default(d) => d.color,
            _ => DVec3::ZERO,
        }
    }
}

pub struct Default {
    pub color: DVec3
}

impl Default {
    pub fn shade(&self, h: &Hit) -> DVec3 {
        crate::tracer::phong::phong_shading(
            h,
            DVec3::splat(0.9),
            3.0
        )
    }
}

pub struct Mirror {}

impl Mirror {
    pub fn reflect(&self, h: &Hit) -> Option<Ray> {
        Some(Ray {
            origin: h.p,
            dir: h.p - 2.0 * h.n * h.p.dot(h.n)
        })
    }
}

pub struct Glass {}

impl Glass {
    pub fn refract(&self, h: &Hit, r: &Ray) -> Option<Ray> {
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
}
