#![allow(unused_variables, dead_code)]
use super::*;

/// Axis aligned bounding box
pub struct AaBoundingBox {
    ax_min: DVec3,
    ax_max: DVec3,
}

pub trait Bounded: Object {
    fn bounding_box(&self) -> AaBoundingBox;
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

    pub fn merge(&self, other: &Self) -> Self {
        Self {
            ax_min: self.ax_min.min(other.ax_min),
            ax_max: self.ax_max.max(other.ax_max),
        }
    }
}
