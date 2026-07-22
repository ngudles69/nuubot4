#!/usr/bin/env bash

set -u -o pipefail

runs="${1:-5}"
sweep_id="${2:-6}"
bot_id="${3:-9}"
for value in "$runs" "$sweep_id" "$bot_id"; do
    if [[ ! "$value" =~ ^[1-9][0-9]*$ ]]; then
        echo "usage: bash rtest.sh [runs] [sweep_id] [bot_id]" >&2
        exit 2
    fi
done

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
binary="$repo_root/target/release/nuubot-btrunner.exe"
if [[ ! -x "$binary" ]]; then
    echo "Rust binary not found: $binary" >&2
    exit 2
fi

log_dir="$repo_root/workspace/logs"
mkdir -p "$log_dir"
stamp="$(date -u +%Y%m%dT%H%M%SZ)"
result_log="$log_dir/nuubot4-rtest-s${sweep_id}-b${bot_id}-${runs}-${stamp}.log"
exec > >(tee -a "$result_log") 2>&1

passed=0
total_ms=0
minimum_ms=0
maximum_ms=0

for ((run = 1; run <= runs; run++)); do
    started_ms="$(date +%s%3N)"
    timeout 120s "$binary" "$sweep_id" "$bot_id"
    status=$?
    elapsed_ms=$(( $(date +%s%3N) - started_ms ))
    total_ms=$((total_ms + elapsed_ms))
    if [[ $minimum_ms -eq 0 || $elapsed_ms -lt $minimum_ms ]]; then
        minimum_ms=$elapsed_ms
    fi
    if [[ $elapsed_ms -gt $maximum_ms ]]; then
        maximum_ms=$elapsed_ms
    fi
    if [[ $status -ne 0 ]]; then
        printf 'run=%d result=FAIL exit=%d elapsed_ms=%d\n' "$run" "$status" "$elapsed_ms"
        printf 'requested=%d attempted=%d passed=%d failed=1 total_ms=%d min_ms=%d max_ms=%d log=%s\n' \
            "$runs" "$run" "$passed" "$total_ms" "$minimum_ms" "$maximum_ms" "$result_log"
        exit "$status"
    fi
    ((passed += 1))
    printf 'run=%d result=PASS exit=0 elapsed_ms=%d\n' "$run" "$elapsed_ms"
    if [[ $run -lt $runs ]]; then
        sleep 1
    fi
done

printf 'requested=%d attempted=%d passed=%d failed=0 total_ms=%d average_ms=%d min_ms=%d max_ms=%d log=%s\n' \
    "$runs" "$runs" "$passed" "$total_ms" "$((total_ms / runs))" "$minimum_ms" "$maximum_ms" "$result_log"
