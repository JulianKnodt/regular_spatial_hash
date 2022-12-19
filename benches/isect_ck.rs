use criterion::{black_box, criterion_group, criterion_main, Criterion};
use spatial_hash::SpatialHash;

const FREQ: usize = 256;
const INPUTS: [f32; 10] = [1e-3, 3e-3, 5e-3, 8e-3, 3e-2, 5e-2, 0.0075, 0.01, 0.1, 0.2];

#[test]
fn adjacent_test() {
    let mut sh_cube = SpatialHash::new(CoordinateKind::Cube { side_len: 0.1 });
    let mut sh_tri = SpatialHash::new(CoordinateKind::Tri {
        side_len: TriCoord::height_to_side_len(0.1),
    });
    let mut sh_hex = SpatialHash::new(CoordinateKind::Hex { circumradius: 0.1 });

    panic!(
        "{:?} {:?} {:?}",
        sh_cube.query(0.5, 0.5).count(),
        sh_tri.query(0.5, 0.5).count(),
        sh_hex.query(0.5, 0.5).count(),
    );
}

fn cube_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("Bench Cube");
    for l in INPUTS {
        let mut sh = SpatialHash::cube(l);

        for i in 0..FREQ {
            let i = (i as f32) / (FREQ as f32);
            for j in 0..FREQ {
                let j = (j as f32) / (FREQ as f32);
                sh.add(i, j, ());
            }
        }

        let mut i = 0;
        group.bench_with_input(format!("{l:?}"), &l, |b, _| {
            b.iter(|| {
                i += 1;
                let dx = (i as f32 * 5.97).sin() / 4.;
                let dy = (i as f32 * 3.48).cos() / 4.;
                sh.query(0.5 + black_box(dx), 0.5 + black_box(dy)).count()
            })
        });
    }
    group.finish()
}

fn tri_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("Bench Tri");
    for l in INPUTS {
        let mut sh = SpatialHash::tri_h(l);

        for i in 0..FREQ {
            let i = (i as f32) / (FREQ as f32);
            for j in 0..FREQ {
                let j = (j as f32) / (FREQ as f32);
                sh.add(i, j, ());
            }
        }

        let mut i = 0;
        group.bench_function(format!("{l:?}"), |b| {
            b.iter(|| {
                i += 1;
                let dx = (i as f32 * 5.97).sin() / 4.;
                let dy = (i as f32 * 3.48).cos() / 4.;
                sh.query(0.5 + black_box(dx), 0.5 + black_box(dy)).count()
            })
        });
    }
    group.finish()
}

fn hex_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("Bench Hex");
    for l in INPUTS {
        let mut sh = SpatialHash::hex(l);

        for i in 0..FREQ {
            let i = (i as f32) / (FREQ as f32);
            for j in 0..FREQ {
                let j = (j as f32) / (FREQ as f32);
                sh.add(i, j, ());
            }
        }

        let mut i = 0;
        group.bench_function(format!("{l:?}"), |b| {
            b.iter(|| {
                i += 1;
                let dx = (i as f32 * 5.97).sin() / 4.;
                let dy = (i as f32 * 3.48).cos() / 4.;
                sh.query(0.5 + black_box(dx), 0.5 + black_box(dy)).count()
            })
        });
    }
    group.finish()
}

criterion_group!(benches, cube_benchmark, tri_benchmark, hex_benchmark);

criterion_main!(benches);
