pub use material::Material;
pub use texture::Texture;
pub use object::{Plane, Cube, Sphere, Triangle, Rectangle, AaBoundingBox, Cone};
pub use object::{Disk, Instance, Instanceable, Mesh, KdTree, Object, Bounded};
pub use hit::Hit;
pub use ray::Ray;
pub use integrator::Integrator;

/// Utility struct for orthonormal basis.
mod onb;
/// Abstraction for a hit between a ray and an object.
mod hit;
/// Abstractions for rays.
mod ray;
/// Implementation of different probability density functions for sampling.
mod pdfs;
/// Different BSDFs.
mod bxdfs;
/// Abstractions for objects in the 3D world
mod object;
/// Textures that can be given to some materials
mod texture;
/// Material of an object that defines how it behaves with rays
mod material;
/// Integrator to estimate the irradiance at each point
mod integrator;
/// MFDistribution
mod microfacet;
