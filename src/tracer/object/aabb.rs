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
            ax_min: DVec3::splat(INFINITY),
            ax_max: DVec3::splat(-INFINITY),
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

    pub fn intersect(&self, r: &Ray) -> (f64, f64) {
        let ro_min = (self.ax_min - r.origin).to_array();
        let ro_max = (self.ax_max - r.origin).to_array();
        let rd = r.dir.to_array();

        // UNROLL LOOPP??

        // axis parallel rays?
        let mut t_start = -INFINITY;
        let mut t_end = INFINITY;

        for ax in 0..=2 {
            if rd[ax].abs() < EPSILON {
                if ro_min[ax] > 0.0 || ro_max[ax] < 0.0 {
                    return (INFINITY, -INFINITY);
                } else {
                    continue;
                }
            }

            let (mut t1, mut t2) =
                (ro_min[ax] / rd[ax], ro_max[ax] / rd[ax]);

            if t1 > t2 {
                std::mem::swap(&mut t1, &mut t2);
            }

            t_start = t_start.max(t1);
            t_end = t_end.min(t2);
        }

        (t_start, t_end)
    }

    /// Combine self and other to a new bigger AABB
    pub fn merge(&self, other: &Self) -> Self {
        Self::new(
            self.ax_min.min(other.ax_min),
            self.ax_max.max(other.ax_max),
        )
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
