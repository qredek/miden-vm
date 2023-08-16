/// Instantiates a test with Miden standard library included.
#[macro_export]
macro_rules! build_test {
    ($($params:tt)+) => {{
        let mut test = test_utils::build_test_by_mode!(false, $($params)+);
        test.libraries = vec![miden_stdlib::StdLibrary::default().into()];
        test
    }}
}

mod collections;
mod crypto;
mod math;
mod mem;
mod sys;
