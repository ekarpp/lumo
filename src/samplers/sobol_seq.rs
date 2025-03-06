const DEG: usize = 10;

pub const SOBOL_MAX_LEN: usize = (1 << DEG) - 1;

// dim = 119
pub const VS1: [u64; DEG] = map_m_v([1, 1, 7, 15, 5, 19, 69, 51, 121, 695]);
// dim = 103
pub const VS2: [u64; DEG] = map_m_v([1, 1, 7, 7, 7, 53, 57, 229, 473, 533]);

const fn map_m_v(mut ms: [u64; DEG]) -> [u64; DEG] {
    let mut i = 0;
    loop {
        ms[i] <<= 32 - i - 1;

        i += 1;
        if i >= DEG {
            break;
        }
    }
    ms
}
