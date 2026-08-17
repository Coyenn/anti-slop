fn take(result: Result<i32, &'static str>) -> i32 {
    if result.is_ok() {
        result.unwrap()
    } else {
        0
    }
}

fn opt(value: Option<i32>) -> i32 {
    if value.is_some() {
        value.expect("present")
    } else {
        0
    }
}
