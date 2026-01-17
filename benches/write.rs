use {
    criterion::{Criterion, criterion_group, criterion_main},
    growlock::grow_lock,
    std::{hint::black_box, sync::Arc, thread},
};

fn concurrent_push(crit: &mut Criterion) {
    let mut group = crit.benchmark_group("concurrent_push");
    for threads in [1, 2, 4, 8, 16] {
        group.bench_with_input(
            format!("threads_{threads}"),
            &threads,
            |bencher, &n_threads| {
                bencher.iter(|| {
                    let lock = Arc::new(grow_lock!(n_threads * 100));
                    let mut handles = Vec::with_capacity(n_threads);
                    for _ in 0..threads {
                        handles.push(thread::spawn({
                            let lock = Arc::clone(&lock);
                            move || {
                                let mut guard = lock.write().unwrap();
                                for i in 0..100 {
                                    guard.push(black_box(i));
                                }
                            }
                        }));
                    }
                    for handle in handles {
                        handle.join().unwrap();
                    }
                });
            },
        );
    }
    group.finish();
}

criterion_group!(benches, concurrent_push);
criterion_main!(benches);
