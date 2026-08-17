fn handle(r: Result<i32, AppError>) -> Result<i32, AppError> {
    match r {
        Ok(v) => Ok(v),
        Err(e) => Err(e),
    }
}

fn named(r: Result<i32, AppError>) {
    match r {
        Ok(_) => {}
        Err(e) => {
            let _err = e;
        }
    }
}

struct AppError;
