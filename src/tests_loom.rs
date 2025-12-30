//! Run these tests with
//! `RUSTFLAGS="--cfg loom" cargo test tests_loom --release`

use {
    crate::grow_lock,
    loom::{sync::Arc, thread},
};

/// The max loom's thread pool is 4. To keep the tests fast, we use
/// a maximum of 3.
const THREADS: usize = 3;

/// Tests that when a reader sees the length increment, the last element is
/// correctly initialized
#[test]
fn length_visibility() {
    loom::model(|| {
        let lock = Arc::new(grow_lock!(5));
        thread::spawn({
            let lock = Arc::clone(&lock);
            move || {
                let mut guard = lock.write().unwrap();
                guard.extend([0, 42, 67, 39, 11]);
            }
        });

        let len = lock.len();
        assert_eq!(&lock[..len], &[0, 42, 67, 39, 11][..len]);
    });
}

#[test]
fn write_contention() {
    loom::model(|| {
        let lock = Arc::new(grow_lock!(THREADS));
        let mut handles = Vec::with_capacity(THREADS);
        for i in 0..THREADS {
            handles.push(thread::spawn({
                let lock = Arc::clone(&lock);
                move || {
                    let mut guard = lock.write().unwrap();
                    guard.push(i);
                }
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }
        assert_eq!(lock.len(), THREADS);
    });
}

#[test]
fn length_consistency_panic() {
    loom::model(|| {
        let lock = Arc::new(grow_lock!(1));
        let _ = thread::spawn({
            let lock = Arc::clone(&lock);
            move || {
                // loom thread will panic if this panics
                // so we catch it );
                let _ = std::panic::catch_unwind(|| {
                    let mut guard = lock.write().unwrap();
                    guard.push(1);
                    // this will panic
                    guard.push(2);
                });
            }
        })
        .join();

        // even if `push` panics, the length needs to stay consistent
        assert_eq!(lock.len(), 1);
    });
}
