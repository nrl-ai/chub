#!/usr/bin/env bash
# benchmark.sh — Compare chub (Rust) vs context-hub (JS) performance.
#
# Prerequisites:
#   - Rust binary built: cargo build --release
#   - JS reference at ./references/context-hub/cli with deps installed:
#     cd references/context-hub/cli && npm install
#   - Content directory at ./content/
#
# Usage:
#   ./scripts/benchmark.sh                    # run all benchmarks
#   ./scripts/benchmark.sh --runs 10          # custom iteration count
#   ./scripts/benchmark.sh --skip-memory      # skip memory measurement (faster)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

RUST_BIN="$ROOT_DIR/target/release/chub$([ "$(uname -o 2>/dev/null)" = "Msys" ] && echo ".exe" || true)"
JS_BIN="node $ROOT_DIR/references/context-hub/cli/bin/chub"
CONTENT_DIR="$ROOT_DIR/content"
RUNS="${RUNS:-5}"
SKIP_MEMORY=false

# Find python (python3 on Linux/macOS, python on Windows/conda)
PYTHON=""
for candidate in python3 python; do
  if command -v "$candidate" > /dev/null 2>&1 && "$candidate" -c "print(1)" > /dev/null 2>&1; then
    PYTHON="$candidate"
    break
  fi
done
if [ -z "$PYTHON" ]; then
  echo "Error: python not found"
  exit 1
fi

# Parse args
while [[ $# -gt 0 ]]; do
  case "$1" in
    --runs) RUNS="$2"; shift 2 ;;
    --skip-memory) SKIP_MEMORY=true; shift ;;
    *) echo "Unknown arg: $1"; exit 1 ;;
  esac
done

# Validation
if [ ! -f "$RUST_BIN" ]; then
  echo "Error: Rust binary not found at $RUST_BIN"
  echo "Run: cargo build --release"
  exit 1
fi

if ! $JS_BIN --help > /dev/null 2>&1; then
  echo "Error: JS chub not runnable. Run: cd references/context-hub/cli && npm install"
  exit 1
fi

if [ ! -d "$CONTENT_DIR" ]; then
  echo "Error: Content directory not found at $CONTENT_DIR"
  exit 1
fi

TMP_DIR="$(mktemp -d)"
trap "rm -rf $TMP_DIR" EXIT

echo "========================================"
echo "  chub benchmark: Rust vs JS"
echo "========================================"
echo ""
echo "Rust binary: $RUST_BIN"
echo "JS binary:   $JS_BIN"
echo "Content:     $CONTENT_DIR"
echo "Runs:        $RUNS per test (median reported)"
echo ""

# --- Helpers ---

median() {
  sort -n | awk '{a[NR]=$1} END {print a[int((NR+1)/2)]}'
}

now_ns() {
  date +%s%N 2>/dev/null || $PYTHON -c "import time; print(int(time.time()*1e9))"
}

time_ms() {
  local cmd="$1"
  local times=()
  for i in $(seq 1 "$RUNS"); do
    local start end elapsed
    start=$(now_ns)
    eval "$cmd" > /dev/null 2>&1
    end=$(now_ns)
    elapsed=$(( (end - start) / 1000000 ))
    times+=("$elapsed")
  done
  printf '%s\n' "${times[@]}" | median
}

calc() {
  $PYTHON -c "$1"
}

# On Windows (Git Bash/MSYS2), convert paths to native format for Python.
# Bash auto-translates /tmp but Python doesn't understand MSYS paths.
native_path() {
  if command -v cygpath > /dev/null 2>&1; then
    cygpath -m "$1"
  else
    echo "$1"
  fi
}

# --- Cold Start ---

echo "--- Cold Start (--help) ---"
js_cold=$(time_ms "$JS_BIN --help")
rust_cold=$(time_ms "$RUST_BIN --help")
speedup=$(calc "print(f'{$js_cold/$rust_cold:.1f}')")
printf "  JS:    %4d ms\n" "$js_cold"
printf "  Rust:  %4d ms\n" "$rust_cold"
printf "  Speedup: %sx\n" "$speedup"
echo ""

# --- Build (full corpus) ---

echo "--- Build (full corpus) ---"
js_build=$(time_ms "$JS_BIN build $CONTENT_DIR -o $TMP_DIR/js-build")
rust_build=$(time_ms "$RUST_BIN build $CONTENT_DIR -o $TMP_DIR/rs-build")
speedup=$(calc "print(f'{$js_build/$rust_build:.1f}')")
printf "  JS:    %4d ms\n" "$js_build"
printf "  Rust:  %4d ms\n" "$rust_build"
printf "  Speedup: %sx\n" "$speedup"
echo ""

