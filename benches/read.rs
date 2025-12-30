use std::sync::atomic::{AtomicBool, Ordering};
use {
    criterion::{Criterion, criterion_group, criterion_main},
    std::{hint::black_box, sync::Arc, thread},
    growlock::grow_lock,
};

fn read_latency(crit: &mut Criterion) {
    let mut group = crit.benchmark_group("read_latency");

    let lock = Arc::new(grow_lock!(1000));
    {
        let mut guard = lock.write().unwrap();
        guard.extend(0..100);
    }

    let running = Arc::new(AtomicBool::new(true));

    let w_handle = thread::spawn({
        let lock = Arc::clone(&lock);
        let running = Arc::clone(&running);
        move || {
            let mut guard = lock.write().unwrap();
            let mut i = 100;
            while running.load(Ordering::Relaxed) {
                let _ = guard.try_push(i);
                i += 1;
            }
        }
    });

    group.bench_function("10_readers_1_writer", |bencher| {
        bencher.iter(|| {
            let slice = black_box(&lock[..]);
            let first = black_box(lock.first());
            let last = black_box(lock.last());

            black_box((slice, first, last));
        });
    });

    running.store(false, Ordering::Relaxed);
    w_handle.join().unwrap();
    group.finish();
}

criterion_group!(benches, read_latency);
criterion_main!(benches);
