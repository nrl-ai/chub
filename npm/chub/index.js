/**
 * Chub JavaScript SDK
 *
 * Programmatic access to chub search, get, annotate, pins, stats, detect,
 * and scan_secrets — all backed by the native chub binary.
 *
 * @example
 * const chub = require('@nrl-ai/chub');
 *
 * const results = await chub.search('stripe payments');
 * const doc     = await chub.get('stripe/package', { lang: 'javascript' });
 * await chub.annotate('stripe/package', 'Always use idempotency keys');
 *
 * // Or use a Client instance:
 * const client = new chub.Client();
 * const stats  = await client.stats();
 */

"use strict";

const { spawn } = require("child_process");
const { join, dirname } = require("path");
const fs = require("fs");
const os = require("os");

// ---------------------------------------------------------------------------
// Binary resolution
// ---------------------------------------------------------------------------

const PLATFORMS = {
  "linux-x64": "@nrl-ai/chub-linux-x64",
  "linux-arm64": "@nrl-ai/chub-linux-arm64",
  "darwin-x64": "@nrl-ai/chub-darwin-x64",
  "darwin-arm64": "@nrl-ai/chub-darwin-arm64",
  "win32-x64": "@nrl-ai/chub-win32-x64",
};

function _resolveBinary() {
  const platformKey = `${process.platform}-${process.arch}`;
  const pkg = PLATFORMS[platformKey];
  if (pkg) {
    try {
      const pkgJson = require.resolve(`${pkg}/package.json`);
      const ext = process.platform === "win32" ? ".exe" : "";
      const candidate = join(dirname(pkgJson), `chub${ext}`);
      if (fs.existsSync(candidate)) return candidate;
    } catch {
      // fall through to PATH
    }
  }
  // Fallback: look for chub on PATH
  return process.platform === "win32" ? "chub.exe" : "chub";
}

// ---------------------------------------------------------------------------
// ChubError
// ---------------------------------------------------------------------------

class ChubError extends Error {
  constructor(message, exitCode) {
    super(message);
    this.name = "ChubError";
    this.exitCode = exitCode;
  }
}

// ---------------------------------------------------------------------------
// Core runner
// ---------------------------------------------------------------------------

/**
 * Spawn the chub binary with `--json` and the given args.
 * Resolves with parsed JSON on exit 0, rejects with ChubError otherwise.
 *
 * @param {string} binary
 * @param {string[]} args
 * @returns {Promise<any>}
 */
function _run(binary, args) {
  return new Promise((resolve, reject) => {
    const proc = spawn(binary, ["--json", ...args], {
      stdio: ["ignore", "pipe", "pipe"],
      env: process.env,
    });

    let stdout = "";
    let stderr = "";
    proc.stdout.on("data", (d) => (stdout += d));
    proc.stderr.on("data", (d) => (stderr += d));

    proc.on("close", (code) => {
      if (code !== 0) {
        const msg = stderr.trim() || `chub exited with code ${code}`;
        return reject(new ChubError(msg, code));
      }
      try {
        resolve(JSON.parse(stdout));
      } catch {
        reject(
          new ChubError(`chub returned invalid JSON: ${stdout.slice(0, 200)}`),
        );
      }
    });

    proc.on("error", (err) => {
      reject(new ChubError(`Failed to spawn chub binary: ${err.message}`));
    });
  });
}

// ---------------------------------------------------------------------------
// Client class
// ---------------------------------------------------------------------------

class Client {
  /**
   * @param {object}  [opts]
   * @param {string}  [opts.binary]  Path to the chub binary (auto-detected if omitted).
   */
  constructor(opts = {}) {
    this._binary = opts.binary || _resolveBinary();
  }

  _run(...args) {
    return _run(this._binary, args);
  }

  /**
   * Search docs by keyword.
   * @param {string} query
   * @returns {Promise<object[]>}
   */
  async search(query) {
    const data = await this._run("search", query);
    return data.results ?? [];
  }

  /**
   * List available docs and skills.
   * @param {string}  [query]
   * @param {object}  [opts]
   * @param {string}  [opts.tags]   Comma-separated tag filter.
   * @param {string}  [opts.lang]   Language filter, e.g. "javascript".
   * @param {string}  [opts.type]   "doc" or "skill".
   * @param {number}  [opts.limit]  Max results.
   * @returns {Promise<object[]>}
   */
  async list(query, opts = {}) {
    const args = ["list"];
    if (query) args.push(query);
    if (opts.tags) args.push("--tags", opts.tags);
    if (opts.lang) args.push("--lang", opts.lang);
    if (opts.type) args.push("--type", opts.type);
    if (opts.limit != null) args.push("--limit", String(opts.limit));
    const data = await this._run(...args);
    return data.results ?? [];
  }

