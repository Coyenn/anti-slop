fn tags(x: Result<i32, AppError>, y: Option<i32>, z: &[u8]) {
    assert!(x.is_ok());
    assert!(!x.is_err());
    assert!(y.is_some());
    assert!((y.is_none()));
    assert!(z.is_empty());
}

struct AppError;
