use crate::benchmarks::{config, helper};

use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use yasl::scan::Scanner;


fn bench(c: &mut Criterion) {
    let mut benchmark = c.benchmark_group("Scanner");
    config::set_default_benchmark_configs(&mut benchmark);

    // Simple arithmetic benchmark
    let simple_source = helper::generate_simple_arithmetic();
    benchmark.bench_with_input(
        BenchmarkId::new("simple_arithmetic", simple_source.len()),
        &simple_source,
        |b, source| {
            b.iter(|| {
                let scanner = Scanner::new(black_box(source));
                scanner.scan().tokens()
            });
        },
    );

    // Complex program benchmark
    let complex_source = helper::generate_complex_program();
    benchmark.bench_with_input(
        BenchmarkId::new("complex_program", complex_source.len()),
        &complex_source,
        |b, source| {
            b.iter(|| {
                let scanner = Scanner::new(black_box(source));
                scanner.scan().tokens()
            });
        },
    );

    // Expression statements benchmark
    let expr_source = helper::generate_expression_statements(1);
    benchmark.bench_with_input(
        BenchmarkId::new("expression_statements", expr_source.len()),
        &expr_source,
        |b, source| {
            b.iter(|| {
                let scanner = Scanner::new(black_box(source));
                scanner.scan().tokens()
            });
        },
    );

    // Scaling benchmark with repeated patterns
    let sizes = [10, 50, 100, 500];
    for &size in &sizes {
        let repeated_source = helper::generate_repeated_pattern(size);
        benchmark.bench_with_input(
            BenchmarkId::new("repeated_pattern", size),
            &repeated_source,
            |b, source| {
                b.iter(|| {
                    let scanner = Scanner::new(black_box(source));
                    scanner.scan().tokens()
                });
            },
        );
    }

    // Full program benchmark
    let full_source = helper::generate_full_program(1);
    benchmark.bench_with_input(
        BenchmarkId::new("full_program", full_source.len()),
        &full_source,
        |b, source| {
            b.iter(|| {
                let scanner = Scanner::new(black_box(source));
                scanner.scan().tokens()
            });
        },
    );

    benchmark.finish()
}

#[cfg(not(target_os = "windows"))]
criterion_group! {
    name = benches;
    config = config::get_default_profiling_configs();
    targets = bench
}
#[cfg(target_os = "windows")]
criterion_group!(benches, bench);

criterion_main!(benches);
