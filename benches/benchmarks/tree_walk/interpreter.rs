use crate::benchmarks::{config, helper};

use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use {
    scan::Scanner,
    tree_walk::{Interpreter, Parser},
};

fn interpret(input: &str) {
    let mut inter = Interpreter::default();
    let scan = Scanner::new(input).scan();
    if let Some(scan_errors) = scan.errors() {
        eprintln!("error scanning {:?}", &scan_errors);
        return;
    };
    let parser = Parser::new(scan.tokens());
    if let Some(parse_errors) = parser.errors() {
        eprintln!("error parsing {:?}", &parse_errors);
        return;
    };
    let stmts = parser.results();
    for stmt in stmts {
        if let Err(_err) = inter.evaluate(stmt) {
            continue;
        };
    }
}

fn bench(c: &mut Criterion) {
    let mut benchmark = c.benchmark_group("Interpreter");
    config::set_default_benchmark_configs(&mut benchmark);

    // Expression statements benchmark
    let expr_source = helper::generate_expression_statements(1);
    benchmark.bench_with_input(
        BenchmarkId::new("expression_statements", expr_source.len()),
        &expr_source,
        |b, tokens| {
            b.iter(|| interpret(black_box(&tokens)));
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
