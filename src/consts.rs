use crate::DVec3;

/* epsilon to avoid self intersection of objects */
pub const EPSILON: f64 = 0.001;

/* number of shadow rays per intersection point */
#[allow(dead_code)]
pub const SHADOW_RAYS: usize = 4*4;

/* refraction pub constant of glass */
pub const ETA: f64 = 1.5;

/* maximum recursion depth of path tracing */
pub const PATH_TRACE_MAX_DEPTH: usize = 5;

/* terminate path with this probability at each iteration */
pub const PATH_TRACE_RR: f64 = 0.1;

/* add these to material itself? */
/* specular lobe coefficient. smaller = bigger lobe */
#[allow(dead_code)]
pub const LOBE_Q: f64 = 10.0;

/* intensity of the specular lobe */
#[allow(dead_code)]
pub const SPECULAR_COEFF: DVec3 = DVec3::splat(0.0);

/**
 * TEXTURES
 */

/* base scale for the size of checker boxes. bigger = smaller boxes */
pub const CHECKER_SCALE: f64 = 13.0;

/* number of points in the perlin noise lattice */
pub const PERLIN_POINTS: usize = 256;

/* scale of points in perlin. bigger = more noticeable effect */
pub const PERLIN_SCALE: f64 = 4.0;

/* frequency of noise in perlin noise. bigger = more frequent */
pub const PERLIN_FREQ: f64 = 60.0;

/* amplitude of the noise pattern in perlin noise */
pub const PERLIN_AMP: f64 = 20.0;

/* recursion depth in perlin turbulence */
pub const PERLIN_OCTAVES: usize = 6;

/* scale of each term in turbulence. should be < 1.0 */
pub const PERLIN_GAIN: f64 = 0.5;
