use std::hash::{BuildHasherDefault, Hasher};
use std::mem::transmute;

#[derive(Default)]
pub struct SimpleHash {
    state: u64,
    count: usize,
}
const MS: [i64; 3] = [1597334677, 3812015801, 2489301273];

impl Hasher for SimpleHash {
    fn write(&mut self, _bytes: &[u8]) {
        unimplemented!();
    }
    fn write_i32(&mut self, v: i32) {
        self.state ^= unsafe { transmute::<i64, u64>((v as i64) * MS[self.count]) };
        self.count += 1;
    }
    fn finish(&self) -> u64 {
        self.state
    }
}

pub type SimpleHashBuilder = BuildHasherDefault<SimpleHash>;
