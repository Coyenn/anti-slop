use std::env;
use std::io::Write;
use std::path::PathBuf;
use std::process::ExitCode;

use anti_slop::{color_enabled, lint_path, render_text, ColorMode, Config, RULES};
use clap::{Parser, ValueEnum};

#[derive(Parser, Debug)]
#[command(
    name = "cargo-anti-slop",
    bin_name = "cargo anti-slop",
    version,
    about = "Opinionated lints that reject low-evidence Rust patterns."
)]
struct Cli {
    /// Files or directories to lint (default: current directory).
    paths: Vec<PathBuf>,

    /// Disable a rule (repeatable).
    #[arg(long = "allow", value_name = "RULE")]
    allow: Vec<String>,

    /// Enable a rule. Default: every rule.
    #[arg(long = "deny", value_name = "RULE")]
    deny: Vec<String>,

    /// Output format.
    #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
    format: OutputFormat,

    /// Color diagnostics, like rustc.
    #[arg(long, value_enum, default_value_t = ColorArg::Auto)]
    color: ColorArg,

    /// Extra path glob or directory name to skip (default: target, .git).
    #[arg(long, value_name = "GLOB")]
    exclude: Vec<String>,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum OutputFormat {
    Text,
    Json,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum ColorArg {
    Auto,
    Always,
    Never,
}

impl From<ColorArg> for ColorMode {
    fn from(value: ColorArg) -> Self {
        match value {
            ColorArg::Auto => ColorMode::Auto,
            ColorArg::Always => ColorMode::Always,
            ColorArg::Never => ColorMode::Never,
        }
    }
}

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(err) => {
            eprintln!("error: {err}");
            ExitCode::from(2)
        }
    }
}

fn run() -> anyhow::Result<ExitCode> {
    let args = plugin_args(env::args().collect());
    let cli = Cli::parse_from(args);

    for name in cli.allow.iter().chain(cli.deny.iter()) {
        if !RULES.contains(&name.as_str()) {
            anyhow::bail!("unknown rule `{name}`; known rules: {}", RULES.join(", "));
        }
    }

    let config = Config {
        allow: cli.allow,
        deny: cli.deny,
        exclude: {
            let mut ex = cli.exclude;
            for extra in ["target", ".git"] {
                if !ex.iter().any(|e| e == extra) {
                    ex.push(extra.to_string());
                }
            }
            ex
        },
    };

    let paths = if cli.paths.is_empty() {
        vec![PathBuf::from(".")]
    } else {
        cli.paths
    };

    let mut findings = Vec::new();
    for path in &paths {
        findings.extend(lint_path(path, &config)?);
    }
    findings.sort_by(|a, b| {
        (&a.file, a.line, a.column, &a.rule).cmp(&(&b.file, b.line, b.column, &b.rule))
    });

    match cli.format {
        OutputFormat::Text => {
            let color = color_enabled(cli.color.into());
            let text = render_text(&findings, color);
            let mut out = std::io::stdout().lock();
            out.write_all(text.as_bytes())?;
        }
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&findings)?);
        }
    }

    if findings.is_empty() {
        Ok(ExitCode::SUCCESS)
    } else {
        Ok(ExitCode::from(1))
    }
}

/// Cargo plugin convention: `cargo anti-slop` invokes `cargo-anti-slop anti-slop ...`.
fn plugin_args(mut args: Vec<String>) -> Vec<String> {
    if args.get(1).map(String::as_str) == Some("anti-slop") {
        args.remove(1);
    }
    args
}
