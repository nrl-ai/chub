# Contributing to Chub

Thank you for your interest in contributing to Chub! This guide covers both code contributions and documentation/skill contributions.

## Development Setup

### Prerequisites

- Rust 1.70+ (install via [rustup](https://rustup.rs/))
- Git

### Getting Started

```bash
git clone https://github.com/nrl-ai/chub.git
cd chub
cargo build
```

### Running the CLI locally

```bash
cargo run -- --help
cargo run -- build content/ --validate-only
cargo run -- search "stripe"
```

### Running Tests

```bash
cargo test              # run all tests
cargo test -- --nocapture  # with stdout output
```

### Linting

```bash
cargo fmt --check       # check formatting
cargo clippy            # lint
```

## Code Contributions

### Pull Request Process

1. Fork the repo and create a branch from `main`
2. Make your changes
3. Add or update tests as needed
4. Ensure all tests pass: `cargo test`
5. Ensure formatting: `cargo fmt`
6. Ensure no clippy warnings: `cargo clippy`
7. Validate the build: `cargo run -- build content/ --validate-only`
8. Submit a pull request

### Code Style

- Follow standard Rust conventions (`cargo fmt`)
- Minimal dependencies — prefer std library where practical
- Dual-mode output: every command supports `--json` for machine-readable output
- Business logic in `chub-core`, CLI/MCP in `chub-cli`

### Project Structure

```
crates/
  chub-core/              # Library: types, search, build, cache, registry
  chub-cli/               # Binary: CLI commands + MCP server
content/                  # Public content registry source
docs/                     # Design docs and roadmap
tests/fixtures/           # Test fixtures
npm/                      # npm distribution packages
```

## Content Contributions

Contributing curated documentation or skills is one of the most impactful ways to help.

### Contributing a Doc

1. Create a directory under `content/<author>/docs/<name>/`
2. Add a `DOC.md` with YAML frontmatter:

```yaml
---
name: my-api
description: Short description of what this doc covers
metadata:
  languages: "python,javascript"
  versions: "1.0.0"
  source: community
  tags: "api,rest"
  updated-on: "2026-03-21"
---
# Content here...
```

3. Add reference files in a `references/` subdirectory if needed
4. Validate: `cargo run -- build content/ --validate-only`

### Contributing a Skill

1. Create a directory under `content/<author>/skills/<name>/`
2. Add a `SKILL.md` with YAML frontmatter:

```yaml
---
name: my-skill
description: What this skill teaches agents to do
metadata:
  source: community
  tags: "automation,testing"
  updated-on: "2026-03-21"
---
# Skill content here...
```

### Content Quality Guidelines

- Write for LLMs: clear structure, code examples, explicit parameter names
- Use progressive disclosure: entry point (DOC.md/SKILL.md) should be < 500 lines
- Put detailed references in companion files with relative links
- Keep content up to date with the latest API versions
- Include practical code examples, not just API signatures

## Reporting Issues

- **Bugs**: [Open an issue](https://github.com/nrl-ai/chub/issues/new?template=bug_report.md)
- **Features**: [Request a feature](https://github.com/nrl-ai/chub/issues/new?template=feature_request.md)
- **Security**: See [SECURITY.md](SECURITY.md)

## License

By contributing, you agree that your contributions will be licensed under the [MIT License](LICENSE).
