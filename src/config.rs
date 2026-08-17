//! Process settings parsed once from the environment.

pub(crate) struct ColorSettings {
    pub no_color: bool,
}

impl ColorSettings {
    pub(crate) fn from_env() -> Self {
        Self {
            no_color: std::env::var_os("NO_COLOR").is_some(),
        }
    }
}
