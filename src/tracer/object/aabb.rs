#![allow(unused_variables, dead_code)]
use super::*;

/// Axis aligned bounding box
#[derive(Copy, Clone)]
pub struct AaBoundingBox {
    pub ax_min: DVec3,
    pub ax_max: DVec3,
}

impl Default for AaBoundingBox {
    fn default() -> Self {
        Self {
            ax_min: DVec3::splat(-INFINITY),
            ax_max: DVec3::splat(INFINITY),
        }
    }
}

impl AaBoundingBox {
    pub fn new(ax_min: DVec3, ax_max: DVec3) -> Self {
        Self {
            ax_min,
            ax_max,
        }
    }

    /// Get minimum value in `axis` (`0=x, 1=y, 2=z`).
    pub fn get_min_axis(&self, axis: usize) -> f64 {
        match axis {
            0 => self.ax_min.x,
            1 => self.ax_min.y,
            2 => self.ax_min.z,
            _ => panic!(),
        }
    }

    /// Get maximum value in `axis` (`0=x, 1=y, 2=z`).
    pub fn get_max_axis(&self, axis: usize) -> f64 {
        match axis {
            0 => self.ax_max.x,
            1 => self.ax_max.y,
            2 => self.ax_max.z,
            _ => panic!(),
        }
    }

    pub fn intersect(&self, r: &Ray) -> bool {
        let mut ts = -f64::INFINITY;
        let mut te = f64::INFINITY;
        let ro_min = (self.ax_min - r.origin).to_array();
        let ro_max = (self.ax_max - r.origin).to_array();
        let rd = r.dir.to_array();

        (0..3).for_each(|i: usize| {
            let t1 = ro_min[i] / rd[i];
            let t2 = ro_max[i] / rd[i];
            ts = ts.max(t1);
            te = te.min(t2);
        });

        ts < te && te > EPSILON
    }

    /// Combine self and other to a new bigger AABB
    pub fn merge(&self, other: &Self) -> Self {
        Self {
            ax_min: self.ax_min.min(other.ax_min),
            ax_max: self.ax_max.max(other.ax_max),
        }
    }

    /// Split `self` along `axis` (x=0, y=1, z=2) at `value`
    pub fn split(&self, axis: usize, value: f64) -> (Self, Self) {
        let mut ax_mid_max = self.ax_max;
        let mut ax_mid_min = self.ax_min;
        match axis {
            0 => {
                ax_mid_max.x = value;
                ax_mid_min.x = value;
            }
            1 => {
                ax_mid_max.y = value;
                ax_mid_min.y = value;
            }
            2 => {
                ax_mid_max.z = value;
                ax_mid_min.z = value;
            }
            _ => panic!(),
        }

        (
            Self::new(self.ax_min, ax_mid_max),
            Self::new(ax_mid_min, self.ax_max),
        )
    }
}
