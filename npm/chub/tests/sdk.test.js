/**
 * Tests for the chub JavaScript SDK.
 *
 * Runs against the dev binary at ../../target/debug/chub[.exe].
 * Set CHUB_TEST_BINARY to override.
 *
 * Usage:
 *   node --test tests/sdk.test.js
 */

"use strict";

const { test, describe, before } = require("node:test");
const assert = require("node:assert/strict");
const path = require("node:path");
const fs = require("node:fs");
const os = require("node:os");

// Point at local source, not installed package.
const chub = require("../index.js");
const { Client, ChubError } = chub;

const REPO_ROOT = path.resolve(__dirname, "..", "..", "..");
const EXE = process.platform === "win32" ? "chub.exe" : "chub";
const DEV_BINARY =
  process.env.CHUB_TEST_BINARY || path.join(REPO_ROOT, "target", "debug", EXE);

function client() {
  return new Client({ binary: DEV_BINARY });
}

// Point module-level helpers at dev binary too.
before(() => {
  chub._defaultClient = client();
});

// ---------------------------------------------------------------------------
// search
// ---------------------------------------------------------------------------

describe("search", () => {
  test("returns an array", async () => {
    const results = await client().search("stripe");
    assert.ok(Array.isArray(results));
    assert.ok(results.length > 0);
  });

  test("result shape has id, description, type", async () => {
    const results = await client().search("openai");
    for (const r of results) {
      assert.ok("id" in r);
      assert.ok("description" in r);
      assert.ok("type" in r);
    }
  });

  test("unknown query returns empty array", async () => {
    const results = await client().search("xyzzy_nonexistent_12345");
    assert.deepEqual(results, []);
  });
});

// ---------------------------------------------------------------------------
// list
// ---------------------------------------------------------------------------

describe("list", () => {
  test("returns an array", async () => {
    const results = await client().list();
    assert.ok(Array.isArray(results));
    assert.ok(results.length > 0);
  });

  test("limit is respected", async () => {
    const results = await client().list(undefined, { limit: 5 });
    assert.ok(results.length <= 5);
  });

  test("lang filter keeps only matching entries", async () => {
    const results = await client().list(undefined, {
      lang: "python",
      limit: 20,
    });
    for (const r of results) {
      assert.ok(
        (r.languages ?? []).includes("python"),
        `expected python in languages, got ${JSON.stringify(r.languages)}`,
      );
    }
  });

  test("type filter keeps only matching entries", async () => {
    const results = await client().list(undefined, { type: "doc", limit: 10 });
    for (const r of results) {
      assert.equal(r.type, "doc");
    }
  });

  test("query narrows results", async () => {
    const all = await client().list(undefined, { limit: 100 });
    const filtered = await client().list("stripe", { limit: 100 });
    assert.ok(filtered.length <= all.length);
    assert.ok(filtered.some((r) => r.id.includes("stripe")));
  });
});

// ---------------------------------------------------------------------------
// get
// ---------------------------------------------------------------------------

describe("get", () => {
  test("returns markdown string", async () => {
    const content = await client().get("stripe/package", {
      lang: "javascript",
    });
    assert.equal(typeof content, "string");
    assert.ok(content.length > 100);
    assert.ok(content.toLowerCase().includes("stripe"));
  });

  test("lang option is applied", async () => {
    const content = await client().get("stripe/package", {
      lang: "javascript",
    });
    assert.ok(content.toLowerCase().includes("stripe"));
  });

  test("rejects with ChubError for unknown id", async () => {
    await assert.rejects(
      () => client().get("nonexistent/entry_xyz"),
      (err) => {
        assert.ok(err instanceof ChubError);
        return true;
      },
    );
  });
});

// ---------------------------------------------------------------------------
// pins
// ---------------------------------------------------------------------------

describe("pins", () => {
  test("returns an array", async () => {
    const pins = await client().pins();
    assert.ok(Array.isArray(pins));
  });

  test("each pin has an id", async () => {
    const pins = await client().pins();
    for (const p of pins) {
      assert.ok("id" in p);
    }
  });
});

// ---------------------------------------------------------------------------
// stats
// ---------------------------------------------------------------------------

describe("stats", () => {
  test("returns an object", async () => {
    const data = await client().stats();
    assert.equal(typeof data, "object");
    assert.ok(!Array.isArray(data));
  });

  test("has expected keys", async () => {
    const data = await client().stats();
    assert.ok("period_days" in data);
    assert.ok("total_fetches" in data);
    assert.ok("total_searches" in data);
  });

  test("days option is applied", async () => {
    const data = await client().stats({ days: 7 });
    assert.equal(data.period_days, 7);
  });
});

// ---------------------------------------------------------------------------
// detect
// ---------------------------------------------------------------------------

