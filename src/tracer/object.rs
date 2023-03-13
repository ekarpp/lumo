use crate::{DVec3, DMat3, DVec2, DAffine3, DQuat};
use std::f64::consts::PI;
use crate::onb::Onb;
use crate::rand_utils;
use crate::consts::EPSILON;
use crate::tracer::ray::Ray;
use crate::tracer::hit::Hit;
use crate::tracer::material::Material;
use crate::tracer::object::triangle::Triangle;
use crate::tracer::object::rectangle::Rectangle;

/// Defines infinite planes
pub mod plane;
/// Defines cuboids. Built from six rectangles.
pub mod cuboid;
/// Defines spheres.
pub mod sphere;
/// Defines triangles.
pub mod triangle;
/// Defines rectangles. Built from two triangles.
pub mod rectangle;

/* given a triangle, mirror the middle point around to get a rectangle.
 * this is a dumb way... the triangle order matters now.*/
/// Given a triangle, with points read from the columns of the matrix `abc`,
/// returns `b` mirrored to define a rectangle.
fn _triangle_to_rect(abc: DMat3) -> DVec3 {
    abc.col(1) + (abc.col(0) - abc.col(1)) + (abc.col(2) - abc.col(1))
}

/// Common functionality shared between all objects.
pub trait Object: Sync {
    /* unit length normal for r at p. only called during hit creation
     * => no need to implement for rectangle or cuboid. */
    fn normal_at(&self, _p: DVec3) -> DVec3 { todo!() }
    fn is_translucent(&self) -> bool { self.material().is_translucent() }
    /// Number of objects this object consists of.
    fn size(&self) -> usize { 1 }
    /// Is the ray inside this object?
    fn inside(&self, _r: &Ray) -> bool { false }
    /// Sample a random point from our surface that is visible from `p`
    fn sample_towards(&self, _p: DVec3, _rand_sq: DVec2) -> DVec3 { todo!() }
    fn hit(&self, r: &Ray) -> Option<Hit>;
    fn material(&self) -> &Material;
    fn area(&self) -> f64;

    /* default pdf, uniformly at random from surface. */
    fn sample_pdf(&self, p: DVec3, dir: DVec3, h: &Hit) -> f64 {
        p.distance_squared(h.p)
            / (h.norm.dot(-dir.normalize()).max(0.0) * self.area())
    }
}
