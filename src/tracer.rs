pub use camera::Camera;
pub use color::Color;
pub use film::{Film, FilmSample};
pub use integrator::Integrator;
pub use material::Material;
pub use object::{
    Bounded, Disk, Instance, Instanceable, KdTree, Object,
    Cone, Cube, Cylinder, Plane, Rectangle, Sphere, Triangle,
    Sampleable, Medium, TriangleMesh, Face, Mesh
};
pub use scene::Scene;
pub use texture::Texture;
pub use filter::Filter;

/// Different BSDFs.
mod bxdfs;
/// Abstraction for a camera
mod camera;
/// Color struct
mod color;
/// Film contains the image being rendered
mod film;
/// Abstraction for a hit between a ray and an object.
mod hit;
/// Integrator to estimate the irradiance at each point
mod integrator;
/// Material of an object that defines how it behaves with rays
mod material;
/// MFDistribution
mod microfacet;
/// Abstractions for objects in the 3D world
mod object;
/// Utility struct for orthonormal basis.
mod onb;
/// Implementation of different probability density functions for sampling.
mod pdfs;
/// Abstractions for rays.
mod ray;
/// Scene that describes the 3D world to render.
mod scene;
/// Textures that can be given to some materials
mod texture;
/// Filters for film samples
mod filter;
