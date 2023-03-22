/// Abstraction for a hit between a ray and an object.
pub mod hit;
/// Abstractions for rays.
pub mod ray;
/// Different BSDFs.
pub mod bxdfs;
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
