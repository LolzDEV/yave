use criterion::{criterion_group, criterion_main, Criterion};
use yave::client::voxel::VoxelVertex;

pub fn compress(c: &mut Criterion) {
    c.bench_function("Vertex compression", |b| {
        b.iter(|| {
            VoxelVertex::new(16, 13, 4, 1, 3);
        })
    });
}

pub fn decompress(c: &mut Criterion) {
    let compressed = VoxelVertex::new(16, 13, 4, 1, 3);
    c.bench_function("Vertex decompression", |b| {
        b.iter(|| {
            compressed.x();
            compressed.y();
            compressed.z();
        })
    });
}

criterion_group!(benches, compress, decompress);
criterion_main!(benches);
