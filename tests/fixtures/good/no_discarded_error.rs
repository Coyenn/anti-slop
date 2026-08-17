fn keep(r: Result<i32, AppError>) -> Result<i32, AppError> {
    match r {
        Ok(v) => Ok(v),
        Err(e) => Err(e),
    }
}

fn propagate(r: Result<i32, AppError>) -> Result<i32, AppError> {
    let v = r?;
    Ok(v)
}

fn from_option(o: Option<i32>) -> Result<i32, AppError> {
    o.ok_or(AppError)
}

fn from_option_else(o: Option<i32>) -> Result<i32, AppError> {
    o.ok_or_else(|| AppError)
}

struct AppError;
