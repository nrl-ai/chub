# Installation

Chub is available as a single native binary. Install it with your preferred package manager or download it directly.

## npm (recommended)

The easiest way to install Chub. Works on all supported platforms.

```sh
npm install -g @nrl-ai/chub
```

The npm package (`@nrl-ai/chub`) is a thin wrapper that automatically downloads the correct platform-specific binary via `optionalDependencies`. No Node.js runtime is needed at execution time — it runs the native Rust binary directly.

**Supported platforms via npm:**

| Package | Platform |
|---|---|
| `@nrl-ai/chub-linux-x64` | Linux x86_64 |
| `@nrl-ai/chub-linux-arm64` | Linux ARM64 |
| `@nrl-ai/chub-darwin-x64` | macOS Intel |
| `@nrl-ai/chub-darwin-arm64` | macOS Apple Silicon |
| `@nrl-ai/chub-win32-x64` | Windows x86_64 |

::: tip
If you already have Node.js installed for your project, npm is the fastest path. The binary is ~1.2 MB.
:::

## pip (Python)

```sh
pip install chub
```

Pre-built wheels include the native binary for each platform:

- Linux x86_64, ARM64
- macOS x86_64, Apple Silicon (ARM64)
- Windows x86_64

You can also invoke Chub as a Python module:

```sh
python -m chub search "stripe"
```

::: info How it works
The Python package is a thin wrapper — no Python runtime overhead. When you run `chub` or `python -m chub`, it delegates to the compiled Rust binary bundled inside the wheel.
:::

## Cargo (build from source)

If you have the Rust toolchain installed:

```sh
cargo install chub
```

This compiles Chub from source and installs the binary to `~/.cargo/bin/`. Requires Rust 1.70+.

## Homebrew (macOS / Linux)

```sh
brew install nrl-ai/tap/chub
```

## Binary download

Download prebuilt binaries directly from [GitHub Releases](https://github.com/nrl-ai/chub/releases).

### Linux

```sh
# x86_64
curl -fsSL https://github.com/nrl-ai/chub/releases/latest/download/chub-linux-x64 -o chub
chmod +x chub
sudo mv chub /usr/local/bin/

# ARM64
curl -fsSL https://github.com/nrl-ai/chub/releases/latest/download/chub-linux-arm64 -o chub
chmod +x chub
sudo mv chub /usr/local/bin/
```

### macOS

```sh
# Intel
curl -fsSL https://github.com/nrl-ai/chub/releases/latest/download/chub-darwin-x64 -o chub
chmod +x chub
sudo mv chub /usr/local/bin/

# Apple Silicon
curl -fsSL https://github.com/nrl-ai/chub/releases/latest/download/chub-darwin-arm64 -o chub
chmod +x chub
sudo mv chub /usr/local/bin/
```

### Windows

Download `chub-win32-x64.exe` from [GitHub Releases](https://github.com/nrl-ai/chub/releases) and add it to your `PATH`.

Or with PowerShell:

```powershell
Invoke-WebRequest -Uri "https://github.com/nrl-ai/chub/releases/latest/download/chub-win32-x64.exe" -OutFile "$env:USERPROFILE\.cargo\bin\chub.exe"
```

## Verify installation

```sh
chub --version
```

You should see output like:

```
chub 0.1.1
```

## Uninstall

```sh
# npm
npm uninstall -g @nrl-ai/chub

# pip
pip uninstall chub

# Cargo
cargo uninstall chub

# Manual binary
rm /usr/local/bin/chub
```

## Next steps

- [Getting Started](/guide/getting-started) — your first commands
- [CLI Reference](/reference/cli) — all commands and flags
- [MCP Server](/reference/mcp-server) — connect to AI agents
