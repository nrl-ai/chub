#!/bin/bash
# chub-freshness-check.sh — warn if pinned docs are stale before committing
# Runs as a PreToolUse hook on Bash commands containing "git commit"

INPUT=$(cat)

# Only trigger on git commit commands (grep without jq dependency)
if ! echo "$INPUT" | grep -q '"git commit'; then
  exit 0
fi

# Check if chub is available
if ! command -v chub &> /dev/null; then
  exit 0
fi

# Check if .chub/ exists (project uses chub)
if [ ! -d ".chub" ]; then
  exit 0
fi

# Run freshness check (parse stale count without jq)
CHECK_OUTPUT=$(chub check --json 2>/dev/null)
if echo "$CHECK_OUTPUT" | grep -q '"stale"'; then
  STALE_COUNT=$(echo "$CHECK_OUTPUT" | grep -o '"stale"' | wc -l)
  if [ "$STALE_COUNT" -gt 0 ]; then
    echo "Warning: some pinned docs may be stale. Run 'chub check' for details." >&2
  fi
fi

# Don't block the commit, just warn
exit 0
