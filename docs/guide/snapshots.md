# Snapshots & Freshness

This page covers three related features: doc snapshots, freshness checks, and usage analytics.

## Doc Snapshots

Capture point-in-time snapshots of all pins for reproducible builds and regression audits.

```sh
chub snapshot create v2.1.0        # Save current pins
chub snapshot list                 # List all snapshots
chub snapshot restore v2.1.0       # Restore exact pin versions
chub snapshot diff v2.0.0 v2.1.0   # What changed between releases
```

### What's stored

A snapshot captures the full state of `.chub/pins.yaml` at a point in time — every pin with its ID, version, language, reason, and source. Snapshots are saved as YAML files in `.chub/snapshots/` with an ISO 8601 timestamp.

### When to create snapshots

- **Before releases** — tag the doc context that shipped with each version
- **Before major upgrades** — capture the working state before changing pins
- **After `chub check --fix`** — record the new state for comparison later

### Diff output

`chub snapshot diff` compares two snapshots by pin ID and reports:

```
Comparing v2.0.0 → v2.1.0:

  + Added:   fastapi/app (v0.100)
  - Removed: flask/routing (v2.3)
  ~ Changed: openai/chat v4.0 → v4.5
  ~ Changed: stripe/api v2024-01 → v2024-06

  ✓ Unchanged: 8 pins
```

Version comparison uses exact string matching — `4.0` and `4.0.0` are treated as different versions.

### Rollback

Restoring a snapshot overwrites your current pins. Chub automatically creates a backup snapshot named `pre-restore-{name}` before restoring, so you can always undo:

```sh
# Something broke after updating pins
chub snapshot restore v2.0.0

# Changed your mind? Go back
chub snapshot restore pre-restore-v2.0.0
```

### Use case: debugging regressions

"The agent generated correct code in staging but wrong code in production."

1. Check which snapshot was active in each environment
2. Run `chub snapshot diff staging-v3 production-v3`
3. If a doc version changed, that's likely the cause
4. Restore the known-good snapshot and re-test

## Doc Freshness

Detect when pinned doc versions lag behind the library version actually installed in your project. Freshness checking reads your dependency files (`package.json`, `Cargo.toml`, `requirements.txt`, etc.) and compares installed versions against pinned doc versions.

Version comparison strips common prefixes (`^`, `~`, `=`, `>`, `<`, `v`) before comparing strings. For example, a pin of `4.0` is considered current if `requirements.txt` has `openai==4.0` or `openai~=4.0`.

```sh
chub check         # Compare pinned vs installed versions
chub check --fix   # Auto-update outdated pins
```

### Example output

```
⚠  openai/chat pinned to v4.0 docs, but openai==4.52.0 is installed
   → chub pin add openai/chat --version 4.52.0

✓  stripe/api docs are current
✓  redis/cache docs are current

1 outdated pin found. Run `chub check --fix` to update.
```

### Freshness in CI

Add a freshness check to your CI pipeline to catch drift early:

```sh
chub check
# Exits non-zero if any pin is outdated
```

See [CI/CD Integration](/guide/ci-cd) for full workflow examples.

## Usage Analytics

Track which docs are actually used. All data stays local, opt-in only.

```sh
chub stats           # Show usage analytics
chub stats --days 7  # Last 7 days
chub stats --json    # Machine-readable output
```

### Example output

```
Most fetched docs (last 30 days):
  1. openai/chat          — 142 fetches
  2. stripe/api           — 89 fetches

Never fetched (pinned but unused):
  - redis/cache           — pinned 45 days ago, 0 fetches

Suggestion: unpin unused docs to reduce noise.
```

### Acting on analytics

- **Heavily used docs** — make sure they're pinned to the right version and have annotations for known issues
- **Never fetched** — consider unpinning to reduce agent context noise
- **Spike in usage** — a team member is actively working with that library; good time to add annotations
