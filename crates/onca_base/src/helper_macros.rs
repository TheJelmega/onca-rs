//! Helper macros


/// Count the number of token trees
// mutliple version to limit recusion
#[macro_export]
macro_rules! count_tt {
    ($_a:tt $_b:tt $_c:tt $_d:tt $_e:tt
     $_f:tt $_g:tt $_h:tt $_i:tt $_j:tt
     $_k:tt $_l:tt $_m:tt $_n:tt $_o:tt
     $_p:tt $_q:tt $_r:tt $_s:tt $_t:tt
     $($rest:tt)*) => {
        20usize + onca_common::count_tt!($($rest)*)
    };
    ($_a:tt $_b:tt $_c:tt $_d:tt $_e:tt
     $_f:tt $_g:tt $_h:tt $_i:tt $_j:tt
     $($rest:tt)*) => {
        10usize + onca_common::count_tt!($($rest)*)
    };
    ($_a:tt $_b:tt $_c:tt $_d:tt $_e:tt
     $($rest:tt)*) => {
        5usize + onca_common::count_tt!($($rest)*)
    };
    ($_first:tt $($rest:tt)*) => {
        1usize + onca_common::count_tt!($($rest)*)
    };
    () => {
        0usize
    };
}

/// Count the number of comma separated expressions
// mutliple version to limit recusion
#[macro_export]
macro_rules! count_exprs {
    ($_a:expr, $_b:expr, $_c:expr, $_d:expr, $_e:expr,
     $_f:expr, $_g:expr, $_h:expr, $_i:expr, $_j:expr,
     $_k:expr, $_l:expr, $_m:expr, $_n:expr, $_o:expr,
     $_p:expr, $_q:expr, $_r:expr, $_s:expr, $_t:expr,
     $($rest:expr),* $(,)?) => {
        20usize + onca_common::count_exprs!($($rest),*)
    };
    ($_a:expr, $_b:expr, $_c:expr, $_d:expr, $_e:expr,
     $_f:expr, $_g:expr, $_h:expr, $_i:expr, $_j:expr,
     $($rest:expr),* $(,)?) => {
        10usize + onca_common::count_exprs!($($rest),*)
    };
    ($_a:expr, $_b:expr, $_c:expr, $_d:expr, $_e:expr,
     $($rest:expr),* $(,)?) => {
        5usize + onca_common::count_exprs!($($rest),*)
    };
    ($_first:expr, $($rest:expr),* $(,)?) => {
        1usize + onca_common::count_exprs!($($rest),*)
    };
    ($_first:expr $(,)?) => {
        1usize
    };
    () => {
        0usize
    };
}

/// Get the name of the surrounding function
#[macro_export]
macro_rules! func_name {
    () => {{
        fn f() {}
        fn type_name_of<T>(_: T) -> &'static str {
            core::any::type_name::<T>()
        }
        type_name_of(f).strip_suffix("::f").unwrap()
    }};
}