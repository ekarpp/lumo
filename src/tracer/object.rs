use crate::{DVec3, DMat3, DVec2, DAffine3, DQuat};
use std::f64::{INFINITY, consts::PI};
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
/// Medium, fog, smoke, etc
pub mod medium;
/// Axis aligned bounding boxes
pub mod aabb;
/// Bounding volume hierarchy
pub mod bvh;

/// Common functionality shared between all objects.
pub trait Object: Sync {
    /// Does the ray hit the object?
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<Hit>;

    /// dumb
    fn material(&self) -> &Material;

    /// Is the point inside the object? Care with epsilons here.
    /// How about planar/solid objects?
    fn inside(&self, _p: DVec3) -> bool { false }

    /// Sample random point on the surface of the object
    fn sample_on(&self, _rand_sq: DVec2) -> DVec3;

    /// Sample random ray from `xo` towards area of object
    /// that is visible form `xo`
    ///
    /// # Arguments
    /// * `xo` - Point on the "from" object
    /// * `rand_sq` - Uniformly random point on unit square
    fn sample_towards(&self, _xo: DVec3, _rand_sq: DVec2) -> Ray;

    /// PDF for sampling points on the surface uniformly at random
    ///
    /// # Arguments
    /// * `ri` - Sampled ray from `xo` to `xi`
    fn sample_towards_pdf(&self, ri: &Ray) -> f64;
}
