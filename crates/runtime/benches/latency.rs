use criterion::{black_box, criterion_group, criterion_main, Criterion};
use runtime::{engine::SimEngine, metrics::DecisionLatencyMetrics, TARGET_ORDERS_PER_SEC};
use std::time::Instant;
use tokio::runtime::Builder;

const LATENCY_SAMPLES: usize = 5_000;

fn bench_runtime_latency(c: &mut Criterion) {
    let runtime = Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("tokio runtime should build");

    let mut metrics = DecisionLatencyMetrics::new();
    runtime.block_on(async {
        let mut engine = SimEngine::for_test_seed(11);
        for _ in 0..LATENCY_SAMPLES {
            let started = Instant::now();
            let events = engine.step_once().await;
            let elapsed_nanos = started.elapsed().as_nanos() as u64;
            metrics.record_latency_nanos(elapsed_nanos);
            black_box(events);
        }
        black_box(&engine);
    });

    if let Some(report) = metrics.percentiles() {
        let budget_nanos = 1_000_000_000 / TARGET_ORDERS_PER_SEC;
        println!(
            "latency_budget_nanos={budget_nanos} p50_nanos={} p95_nanos={} p99_nanos={} max_nanos={} samples={}",
            report.p50_nanos, report.p95_nanos, report.p99_nanos, report.max_nanos, report.count
        );
    }

    c.bench_function("runtime_latency_step_once", |b| {
        let mut engine = SimEngine::for_test_seed(13);
        b.iter(|| {
            runtime.block_on(async {
                let events = engine.step_once().await;
                black_box(events);
            });
            black_box(&engine);
        });
    });
}

criterion_group!(benches, bench_runtime_latency);
criterion_main!(benches);
