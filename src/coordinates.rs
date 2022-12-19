use std::hash::{Hash, Hasher};

/// A coordinate on a regular grid.
pub trait RegularCoord: Hash {
    const NEIGHBORS: usize;

    fn from_euclidean(x: f32, y: f32, param: f32) -> Self;

    fn one_ring(&self) -> [Self; Self::NEIGHBORS]
    where
        Self: Sized;
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HexAxial<T> {
    q: T,
    r: T,
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

#[derive(Debug, Copy, Clone)]
pub struct Euclidean<T> {
    x: T,
    y: T,
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
            [0, -1],
            [-1, 1],
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
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct TriCoord<T> {
    s: T,
    t: T,
    u: T,
}

impl Hash for TriCoord<i32> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_i32(self.s);
        state.write_i32(self.t);
        state.write_i32(self.u);
    }
}

impl TriCoord<i32> {
    pub fn height_to_side_len(h: f32) -> f32 {
        let root3: f32 = (3.0f32).sqrt();
        h * 2. / root3
    }
    pub fn points_up(&self) -> bool {
        self.s + self.t + self.u == 2
    }
    pub fn new(x: f32, y: f32, side_len: f32) -> Self {
        let root3: f32 = (3.0f32).sqrt();
        let y = y + 1e-5;

        let s = ((x - y * root3 / 3.) / side_len).ceil() as i32;
        let t = ((y * root3 * 2. / 3.) / side_len).floor();
        let t = t as i32 + 1;
        let u = ((-x - y * root3 / 3.) / side_len).ceil() as i32;
        let sum = s + t + u;

        if sum == 0 {
            return Self::new(x + 1e-8, y, side_len);
        }

        assert!(
            sum == 1 || sum == 2,
            "Internal error, unexpected {sum} {s} {t} {u} {x} {y}"
        );
        Self { s, t, u }
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
                [-1, 1, 1],
                [1, -1, 1],
                [1, 1, -1],
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
                [1, -1, -1],
                [-1, 1, -1],
                [-1, -1, 1],
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
