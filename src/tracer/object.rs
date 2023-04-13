use crate::rand_utils;
use crate::Axis;
use crate::EPSILON;
use crate::tracer::hit::Hit;
use crate::tracer::material::Material;
use crate::tracer::onb::Onb;
use crate::tracer::ray::Ray;
use glam::{DAffine3, DMat3, DVec2, DVec3};
use std::f64::{consts::PI, INFINITY};

pub use aabb::AaBoundingBox;
pub use cone::Cone;
pub use cube::Cube;
pub use cylinder::Cylinder;
pub use disk::Disk;
pub use instance::{Instance, Instanceable};
pub use kdtree::{KdTree, Mesh};
pub use medium::Medium;
pub use plane::Plane;
pub use rectangle::Rectangle;
pub use sphere::Sphere;
pub use triangle::Triangle;

/// Axis aligned bounding boxes
mod aabb;
/// Defines cones
mod cone;
/// Defines a unit cube. Transform to desired shape with instances.
mod cube;
/// Defines y axis aligned cylinders
mod cylinder;
/// Defines disks
mod disk;
/// Instance of an object i.e. an object to wich Euclidean (+ scaling)
/// transformations have been applied to.
mod instance;
/// kD-trees, used for complex meshes
mod kdtree;
/// Medium, fog, smoke, etc
mod medium;
/// Defines infinite planes
mod plane;
/// Defines rectangles. Built from two triangles.
mod rectangle;
/// Defines spheres.
mod sphere;
/// Defines triangles.
mod triangle;

/// Common functionality shared between all objects.
pub trait Object: Sync {
    /// Does the ray hit the object? NOTE: ray direction can be unnormalized
    /// for instanced objects. Is this an issue?
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<Hit>;

    /// dumb
    fn material(&self) -> &Material;
}

/// Objects that can be contained within an AABB
pub trait Bounded: Object {
    /// Axis aligned box that contains the object
    fn bounding_box(&self) -> AaBoundingBox;
}

/// An object that has volume in three dimensions
pub trait Solid: Object {
    /// Is `xo` inside the solid?
    fn inside(&self, xo: DVec3) -> bool;
}

/// Object towards which rays can be sampled
pub trait Sampleable: Object {
    /// Sample random point on the surface of the object
    fn sample_on(&self, rand_sq: DVec2) -> DVec3;

    /// Sample random ray from `xo` towards area of object
    /// that is visible form `xo`
    ///
    /// # Arguments
    /// * `xo` - Point on the "from" object
    /// * `rand_sq` - Uniformly random point on unit square
    fn sample_towards(&self, xo: DVec3, rand_sq: DVec2) -> Ray;

    /// PDF for sampling points on the surface uniformly at random. Returns PDF
    /// and normal at intersection, if found.
    ///
    /// # Arguments
    /// * `ri` - Sampled ray from `xo` to `xi`
    fn sample_towards_pdf(&self, ri: &Ray) -> (f64, DVec3);
}
