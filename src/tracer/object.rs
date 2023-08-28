use crate::{
    rand_utils, Axis, efloat::{self, EFloat},
    Point, Normal, Direction, Float,
    Vec3, Vec2, Transform, Mat3
};
use crate::tracer::{
    hit::Hit, material::Material,
    onb::Onb, ray::Ray
};
use std::sync::Arc;

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
pub use triangle_mesh::{TriangleMesh, Face};

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
/// Volumetric mediums
mod medium;
/// Defines infinite planes
mod plane;
/// Defines rectangles. Built from two triangles.
mod rectangle;
/// Defines spheres.
mod sphere;
/// Defines triangles.
mod triangle;
/// Triangle meshes, stores vertices, normals and texture coordinates to save space
mod triangle_mesh;

/// Common functionality shared between all objects.
pub trait Object: Sync {
    /// Does the ray hit the object? NOTE: ray direction can be unnormalized
    /// for instanced objects. Is this an issue?
    fn hit(&self, r: &Ray, t_min: Float, t_max: Float) -> Option<Hit>;
}

/// Objects that can be contained within an AABB
pub trait Bounded: Object {
    /// Axis aligned box that contains the object
    fn bounding_box(&self) -> AaBoundingBox;
}

/// Object towards which rays can be sampled
pub trait Sampleable: Object {
    /// Returns surface area of the object
    fn area(&self) -> Float;

    /// Samples a ray leaving at random point on the surface of the object.
    /// Direction cos weighed on the hemisphere. Returns also normal at ray origin
    fn sample_leaving(&self, rand_sq0: Vec2, rand_sq1: Vec2) -> (Ray, Hit) {
        let ho = self.sample_on(rand_sq0);
        let ns = ho.ns;
        let xo = ho.p;
        let uvw = Onb::new(ns);
        let wi_local = rand_utils::square_to_cos_hemisphere(rand_sq1);
        let wi = uvw.to_world(wi_local);
        // pdf start = 1 / area
        // pdf dir = cos hemisphere
        // prob want to make sample_leaving_pdf function
        (Ray::new(xo, wi), ho)
    }

    /// Returns PDF for sampled ray (i) origin and (ii) direction
    fn sample_leaving_pdf(&self, r: &Ray, ng: Normal) -> (Float, Float) {
        let pdf_origin = 1.0 / self.area();
        let wi = r.dir;
        let cos_theta = ng.dot(wi);
        let pdf_dir = cos_theta / crate::PI;

        (pdf_origin, pdf_dir)
    }

    /// Returns randomly sampled point on the surface of the object
    /// and the normal at the point.
    fn sample_on(&self, rand_sq: Vec2) -> Hit;

    /// Sample random direction from `xo` towards area of object
    /// that is visible form `xo`
    ///
    /// # Arguments
    /// * `xo` - Point on the "from" object
    /// * `rand_sq` - Uniformly random point on unit square
    fn sample_towards(&self, xo: Point, rand_sq: Vec2) -> Direction {
        let xi = self.sample_on(rand_sq).p;
        xi - xo
    }

    /// PDF for sampling points on the surface uniformly at random. Returns PDF
    /// with respect to area and hit to self, if found.
    ///
    /// # Arguments
    /// * `ri` - Sampled ray from `xo` to `xi`
    fn sample_towards_pdf(&self, ri: &Ray) -> (Float, Option<Hit>) {
        match self.hit(ri, 0.0, crate::INF) {
            None => (0.0, None),
            Some(hi) => {
                // might not work for solids, cause might be multiple hits
                let p = 1.0 / self.area();

                (p, Some(hi))
            }
        }
    }
}
