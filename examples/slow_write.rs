use {
    growlock::grow_lock,
    std::{sync::Arc, thread, time::Duration},
};

fn main() {
    // we initialize the lock
    let lock = Arc::new(grow_lock!(10, [1, 2, 3]));

    let lock_clone = Arc::clone(&lock);
    let handle = thread::spawn(move || {
        let mut guard = lock_clone.write().unwrap();

        // we simulate a very slow write from another thread
        guard.push(4);
        thread::sleep(Duration::from_millis(1000));
        guard.push(5);
    });

    // wait for the second thread to lock `lock`
    thread::sleep(Duration::from_millis(30));

    assert!(lock.len() >= 3);

    // we can still read
    assert_eq!(&lock[..3], &[1, 2, 3]);

    // fourth element could have been already pushed,
    // and if it is we can access it, even if `handle`
    // still controls the lock
    if let Some(&fourth) = lock.get(3) {
        assert_eq!(fourth, 4);
    }

    handle.join().unwrap();

    // here both 4 and 5 are certainly already been pushed
    assert_eq!(lock.len(), 5);
    assert_eq!(&lock[3..], &[4, 5]);
    println!("lock is: {lock:?}"); // -> [1, 2, 3, 4, 5]
}
