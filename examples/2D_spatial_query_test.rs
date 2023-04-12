#![feature(return_position_impl_trait_in_trait)]
#![allow(incomplete_features)]

use spatial_hash::SpatialHash;
use surveyor::{generate_test, Spatial2DQuery, Spatial2DQueryTest};

struct ImplSpatial2DQuery<T> {
    sh: SpatialHash<([f32; 2], T)>,
    r: f32,
}

impl<T> Spatial2DQuery<T> for ImplSpatial2DQuery<T> {
    fn new(r: f32) -> Self {
        Self {
            sh: SpatialHash::cube(r),
            r,
        }
    }
    fn insert(&mut self, [x, y]: [f32; 2], val: T) {
        self.sh.add(x, y, ([x, y], val));
    }
    fn query(&self, p @ [x, y]: [f32; 2]) -> impl Iterator<Item = [f32; 2]> + '_ {
        self.sh
            .query(x, y)
            .map(|v| v.0)
            .filter(move |&v| Self::dist(v, p) <= self.r)
    }
}

generate_test!(Spatial2DQueryTest, ImplSpatial2DQuery<()>);
