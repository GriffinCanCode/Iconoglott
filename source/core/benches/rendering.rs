//! Rendering and diffing benchmarks
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};

// Note: These benchmarks are placeholders.
// To run meaningful benchmarks, the lib needs to be built as rlib for bench access.

fn bench_fnv1a_hashing(c: &mut Criterion) {
    let data = b"<rect x=\"100\" y=\"200\" width=\"300\" height=\"400\" fill=\"#fff\"/>";
    
    c.bench_function("fnv1a_hash_svg", |b| {
        b.iter(|| {
            let mut hash: u64 = 0xcbf29ce484222325;
            for byte in data.iter() {
                hash ^= *byte as u64;
                hash = hash.wrapping_mul(0x100000001b3);
            }
            black_box(hash)
        })
    });
}

fn bench_element_indexing(c: &mut Criterion) {
    let mut group = c.benchmark_group("indexing");
    
    for count in [10, 100, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(count),
            count,
            |b, &count| {
                b.iter(|| {
                    // Simulate indexing N elements
                    let mut ids = Vec::with_capacity(count);
                    for i in 0..count {
                        let mut hash: u64 = 0xcbf29ce484222325;
                        hash ^= i as u64;
                        hash = hash.wrapping_mul(0x100000001b3);
                        ids.push(hash);
                    }
                    black_box(ids)
                })
            },
        );
    }
    group.finish();
}

criterion_group!(benches, bench_fnv1a_hashing, bench_element_indexing);
criterion_main!(benches);
