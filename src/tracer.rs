pub use scene::Scene;
pub use camera::Camera;
pub use texture::Texture;
pub use material::Material;
pub use integrator::Integrator;
pub use object::{Plane, Cube, Sphere, Triangle, Rectangle, Cone, Cylinder};
pub use object::{Disk, Instance, Instanceable, Mesh, KdTree, Object, Bounded};

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
/// Scene that describes the 3D world to render.
mod scene;
/// Abstraction for a camera
mod camera;
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
