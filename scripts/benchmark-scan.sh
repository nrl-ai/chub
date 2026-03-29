#!/usr/bin/env bash
# Benchmark: chub scan vs gitleaks vs betterleaks
# Measures wall-clock time for directory scanning and git scanning.
#
# Usage: bash scripts/benchmark-scan.sh

set -euo pipefail

export PATH="/c/Program Files/Go/bin:$HOME/go/bin:$PATH:/c/Users/vieta/AppData/Local/Microsoft/WinGet/Links"

CHUB="./target/release/chub.exe"
GITLEAKS="gitleaks"
BETTERLEAKS="./references/betterleaks/betterleaks.exe"

# Betterleaks on Windows re-runs `git config --global --add safe.directory` each
# invocation; if the path already exists, git prints a hint to stderr which
# betterleaks treats as fatal (0 commits scanned). Pass safe.directory via
# GIT_CONFIG_COUNT env vars so git never touches the global config file.
REPO_ABS="$(pwd -W 2>/dev/null || pwd)"
export GIT_CONFIG_COUNT=1
export GIT_CONFIG_KEY_0=safe.directory
export GIT_CONFIG_VALUE_0="$REPO_ABS"

REPO_DIR="."
SMALL_DIR="./references/betterleaks/testdata/repos/small"
BENCH_DIR="./target/bench-corpus"
RESULTS_FILE="./target/benchmark-results.json"
RUNS=5

# Colors
BOLD="\033[1m"
CYAN="\033[36m"
GREEN="\033[32m"
YELLOW="\033[33m"
RESET="\033[0m"

echo -e "${BOLD}${CYAN}Chub Scan Benchmark${RESET}"
echo -e "Comparing: chub (Rust) vs gitleaks (Go) vs betterleaks (Go)"
echo ""

# Verify tools
echo -e "${BOLD}Tool versions:${RESET}"
echo -n "  chub:        "; $CHUB --version 2>&1 | head -1
echo -n "  gitleaks:    "; $GITLEAKS version 2>&1 | head -1
echo -n "  betterleaks: "; $BETTERLEAKS version 2>&1 | head -1 || echo "(version unavailable)"
echo ""

# ----- Generate synthetic corpus -----
echo -e "${BOLD}Generating synthetic benchmark corpus...${RESET}"
mkdir -p "$BENCH_DIR"

# Generate files with realistic code containing some secrets
generate_corpus() {
    local dir="$1"
    local count="$2"
    mkdir -p "$dir"

    for i in $(seq 1 "$count"); do
        cat > "$dir/file_$i.py" <<'PYEOF'
import os
import requests

# Configuration
API_URL = "https://api.example.com/v1"
DEBUG = os.getenv("DEBUG", "false")

def get_data():
    """Fetch data from the API."""
    headers = {"Authorization": f"Bearer {os.getenv('API_TOKEN')}"}
    response = requests.get(API_URL, headers=headers)
    return response.json()

class DataProcessor:
    def __init__(self, config):
        self.config = config
        self.cache = {}

    def process(self, data):
        result = []
        for item in data:
            if item["status"] == "active":
                result.append(self.transform(item))
        return result

    def transform(self, item):
        return {
            "id": item["id"],
            "name": item["name"].strip(),
            "value": float(item["value"]),
        }

def main():
    data = get_data()
    processor = DataProcessor({"batch_size": 100})
    results = processor.process(data)
    print(f"Processed {len(results)} items")

if __name__ == "__main__":
    main()
PYEOF
    done

    # Add some files with actual secrets (to ensure scanners find something)
    cat > "$dir/config_leaked.env" <<'ENVEOF'
AWS_ACCESS_KEY_ID=AKIAK4JM7NR2PX6SWT3B
AWS_SECRET_ACCESS_KEY=wJalrXUtnFEMI/K7MDENG/bPxRfiCYzk4Jm8nR2pX
GITHUB_TOKEN=ghp_k4Jm8nR2pX6sW9vB3fH7dL5qA1cY0eT2uI
STRIPE_SECRET_KEY=sk_live_4eC39HqLyjWDarjtT1zdp7dc
OPENAI_API_KEY=sk-proj-k4Jm8nR2pX6sW9vB3fH7dL5qA1cY0eT2uIgKxZwN4rP8mVjQ3bF
ANTHROPIC_API_KEY=sk-ant-api03-k4Jm8nR2pX6sW9vB3fH7dL5qA1cY0eT2uIgKxZwN4rP8mVjQ3bF6yDhLsWtR7nXcJpGk2aE9fUoMiAA
SLACK_BOT_TOKEN=xoxb-123456789012-1234567890123-k4Jm8nR2pX6sW9vB3fH7dLqR
ENVEOF

    # Add an AI transcript file
    cat > "$dir/agent_transcript.log" <<'LOGEOF'
[2024-01-15 10:30:00] User: Can you help me set up the AWS SDK?
[2024-01-15 10:30:01] Agent: Sure! Here's how to configure it:

```python
import boto3

# Using your credentials
client = boto3.client(
    's3',
    aws_access_key_id='AKIAK4JM7NR2PX6SWT3B',
    aws_secret_access_key='wJalrXUtnFEMI/K7MDENG/bPxRfiCYzk4Jm8nR2pX'
)
```

[2024-01-15 10:31:00] User: Now let me set up OpenAI
[2024-01-15 10:31:01] Agent: Here's the setup:

```python
import openai
openai.api_key = "sk-proj-k4Jm8nR2pX6sW9vB3fH7dL5qA1cY0eT2uIgKxZwN4rP8mVjQ3bF"
```
LOGEOF
}