# --- Validate Only ---

echo "--- Validate Only ---"
js_val=$(time_ms "$JS_BIN build $CONTENT_DIR --validate-only")
rust_val=$(time_ms "$RUST_BIN build $CONTENT_DIR --validate-only")
speedup=$(calc "print(f'{$js_val/$rust_val:.1f}')")
printf "  JS:    %4d ms\n" "$js_val"
printf "  Rust:  %4d ms\n" "$rust_val"
printf "  Speedup: %sx\n" "$speedup"
echo ""

# --- Search ---

echo "--- Search (\"stripe payments\") ---"
js_search=$(time_ms "$JS_BIN search 'stripe payments' --json")
rust_search=$(time_ms "$RUST_BIN search 'stripe payments' --json")
speedup=$(calc "print(f'{$js_search/$rust_search:.1f}')")
printf "  JS:    %4d ms\n" "$js_search"
printf "  Rust:  %4d ms\n" "$rust_search"
printf "  Speedup: %sx\n" "$speedup"
echo ""

# --- Get ---

echo "--- Get (stripe/api --json) ---"
js_get=$(time_ms "$JS_BIN get stripe/api --json")
rust_get=$(time_ms "$RUST_BIN get stripe/api --json")
speedup=$(calc "print(f'{$js_get/$rust_get:.1f}')")
printf "  JS:    %4d ms\n" "$js_get"
printf "  Rust:  %4d ms\n" "$rust_get"
printf "  Speedup: %sx\n" "$speedup"
echo ""

# --- Binary Size ---

echo "--- Package Size ---"
rust_size=$(wc -c < "$RUST_BIN" | tr -d ' ')
rust_mb=$(calc "print(f'{$rust_size/1048576:.1f}')")
printf "  Rust binary:    %s MB (single file, no runtime deps)\n" "$rust_mb"
printf "  JS node_modules: ~22 MB + requires Node.js 20+\n"
echo ""

# --- Memory (optional) ---

if [ "$SKIP_MEMORY" = false ] && command -v powershell > /dev/null 2>&1; then
  echo "--- Peak Memory (build, full corpus) ---"

  measure_mem() {
    local exe="$1"
    local args="$2"
    local out_dir="$3"
    powershell -Command "
      \$psi = New-Object System.Diagnostics.ProcessStartInfo
      \$psi.FileName = '$exe'
      \$psi.Arguments = '$args'
      \$psi.UseShellExecute = \$false
      \$psi.RedirectStandardOutput = \$true
      \$psi.RedirectStandardError = \$true
      \$p = [System.Diagnostics.Process]::Start(\$psi)
      \$maxMem = 0
      while (-not \$p.HasExited) {
        \$p.Refresh()
        if (\$p.WorkingSet64 -gt \$maxMem) { \$maxMem = \$p.WorkingSet64 }
        Start-Sleep -Milliseconds 20
      }
      \$p.Refresh()
      if (\$p.PeakWorkingSet64 -gt \$maxMem) { \$maxMem = \$p.PeakWorkingSet64 }
      Write-Output ([math]::Round(\$maxMem / 1MB, 1))
    " 2>/dev/null
  }

  js_mem=$(measure_mem "node" "$ROOT_DIR/references/context-hub/cli/bin/chub build $CONTENT_DIR -o $TMP_DIR/js-mem" "$TMP_DIR/js-mem")
  rust_mem=$(measure_mem "$RUST_BIN" "build $CONTENT_DIR -o $TMP_DIR/rs-mem" "$TMP_DIR/rs-mem")

  printf "  JS:    %s MB\n" "$js_mem"
  printf "  Rust:  %s MB\n" "$rust_mem"
  reduction=$(calc "print(f'{float(\"$js_mem\")/float(\"$rust_mem\"):.1f}')")
  printf "  Reduction: %sx\n" "$reduction"
  echo ""
