use cmaze::maze::{Dims3D, MazeAlgorithm, RndKruskals};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

const DIMS: Dims3D = Dims3D(30, 30, 10);

pub fn kruskals_no_rayon(c: &mut Criterion) {
    c.bench_function("kruskals_no_rayon", |b| {
        b.iter(|| {
            let (handle, _stop, _progress) =
                RndKruskals::generate(black_box(DIMS), black_box(false), false).unwrap();

            let _ = handle.join().unwrap();
        })
    });
}

pub fn kruskals_with_rayon(c: &mut Criterion) {
    c.bench_function("kruskals_with_rayon", |b| {
        b.iter(|| {
            let (handle, _stop, _progress) =
                RndKruskals::generate(black_box(DIMS), black_box(true), false).unwrap();

            let _ = handle.join().unwrap();
        })
    });
}

criterion_group! {name = benches; config = Criterion::default().sample_size(10); targets = kruskals_no_rayon, kruskals_with_rayon}
criterion_main!(benches);
