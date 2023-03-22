use crate::{DVec3, DMat3, DVec2, DAffine3};
use std::f64::{INFINITY, consts::PI};
use crate::tracer::onb::Onb;
use crate::rand_utils;
use crate::consts::EPSILON;
use crate::tracer::ray::Ray;
use crate::tracer::hit::Hit;
use crate::tracer::material::Material;

/* given a triangle, mirror the middle point around to get a rectangle.
 * this is a dumb way... the triangle order matters now.*/
/// Given a triangle, with points read from the columns of the matrix `abc`,
/// returns `b` mirrored to define a rectangle.
fn _triangle_to_rect(abc: DMat3) -> DVec3 {
    abc.col(1) + (abc.col(0) - abc.col(1)) + (abc.col(2) - abc.col(1))
}

pub use plane::Plane;
pub use cube::Cube;
pub use sphere::Sphere;
pub use triangle::Triangle;
pub use rectangle::Rectangle;
pub use aabb::AaBoundingBox;
pub use kdtree::*;
pub use instance::*;

/// Defines infinite planes
pub mod plane;
/// Defines a unit cube. Transform to desired shape with instances.
pub mod cube;
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
/// kD-trees, used for complex meshes
pub mod kdtree;
/// Instance of an object i.e. an object to wich Euclidean (+ scaling)
/// transformations have been applied to.
pub mod instance;

/// Common functionality shared between all objects.
pub trait Object: Sync {
    /// Does the ray hit the object?
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<Hit>;

    /// dumb
    fn material(&self) -> &Material;

    /// Is the point inside the object? Care with epsilons here.
    /// How about planar/solid objects? Get rid of this, just check from normal
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

/// Objects that can be contained within an AABB
pub trait Bounded: Object {
    /// Axis aligned box that contains the object
    fn bounding_box(&self) -> AaBoundingBox;
}
