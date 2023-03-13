use crate::{DVec3, DMat3, DVec2, DAffine3, DQuat};
use std::f64::consts::PI;
use crate::onb::Onb;
use crate::rand_utils;
use crate::pdfs::{Pdf, CosPdf};
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
    /// Normal of the object at the point `p`. Does not check if `p` is
    /// actually on the object.
    fn normal_at(&self, _p: DVec3) -> DVec3 { todo!() }

    fn is_translucent(&self) -> bool { self.material().is_translucent() }

    /// Number of objects the object consists of.
    fn size(&self) -> usize { 1 }

    /// Is the ray inside the object?
    fn inside(&self, _r: &Ray) -> bool { false }

    /// Sample a random point from surface of the object that is visible from `p`
    /// Returns direction from `p` to us. DO THIS BETTER
    fn sample_towards(&self, _p: DVec3, _rand_sq: DVec2) -> DVec3 { todo!() }

    /// Random point on the surface of the object
    fn sample_on(&self, _rand_sq: DVec2) -> DVec3 { todo!() }

    /// Sample random ray leaving the object
    fn sample_from(&self, rand_sq_o: DVec2, rand_sq_d: DVec2) -> Ray {
        let origin = self.sample_on(rand_sq_o);
        let cos_pdf = CosPdf::new(self.normal_at(origin));
        let dir = cos_pdf.generate_dir(rand_sq_d);
        Ray::new(
            origin,
            dir,
        )
    }

    /// Does the ray hit the object?
    fn hit(&self, r: &Ray) -> Option<Hit>;

    fn material(&self) -> &Material;

    /// Surface area of the object
    fn area(&self) -> f64;

    /* TODO: THIS SHOULD BE DONE BETTER */
    /// PDF for sampling points on the surface that are visible from `p`
    ///
    /// # Arguments
    /// * `p` - The point form which the point sampled on the object should
    /// be visible
    /// * `dir` - Direction from `p` to the sampled point on surface
    /// * `h` - Hit on the surface from `p`
    fn sample_towards_pdf(&self, p: DVec3, dir: DVec3, h: &Hit) -> f64 {
        p.distance_squared(h.p)
            / (h.norm.dot(-dir.normalize()).max(0.0) * self.area())
    }
}
