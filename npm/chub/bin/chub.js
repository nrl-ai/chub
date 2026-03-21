#!/usr/bin/env node

/**
 * Thin JS wrapper that detects the current platform and executes
 * the native chub-turbo binary from the appropriate optional dependency.
 *
 * Pattern used by SWC, Biome, Oxlint, etc.
 */

const { execFileSync } = require("child_process");
const { join } = require("path");

const PLATFORMS = {
  "linux-x64": "@nrl-ai/chub-linux-x64",
  "linux-arm64": "@nrl-ai/chub-linux-arm64",
  "darwin-x64": "@nrl-ai/chub-darwin-x64",
  "darwin-arm64": "@nrl-ai/chub-darwin-arm64",
  "win32-x64": "@nrl-ai/chub-win32-x64",
};

function getBinaryPath() {
  const platformKey = `${process.platform}-${process.arch}`;
  const pkg = PLATFORMS[platformKey];

  if (!pkg) {
    console.error(
      `Unsupported platform: ${platformKey}\n` +
        `chub-turbo supports: ${Object.keys(PLATFORMS).join(", ")}`,
    );
    process.exit(1);
  }

  try {
    const binDir = require.resolve(`${pkg}/package.json`);
    const ext = process.platform === "win32" ? ".exe" : "";
    return join(binDir, "..", `chub${ext}`);
  } catch {
    console.error(
      `Could not find the chub binary for ${platformKey}.\n` +
        `Expected package: ${pkg}\n` +
        `Try reinstalling: npm install @nrl-ai/chub`,
    );
    process.exit(1);
  }
}

const binary = getBinaryPath();

try {
  execFileSync(binary, process.argv.slice(2), {
    stdio: "inherit",
    env: process.env,
  });
} catch (err) {
  if (err.status !== undefined) {
    process.exit(err.status);
  }
  throw err;
}
