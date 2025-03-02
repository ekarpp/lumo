use super::*;

// limits spawned threads to 2^{max_depth}
const PARALLEL_MAX_DEPTH: usize = 8;
const PARALLEL_THRESHOLD: usize = 16384;

const COST_TRAVERSE: Float = 15.0;
const COST_INTERSECT: Float = 20.0;
const EMPTY_BONUS: Float = 0.2;

/// A node in the kD-tree. Can be either a plane split or a leaf node.
pub enum KdNode {
    /// X-split, axis (x = 0, y = 1, z = 2), split point and child nodes
    Split(Axis, Float, Box<KdNode>, Box<KdNode>),
    /// Stores indices to the object vector in the kD-tree
    Leaf(Vec<usize>),
}

#[derive(PartialEq)]
enum KdSide {
    Left = -1, Both = 0, Right = 1,
}

impl KdNode {
    /// Computes the cost for split along `axis` at `point`.
    fn cost(
        boundary: &AaBoundingBox,
        axis: Axis,
        point: Float,
        num_left: usize,
        num_planar: usize,
        num_right: usize,
    ) -> (Float, KdSide) {
        if !boundary.cuts(axis, point) {
            (crate::INF, KdSide::Both)
        } else {
            let (left, right) = boundary.split(axis, point);

            let area_left = left.area() / boundary.area();
            let area_right = right.area() / boundary.area();

            let cut = |num_left: usize, num_right: usize| -> Float {
                let cost = COST_TRAVERSE + COST_INTERSECT *
                    (num_left as Float * area_left + num_right as Float * area_right);
                if num_left == 0 || num_right == 0 {
                    (1.0 - EMPTY_BONUS) * cost
                } else {
                    cost
                }
            };

            let cost_left = cut(num_left + num_planar, num_right);
            let cost_right = cut(num_left, num_planar + num_right);

            if cost_left < cost_right {
                (cost_left, KdSide::Left)
            } else {
                (cost_right, KdSide::Right)
            }
        }
    }

    /// Finds the best split according to SAH.
    fn find_best_split(
        events: &[KdEvent],
        boundary: &AaBoundingBox,
        primitives: usize,
    ) -> (Axis, Float, Float, usize, usize, KdSide) {
        let mut best_cost = crate::INF;
        let mut best_point = crate::INF;
        let mut best_axis = Axis::X;
        let mut best_side = KdSide::Both;
        let mut best_left = 0;
        let mut best_right = 0;

        for axis in [Axis::X, Axis::Y, Axis::Z] {
            let mut num_left = 0;
            let mut num_right = primitives;
            events.iter()
                .filter(|ev| ev.a == axis)
                .for_each(|ev| {
                    let end = matches!(ev.t, KdEventType::End);
                    let planar = matches!(ev.t, KdEventType::Planar);
                    let start = matches!(ev.t, KdEventType::Start);
                    let num_planar = if planar { 1 } else { 0 };

                    if end || planar { num_right -= 1; }

                    let (cost, cut_side) = Self::cost(
                        boundary, ev.a, ev.p,
                        num_left, num_planar, num_right,
                    );

                    if cost < best_cost {
                        best_cost = cost;
                        best_point = ev.p;
                        best_axis = axis;
                        best_side = cut_side;
                        best_left = num_left;
                        best_right = num_right;
                        if matches!(best_side, KdSide::Left) {
                            best_left += num_planar;
                        } else {
                            best_right += num_planar;
                        }
                    }

                    if start || planar { num_left += 1; }
                })
        }

        (best_axis, best_point, best_cost, best_left, best_right, best_side)
    }

    /// Partitions indices to left and right parts along `axis` at `point`
    fn partition(
        events: &[KdEvent],
        primitives: usize,
        axis: Axis,
        point: Float,
        side: KdSide,
    ) -> FxHashMap<usize, KdSide> {
        let mut partition = FxHashMap::default();
        partition.reserve(primitives);

        for ev in events {
            if ev.a != axis { continue; }

            match ev.t {
                KdEventType::End => {
                    if ev.p <= point { partition.insert(ev.idx, KdSide::Left); }
                }
                KdEventType::Start => {
                    if ev.p >= point { partition.insert(ev.idx, KdSide::Right);  }
                }
                KdEventType::Planar => {
                    if ev.p < point || (ev.p == point && side == KdSide::Left) {
                        partition.insert(ev.idx, KdSide::Left);
                    } else if ev.p > point || (ev.p == point && side == KdSide::Right) {
                        partition.insert(ev.idx, KdSide::Right);
                    }
                }
            }
        }

        partition
    }

    /// Constructs nodes of the kD-tree recursively with SAH in n log n.
    /// `events` should be sorted on first call. if `tx` is provided, node is sent there,
    /// otherwise returned inside an option.
    pub fn construct(
        events: Vec<KdEvent>,
        primitives: usize,
        boundary: &AaBoundingBox,
        depth: usize,
        tx: Option<mpsc::Sender<Box<Self>>>,
    ) -> Option<Box<Self>> {
        let (axis, point, cost, n_l, n_r, side) =
            Self::find_best_split(&events, boundary, primitives);

        let cost_leaf = COST_INTERSECT * primitives as Float;
        let node = if cost > cost_leaf {
            let mut indices = Vec::with_capacity(primitives);
            let mut haves = FxHashSet::default();
            haves.reserve(primitives);
            for e in events {
                if !haves.contains(&e.idx) {
                    indices.push(e.idx);
                    haves.insert(e.idx);
                }
            }
            Box::new(Self::Leaf(indices))
        } else {
            let partition = Self::partition(
                &events,
                primitives,
                axis,
                point,
                side,
            );
            let mut events_l = Vec::with_capacity(2 * n_l);
            let mut events_r = Vec::with_capacity(2 * n_r);

            // TODO: hashmap is slow here
            for ev in events {
                if let Some(s) = partition.get(&ev.idx) {
                    if matches!(s, KdSide::Left) {
                        events_l.push(ev);
                    } else if matches!(s, KdSide::Right) {
                        events_r.push(ev);
                    } else {
                        unreachable!();
                    }
                } else {
                    events_l.push(ev.clone());
                    events_r.push(ev);
                }
            }
            drop(partition);

            let (bound_l, bound_r) = boundary.split(axis, point);

            let do_sequential = n_l + n_r < PARALLEL_THRESHOLD
                || depth >= PARALLEL_MAX_DEPTH;

            let (left, right) = if do_sequential {
                (
                    Self::construct(events_l, n_l, &bound_l, depth + 1, None).unwrap(),
                    Self::construct(events_r, n_r, &bound_r, depth + 1, None).unwrap(),
                )
            } else {
                let (tx_l, rx_l) = mpsc::channel();
                let _tl = thread::spawn(move || {
                    Self::construct(events_l, n_l, &bound_l, depth + 1, Some(tx_l));
                });
                let (tx_r, rx_r) = mpsc::channel();
                let _tr = thread::spawn(move || {
                    Self::construct(events_r, n_r, &bound_r, depth + 1, Some(tx_r));
                });

                (
                    rx_l.recv().unwrap(),
                    rx_r.recv().unwrap(),
                )
            };

            Box::new(Self::Split(
                axis,
                point,
                left,
                right,
            ))
        };

        if let Some(tx) = tx {
            tx.send(node).unwrap();
            None
        } else {
            Some( node )
        }
    }
}
