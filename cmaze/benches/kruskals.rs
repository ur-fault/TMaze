use criterion::{black_box, criterion_group, criterion_main, Criterion};
use cmaze::maze::{RndKruskals, MazeAlgorithm, Dims3D};

pub fn kruskals_no_rayon(c: &mut Criterion) {
    c.bench_function("kruskals_no_rayon", |b| {
        b.iter(|| {
            let (handle, _stop, _progress) = RndKruskals::generate(
                black_box(Dims3D(10, 10, 10)),
                black_box(false),
                black_box(false),
            ).unwrap();

            let _ = handle.join().unwrap();
        })
    });
}

pub fn kruskals_rayon(c: &mut Criterion) {
    c.bench_function("kruskals_rayon", |b| {
        b.iter(|| {
            let (handle, _stop, _progress) = RndKruskals::generate(
                black_box(Dims3D(10, 10, 10)),
                black_box(false),
                black_box(true),
            ).unwrap();

            let _ = handle.join().unwrap();
        })
    });
}

criterion_group!(benches, kruskals_no_rayon, kruskals_rayon);
criterion_main!(benches);