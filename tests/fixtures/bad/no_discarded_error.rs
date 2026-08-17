fn drop_err(r: Result<i32, &'static str>) -> Option<i32> {
    r.ok()
}

fn fallback(r: Result<i32, &'static str>) -> i32 {
    r.unwrap_or_else(|_| 0)
}

fn defaulted(r: Result<i32, &'static str>) -> i32 {
    r.unwrap_or_default()
}
