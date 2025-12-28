use {crate::AtomicVec, std::alloc::System};
use crate::cap::Cap;
// > NOTE: using wildcard allows `miri` to tell the exact line
// > that causes UB. This is because the [`AtomicVec`] is
// > instantly dropped.

// ------------------- test constructors -------------------

/// Tests constructors and [`AtomicVec::drop`] with different kind of types and
/// capacities.
#[test]
fn new_empty_drop1() {
    let _ = AtomicVec::<u32>::try_new(0);
    let _ = AtomicVec::<char>::new(1 << 20);
    let _ = AtomicVec::<(i64, *mut char)>::new(12);
    let _ = AtomicVec::<bool, _>::new_in(5, System);
    let _ = AtomicVec::<[i8; 12], _>::try_new_in(23, System);
}

/// Tests constructors and [`AtomicVec::drop`] with more complicated types
#[test]
fn new_empty_drop2() {
    use std::{collections::HashMap, rc::Rc, sync::Arc};

    let _ = AtomicVec::<String>::try_new(0);
    let _ = AtomicVec::<Vec<u16>>::new(3);
    let _ = AtomicVec::<HashMap<u32, &'static str>>::new(1 << 30);
    let _ = AtomicVec::<Arc<u64>>::new(46);
    let _ = AtomicVec::<Rc<i64>>::new(46);
}

/// Tests constructors and [`AtomicVec::drop`] with ZSTs
///
/// > NOTE: capacity is automatically set as 0 for ZSTs
#[test]
fn new_empty_drop3() {
    struct MyZST;
    let _ = AtomicVec::<()>::new(0);
    let _ = AtomicVec::<MyZST>::try_new(1 << 60);
    let _ = AtomicVec::<(), _>::try_new_in(isize::MAX as usize, System);
    let v = AtomicVec::<MyZST, _>::new_in(usize::MAX, System);
    assert_eq!(v.capacity(), usize::MAX);
    assert_eq!(v.buf.raw_cap(), Cap::ZERO);
}

// ------------------- test drop -------------------

#[test]
fn initialized_drop() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    static COUNTER: AtomicUsize = AtomicUsize::new(0);
    struct AddOnDrop;
    impl Drop for AddOnDrop {
        fn drop(&mut self) {
            COUNTER.fetch_add(1, Ordering::Relaxed);
        }
    }

    {
        let vec = AtomicVec::new(200);
        let mut guard = vec.lock().unwrap();
        for _ in 0..100 {
            guard.push(AddOnDrop);
        }
        // here `vec` is dropped
    }
    assert_eq!(COUNTER.load(Ordering::Relaxed), 100);
}

// ------------------- test write -------------------

#[test]
fn write_contention() {
    use std::{sync::Arc, thread};
    const THREADS: usize = 10;
    const CAP: usize = 1000;

    let vec = Arc::new(AtomicVec::new(CAP));
    let mut handles = Vec::with_capacity(THREADS);
    for t in 0..THREADS {
        let v = Arc::clone(&vec);
        handles.push(thread::spawn(move || {
            for i in 0..(CAP / THREADS) {
                let mut guard = v.lock().unwrap();
                guard.push(t * (CAP / THREADS) + i);
            }
        }));
    }
    for handle in handles {
        handle.join().unwrap();
    }

    assert_eq!(vec.len(), CAP);
}