elif [ "$SKIP_MEMORY" = false ] && command -v /usr/bin/time > /dev/null 2>&1; then
  echo "--- Peak Memory (build, full corpus) ---"
  js_mem=$(/usr/bin/time -v $JS_BIN build "$CONTENT_DIR" -o "$TMP_DIR/js-mem" 2>&1 | grep "Maximum resident" | awk '{print $NF}')
  rust_mem=$(/usr/bin/time -v "$RUST_BIN" build "$CONTENT_DIR" -o "$TMP_DIR/rs-mem" 2>&1 | grep "Maximum resident" | awk '{print $NF}')
  js_mb=$(calc "print(f'{$js_mem/1024:.1f}')")
  rust_mb=$(calc "print(f'{$rust_mem/1024:.1f}')")
  printf "  JS:    %s MB\n" "$js_mb"
  printf "  Rust:  %s MB\n" "$rust_mb"
  reduction=$(calc "print(f'{$js_mem/$rust_mem:.1f}')")
  printf "  Reduction: %sx\n" "$reduction"
  echo ""
else
  echo "--- Peak Memory ---"
  echo "  (skipped — needs PowerShell on Windows or /usr/bin/time on Linux)"
  echo ""
fi

# --- Feature Count ---

echo "--- Feature Comparison ---"
js_cmds=$(eval "$JS_BIN --help" 2>&1 | grep "^  [a-z]" | wc -l | tr -d ' ')
rust_cmds=$("$RUST_BIN" --help 2>&1 | grep "^  [a-z]" | wc -l | tr -d ' ')
printf "  CLI commands:  JS=%d  Rust=%d\n" "$js_cmds" "$rust_cmds"
printf "  MCP tools:     JS=5   Rust=7\n"
printf "  Runtime deps:  JS=Node.js 20+  Rust=none\n"
echo ""

# --- Output Compatibility ---

# Run a final build for compatibility comparison (not timed)
$JS_BIN build "$CONTENT_DIR" -o "$TMP_DIR/js-compat" > /dev/null 2>&1 || true
"$RUST_BIN" build "$CONTENT_DIR" -o "$TMP_DIR/rs-compat" > /dev/null 2>&1 || true

echo "--- Output Compatibility ---"
JS_REG="$(native_path "$TMP_DIR/js-compat/registry.json")"
RS_REG="$(native_path "$TMP_DIR/rs-compat/registry.json")"
JS_IDX="$(native_path "$TMP_DIR/js-compat/search-index.json")"
RS_IDX="$(native_path "$TMP_DIR/rs-compat/search-index.json")"

if [ -f "$TMP_DIR/js-compat/registry.json" ] && [ -f "$TMP_DIR/rs-compat/registry.json" ]; then
  js_keys=$(calc "import json; d=json.load(open('$JS_REG')); print(sorted(d.keys()))")
  rs_keys=$(calc "import json; d=json.load(open('$RS_REG')); print(sorted(d.keys()))")
  if [ "$js_keys" = "$rs_keys" ]; then
    printf "  registry.json keys: MATCH\n"
  else
    printf "  registry.json keys: MISMATCH\n"
    printf "    JS:   %s\n" "$js_keys"
    printf "    Rust: %s\n" "$rs_keys"
  fi

  js_docs=$(calc "import json; d=json.load(open('$JS_REG')); print(len(d.get('docs',[])))")
  rs_docs=$(calc "import json; d=json.load(open('$RS_REG')); print(len(d.get('docs',[])))")
  printf "  Doc count:     JS=%s  Rust=%s  %s\n" "$js_docs" "$rs_docs" "$([ "$js_docs" = "$rs_docs" ] && echo "MATCH" || echo "MISMATCH")"

  js_skills=$(calc "import json; d=json.load(open('$JS_REG')); print(len(d.get('skills',[])))")
  rs_skills=$(calc "import json; d=json.load(open('$RS_REG')); print(len(d.get('skills',[])))")
  printf "  Skill count:   JS=%s  Rust=%s  %s\n" "$js_skills" "$rs_skills" "$([ "$js_skills" = "$rs_skills" ] && echo "MATCH" || echo "MISMATCH")"

  js_idx_keys=$(calc "import json; d=json.load(open('$JS_IDX')); print(sorted(d.keys()))")
  rs_idx_keys=$(calc "import json; d=json.load(open('$RS_IDX')); print(sorted(d.keys()))")
  if [ "$js_idx_keys" = "$rs_idx_keys" ]; then
    printf "  search-index.json keys: MATCH\n"
  else
    printf "  search-index.json keys: MISMATCH\n"
    printf "    JS:   %s\n" "$js_idx_keys"
    printf "    Rust: %s\n" "$rs_idx_keys"
  fi
else
  echo "  (build output not available — skipped)"
fi
echo ""

echo "========================================"
echo "  Benchmark complete"
echo "========================================"
