use super::*;

#[derive(PartialEq, PartialOrd, Clone, Copy)]
pub enum KdEventType {
    End = 0, Planar = 1, Start = 2,
}

#[derive(Clone)]
/// Represents a possible split even during kD tree construction
pub struct KdEvent {
    /// Point of the split
    pub p: Float,
    /// Axis of the split
    pub a: Axis,
    /// Split event type
    pub t: KdEventType,
    /// Index of the primitive
    pub idx: usize,
}

impl KdEvent {
    pub fn new(p: Float, a: Axis, t: KdEventType, idx: usize) -> Self {
        Self { p, a, t, idx }
    }

    pub fn cmp(&self, other: &KdEvent) -> Ordering {
        if self.p < other.p {
            Ordering::Less
        } else if self.p > other.p {
            Ordering::Greater
        } else /* if self.p == other.p */ {
            if self.a < other.a {
                Ordering::Less
            } else if self.a > other.a {
                Ordering::Greater
            } else /* if self.a == other.a */ {
                if self.t < other.t {
                    Ordering::Less
                } else if self.t > other.t {
                    Ordering::Greater
                } else /* if self.t == other.t */ {
                    Ordering::Equal
                }
            }
        }
    }
}
