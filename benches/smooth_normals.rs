use std::hint::black_box;

use criterion::{criterion_group, criterion_main, Criterion};
use geometry_tools::vectors::calculate_smooth_normals;

fn criterion_benchmark(c: &mut Criterion) {
    let positions = vec![glam::Vec3A::ZERO; 10000];
    let indices: Vec<_> = (0..10000).chain(0..10000).chain(0..10000).collect();

    c.bench_function("calculate_smooth_normals", |b| {
        b.iter(|| calculate_smooth_normals(black_box(&positions), black_box(&indices)))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
