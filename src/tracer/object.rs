use glam::{DVec3, DMat3, DVec2, DAffine3};
use std::f64::{INFINITY, consts::PI};
use crate::tracer::onb::Onb;
use crate::rand_utils;
use crate::EPSILON;
use crate::tracer::ray::Ray;
use crate::tracer::hit::Hit;
use crate::tracer::material::Material;

pub use plane::Plane;
pub use cube::Cube;
pub use sphere::Sphere;
pub use triangle::Triangle;
pub use rectangle::Rectangle;
pub use aabb::AaBoundingBox;
pub use cylinder::Cylinder;
pub use cone::Cone;
pub use disk::Disk;
pub use kdtree::{Mesh, KdTree};
pub use instance::{Instance, Instanceable};

/// Defines disks
mod disk;
/// Defines cones
mod cone;
/// Defines infinite planes
mod plane;
/// Defines a unit cube. Transform to desired shape with instances.
mod cube;
/// Defines spheres.
mod sphere;
/// Defines triangles.
mod triangle;
/// Defines rectangles. Built from two triangles.
mod rectangle;
/// Defines y axis aligned cylinders
mod cylinder;
/// Medium, fog, smoke, etc
mod medium;
/// Axis aligned bounding boxes
mod aabb;
/// kD-trees, used for complex meshes
mod kdtree;
/// Instance of an object i.e. an object to wich Euclidean (+ scaling)
/// transformations have been applied to.
mod instance;

/// Common functionality shared between all objects.
pub trait Object: Sync {
    /// Does the ray hit the object? NOTE: ray direction can be unnormalized
    /// for instanced objects. Is this an issue?
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<Hit>;

    /// dumb
    fn material(&self) -> &Material;

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
