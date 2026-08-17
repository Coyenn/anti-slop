fn drop_err(r: Result<i32, AppError>) -> Option<i32> {
    r.ok()
}

fn fallback(r: Result<i32, AppError>) -> i32 {
    r.unwrap_or_else(|_| 0)
}

fn defaulted(r: Result<i32, AppError>) -> i32 {
    r.unwrap_or_default()
}

fn mapped(r: Result<i32, AppError>) -> i32 {
    r.map_or(0, |v| v)
}

fn insert(slot: &mut Option<i32>) -> &mut i32 {
    slot.get_or_insert(0)
}

struct AppError;
