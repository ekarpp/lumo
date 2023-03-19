/// Abstraction for a hit between a ray and an object.
pub mod hit;
/// Abstractions for rays.
pub mod ray;
/// Different BSDFs.
pub mod bxdfs;
/// Scene that describes the 3D world to render.
pub mod scene;
/// Abstraction for a camera
pub mod camera;
/// Abstractions for objects in the 3D world
pub mod object;
/// Textures that can be given to some materials
pub mod texture;
/// Material of an object that defines how it behaves with rays
pub mod material;
/// Integrator to estimate the irradiance at each point
pub mod integrator;
/// MFDistributio
pub mod microfacet;
