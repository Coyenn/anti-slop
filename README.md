# anti-slop

Opinionated lints that reject low-evidence and low-signal Rust patterns.

Prefer a **named owner / domain type**, parse once at the **I/O boundary**, and keep **evidence**. This crate is a port of [dmmulroy/anti-slop](https://github.com/dmmulroy/anti-slop)'s evidence thesis to Rust, plus deterministic **language-slop** checks on comments and docs (AI-writing tells, not authorship claims).

This project is meant to be opinionated. Read the rules, change them, `--allow` the ones that do not match your team. The bundled defaults are a thesis, not a constitution.

## Install

```bash
cargo install anti-slop
```

The binary is named `cargo-anti-slop`, so Cargo's plugin convention works:

```bash
cargo anti-slop
cargo anti-slop src/
cargo anti-slop --allow no_discarded_error --format json
cargo anti-slop --lint-strings src/
```

Exit status is `1` if any finding is reported.

By default, language rules lint **comments and doc comments only** (`//`, `///`, `//!`, `/* */`). Pass `--lint-strings` (or set `Config.lint_strings = true` in the library) to also scan string literals. User-facing copy in strings is your choice; the default keeps strings off.

`--color always|auto|never` matches rustc. Piped output is plain unless you pass `--color always`. `NO_COLOR` disables color in `auto`.

## Output

Diagnostics use rustc / Clippy's human format: lint id, file location, source snippet, caret, help, and a deny-by-default note.

```text
error[anti_slop::no_erasure_cast]: This `as _` cast discards type evidence.
  --> src/lib.rs:12:7
   |
12 |     x as _
   |       ^^^^
   |
   = help: Name the target type so the conversion stays visible.
   = note: `#[deny(anti_slop::no_erasure_cast)]` on by default

error: aborting due to 1 previous error
```

## Rules

Every rule is deny-by-default. Disable one with `--allow RULE`.

- `no_erasure_cast` — rejects `as _` and pointer-cast chains that discard type evidence.
- `no_any_bag` — rejects `dyn Any` / `Box<dyn Any>` / `Arc<dyn Any>` / `Rc<dyn Any>` as a param, return, field, or alias.
- `no_value_soup` — rejects `serde_json::Value`, `toml::Value`, and `serde_yaml::Value` as a domain type.
- `no_untyped_map` — rejects `HashMap` / `BTreeMap` / `IndexMap` whose key is `String`/`str` and whose value is `Value` or `dyn Any`.
- `no_discarded_error` — rejects `.ok()`, `.unwrap_or_else(...)`, `.unwrap_or_default()`, `.map_or(...)`, `.map_or_else(...)`, `.get_or_insert(...)`, and `.get_or_insert_with(...)`. Also hits `Option`; that is intentional. Does not flag `ok_or` / `ok_or_else`.
- `no_automock` — rejects `#[automock]`, `mockall::mock!`, and `mock!`.
- `no_dead_guard` — rejects same-ident `is_ok()` / `is_some()` then `.unwrap()` / `.expect()` / `.unwrap_unchecked()` in that branch. Evidence already known. Inspired by [AndreyMashukov/rust-lint](https://github.com/AndreyMashukov/rust-lint).
- `no_json_roundtrip` — rejects `serde_json::from_value(serde_json::to_value(...))` and the same laundering via a local binding.
- `no_error_swallow` — rejects an `Err(_)` match arm that is empty, `()`, or only `let _ =`. Evidence-shaped form of [styrene-lab/lipstyk](https://github.com/styrene-lab/lipstyk) error swallowing.
- `no_empty_safety` — rejects a `SAFETY:` comment whose body is empty or only generic words.
- `no_json_index` — rejects `Value[i]` / `val[0]["k"]` indexing. serde_json `Index` returns `Null` when missing.
- `no_type_alias_bag` — rejects a `type` alias whose target is `Value`, `dyn Any`, or a string-keyed untyped map.
- `no_boxed_error` — rejects `Box<dyn Error>`, `anyhow::Error`, and `eyre::Report` as a domain type.
- `no_stringly_error` — rejects `Result<T, String>` / `str` / `Cow<str>` as a domain type.
- `no_typeid_branch` — rejects `TypeId::of`, `type_name`, `type_name_of_val`, and `downcast` / `downcast_ref` / `downcast_mut`.
- `no_type_only_assert` — rejects `assert!(x.is_ok())` (and `is_err` / `is_some` / `is_none` / `is_empty`), including a leading `!`. Skips `#[test]`, `#[cfg(test)]`, and integration-test files.
- `no_widen_then_narrow` — rejects numeric `as` widen-then-narrow chains and `(x as T).try_into().unwrap()`.
- `no_ad_hoc_env` — rejects `std::env::var` / `var_os` / `vars` / `vars_os` outside a `config` or `settings` module.
- `no_over_1k` — rejects a `.rs` file over 1000 lines. From [thermo-nuclear-code-quality-review](https://github.com/cursor/plugins/blob/main/cursor-team-kit/skills/thermo-nuclear-code-quality-review/SKILL.md).

### Language slop (comments and docs by default)

Deterministic regex-style checks inspired by [stopslop](https://github.com/mgiovani/stopslop) and [ai-slop-detect](https://github.com/antydizajn/ai-slop-detect). No LLM, no prose-density scorer. Each finding includes a concrete `help:` rewrite hint.

- `no_emdash` — U+2014, mid-prose U+2013, or spaced `--` in comments. Exempt numeric ranges (`2020–2024`, `1.0–2.0`) and attribution-only lines (`— MIT`). Help: rewrite as two sentences, a comma, or parentheses.
- `no_smart_quotes` — curly quotes/apostrophes (`‘’“”`) in comments. Help: use ASCII `'` / `"`.
- `no_chat_preamble` — comment starts with assistant residue (`Certainly`, `Of course`, `Great question`, `I'd be happy to`, `Here's the updated`, `Let me know if`, …). Help: delete the preamble; keep any technical note.
- `no_elision_comment` — `rest of the code unchanged`, `... existing code ...`, `insert code here`. Help: delete; the edit should be complete.
- `no_fence_comment` — a comment that is only a markdown ` ``` ` fence. Help: delete.

## Violation examples

Each snippet below is rejected by the named rule.

### `no_erasure_cast`

```rust
let n = x as _;
let q = p as *const u8 as *const Header;
```

### `no_any_bag`

```rust
fn take(value: Box<dyn std::any::Any>) {}
```

### `no_value_soup`

```rust
fn take(value: serde_json::Value) {}
```

### `no_untyped_map`

```rust
fn take(map: HashMap<String, serde_json::Value>) {}
```

### `no_discarded_error`

```rust
let n = result.ok();
let n = result.unwrap_or_default();
let n = result.map_or(0, |v| v);
```

### `no_automock`

```rust
#[automock]
trait Store {
    fn get(&self) -> u32;
}
```

### `no_dead_guard`

```rust
if result.is_ok() {
    result.unwrap()
}
```

### `no_json_roundtrip`

```rust
let y = serde_json::from_value(serde_json::to_value(x));
```

### `no_error_swallow`

```rust
match result {
    Ok(v) => v,
    Err(_) => {}
}
```

### `no_empty_safety`

```rust
// SAFETY: trust me
let n = x as _;
```

### `no_json_index`

```rust
let _ = serde_json::Value[0]["k"];
```

### `no_type_alias_bag`

```rust
type Soup = serde_json::Value;
```

### `no_boxed_error`

```rust
fn load() -> Result<u32, Box<dyn std::error::Error>> {}
```

### `no_stringly_error`

```rust
fn load() -> Result<u32, String> {}
```

### `no_typeid_branch`

```rust
let id = std::any::TypeId::of::<u32>();
let n = x.downcast_ref::<u32>();
```

### `no_type_only_assert`

```rust
assert!(x.is_ok());
assert!(!y.is_some());
```

### `no_widen_then_narrow`

```rust
let n = x as u32 as u8;
let n: u32 = (x as u64).try_into().unwrap();
```

### `no_ad_hoc_env`

```rust
let port = std::env::var("PORT");
```

### `no_over_1k`

A `.rs` file whose line count is greater than 1000.

### `no_emdash`

```rust
// parse once — keep evidence
```

### `no_smart_quotes`

```rust
// say “hello”
```

### `no_chat_preamble`

```rust
// Certainly! Here's the updated handler.
```

### `no_elision_comment`

```rust
// ... rest of the code unchanged ...
```

### `no_fence_comment`

```rust
// ```rust
```

## SAFETY escape

The only comment escape is `SAFETY:` (case-sensitive, word-boundary, optional space, colon) on the line immediately before the offending construct, stating an invariant. Empty or generic text (`safe`, `trust me`, `it's fine`, `this is ok`, `okay`, `lints`) does not count.

```rust
// SAFETY: parse_user_id validated the identifier before branding it.
let id = raw as UserId;
```

A block comment immediately before the span works the same way.

## LANGUAGE escape

Put `LANGUAGE:` (case-sensitive, word-boundary, optional space, colon) in a comment to keep intentional prose there. Same spirit as `SAFETY:`, but for language-slop lints on that comment (or string literal when `--lint-strings` is on).

```rust
// LANGUAGE: keep — stylistic dash in this attribution note.
```

Disable a rule project-wide with `--allow no_emdash` (or any other rule name).

## Companion Clippy

Turn on Clippy's `undocumented_unsafe_blocks` and nothing else from this thesis. anti-slop is about evidence in safe Rust; Clippy already covers `unsafe` blocks.

## What this is not

- Not a full [stopslop](https://github.com/mgiovani/stopslop) port. No markdown-density suite, no document-wide “delve / tapestry / leverage” vocabulary gate, no narrative-comment-restates-code heuristic (too fuzzy at this precision bar).
- Not [lipstyk](https://github.com/styrene-lab/lipstyk) style-slop. No clone-overuse, no unwrap-used-as-panic, no Shape-in-names.
- Not an LLM classifier or authorship detector. Language rules are high-confidence formatting and phrasing artifacts only.
- Not dylint, rustc_private, or clippy_utils. Stable Rust only, so it can publish to crates.io.

The rust-lint dead-guard / silent-fallback idea and the lipstyk error-swallowing idea are used only in their evidence-shaped form: you already knew the variant, or you threw the `Err` away.

## Library

```rust
use anti_slop::{lint_path, Config};

let findings = lint_path(std::path::Path::new("src"), &Config::default())?;
```

Each finding is `{ rule, message, help, file, line, column }`.

## License

MIT
