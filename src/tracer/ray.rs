use glam::f32::Vec3;
use crate::tracer::sphere::Sphere;

pub struct Ray {
    pub origin: Vec3,
    pub dir: Vec3
}

impl Ray {
    pub fn color(&self, scene: &Vec<Sphere>) -> Vec3 {
        if scene[0].hit(self) {
            return Vec3::new(0.0, 1.0, 0.0);
        }
        let u = self.dir.normalize();
        let t: f32 = 0.5*(u.y + 1.0);
        let c = Vec3::splat(1.0 - t) + t*Vec3::new(0.52, 0.81, 0.92);

        c / c.max_element()
    }
}
