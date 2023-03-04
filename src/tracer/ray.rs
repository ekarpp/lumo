use glam::f32::Vec3;
use crate::tracer::sphere::Sphere;
use crate::tracer::hit::Hit;

pub struct Ray {
    pub origin: Vec3,
    pub dir: Vec3
}

impl Ray {
    pub fn at(&self, t: f32) -> Vec3 {
        self.origin + t*self.dir
    }

    pub fn color(&self, scene: &Vec<Sphere>) -> Vec3 {
        let mut closest_hit: Option<Hit> = None;
        for sphere in scene {
            let h = sphere.hit(self);
            if closest_hit.is_none() {
                closest_hit = h;
            }
            else if h.is_some() && h < closest_hit {
                closest_hit = h;
            }
        }

        match closest_hit {
            Some(h) => h.normal,
            None => {
                let u = self.dir.normalize();
                let t: f32 = 0.5*(u.y + 1.0);
                let c = Vec3::splat(1.0 - t) + t*Vec3::new(0.52, 0.81, 0.92);

                c / c.max_element()
            }
        }
    }
}
