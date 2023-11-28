use criterion::{black_box, criterion_group, criterion_main, Criterion};
use geometry_tools::vectors::calculate_tangents_bitangents;

fn criterion_benchmark(c: &mut Criterion) {
    let positions = vec![glam::Vec3A::ZERO; 10000];
    let normals = vec![glam::Vec3A::ZERO; 10000];
    let uvs = vec![glam::Vec2::ZERO; 10000];
    let indices: Vec<_> = (0..10000).chain(0..10000).chain(0..10000).collect();

    c.bench_function("calculate_tangents_bitangents", |b| {
        b.iter(|| {
            calculate_tangents_bitangents(
                black_box(&positions),
                black_box(&normals),
                black_box(&uvs),
                black_box(&indices),
            )
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