# Small corpus: 100 files
generate_corpus "$BENCH_DIR/small" 100

# Medium corpus: 500 files
generate_corpus "$BENCH_DIR/medium" 500

# Large corpus: 1000 files
generate_corpus "$BENCH_DIR/large" 1000

echo "  small:  $(find "$BENCH_DIR/small" -type f | wc -l) files"
echo "  medium: $(find "$BENCH_DIR/medium" -type f | wc -l) files"
echo "  large:  $(find "$BENCH_DIR/large" -type f | wc -l) files"
echo ""

# ----- Timing helper -----
# Returns elapsed time in milliseconds
time_cmd() {
    local start end elapsed
    start=$(date +%s%N 2>/dev/null || python3 -c "import time; print(int(time.time()*1e9))")
    eval "$@" > /dev/null 2>&1 || true
    end=$(date +%s%N 2>/dev/null || python3 -c "import time; print(int(time.time()*1e9))")
    elapsed=$(( (end - start) / 1000000 ))
    echo "$elapsed"
}

# Run N times, return median
median_time() {
    local cmd="$1"
    local times=()
    for _ in $(seq 1 $RUNS); do
        t=$(time_cmd "$cmd")
        times+=("$t")
    done
    # Sort and pick median
    IFS=$'\n' sorted=($(sort -n <<<"${times[*]}")); unset IFS
    mid=$((RUNS / 2))
    echo "${sorted[$mid]}"
}

# ----- Directory scan benchmarks -----
echo -e "${BOLD}${CYAN}=== Directory Scan Benchmarks (median of $RUNS runs) ===${RESET}"
echo ""

declare -A results

for size in small medium large; do
    dir="$BENCH_DIR/$size"
    file_count=$(find "$dir" -type f | wc -l)
    echo -e "${BOLD}--- $size ($file_count files) ---${RESET}"

    # Chub
    t=$(median_time "$CHUB scan secrets dir $dir --no-banner -r /dev/null")
    results["chub_dir_$size"]=$t
    echo -e "  chub:        ${GREEN}${t} ms${RESET}"

    # Gitleaks
    t=$(median_time "$GITLEAKS dir --no-banner -r /dev/null $dir")
    results["gitleaks_dir_$size"]=$t
    echo -e "  gitleaks:    ${YELLOW}${t} ms${RESET}"

    # Betterleaks
    t=$(median_time "$BETTERLEAKS dir --no-banner -r /dev/null $dir")
    results["betterleaks_dir_$size"]=$t
    echo -e "  betterleaks: ${YELLOW}${t} ms${RESET}"

    echo ""
done

# ----- Git scan benchmark -----
echo -e "${BOLD}${CYAN}=== Git History Scan Benchmark (this repo, $(git log --oneline | wc -l) commits) ===${RESET}"
echo ""

