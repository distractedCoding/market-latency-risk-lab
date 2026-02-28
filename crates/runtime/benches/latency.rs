use criterion::{criterion_group, criterion_main, Criterion};
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
            let _ = engine.step_once().await;
            let micros = started.elapsed().as_micros() as u64;
            metrics.record_latency_micros(micros.max(1));
        }
    });

    if let Some(report) = metrics.percentiles() {
        let budget_micros = 1_000_000 / TARGET_ORDERS_PER_SEC;
        println!(
            "latency_budget_micros={budget_micros} p50={} p95={} p99={} max={} samples={}",
            report.p50_micros, report.p95_micros, report.p99_micros, report.max_micros, report.count
        );
    }

    c.bench_function("runtime_latency_step_once", |b| {
        let mut engine = SimEngine::for_test_seed(13);
        b.iter(|| {
            runtime.block_on(async {
                let _ = engine.step_once().await;
            });
        });
    });
}

criterion_group!(benches, bench_runtime_latency);
criterion_main!(benches);
