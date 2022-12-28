use std::hash::{Hash, Hasher};

#[inline]
fn sqr(v: f32) -> f32 {
    v * v
}

/// distance between two points
fn dist_sqr([x, y]: [f32; 2], [a, b]: [f32; 2]) -> f32 {
    sqr(x - a) + sqr(y - b)
}

/// A coordinate on a regular grid.
pub trait RegularCoord: Hash {
    const NEIGHBORS: usize;

    fn from_euclidean(x: f32, y: f32, param: f32) -> Self;

    fn one_ring(&self) -> [Self; Self::NEIGHBORS]
    where
        Self: Sized;

    /// A specialized function for performing clipping on neighbors if they do not need to be
    /// checked.
    fn one_ring_clipped(&self, x: f32, y: f32, param: f32) -> impl Iterator<Item = Self>
    where
        Self: Sized,
        [Self; Self::NEIGHBORS]:,
    {
        let _ = (x, y, param);
        self.one_ring().into_iter()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct HexAxial<T> {
    pub q: T,
    pub r: T,
}

impl Hash for HexAxial<i32> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_i32(self.q);
        state.write_i32(self.r);
    }
}

impl HexAxial<f32> {
    fn new(x: f32, y: f32, circumradius: f32) -> HexAxial<f32> {
        let root3: f32 = (3.0f32).sqrt();
        let q = (x * root3 / 3. - y / 3.) / circumradius;
        let r = (2. * y / 3.) / circumradius;
        HexAxial { q, r }
    }

    #[inline]
    pub fn s(&self) -> f32 {
        -self.q - self.r
    }
    pub fn to_euclidean(&self) -> Euclidean<f32> {
        let root3: f32 = (3.0f32).sqrt();
        let x = root3 * self.q + self.r * root3 / 2.0;
        let y = 1.5 * self.r;
        Euclidean { x, y }
    }

    pub fn round(&self) -> HexAxial<i32> {
        let q = self.q.round();
        let r = self.r.round();
        let og_s = self.s();
        let s = og_s.round();

        let q_diff = (q - self.q).abs();
        let r_diff = (r - self.r).abs();
        let s_diff = (s - og_s).abs();

        let q = q as i32;
        let r = r as i32;
        let s = s as i32;

        let [q, r] = if q_diff > r_diff && q_diff > s_diff {
            [-r - s, r]
        } else if r_diff > s_diff {
            [q, -q - s]
        } else {
            [q, r]
        };
        HexAxial { q, r }
    }
}

impl RegularCoord for HexAxial<i32> {
    const NEIGHBORS: usize = 6;
    fn one_ring(&self) -> [HexAxial<i32>; 6] {
        Self::neighbor_indices().map(move |[dq, dr]| HexAxial {
            q: self.q + dq,
            r: self.r + dr,
        })
    }

    fn from_euclidean(x: f32, y: f32, circumradius: f32) -> Self {
        HexAxial::<f32>::new(x, y, circumradius).round()
    }
}

impl HexAxial<i32> {
    pub fn s(&self) -> i32 {
        -self.q - self.r
    }
    fn neighbor_indices() -> [[i32; 2]; 6] {
        [[1, 0], [1, -1], [0, -1], [-1, 0], [-1, 1], [0, 1]]
    }
    pub fn offset(self, dq: i32, dr: i32) -> HexAxial<i32> {
        HexAxial {
            q: self.q + dq,
            r: self.r + dr,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Euclidean<T> {
    pub x: T,
    pub y: T,
}

impl Hash for Euclidean<i32> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_i32(self.x);
        state.write_i32(self.y);
    }
}

impl Euclidean<f32> {
    pub fn round(&self) -> Euclidean<i32> {
        let x = self.x.floor() as i32;
        let y = self.y.floor() as i32;
        Euclidean { x, y }
    }
}

impl Euclidean<i32> {
    fn neighbor_indices() -> [[i32; 2]; 8] {
        [
            [-1, -1],
            [-1, 0],
            [-1, 1],
            //
            [0, -1],
            [0, 1],
            //
            [1, -1],
            [1, 0],
            [1, 1],
        ]
    }
    pub fn offset(self, dx: i32, dy: i32) -> Euclidean<i32> {
        Euclidean {
            x: self.x + dx,
            y: self.y + dy,
        }
    }
}

impl RegularCoord for Euclidean<i32> {
    fn from_euclidean(x: f32, y: f32, side_len: f32) -> Self {
        Euclidean {
            x: (x / side_len).floor() as i32,
            y: (y / side_len).floor() as i32,
        }
    }
    const NEIGHBORS: usize = 8;
    fn one_ring(&self) -> [Euclidean<i32>; 8] {
        Self::neighbor_indices().map(move |[dx, dy]| Euclidean {
            x: self.x + dx,
            y: self.y + dy,
        })
    }

