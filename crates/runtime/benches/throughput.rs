use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use runtime::{
    benchmark::{calculate_orders_per_sec, meets_target_orders_per_sec},
    engine::SimEngine,
    TARGET_ORDERS_PER_SEC,
};
use std::time::Instant;
use tokio::runtime::Builder;

const BENCH_STEPS: u64 = 10_000;

fn bench_runtime_throughput(c: &mut Criterion) {
    let runtime = Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("tokio runtime should build");

    let mut group = c.benchmark_group("runtime_throughput");
    group.throughput(Throughput::Elements(BENCH_STEPS));

    let (elapsed_nanos, total_events) = runtime.block_on(async {
        let mut engine = SimEngine::for_test_seed(17);
        let started = Instant::now();
        let mut total_events = 0_u64;
        for _ in 0..BENCH_STEPS {
            let events = engine.step_once().await;
            total_events += events.len() as u64;
            black_box(events);
        }
        black_box(&engine);
        (started.elapsed().as_nanos(), total_events)
    });
    let achieved_orders_per_sec = calculate_orders_per_sec(BENCH_STEPS, elapsed_nanos);
    let meets_target = meets_target_orders_per_sec(achieved_orders_per_sec, TARGET_ORDERS_PER_SEC);

    println!(
        "target_orders_per_sec={TARGET_ORDERS_PER_SEC} achieved_orders_per_sec={achieved_orders_per_sec} throughput_target_met={meets_target} throughput_events_processed={total_events}"
    );

    group.bench_function(BenchmarkId::new("step_once", BENCH_STEPS), |b| {
        b.iter(|| {
            runtime.block_on(async {
                let mut engine = SimEngine::for_test_seed(7);
                let mut total_events = 0_u64;
                for _ in 0..BENCH_STEPS {
                    let events = engine.step_once().await;
                    total_events += events.len() as u64;
                    black_box(events);
                }
                black_box(total_events);
                black_box(&engine);
            });
        });
    });

    group.finish();
}

criterion_group!(benches, bench_runtime_throughput);
criterion_main!(benches);
