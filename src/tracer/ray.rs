use glam::f32::Vec3;
use crate::tracer::sphere::Sphere;

pub struct Ray {
    pub origin: Vec3,
    pub dir: Vec3
}

impl Ray {
    pub fn at(&self, t: f32) -> Vec3 {
        self.origin + t*self.dir
    }

    pub fn color(&self, scene: &Vec<Sphere>) -> Vec3 {
        for sphere in scene {
            let t = sphere.hit(self);
            if t > 0.0 {
                let normal = (self.at(t) - sphere.origin).normalize();
                return 0.5*(normal + Vec3::ONE);
            }
        }
        let u = self.dir.normalize();
        let t: f32 = 0.5*(u.y + 1.0);
        let c = Vec3::splat(1.0 - t) + t*Vec3::new(0.52, 0.81, 0.92);

        c / c.max_element()
    }
}
