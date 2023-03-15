use crate::{DVec3, DMat3, DVec2, DAffine3, DQuat};
use std::f64::consts::PI;
use crate::onb::Onb;
use crate::rand_utils;
use rand_utils::RandomShape;
use crate::consts::EPSILON;
use crate::tracer::ray::Ray;
use crate::tracer::hit::Hit;
use crate::tracer::material::Material;
use crate::tracer::object::triangle::Triangle;
use crate::tracer::object::rectangle::Rectangle;
use crate::tracer::object::aabb::AxisAlignedBoundingBox;

/* given a triangle, mirror the middle point around to get a rectangle.
 * this is a dumb way... the triangle order matters now.*/
/// Given a triangle, with points read from the columns of the matrix `abc`,
/// returns `b` mirrored to define a rectangle.
fn _triangle_to_rect(abc: DMat3) -> DVec3 {
    abc.col(1) + (abc.col(0) - abc.col(1)) + (abc.col(2) - abc.col(1))
}

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
/// Axis aligned bounding boxes
pub mod aabb;
/// Bounding volume hierarchy
pub mod bvh;

/// Common functionality shared between all objects.
pub trait Object: Sync {
    /// Normal of the object at the point `p`. Does not check if `p` is
    /// actually on the object.
    fn normal_at(&self, _p: DVec3) -> DVec3;
    /// Does the ray hit the object?
    fn hit(&self, r: &Ray) -> Option<Hit>;
    fn material(&self) -> &Material;
    /// Surface area of the object
    fn area(&self) -> f64;
    /// Number of objects the object consists of.
    fn size(&self) -> usize { 1 }

    /// Is the point inside the object? Care with epsilons here
    fn inside(&self, _p: DVec3) -> bool { false }

    /// Sample random ray from `xo` towards area of object
    /// that is visible form `xo`
    ///
    /// # Arguments
    /// * `xo` - Point on the "from" object
    /// * `rand_sq` - Uniformly random point on unit square
    fn sample_towards(&self, _xo: DVec3, _rand_sq: DVec2) -> Ray;

    /// Sample random point on the surface of the object
    fn sample_on(&self, _rand_sq: DVec2) -> DVec3;

    /// Sample random ray leaving the object. Random unit square points for
    /// ray direction and origin separately. Special care for sphere/cuboid?
    /// What if non-solids face wrong direction?
    fn sample_from(&self, _rand_sq_o: DVec2, _rand_sq_d: DVec2) -> (Ray, f64) {
        todo!()
        /*
        let xi = self.sample_on(rand_sq_o);
        let cos_pdf = CosPdf::new(self.normal_at(xi));
        let wi = cos_pdf.generate_dir(rand_sq_d);
        ( Ray::new(xi, wi), cos_pdf.pdf_val(wi) + /* origin prob */ 0.0 )
         */
    }

    /* TODO: THIS SHOULD BE DONE BETTER */
    /// PDF for sampling points on the surface uniformly at random
    ///
    /// # Arguments
    /// * `xo` - Point on the "from" object
    /// * `hi` - Hit on the "towards" object in the sampled direction
    /// * `xi` - Randomly drawn point on the "towards" object
    /// * `wi` - Direction towards `xi` from `xo`. Not normalized.
    /// * `ni` - Normal of "towards" object at `xi`
    //fn sample_area_pdf(&self, xo: DVec3, xi: DVec3, wi: DVec3, ni: DVec3)
    fn sample_area_pdf(&self, xo: DVec3, wi: DVec3, hi: &Hit)
                       -> f64 {
        let xi = hi.p;
        let ni = hi.norm;
        xo.distance_squared(xi)
            / (ni.dot(wi.normalize()).abs() * self.area())
    }
}
