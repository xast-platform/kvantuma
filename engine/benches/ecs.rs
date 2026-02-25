#[derive(Clone, Copy)]
struct Position {
    x: f32,
    y: f32,
}

#[derive(Clone, Copy)]
struct Velocity {
    x: f32,
    y: f32,
}

component! { POD: Velocity }
component! { POD: Position }

use criterion::{black_box, Criterion};
use hecs::World as HecsWorld;
use xastge::{component, ecs::world::World};
use xastge::ecs::world::ComponentWrite;

fn bench_hecs_spawn(c: &mut Criterion) {
    c.bench_function("hecs spawn", |b| {
        b.iter(|| {
            let mut world = HecsWorld::new();
            for _ in 0..100_000 {
                world.spawn((Position { x: 0.0, y: 0.0 }, Velocity { x: 1.0, y: 1.0 }));
            }
        });
    });
}

fn bench_hecs_query(c: &mut Criterion) {
    let mut world = HecsWorld::new();
    for _ in 0..100_000 {
        world.spawn((Position { x: 0.0, y: 0.0 }, Velocity { x: 1.0, y: 1.0 }));
    }
    c.bench_function("hecs query", |b| {
        b.iter(|| {
            for (_, (pos, vel)) in world.query_mut::<(&mut Position, &Velocity)>() {
                pos.x = black_box(pos.x + vel.x);
                pos.y = black_box(pos.y + vel.y);
            }
        });
    });
}

fn bench_kvantuma_spawn(c: &mut Criterion) {
    c.bench_function("xengine ecs spawn", |b| {
        b.iter(|| {
            let mut world = World::new();
            for _ in 0..100_000 {
                world.spawn((Position { x: 0.0, y: 0.0 }, Velocity { x: 1.0, y: 1.0 }));
            }
        });
    });
}

fn bench_kvantuma_query(c: &mut Criterion) {
    let mut world = World::new();
    for _ in 0..100_000 {
        world.spawn((Position { x: 0.0, y: 0.0 }, Velocity { x: 1.0, y: 1.0 }));
    }
    let mut batch = Vec::<ComponentWrite>::with_capacity(100_000);

    c.bench_function("xengine ecs query", |b| {
        b.iter(|| {
            world.for_each::<(&Position, &Velocity), _>(|e, (pos, vel)| {
                batch.push(black_box(ComponentWrite::new(e, Position {
                    x: pos.x + vel.x,
                    y: pos.y + vel.y,
                })));
            });

            for w in &batch {
                world.apply(w);
            }

            batch.clear();
        });
    });
}

criterion::criterion_group!(
    ecs,
    bench_hecs_spawn,
    bench_hecs_query,
    bench_kvantuma_spawn,
    bench_kvantuma_query
);
criterion::criterion_main!(ecs);