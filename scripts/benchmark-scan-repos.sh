#!/usr/bin/env bash
# Benchmark: chub scan vs gitleaks vs betterleaks on 10 real public repos
# Usage: bash scripts/benchmark-scan-repos.sh

set -euo pipefail

export PATH="/c/Program Files/Go/bin:$HOME/go/bin:$PATH:/c/Users/vieta/AppData/Local/Microsoft/WinGet/Links"

CHUB="./target/release/chub.exe"
GITLEAKS="gitleaks"
BETTERLEAKS="./references/betterleaks/betterleaks.exe"
RUNS=3

# Betterleaks safe.directory workaround
REPO_ABS="$(pwd -W 2>/dev/null || pwd)"
export GIT_CONFIG_COUNT=1
export GIT_CONFIG_KEY_0=safe.directory
export GIT_CONFIG_VALUE_0="*"

BOLD="\033[1m"
CYAN="\033[36m"
GREEN="\033[32m"
YELLOW="\033[33m"
RESET="\033[0m"

echo -e "${BOLD}${CYAN}Chub Real-Repo Scan Benchmark${RESET}"
echo -e "Repos: 10 public GitHub repositories. Median of $RUNS runs."
echo ""

echo -e "${BOLD}Tool versions:${RESET}"
echo -n "  chub:        "; $CHUB --version 2>&1 | head -1
echo -n "  gitleaks:    "; $GITLEAKS version 2>&1 | head -1
echo -n "  betterleaks: "; $BETTERLEAKS version 2>&1 | head -1 || echo "(version unavailable)"
echo ""

# Timing helper (milliseconds)
time_cmd() {
    local start end
    start=$(python -c "import time; print(int(time.time()*1e9))")
    eval "$@" > /dev/null 2>&1 || true
    end=$(python -c "import time; print(int(time.time()*1e9))")
    echo $(( (end - start) / 1000000 ))
}

median_time() {
    local cmd="$1"
    local times=()
    for _ in $(seq 1 $RUNS); do
        times+=("$(time_cmd "$cmd")")
    done
    IFS=$'\n' sorted=($(sort -n <<<"${times[*]}")); unset IFS
    echo "${sorted[$((RUNS / 2))]}"
}

# Repos to benchmark (name:path)
declare -A REPOS=(
    ["axios"]="./axios"
    ["deno"]="./deno"
    ["django"]="./django"
    ["express"]="./express"
    ["fastapi"]="./fastapi-real"
    ["golang/go"]="./golang"
    ["tokio"]="./tokio"
    ["hashicorp/vault"]="./vault"
    ["flask"]="./flask-repo"
    ["openai-python"]="./openai-python"
)

# Ordered list for consistent output
REPO_NAMES=(axios deno django express fastapi "golang/go" tokio "hashicorp/vault" flask openai-python)

echo -e "${BOLD}${CYAN}=== Directory Scan (median of $RUNS runs) ===${RESET}"
echo ""
printf "%-20s %8s %8s %8s %8s %9s\n" "Repo" "Files" "Chub" "Gitleaks" "BLeaks" "Speedup"
printf "%-20s %8s %8s %8s %8s %9s\n" "--------------------" "-------" "-------" "--------" "------" "--------"

declare -A dir_results_chub
declare -A dir_results_gitleaks
declare -A dir_results_betterleaks
declare -A dir_file_counts

