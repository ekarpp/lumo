use crate::DVec3;

/// Epsilon to avoid self intersection of objects
pub const EPSILON: f64 = 0.0001;

/// number of shadow rays per intersection point. UNUSED
#[allow(dead_code)]
pub const SHADOW_RAYS: usize = 4*4;

/// Refraction constant of glass
pub const ETA: f64 = 1.5;

/// Maximum recursion depth of path tracing. UNUSED, russian roulette instead
#[allow(dead_code)]
pub const PATH_TRACE_MAX_DEPTH: usize = 5;

/// Russian roulette probability for the path tracer.
/// Terminates a path at each step with this probability.
/// Computed values are multiplied by the reciprocal of the inverse probability.
pub const PATH_TRACE_RR: f64 = 0.4;

/// specular lobe coefficient. smaller = bigger lobe UNUSED
#[allow(dead_code)]
pub const LOBE_Q: f64 = 10.0;

/// Intensity of the specular lobe UNUSED
#[allow(dead_code)]
pub const SPECULAR_COEFF: DVec3 = DVec3::splat(0.0);

/*
 * TEXTURES
 */

/// Base scale for the size of checker boxes. bigger = smaller boxes
pub const CHECKER_SCALE: f64 = 13.0;

/// Number of points in the perlin noise lattice
pub const PERLIN_POINTS: usize = 256;

/// Scale of points in perlin. bigger = more noticeable effect
pub const PERLIN_SCALE: f64 = 4.0;

/// Frequency of noise in perlin noise. bigger = more frequent
pub const PERLIN_FREQ: f64 = 60.0;

/// Amplitude of the noise pattern in perlin noise
pub const PERLIN_AMP: f64 = 20.0;

/// Recursion depth in perlin turbulence
pub const PERLIN_OCTAVES: usize = 6;

/// Scale of each term in turbulence. should be less than 1.0
pub const PERLIN_GAIN: f64 = 0.5;
