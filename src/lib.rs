#![feature(generic_const_exprs)]
#![allow(incomplete_features)]
#![feature(generic_arg_infer)]
#![feature(return_position_impl_trait_in_trait)]

pub mod coordinates;
pub mod hash;
pub mod lines;

#[cfg(test)]
mod tests;

use coordinates::{Euclidean, HexAxial, RegularCoord, TriCoord};
use std::collections::hash_map::RandomState;
use std::collections::BTreeMap;
use std::default::Default;
use std::hash::{BuildHasher, Hasher};
use std::iter;

type DefaultHashBuilder = RandomState;
//type DefaultHashBuilder = hash::SimpleHashBuilder;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CoordinateKind {
    Cube { side_len: f32 },
    Hex { circumradius: f32 },
    Tri { side_len: f32 },
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Tri<S, T, U> {
    A(S),
    B(T),
    C(U),
}

impl<I, S: Iterator<Item = I>, T: Iterator<Item = I>, U: Iterator<Item = I>> Iterator
    for Tri<S, T, U>
{
    type Item = I;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Tri::A(i) => i.next(),
            Tri::B(i) => i.next(),
            Tri::C(i) => i.next(),
        }
    }
}

/// A Hexagonal Spatial Hash.
/// Unlike most spatial hashes that use cubes, this uses hexagons.
#[derive(Debug, Clone)]
pub struct SpatialHash<T, const N: usize = 256, S = DefaultHashBuilder> {
    /// Where the items are actually stored
    data: [BTreeMap<[i32; 2], Vec<T>>; N],

    /// Hash State
    state: S,

    pub kind: CoordinateKind,
}

impl<T> Default for SpatialHash<T, 256, DefaultHashBuilder> {
    fn default() -> Self {
        Self::new(CoordinateKind::Tri { side_len: 1. })
    }
}

impl<T> SpatialHash<T, 256, DefaultHashBuilder> {
    /// Create an empty hex spatial hash
    pub fn new(kind: CoordinateKind) -> Self {
        SpatialHash {
            data: [(); _].map(|_| BTreeMap::new()),
            kind,
            state: Default::default(),
        }
    }
    pub fn cube(side_len: f32) -> Self {
        Self::new(CoordinateKind::Cube { side_len })
    }
    /// Primary use case.
    /// Height should be equivalent to query radius.
    pub fn tri_h(height: f32) -> Self {
        let side_len = TriCoord::height_to_side_len(height);
        Self::new(CoordinateKind::Tri { side_len })
    }
    pub fn hex(circumradius: f32) -> Self {
        Self::new(CoordinateKind::Hex { circumradius })
    }
}

impl<T, const N: usize, S> SpatialHash<T, N, S> {
    /// Create an empty hex spatial hash
    pub fn with_hasher(self, state: S) -> Self {
        SpatialHash { state, ..self }
    }

    /// Remove all items from this spatial hash.
    pub fn clear(&mut self) {
        for d in &mut self.data {
            d.clear()
        }
    }
}

impl<T, const N: usize, S: BuildHasher + Default> SpatialHash<T, N, S> {
    pub fn idx(&self, x: f32, y: f32) -> (usize, [i32; 2]) {
        match self.kind {
            CoordinateKind::Cube { side_len } => {
                let ec = Euclidean::from_euclidean(x, y, side_len);
                (self.coord_idx(ec), [ec.x, ec.y])
            }
            CoordinateKind::Tri { side_len } => {
                let ec = TriCoord::from_euclidean(x, y, side_len);
                (self.coord_idx(ec), ec.canon2d())
            }
            CoordinateKind::Hex { circumradius } => {
                let ec = HexAxial::from_euclidean(x, y, circumradius);
                (self.coord_idx(ec), [ec.q, ec.r])
            }
        }
    }
    #[inline]
    pub fn coord_idx(&self, ax: impl RegularCoord) -> usize {
        let mut h = self.state.build_hasher();
        ax.hash(&mut h);
        (h.finish() as usize) % N
    }
    /// Iterates over each bin in this spatial hash, returning the 2D coordinate in floating
    /// point, and all the stored values.
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = ([f32; 2], &[T])> {
        self.data.iter().flat_map(|bins| {
            bins.iter().filter_map(|(&[u, v], vals)| {
                if vals.is_empty() {
                    return None;
                }
                let coord = match self.kind {
                    CoordinateKind::Cube { side_len } => {
                        Euclidean { x: u, y: v }.to_euclidean(side_len)
                    }
                    CoordinateKind::Tri { side_len: _ } => {
                        todo!("TODO convert uv to TriCoord")
                    }
                    CoordinateKind::Hex { circumradius } => {
                        HexAxial { q: u, r: v }.to_euclidean(circumradius)
                    }
                };
                Some((coord, vals.as_slice()))
            })
        })
    }

    /// Adds an item to this spatial hash. Returns the item set that it was added to.
    /// This can be used to sort the items for later querying.
    /// Mainly exists so you can have a z buffer in it.
    pub fn add(&mut self, x: f32, y: f32, t: T) -> &mut [T] {
        let (idx, key) = self.idx(x, y);
        let v = self.data[idx].entry(key).or_insert_with(Vec::new);
        v.push(t);
        v
    }