describe("detect", () => {
  test("returns an object", async () => {
    const data = await client().detect();
    assert.equal(typeof data, "object");
  });

  test("has matches array", async () => {
    const data = await client().detect();
    assert.ok(Array.isArray(data.matches));
  });

  test("match shape has dependency, doc_id, confidence", async () => {
    const data = await client().detect();
    for (const m of data.matches) {
      assert.ok("dependency" in m);
      assert.ok("doc_id" in m);
      assert.ok("confidence" in m);
    }
  });
});

// ---------------------------------------------------------------------------
// scanSecrets
// ---------------------------------------------------------------------------

describe("scanSecrets", () => {
  test("clean dir returns empty findings", async () => {
    const tmp = fs.mkdtempSync(path.join(os.tmpdir(), "chub-test-"));
    try {
      fs.writeFileSync(path.join(tmp, "clean.js"), "const x = 1;\n");
      const data = await client().scanSecrets(tmp);
      assert.ok("findings" in data);
      assert.deepEqual(data.findings, []);
    } finally {
      fs.rmSync(tmp, { recursive: true, force: true });
    }
  });

  test("detects a fake AWS key", async () => {
    const tmp = fs.mkdtempSync(path.join(os.tmpdir(), "chub-test-"));
    try {
      fs.writeFileSync(
        path.join(tmp, "config.js"),
        'const AWS_SECRET = "AKIAIOSFODNN7EXAMPLE/wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY";\n',
      );
      const data = await client().scanSecrets(tmp);
      assert.ok("findings" in data);
      assert.ok(data.findings.length > 0);
    } finally {
      fs.rmSync(tmp, { recursive: true, force: true });
    }
  });

  test("finding shape has RuleID or Description", async () => {
    const tmp = fs.mkdtempSync(path.join(os.tmpdir(), "chub-test-"));
    try {
      fs.writeFileSync(
        path.join(tmp, "config.js"),
        'const AWS_SECRET = "AKIAIOSFODNN7EXAMPLE/wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY";\n',
      );
      const data = await client().scanSecrets(tmp);
      for (const f of data.findings) {
        assert.ok(
          "RuleID" in f || "rule_id" in f || "Description" in f,
          `unexpected finding shape: ${JSON.stringify(f)}`,
        );
      }
    } finally {
      fs.rmSync(tmp, { recursive: true, force: true });
    }
  });

  test("git repo scan returns findings dict", async () => {
    const data = await client().scanSecrets(REPO_ROOT);
    assert.ok("findings" in data);
    assert.ok(Array.isArray(data.findings));
  });
});

// ---------------------------------------------------------------------------
// annotate (personal only — avoids mutating shared team state)
// ---------------------------------------------------------------------------

describe("annotate", () => {
  test("returns an object", async () => {
    const data = await client().annotate(
      "stripe/package",
      "JS SDK test annotation",
    );
    assert.equal(typeof data, "object");
  });

  test("kind option is accepted", async () => {
    const data = await client().annotate("stripe/package", "test practice", {
      kind: "practice",
    });
    assert.equal(typeof data, "object");
  });
});

// ---------------------------------------------------------------------------
// Module-level convenience API
// ---------------------------------------------------------------------------

describe("module-level helpers", () => {
  test("search", async () => {
    const results = await chub.search("stripe");
    assert.ok(results.length > 0);
  });

  test("list", async () => {
    const results = await chub.list(undefined, { limit: 5 });
    assert.ok(results.length <= 5);
  });

  test("get", async () => {
    const content = await chub.get("stripe/package", { lang: "javascript" });
    assert.ok(content.length > 100);
  });

  test("pins", async () => {
    assert.ok(Array.isArray(await chub.pins()));
  });

  test("stats", async () => {
    const data = await chub.stats();
    assert.ok("total_fetches" in data);
  });

  test("detect", async () => {
    const data = await chub.detect();
    assert.ok("matches" in data);
  });

  test("scanSecrets", async () => {
    const tmp = fs.mkdtempSync(path.join(os.tmpdir(), "chub-test-"));
    try {
      fs.writeFileSync(path.join(tmp, "f.txt"), "nothing here\n");
      const data = await chub.scanSecrets(tmp);
      assert.ok("findings" in data);
    } finally {
      fs.rmSync(tmp, { recursive: true, force: true });
    }
  });
});

// ---------------------------------------------------------------------------
// Error handling
// ---------------------------------------------------------------------------

describe("errors", () => {
  test("ChubError on bad id", async () => {
    await assert.rejects(
      () => client().get("totally/invalid/id/xyz"),
      ChubError,
    );
  });

  test("ChubError has exitCode", async () => {
    try {
      await client().get("totally/invalid/id/xyz");
      assert.fail("should have thrown");
    } catch (err) {
      assert.ok(err instanceof ChubError);
      assert.equal(typeof err.exitCode, "number");
    }
  });
});
