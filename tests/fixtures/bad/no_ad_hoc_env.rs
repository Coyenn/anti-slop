fn port() -> Result<String, std::env::VarError> {
    std::env::var("PORT")
}

fn home() {
    let _ = std::env::var_os("HOME");
    let _ = std::env::vars();
}