    /// Returns if two coordinates fall into the same bin for this spatial hash
    pub fn same_bin(&self, x: f32, y: f32, a: f32, b: f32) -> bool {
        self.idx(x, y).1 == self.idx(a, b).1
    }
    pub fn add_one_ring(&mut self, x: f32, y: f32, t: T, cb: impl Fn(&mut [T]))
    where
        T: Copy,
    {
        match self.kind {
            CoordinateKind::Cube { side_len } => {
                let ax = Euclidean::from_euclidean(x, y, side_len);
                ax.one_ring()
                    .into_iter()
                    .chain(iter::once(ax))
                    .for_each(move |hax| {
                        let v = self.data[self.coord_idx(hax)]
                            .entry([hax.x, hax.y])
                            .or_insert_with(Vec::new);
                        v.push(t);
                        cb(v)
                    });
            }
            CoordinateKind::Tri { side_len } => {
                let ax = TriCoord::from_euclidean(x, y, side_len);
                ax.one_ring()
                    .into_iter()
                    .chain(iter::once(ax))
                    .for_each(move |hax| {
                        let v = self.data[self.coord_idx(hax)]
                            .entry(hax.canon2d())
                            .or_insert_with(Vec::new);
                        v.push(t);
                        cb(v)
                    });
            }
            CoordinateKind::Hex { circumradius } => {
                let ax = HexAxial::from_euclidean(x, y, circumradius);
                ax.one_ring()
                    .into_iter()
                    .chain(iter::once(ax))
                    .for_each(move |hax| {
                        let v = self.data[self.coord_idx(hax)]
                            .entry([hax.q, hax.r])
                            .or_insert_with(Vec::new);
                        v.push(t);
                        cb(v)
                    });
            }
        }
    }
    /// Adds an item to this spatial hash
    pub fn add_with_conflict_resolution(
        &mut self,
        x: f32,
        y: f32,
        t: T,
        resolve: impl Fn(T, T) -> T,
    ) {
        let (idx, key) = self.idx(x, y);
        use std::collections::btree_map::Entry;
        match self.data[idx].entry(key) {
            Entry::Vacant(v) => {
                v.insert(vec![t]);
            }
            Entry::Occupied(mut o) => {
                assert_eq!(o.get().len(), 1);
                let v = o.get_mut();
                let new = resolve(t, v.pop().unwrap());
                v.push(new);
            }
        }
    }

    /// adds a line to the spatial hash using the bresenham algorithm.
    pub fn add_line_bresenham(&mut self, l_start: [f32; 2], l_end: [f32; 2], t: T)
    where
        T: Copy,
    {
        let (_, l_start) = self.idx(l_start[0], l_start[1]);
        let (_, l_end) = self.idx(l_end[0], l_end[1]);
        for [x, y] in lines::bresenham(l_start, l_end) {
            let idx = self.coord_idx(Euclidean { x, y });
            self.data[idx]
                .entry([x, y])
                .or_insert_with(Vec::new)
                .push(t);
        }
    }

    pub fn query(&self, x: f32, y: f32) -> &[T] {
        let (idx, key) = self.idx(x, y);
        self.data[idx].get(&key).map(Vec::as_slice).unwrap_or(&[])
    }

    /// Query items in a close proximity to a given (x,y) coordinate.
    pub fn query_one_ring(&self, x: f32, y: f32) -> impl Iterator<Item = &[T]> + '_ {
        match self.kind {
            CoordinateKind::Cube { side_len } => {
                let ax = Euclidean::from_euclidean(x, y, side_len);
                let iter = ax
                    //.one_ring_clipped(x,y,side_len)
                    .one_ring()
                    .into_iter()
                    .chain(iter::once(ax))
                    .filter_map(|hax| {
                        self.data[self.coord_idx(hax)]
                            .get(&[hax.x, hax.y])
                            .map(Vec::as_slice)
                    });
                Tri::A(iter)
            }
            CoordinateKind::Tri { side_len } => {
                let ax = TriCoord::from_euclidean(x, y, side_len);
                let iter = ax
                    .one_ring()
                    .into_iter()
                    .chain(iter::once(ax))
                    .filter_map(|hax| {
                        self.data[self.coord_idx(hax)]
                            .get(&hax.canon2d())
                            .map(Vec::as_slice)
                    });
                Tri::B(iter)
            }
            CoordinateKind::Hex { circumradius } => {
                let ax = HexAxial::from_euclidean(x, y, circumradius);
                let iter = ax
                    .one_ring()
                    .into_iter()
                    .chain(iter::once(ax))
                    .filter_map(|hax| {
                        self.data[self.coord_idx(hax)]
                            .get(&[hax.q, hax.r])
                            .map(Vec::as_slice)
                    });
                Tri::C(iter)
            }
        }
    }
    /*
    pub fn query_radius(&self, x: f32, y: f32, rad: f32) -> impl Iterator<Item = &T> + '_ {
        assert!(rad > 0.);
        let num_c_rad = rad / self.hex_circumradius;
        let extra_neighbors = ((num_c_rad.ceil() - 1.0) / 3.0).ceil();
        // (0,1] is mapped to 1 neighbor
        // (1,?] is mapped to 2 neighbors ? = 2.6?
        // (?,4] is mapped to 3 neighbors
        // (4,?) is mapped to 4 neighbors
        // (?,7) is mapped to 5 neighbors
        // 10 would be 7
        let en = extra_neighbors as i32;
        let ax = euclidean_to_axial(x, y, self.hex_circumradius).round();

        (-en..=en).flat_map(move |dq| {
            ((-en).max(-dq - en)..=en.min(en - dq))
                .flat_map(move |dr| &self.data[self.hex_coord_idx(ax.offset(dq, dr))])
        })
    }
    */
}
/*
#[test]
fn hex_spatial_hash_test() {
    let mut hsh = SpatialHash::default();
    hsh.hex_circumradius = 0.1;

    let freq = 128;
    for i in 0..freq {
        let i = (i as f32) / (freq as f32);
        for j in 0..freq {
            let j = (j as f32) / (freq as f32);
            hsh.add(i, j, ());
        }
    }

    panic!("{:?}", hsh.query(0.5, 0.5).count());
}
*/
