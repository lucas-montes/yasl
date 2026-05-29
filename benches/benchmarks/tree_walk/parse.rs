use crate::benchmarks::{config, helper};

use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use tree_walk::{Parser, Scanner};

fn bench(c: &mut Criterion) {
    let mut benchmark = c.benchmark_group("Parser");
    config::set_default_benchmark_configs(&mut benchmark);

    // Simple arithmetic benchmark
    let simple_source = helper::generate_simple_arithmetic();
    let simple_tokens = Scanner::new(&simple_source).scan().tokens();
    benchmark.bench_with_input(
        BenchmarkId::new("simple_arithmetic", simple_tokens.len()),
        &simple_tokens,
        |b, tokens| {
            b.iter(|| {
                let parser = Parser::new(black_box(tokens.to_vec()));
                parser.results()
            });
        },
    );

    // Complex program benchmark
    let complex_source = helper::generate_complex_program();
    let complex_tokens = Scanner::new(&complex_source).scan().tokens();
    benchmark.bench_with_input(
        BenchmarkId::new("complex_program", complex_tokens.len()),
        &complex_tokens,
        |b, tokens| {
            b.iter(|| {
                let parser = Parser::new(black_box(tokens.to_vec()));
                parser.results()
            });
        },
    );

    // Expression statements benchmark
    let expr_source = helper::generate_expression_statements(1);
    let expr_tokens = Scanner::new(&expr_source).scan().tokens();
    benchmark.bench_with_input(
        BenchmarkId::new("expression_statements", expr_tokens.len()),
        &expr_tokens,
        |b, tokens| {
            b.iter(|| {
                let parser = Parser::new(black_box(tokens.to_vec()));
                parser.results()
            });
        },
    );

    // Nested expression depth benchmark
    let depths = [5, 10, 20, 50];
    for &depth in &depths {
        let nested_source = helper::generate_nested_expression(depth);
        let nested_tokens = Scanner::new(&nested_source).scan().tokens();
        benchmark.bench_with_input(
            BenchmarkId::new("nested_expression", depth),
            &nested_tokens,
            |b, tokens| {
                b.iter(|| {
                    let parser = Parser::new(black_box(tokens.to_vec()));
                    parser.results()
                });
            },
        );
    }

    // Repeated pattern scaling benchmark
    let sizes = [10, 50, 100, 500];
    for &size in &sizes {
        let repeated_source = helper::generate_repeated_pattern(size);
        let repeated_tokens = Scanner::new(&repeated_source).scan().tokens();
        benchmark.bench_with_input(
            BenchmarkId::new("repeated_pattern", size),
            &repeated_tokens,
            |b, tokens| {
                b.iter(|| {
                    let parser = Parser::new(black_box(tokens.to_vec()));
                    parser.results()
                });
            },
        );
    }

    // Full program benchmark
    let full_source = helper::generate_full_program(1);
    let full_tokens = Scanner::new(&full_source).scan().tokens();
    benchmark.bench_with_input(
        BenchmarkId::new("full_program", full_tokens.len()),
        &full_tokens,
        |b, tokens| {
            b.iter(|| {
                let parser = Parser::new(black_box(tokens.to_vec()));
                parser.results()
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
