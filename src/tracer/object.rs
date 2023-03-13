use crate::{DVec3, DMat3, DVec2, DAffine3};
use std::f64::consts::PI;
use crate::onb::Onb;
use crate::rand_utils;
use crate::consts::EPSILON;
use crate::tracer::ray::Ray;
use crate::tracer::hit::Hit;
use crate::tracer::material::Material;
use crate::tracer::object::triangle::Triangle;
use crate::tracer::object::rectangle::Rectangle;

pub mod plane;
pub mod cuboid;
pub mod sphere;
pub mod triangle;
pub mod rectangle;

/* make sure normal points the right way for triangle/plane/sphere */
fn _orient_normal(n: DVec3, r: &Ray) -> DVec3 {
    if n.dot(r.dir) > 0.0 { -n } else { n }
}

/* given a triangle, mirror the middle point around to get a rectangle.
 * this is a dumb way... the triangle order matters now.*/
fn _triangle_to_rect(abc: DMat3) -> DVec3 {
    abc.col(1) + (abc.col(0) - abc.col(1)) + (abc.col(2) - abc.col(1))
}

pub trait Object: Sync {
    /* unit length normal for r at p. only called during hit creation
     * => no need to implement for rectangle or cuboid. */
    fn normal_for_at(&self, _r: &Ray, _p: DVec3) -> DVec3 { todo!() }
    fn is_translucent(&self) -> bool { self.material().is_translucent() }
    fn size(&self) -> usize { 1 }
    fn inside(&self, _r: &Ray) -> bool { false }
    fn sample_from(&self, _p: DVec3, _rand_sq: DVec2) -> DVec3 { todo!() }
    fn hit(&self, r: &Ray) -> Option<Hit>;
    fn material(&self) -> &Material;
    fn area(&self) -> f64;

    /* default pdf, uniformly at random from surface. */
    /* do this in scene.rs */
    fn sample_pdf(&self, p: DVec3, dir: DVec3, h: &Hit) -> f64 {
        /* TODO: dont calculate hit */
        p.distance_squared(h.p)
            / (h.norm.dot(-dir.normalize()).max(0.0) * self.area())
    }
}
