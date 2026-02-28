use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use runtime::{engine::SimEngine, TARGET_ORDERS_PER_SEC};
use tokio::runtime::Builder;

const BENCH_STEPS: u64 = 10_000;

fn bench_runtime_throughput(c: &mut Criterion) {
    let runtime = Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("tokio runtime should build");

    let mut group = c.benchmark_group("runtime_throughput");
    group.throughput(Throughput::Elements(BENCH_STEPS));

    group.bench_function(BenchmarkId::new("step_once", BENCH_STEPS), |b| {
        b.iter(|| {
            runtime.block_on(async {
                let mut engine = SimEngine::for_test_seed(7);
                for _ in 0..BENCH_STEPS {
                    let _ = engine.step_once().await;
                }
            });
        });
    });

    group.finish();

    println!("target_orders_per_sec={TARGET_ORDERS_PER_SEC}");
}

criterion_group!(benches, bench_runtime_throughput);
criterion_main!(benches);
