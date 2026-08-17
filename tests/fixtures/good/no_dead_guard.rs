fn take(result: Result<i32, &'static str>) -> i32 {
    match result {
        Ok(v) => v,
        Err(_) => 0,
    }
}

fn opt(value: Option<i32>) -> i32 {
    if let Some(v) = value {
        v
    } else {
        0
    }
}
