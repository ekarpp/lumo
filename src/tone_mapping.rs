use crate::DVec3;

/// Enum for different tone mappers
pub enum ToneMap {
    /// Applies no tone mapping
    NoMap,
    /// Clamps values to \[0,1\]
    Clamp,
}

impl ToneMap {
    /// Tone maps the samples in `samples`
    pub fn map(&self, samples: &mut Vec<DVec3>) {
        match self {
            Self::NoMap => (),
            Self::Clamp => {
                for i in 0..samples.len() {
                    samples[i] =
                        samples[i].clamp(DVec3::ZERO, DVec3::ONE);
                }
            }
        }
    }
}
