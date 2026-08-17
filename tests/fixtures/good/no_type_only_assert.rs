fn present(x: Option<i32>) {
    assert_eq!(x, Some(1));
}

fn ok(x: Result<i32, AppError>) {
    assert_eq!(x, Ok(1));
}

#[test]
fn in_a_test(x: Result<i32, AppError>, y: Option<i32>) {
    assert!(x.is_ok());
    assert!(y.is_some());
}

#[cfg(test)]
mod suite {
    fn helper(x: Option<i32>) {
        assert!(x.is_some());
    }
}

struct AppError;
