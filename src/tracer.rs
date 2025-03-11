pub use camera::{ Camera, CameraBuilder, CameraType };
pub use color::{Color, ColorWavelength, Spectrum, RGB, ColorSpace};
pub use film::{Film, FilmTile, FilmSample};
pub use integrator::Integrator;
pub use material::Material;
pub use medium::Medium;
pub use object::{
    Disk, Instance, Instanceable, KdTree, Object, BVH,
    Cone, Cube, Cylinder, Rectangle, Sphere, Triangle,
    Sampleable, TriangleMesh, Face, Mesh
};
pub use scene::Scene;
pub use texture::Texture;
pub use filter::PixelFilter;

mod bxdf;
mod bsdf;
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
/// Volumetric mediums
mod medium;
/// MFDistribution
mod microfacet;
/// Abstractions for objects in the 3D world
mod object;
/// Utility struct for orthonormal basis.
mod onb;
/// Abstractions for rays.
mod ray;
/// Scene that describes the 3D world to render.
mod scene;
/// Textures that can be given to some materials
mod texture;
/// Filters for film samples
mod filter;
