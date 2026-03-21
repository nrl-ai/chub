---
name: Coding Conventions
description: "Rust coding standards and patterns used in this project"
tags: conventions, rust, patterns
---

# Coding Conventions

## Error Handling

- Use `crate::error::{Error, Result}` for all fallible operations in chub-core
- CLI commands catch errors and call `output::error()` + `std::process::exit(1)`
- Team modules return `Option` or empty collections when `.chub/` is absent (no panics)

## Serialization

- All public types use `#[derive(Serialize, Deserialize)]` from serde
- JSON field names are camelCase (enforced via `#[serde(rename_all = "camelCase")]`)
- YAML files use snake_case keys (pins.yaml, config.yaml, profiles)

## CLI Output

- Human output goes to stderr (via `eprintln!`)
- Machine output (--json) goes to stdout (via `println!`)
- Colors via `owo-colors` crate, never hard-coded ANSI codes
- Every command supports `--json` flag for scripting

## Testing

- Unit tests are inline `#[cfg(test)]` modules
- Integration tests in `crates/chub-cli/tests/`
- Run: `cargo test --all`

## File Organization

- One command per file in `crates/chub-cli/src/commands/`
- Team feature modules in `crates/chub-core/src/team/`
- Each module is self-contained with its own types and logic