  /**
   * Fetch a doc by ID. Returns the markdown content as a string.
   * @param {string}  id             Entry ID, e.g. "stripe/payments".
   * @param {object}  [opts]
   * @param {string}  [opts.lang]    Language variant.
   * @param {string}  [opts.version] Pinned version string.
   * @param {string}  [opts.file]    Specific file path within the entry.
   * @returns {Promise<string>}
   */
  async get(id, opts = {}) {
    const args = ["get", id];
    if (opts.lang) args.push("--lang", opts.lang);
    if (opts.version) args.push("--version", opts.version);
    if (opts.file) args.push("--file", opts.file);
    const data = await this._run(...args);
    return data.content ?? "";
  }

  /**
   * Attach a note to a doc entry.
   * @param {string}  id             Entry ID.
   * @param {string}  note           Annotation text.
   * @param {object}  [opts]
   * @param {string}  [opts.kind]    "note" | "issue" | "fix" | "practice".
   * @param {boolean} [opts.team]    Save as git-tracked team annotation.
   * @param {string}  [opts.author]  Author name for team annotations.
   * @returns {Promise<object>}
   */
  async annotate(id, note, opts = {}) {
    const args = ["annotate", id, note];
    if (opts.kind) args.push("--kind", opts.kind);
    if (opts.team) args.push("--team");
    if (opts.author) args.push("--author", opts.author);
    return this._run(...args);
  }

  /**
   * Return all currently pinned docs.
   * @returns {Promise<object[]>}
   */
  async pins() {
    const data = await this._run("pin", "list");
    return data.pins ?? [];
  }

  /**
   * Return usage analytics.
   * @param {object} [opts]
   * @param {number} [opts.days=30]
   * @returns {Promise<object>}
   */
  async stats(opts = {}) {
    return this._run("stats", "--days", String(opts.days ?? 30));
  }

  /**
   * Detect project dependencies and match to available docs.
   * @param {object}  [opts]
   * @param {boolean} [opts.pin]  Auto-pin all detected docs.
   * @returns {Promise<object>}
   */
  async detect(opts = {}) {
    const args = ["detect"];
    if (opts.pin) args.push("--pin");
    return this._run(...args);
  }

  /**
   * Scan a path for secrets. Returns ``{ findings: [...] }``.
   * Never rejects for found secrets — only for scan errors.
   *
   * @param {string}  path           Directory or git repo root.
   * @param {object}  [opts]
   * @param {boolean} [opts.staged]  Scan staged changes only (git repos).
   * @param {string}  [opts.config]  Path to .chub-scan.toml / .gitleaks.toml.
   * @param {string}  [opts.baseline] Path to a baseline report.
   * @param {number}  [opts.redact]  Redact percentage (0–100).
   * @returns {Promise<{ findings: object[] }>}
   */
  async scanSecrets(path, opts = {}) {
    const isGit = fs.existsSync(join(path, ".git"));
    const args = isGit
      ? ["scan", "secrets", "git", path]
      : ["scan", "secrets", "dir", path];

    if (opts.staged) args.push("--staged");
    if (opts.config) args.push("--config", opts.config);
    if (opts.baseline) args.push("--baseline-path", opts.baseline);
    if (opts.redact != null) args.push("--redact", String(opts.redact));
    // exit code 0 = never fail on "secrets found", only on real errors
    args.push("--exit-code", "0");

    const raw = await this._run(...args);
    return Array.isArray(raw) ? { findings: raw } : raw;
  }
}

// ---------------------------------------------------------------------------
// Module-level convenience API
// ---------------------------------------------------------------------------

let _defaultClient = null;

function _client() {
  if (!_defaultClient) _defaultClient = new Client();
  return _defaultClient;
}

module.exports = {
  ChubError,
  Client,

  search: (query) => _client().search(query),
  list: (query, opts) => _client().list(query, opts),
  get: (id, opts) => _client().get(id, opts),
  annotate: (id, note, opts) => _client().annotate(id, note, opts),
  pins: () => _client().pins(),
  stats: (opts) => _client().stats(opts),
  detect: (opts) => _client().detect(opts),
  scanSecrets: (path, opts) => _client().scanSecrets(path, opts),
};
