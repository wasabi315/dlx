#[macro_export(local_inner_macros)]
macro_rules! hashset {
    (@single $($x:tt)*) => (());
    (@count $($rest:expr),*) => (<[()]>::len(&[$(hashset!(@single $rest)),*]));

    ($($key:expr,)+) => { hashset!($($key),+) };
    ($($key:expr),*) => {
        {
            let _cap = hashset!(@count $($key),*);
            let mut _set = ::std::collections::HashSet::with_capacity_and_hasher(_cap, ::std::default::Default::default());
            $(
                let _ = _set.insert($key);
            )*
            _set
        }
    };
}

#[macro_export]
macro_rules! ecp {
    ($($label:expr => $subset:tt,)+) => { ecp!($($label => $subset),+) };
    ($($label:expr => $subset:tt),*) => {
        ::std::vec![$(
            ($label, hashset! $subset)
        ),*]
    };
}
