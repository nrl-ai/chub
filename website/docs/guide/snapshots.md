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

### Use case

"The agent generated correct code in staging but wrong code in production" — snapshot diffs identify whether a doc update caused the regression.

## Doc Freshness

Detect when pinned doc versions lag behind the library version actually installed in your project.

```sh
chub check         # Compare pinned vs installed versions
chub check --fix   # Auto-update outdated pins
```

### Example output

```
⚠  openai/chat pinned to v4.0 docs, but openai==4.52.0 is installed
   → chub pin openai/chat --version 4.52.0

✓  stripe/api docs are current
✓  redis/cache docs are current

1 outdated pin found. Run `chub check --fix` to update.
```

## Usage Analytics

Track which docs are actually used. All data stays local, opt-in only.

```sh
chub stats           # Show usage analytics
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
