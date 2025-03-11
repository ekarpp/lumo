use super::*;

// limits spawned threads to 2^{max_depth}
const PARALLEL_MAX_DEPTH: usize = 8;
const PARALLEL_THRESHOLD: usize = 16384;

const COST_TRAVERSE: Float = 15.0;
const COST_INTERSECT: Float = 20.0;
const EMPTY_BONUS: Float = 0.2;

/// A node in the kD-tree. Can be either a plane split or a leaf node.
pub enum KdNodeBuilder {
    /// X-split, axis (x = 0, y = 1, z = 2), split point and child nodes
    Split(Axis, Float, Box<KdNodeBuilder>, Box<KdNodeBuilder>),
    /// Stores indices to the object vector in the kD-tree
    Leaf(Vec<usize>),
}

#[derive(PartialEq)]
enum KdSide {
    Left = -1, Both = 0, Right = 1,
}

pub struct KdNode {
    pub axis: Axis,
    pub point: Float,
    pub indices: Vec<usize>,
    pub right: usize,
    pub leaf: bool,
}

impl KdNode {

    pub fn split(axis: Axis, point: Float) -> Self {
        Self {
            axis,
            point,
            indices: vec!(),
            right: IDX_NAN,
            leaf: false,
        }
    }

    pub fn leaf(indices: Vec<usize>) -> Self {
        Self {
            axis: Axis::X,
            point: crate::INF,
            indices,
            right: IDX_NAN,
            leaf: true,
        }
    }
}

impl KdNodeBuilder {
    pub fn num_nodes(&self) -> usize {
        match self {
            Self::Leaf(_) => 1,
            Self::Split(_, _, left, right) => 1 + left.num_nodes() + right.num_nodes(),
        }
    }

    pub fn build(&self, nodes: &mut Vec<KdNode>, parent: usize) {
        match self {
            Self::Leaf(indices) => {
                let node = KdNode::leaf(indices.to_vec());
                nodes.push(node);
                if parent != IDX_NAN {
                    nodes[parent].right = nodes.len() - 1;
                }
            }
            Self::Split(axis, point, left, right) => {
                let node = KdNode::split(*axis, *point);

                nodes.push(node);
                let pos = nodes.len() - 1;
                if parent != IDX_NAN {
                    nodes[parent].right = nodes.len() - 1;
                }
                left.build(nodes, IDX_NAN);
                right.build(nodes, pos);
            }
        }
    }

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
    ) -> (Axis, Float, Float, KdSide) {
        let mut best_cost = crate::INF;
        let mut best_point = crate::INF;
        let mut best_axis = Axis::X;
        let mut best_side = KdSide::Both;

        let mut num_left = [0 ; 3];
        let mut num_planar = [0 ; 3];
        let mut num_right = [primitives ; 3];

        let mut i = 0;
        while i < events.len() {
            let (mut s, mut p, mut e) = (0, 0, 0);
            let event = &events[i];
            while i < events.len()
                && events[i].a == event.a
                && events[i].p == event.p
                && events[i].t == KdEventType::End
            {
                e += 1; i += 1
            }

            while i < events.len()
                && events[i].a == event.a
                && events[i].p == event.p
                && events[i].t == KdEventType::Planar
            {
                p += 1; i += 1
            }

            while i < events.len()
                && events[i].a == event.a
                && events[i].p == event.p
                && events[i].t == KdEventType::Start
            {
                s += 1; i += 1
            }


            let axis = event.a as usize;
            num_planar[axis] = p;
            num_right[axis] -= p;
            num_right[axis] -= e;

            let (cost, cut_side) = Self::cost(
                boundary,
                event.a,
                event.p,
                num_left[axis],
                num_planar[axis],
                num_right[axis],
            );

            if cost < best_cost {
                best_cost = cost;
                best_point = event.p;
                best_axis = event.a;
                best_side = cut_side;
            }

            num_left[axis] += s;
            num_left[axis] += p;
            num_planar[axis] = 0;
        }

        (best_axis, best_point, best_cost, best_side)
    }

    /// Partitions indices to left and right parts along `axis` at `point`
    fn partition(
        events: &[KdEvent],
        primitives: usize,
        axis: Axis,
        point: Float,
        _side: KdSide,
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
                    // TODO: handle the planar side properly, currently breaks bistro
                    if ev.p < point {//|| (ev.p == point && side == KdSide::Left) {
                        partition.insert(ev.idx, KdSide::Left);
                    } else if ev.p > point {//|| (ev.p == point && side == KdSide::Right) {
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
        let (axis, point, cost, side) =
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
            let mut events_l = Vec::with_capacity(3 * primitives);
            let mut events_r = Vec::with_capacity(3 * primitives);

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

            let filt = |e: &&KdEvent| {
                e.a == Axis::X
                    && (e.t == KdEventType::Planar || e.t == KdEventType::Start)
            };
            let n_l = events_l.iter()
                .filter(filt)
                .count();
            let n_r = events_r.iter()
                .filter(filt)
                .count();

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
