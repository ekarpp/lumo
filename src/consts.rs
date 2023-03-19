/// Epsilon to avoid self intersection of objects
pub const EPSILON: f64 = 1e-10;

/// Refraction constant of glass
pub const ETA: f64 = 1.5;

/// Russian roulette probability for the path tracer.
/// Terminates a path at each step with this probability.
/// Computed values are multiplied by the reciprocal of the inverse probability.
pub const PATH_TRACE_RR: f64 = 0.2;

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
pub const PERLIN_OCTAVES: i32 = 6;

/// Scale of each term in turbulence. should be less than 1.0
pub const PERLIN_GAIN: f64 = 0.5;
