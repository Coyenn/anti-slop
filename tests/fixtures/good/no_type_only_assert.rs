fn present(x: Option<i32>) {
    assert_eq!(x, Some(1));
}

fn ok(x: Result<i32, AppError>) {
    assert_eq!(x, Ok(1));
}

struct AppError;
