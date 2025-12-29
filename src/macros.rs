#[macro_export]
macro_rules! atomic_vec {
    () => {
        $crate::AtomicVec::empty()
    };
    ($capacity:expr) => {
        $crate::AtomicVec::with_capacity($capacity)
    };

    ($capacity:expr, [$($elem:expr),*$(,)?]) => {{
        let __v__ = $crate::AtomicVec::with_capacity($capacity);
        {
            let mut __guard__ = __v__.lock().unwrap();
            $(
                __guard__.push($elem);
            )*
        }
        __v__
    }};

    ($elem:expr ; $len:expr) => {{
        let __v__ = $crate::AtomicVec::with_capacity($len);
        {
            let mut __guard__ = __v__.lock().unwrap();
            for _ in 0 .. $len {
                __guard__.push(::std::clone::Clone::clone(&$elem));
            }
        }
        __v__
    }};
    ($capacity:expr, [$elem:expr ; $len:expr]) => {{
        let __v__ = $crate::AtomicVec::with_capacity($capacity);
        {
            let mut __guard__ = __v__.lock().unwrap();
            for _ in 0 .. $len {
                __guard__.push(::std::clone::Clone::clone(&$elem));
            }
        }
        __v__
    }};

    // this is last because everything can match this
    ($($elem:expr),+$(,)?) => {{
        $crate::AtomicVec::from(::std::vec![$($elem),*])
    }};
}