for name in "${REPO_NAMES[@]}"; do
    path="${REPOS[$name]}"
    if [ ! -d "$path" ]; then
        echo "  SKIP $name (not found at $path)"
        continue
    fi
    file_count=$(find "$path" -type f -not -path '*/.git/*' | wc -l)
    dir_file_counts["$name"]=$file_count

    c=$(median_time "$CHUB scan secrets dir $path --no-banner -r /dev/null")
    g=$(median_time "$GITLEAKS dir --no-banner -r /dev/null $path")
    b=$(median_time "$BETTERLEAKS dir --no-banner -r /dev/null --validation=false $path")

    dir_results_chub["$name"]=$c
    dir_results_gitleaks["$name"]=$g
    dir_results_betterleaks["$name"]=$b

    # speedup vs slower competitor
    if [ "$g" -gt "$b" ]; then slowest=$g; else slowest=$b; fi
    if [ "$c" -gt 0 ]; then
        speedup=$(awk "BEGIN {printf \"%.1fx\", $slowest / $c}")
    else
        speedup="inf"
    fi
    printf "%-20s %7d %6dms %7dms %6dms %9s\n" "$name" "$file_count" "$c" "$g" "$b" "$speedup"
done

echo ""
echo -e "${BOLD}${CYAN}=== Git History Scan (median of $RUNS runs) ===${RESET}"
echo ""
printf "%-20s %8s %8s %8s %8s %9s\n" "Repo" "Commits" "Chub" "Gitleaks" "BLeaks" "Speedup"
printf "%-20s %8s %8s %8s %8s %9s\n" "--------------------" "-------" "-------" "--------" "------" "--------"

declare -A git_results_chub
declare -A git_results_gitleaks
declare -A git_results_betterleaks
declare -A git_commit_counts

for name in "${REPO_NAMES[@]}"; do
    path="${REPOS[$name]}"
    if [ ! -d "$path/.git" ]; then
        echo "  SKIP $name"
        continue
    fi
    commit_count=$(git -C "$path" log --oneline | wc -l)
    git_commit_counts["$name"]=$commit_count

    export GIT_CONFIG_VALUE_0="$(cd "$path" && pwd -W 2>/dev/null || pwd)"

    c=$(median_time "$CHUB scan secrets git $path --no-banner -r /dev/null")
    g=$(median_time "$GITLEAKS git --no-banner -r /dev/null --source $path")
    b=$(median_time "$BETTERLEAKS git --no-banner -r /dev/null --validation=false --git-workers=8 $path")

    git_results_chub["$name"]=$c
    git_results_gitleaks["$name"]=$g
    git_results_betterleaks["$name"]=$b

    if [ "$g" -gt "$b" ]; then slowest=$g; else slowest=$b; fi
    if [ "$c" -gt 0 ]; then
        speedup=$(awk "BEGIN {printf \"%.1fx\", $slowest / $c}")
    else
        speedup="inf"
    fi
    printf "%-20s %7d %6dms %7dms %6dms %9s\n" "$name" "$commit_count" "$c" "$g" "$b" "$speedup"
done

echo ""
echo -e "${BOLD}Done.${RESET}"

# Write JSON
mkdir -p ./target
cat > ./target/benchmark-repos-results.json <<JSONEOF
{
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "runs": $RUNS,
  "tools": {
    "chub": "$($CHUB --version 2>&1 | head -1)",
    "gitleaks": "$($GITLEAKS version 2>&1 | head -1)",
    "betterleaks": "source build"
  },
  "dir_scan": {
$(for name in "${REPO_NAMES[@]}"; do
    [ -z "${dir_results_chub[$name]+x}" ] && continue
    echo "    \"$name\": {\"files\": ${dir_file_counts[$name]}, \"chub_ms\": ${dir_results_chub[$name]}, \"gitleaks_ms\": ${dir_results_gitleaks[$name]}, \"betterleaks_ms\": ${dir_results_betterleaks[$name]}},"
done | sed '$ s/,$//')
  },
  "git_scan": {
$(for name in "${REPO_NAMES[@]}"; do
    [ -z "${git_results_chub[$name]+x}" ] && continue
    echo "    \"$name\": {\"commits\": ${git_commit_counts[$name]}, \"chub_ms\": ${git_results_chub[$name]}, \"gitleaks_ms\": ${git_results_gitleaks[$name]}, \"betterleaks_ms\": ${git_results_betterleaks[$name]}},"
done | sed '$ s/,$//')
  }
}
JSONEOF
echo "Results → ./target/benchmark-repos-results.json"