t=$(median_time "$CHUB scan secrets git . --no-banner -r /dev/null")
results["chub_git"]=$t
echo -e "  chub:        ${GREEN}${t} ms${RESET}"

t=$(median_time "$GITLEAKS git --no-banner -r /dev/null .")
results["gitleaks_git"]=$t
echo -e "  gitleaks:    ${YELLOW}${t} ms${RESET}"

t=$(median_time "$BETTERLEAKS git --no-banner -r /dev/null .")
results["betterleaks_git"]=$t
echo -e "  betterleaks: ${YELLOW}${t} ms${RESET}"

echo ""

# ----- Summary table -----
echo -e "${BOLD}${CYAN}=== Summary ===${RESET}"
echo ""
printf "%-25s %10s %10s %10s %12s\n" "Benchmark" "Chub" "Gitleaks" "Betterleaks" "Chub Speedup"
printf "%-25s %10s %10s %10s %12s\n" "-------------------------" "----------" "----------" "-----------" "------------"

for size in small medium large; do
    c=${results["chub_dir_$size"]}
    g=${results["gitleaks_dir_$size"]}
    b=${results["betterleaks_dir_$size"]}
    # Speedup vs slowest competitor
    if [ "$g" -gt "$b" ]; then
        slowest=$g
        slower_name="gitleaks"
    else
        slowest=$b
        slower_name="betterleaks"
    fi
    if [ "$c" -gt 0 ]; then
        speedup=$(awk "BEGIN {printf \"%.1f\", $slowest / $c}")
    else
        speedup="inf"
    fi
    printf "%-25s %8s ms %8s ms %9s ms %10sx vs %s\n" "dir/$size" "$c" "$g" "$b" "$speedup" "$slower_name"
done

c=${results["chub_git"]}
g=${results["gitleaks_git"]}
b=${results["betterleaks_git"]}
if [ "$g" -gt "$b" ]; then
    slowest=$g
    slower_name="gitleaks"
else
    slowest=$b
    slower_name="betterleaks"
fi
if [ "$c" -gt 0 ]; then
    speedup=$(awk "BEGIN {printf \"%.1f\", $slowest / $c}")
else
    speedup="inf"
fi
printf "%-25s %8s ms %8s ms %9s ms %10sx vs %s\n" "git (this repo)" "$c" "$g" "$b" "$speedup" "$slower_name"

echo ""

# ----- Write JSON results -----
cat > "$RESULTS_FILE" <<JSONEOF
{
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "runs": $RUNS,
  "tools": {
    "chub": "$($CHUB --version 2>&1 | head -1)",
    "gitleaks": "$($GITLEAKS version 2>&1 | head -1)",
    "betterleaks": "source build"
  },
  "benchmarks": {
    "dir_small": {
      "files": $(find "$BENCH_DIR/small" -type f | wc -l),
      "chub_ms": ${results["chub_dir_small"]},
      "gitleaks_ms": ${results["gitleaks_dir_small"]},
      "betterleaks_ms": ${results["betterleaks_dir_small"]}
    },
    "dir_medium": {
      "files": $(find "$BENCH_DIR/medium" -type f | wc -l),
      "chub_ms": ${results["chub_dir_medium"]},
      "gitleaks_ms": ${results["gitleaks_dir_medium"]},
      "betterleaks_ms": ${results["betterleaks_dir_medium"]}
    },
    "dir_large": {
      "files": $(find "$BENCH_DIR/large" -type f | wc -l),
      "chub_ms": ${results["chub_dir_large"]},
      "gitleaks_ms": ${results["gitleaks_dir_large"]},
      "betterleaks_ms": ${results["betterleaks_dir_large"]}
    },
    "git_history": {
      "commits": $(git log --oneline | wc -l),
      "chub_ms": ${results["chub_git"]},
      "gitleaks_ms": ${results["gitleaks_git"]},
      "betterleaks_ms": ${results["betterleaks_git"]}
    }
  }
}
JSONEOF

echo -e "Results written to ${BOLD}$RESULTS_FILE${RESET}"
echo ""
echo -e "${BOLD}Done.${RESET}"
