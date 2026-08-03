#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
RUN_DIR=$(mktemp -d)
export LLM_STUB=1
export GONI_RECEIPTS_FILE="$RUN_DIR/receipts.jsonl"
export GONI_SMOKE_REQUIRE_QDRANT=0

cleanup() {
  if [ -n "${PID:-}" ] && kill -0 "$PID" 2>/dev/null; then
    kill "$PID"
    wait "$PID" 2>/dev/null || true
  fi
  rm -rf "$RUN_DIR"
}
trap cleanup EXIT

cd "$ROOT_DIR/software/kernel"
cargo run -p goni-http >"$RUN_DIR/goni-http.log" 2>&1 &
PID=$!

for _ in $(seq 1 60); do
  if curl -fsS http://localhost:7000/healthz >/dev/null 2>&1; then
    break
  fi
  if ! kill -0 "$PID" 2>/dev/null; then
    cat "$RUN_DIR/goni-http.log" >&2
    echo "goni-http exited before becoming ready" >&2
    exit 1
  fi
  sleep 1
done

if ! curl -fsS http://localhost:7000/healthz >/dev/null 2>&1; then
  cat "$RUN_DIR/goni-http.log" >&2
  echo "goni-http did not become ready within 60 seconds" >&2
  exit 1
fi

bash "$ROOT_DIR/scripts/smoke_test.sh"
