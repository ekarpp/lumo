use crate::{Float, Vec2, Vec3, Vec4};
use std::time::SystemTime;

/// Utility functions to help sample from different geometrics
pub mod maps;

pub fn gen_seed() -> u64 {
    let dt = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();

    let s = dt.as_secs();
    let ns = dt.subsec_nanos() as u64;

    // one round of xorshift64
    let mut seed = (ns << 32) ^ s;
    seed ^= seed << 13;
    seed ^= seed >> 7;
    seed ^= seed << 17;

    seed.max(1)
}

pub struct Xorshift {
    hi: u64,
    lo: u64,
}

impl Default for Xorshift {
    fn default() -> Self {
        let seed = gen_seed();
        println!("RNG seed: {}", seed);
        Xorshift::new(seed)
    }
}

impl Xorshift {

    pub fn new(seed: u64) -> Self {
        let mut rng = Self {
            lo: seed.max(1),
            hi: seed.max(1),
        };

        // step three times to seed properly
        rng.step(); rng.step(); rng.step();

        rng
    }

    /// Xorshiftr128+
    fn step(&mut self) -> u64 {
        let lo = self.lo;
        let mut hi = self.hi;
        self.hi = lo;

        hi ^= hi << 23;
        hi ^= hi >> 17;
        hi ^= lo;

        self.lo = hi.wrapping_add(lo);

        hi
    }

    pub fn gen_u64(&mut self) -> u64 {
        self.step()
    }

    /// Random float in `[0,1)`
    pub fn gen_float(&mut self) -> Float {
        let v = self.step() as Float;

        (v * Float::powi(2.0, -64)).min(1.0 - crate::EPSILON)
    }

    /// Random vector in `[0,1)^2`
    pub fn gen_vec2(&mut self) -> Vec2 {
        Vec2::new(
            self.gen_float(),
            self.gen_float(),
        )
    }

    /// Random vector in `[0,1)^3`
    pub fn gen_vec3(&mut self) -> Vec3 {
        Vec3::new(
            self.gen_float(),
            self.gen_float(),
            self.gen_float(),
        )
    }

    pub fn gen_vec4(&mut self) -> Vec4 {
        Vec4::new(
            self.gen_float(),
            self.gen_float(),
            self.gen_float(),
            self.gen_float(),
        )
    }

    /// Generate a random permutation of length `n`
    pub fn gen_perm(&mut self, n: usize) -> Vec<usize> {
        let mut perm: Vec<usize> = (0..n).collect();

        // Fisher-Yates shuffles
        for i in 0..n-1 {
            let rnd = self.gen_u64() as usize;
            // rnd % (n - i) not uniform, small bias
            let j = i + (rnd % (n - i));
            perm.swap(i, j);
        }

        perm
    }
}
