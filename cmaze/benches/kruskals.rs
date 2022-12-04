use cmaze::maze::{Dims3D, MazeAlgorithm, RndKruskals};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

pub fn kruskals_no_rayon(c: &mut Criterion) {
    c.bench_function("kruskals_no_rayon", |b| {
        b.iter(|| {
            let (handle, _stop, _progress) =
                RndKruskals::generate(black_box(Dims3D(10, 10, 10)), black_box(false)).unwrap();

            let _ = handle.join().unwrap();
        })
    });
}


criterion_group!(benches, kruskals_no_rayon);
criterion_main!(benches);
