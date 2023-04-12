use crate::coordinates::TriCoord;
use crate::{CoordinateKind, SpatialHash};

#[test]
fn adjacent_test() {
    let mut sh_cube = SpatialHash::new(CoordinateKind::Cube { side_len: 0.1 });
    let mut sh_tri = SpatialHash::new(CoordinateKind::Tri {
        side_len: TriCoord::height_to_side_len(0.1),
    });
    let mut sh_hex = SpatialHash::new(CoordinateKind::Hex { circumradius: 0.1 });

    let freq = 128;
    for i in 0..freq {
        let i = (i as f32) / (freq as f32);
        for j in 0..freq {
            let j = (j as f32) / (freq as f32);
            sh_cube.add(i, j, ());
            sh_tri.add(i, j, ());
            sh_hex.add(i, j, ());
        }
    }

    panic!(
        "{:?} {:?} {:?}",
        sh_cube.query(0.5, 0.5).len(),
        sh_tri.query(0.5, 0.5).len(),
        sh_hex.query(0.5, 0.5).len(),
    );
}