    fn one_ring_clipped(&self, x: f32, y: f32, side_len: f32) -> impl Iterator<Item = Self> {
        let sx = self.x as f32 * side_len;
        let sy = self.y as f32 * side_len;
        let tl = [sx, sy];
        let tr = [sx + side_len, sy];
        let bl = [sx, sy + side_len];
        let br = [sx + side_len, sy + side_len];
        let s2 = side_len * side_len;
        let sx = self.x;
        let sy = self.y;
        [
            (Some(bl), [-1, -1]),
            (None, [-1, 0]),
            (Some(tl), [-1, 1]),
            //
            (None, [0, -1]),
            (None, [0, 1]),
            //
            (Some(br), [1, -1]),
            (None, [1, 0]),
            (Some(tr), [1, 1]),
        ]
        .into_iter()
        .filter_map(move |(pt, [dx, dy])| match pt {
            None => Some(Euclidean {
                x: sx + dx,
                y: sy + dy,
            }),
            Some(near) => {
                if dist_sqr(near, [x, y]) < s2 {
                    Some(Euclidean {
                        x: sx + dx,
                        y: sy + dy,
                    })
                } else {
                    None
                }
            }
        })
        //
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TriCoord<T> {
    pub s: T,
    pub t: T,
    pub u: T,
}

impl Hash for TriCoord<i32> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let [x, y] = self.canon2d();
        state.write_i32(x);
        state.write_i32(y);
    }
}

impl TriCoord<i32> {
    pub fn height_to_side_len(h: f32) -> f32 {
        let root3: f32 = (3.0f32).sqrt();
        h * 2. / root3
    }
    // down is 1
    // up is 2
    pub fn points_up(&self) -> bool {
        self.s + self.t + self.u == 2
    }
    pub fn new(x: f32, y: f32, side_len: f32) -> Self {
        let root3: f32 = (3.0f32).sqrt();

        let yr3d3 = y * root3 / 3.;
        let s = ((x - yr3d3) / side_len).ceil() as i32;
        let t = ((y * root3 * 2. / 3.) / side_len).floor();
        let t = t as i32 + 1;
        let u = ((-x - yr3d3) / side_len).ceil() as i32;
        let sum = s + t + u;

        debug_assert!(
            sum == 1 || sum == 2,
            "Internal error, unexpected {sum} {s} {t} {u} {x} {y}"
        );

        Self { s, t, u }
    }
    pub fn canon2d(&self) -> [i32; 2] {
        let sum = self.s + self.t + self.u;
        debug_assert!(sum == 1 || sum == 2, "Internal error {}", sum);
        let x = 2 * self.s + if sum == 1 { 0 } else { 1 };
        [x, self.t]
    }
    fn neighbor_indices(up: bool) -> [[i32; 3]; 12] {
        if up {
            [
                [-1, 0, 0],
                [0, -1, 0],
                [0, 0, -1],
                //
                [-1, 1, 0],
                [0, -1, 1],
                [1, 0, -1],
                //
                [1, -1, 0],
                [0, 1, -1],
                [-1, 0, 1],
                //
                [1, -1, -1],
                [-1, 1, -1],
                [-1, -1, 1],
            ]
        } else {
            [
                [1, 0, 0],
                [0, 1, 0],
                [0, 0, 1],
                //
                [-1, 1, 0],
                [0, -1, 1],
                [1, 0, -1],
                //
                [1, -1, 0],
                [0, 1, -1],
                [-1, 0, 1],
                //
                [-1, 1, 1],
                [1, -1, 1],
                [1, 1, -1],
            ]
        }
    }
}

impl RegularCoord for TriCoord<i32> {
    const NEIGHBORS: usize = 12;
    fn from_euclidean(x: f32, y: f32, side_len: f32) -> Self {
        Self::new(x, y, side_len)
    }
    fn one_ring(&self) -> [Self; Self::NEIGHBORS] {
        Self::neighbor_indices(self.points_up()).map(|[ds, dt, du]| TriCoord {
            s: self.s + ds,
            t: self.t + dt,
            u: self.u + du,
        })
    }
}
