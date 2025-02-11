use criterion::{black_box, criterion_group, criterion_main, Criterion};
use one_forty_nine_solver::solve;

fn bench_solver(c: &mut Criterion) {
    let target = 47;
    let mut group = c.benchmark_group("solver");
    group.sample_size(10);
    group.bench_function(format!("solve {}", target), |b| {
        b.iter(|| black_box(solve(target)))
    });
    group.finish();
}

criterion_group!(benches, bench_solver);
criterion_main!(benches);
