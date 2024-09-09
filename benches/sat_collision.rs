use criterion::{criterion_group, criterion_main, Criterion};
use roast_2d::prelude::*;
use roast_2d::sat::*;

fn sat_collide(n: usize) {
    let rect1 = SatRect {
        pos: Vec2 { x: 0.0, y: 0.0 },
        half_size: Vec2 { x: 50.0, y: 30.0 },
        angle: 0.0,
    };

    let rect2 = SatRect {
        pos: Vec2 { x: 70.0, y: 50.0 },
        half_size: Vec2 { x: 30.0, y: 20.0 },
        angle: 0.5,
    };

    for i in 0..n {
        for _j in (i + 1)..n {
            let overlap = sat_collision_overlap(&rect1, &rect2);
            debug_assert!(overlap.is_some());
        }
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("sat collide 100", |b| b.iter(|| sat_collide(100)));
    c.bench_function("sat collide 500", |b| b.iter(|| sat_collide(500)));
    c.bench_function("sat collide 1000", |b| b.iter(|| sat_collide(1000)));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
