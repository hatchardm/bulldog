#!/usr/bin/env bash
set -euo pipefail

# Bulldog run‑mode helper
# Usage:
#   ./scripts/run.sh            → baseline (screen only)
#   ./scripts/run.sh --log      → baseline + serial log → bulldog.log
#   ./scripts/run.sh --harness  → syscall harness (screen only)
#   ./scripts/run.sh --full     → syscall harness + serial log → harness.log

cd ~/bulldog

case "${1:-}" in
  --log)
    echo "[run] Baseline + serial logging → bulldog.log"
    cargo +nightly run -Z bindeps --features "serial_log" > bulldog.log
    ;;
  --harness)
    echo "[run] Syscall harness (screen only)"
    cargo +nightly run -Z bindeps --features "syscall_tests"
    ;;
  --full)
    echo "[run] Syscall harness + serial logging → harness.log"
    cargo +nightly run -Z bindeps --features "syscall_tests serial_log" > harness.log
    ;;
  *)
    echo "[run] Baseline (screen only)"
    cargo +nightly run -Z bindeps
    ;;
esac

