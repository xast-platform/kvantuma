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
use kvantuma::{component, ecs::world::World};

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
    c.bench_function("kvantuma ecs spawn", |b| {
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
    c.bench_function("kvantuma ecs query", |b| {
        b.iter(|| {
            world.for_each::<(&mut Position, &Velocity), _>(|(pos, vel)| {
                pos.x = black_box(pos.x + vel.x);
                pos.y = black_box(pos.y + vel.y);
            });
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