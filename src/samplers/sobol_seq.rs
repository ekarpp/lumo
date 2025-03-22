const DEG: usize = 10;

pub const SOBOL_MAX_LEN: usize = (1 << DEG) - 1;
const NUM_BATCHES: usize = 1 + SOBOL_MAX_LEN / crate::renderer::SAMPLES_INCREMENT as usize;

// dim = 119
pub const VS1: [u64; DEG] = map_m_v([1, 1, 7, 15, 5, 19, 69, 51, 121, 695]);
// dim = 103
pub const VS2: [u64; DEG] = map_m_v([1, 1, 7, 7, 7, 53, 57, 229, 473, 533]);

pub const BATCH_STATES: [(u64, u64); NUM_BATCHES] = get_batch_states();

const fn get_batch_states() -> [(u64, u64); NUM_BATCHES] {
    let mut out = [(0,0); NUM_BATCHES];
    let mut state = 0;
    let mut prev = out[0];

    while state < SOBOL_MAX_LEN {
        if state % crate::renderer::SAMPLES_INCREMENT as usize == 0 {
            out[state / crate::renderer::SAMPLES_INCREMENT as usize] = prev;
        }
        state += 1;
        prev = (
            prev.0 ^ VS1[state.trailing_zeros() as usize],
            prev.1 ^ VS2[state.trailing_zeros() as usize],
        );
    }

    out
}

const fn map_m_v(mut ms: [u64; DEG]) -> [u64; DEG] {
    let mut i = 0;
    while i < DEG {
        ms[i] <<= 64 - i - 1;
        i += 1;
    }
    ms
}
