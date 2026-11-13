#!/bin/sh
set -eu
repo_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cd "$repo_root"

cargo build --workspace --bins
target/debug/ecommerce-api --mode baseline --port 3001 >/tmp/api-compat-baseline.log 2>&1 &
baseline_pid=$!
target/debug/ecommerce-api --mode candidate --port 3002 >/tmp/api-compat-candidate.log 2>&1 &
candidate_pid=$!
target/debug/ecommerce-api --mode compatible --port 3003 >/tmp/api-compat-compatible.log 2>&1 &
compatible_pid=$!
cleanup() { kill "$baseline_pid" "$candidate_pid" "$compatible_pid" 2>/dev/null || true; wait "$baseline_pid" "$candidate_pid" "$compatible_pid" 2>/dev/null || true; }
trap cleanup EXIT HUP INT TERM

wait_for() {
  url=$1
  attempts=0
  until curl -fsS "$url" >/dev/null 2>&1; do
    attempts=$((attempts + 1))
    if [ "$attempts" -ge 40 ]; then echo "Demo API did not become ready: $url" >&2; return 1; fi
    sleep 0.25
  done
}
wait_for http://127.0.0.1:3001/health
wait_for http://127.0.0.1:3002/health
wait_for http://127.0.0.1:3003/health

target/debug/api-compat capture \
  --base-url http://127.0.0.1:3001 \
  --openapi examples/ecommerce-api/openapi/baseline.yaml \
  --cases examples/ecommerce-api/cases \
  --out .api-compat/baseline.json

set +e
target/debug/api-compat check \
  --baseline .api-compat/baseline.json \
  --candidate-url http://127.0.0.1:3002 \
  --old-openapi examples/ecommerce-api/openapi/baseline.yaml \
  --new-openapi examples/ecommerce-api/openapi/candidate.yaml \
  --out reports/candidate
candidate_status=$?
set -e
if [ "$candidate_status" -ne 2 ] && [ "$candidate_status" -ne 3 ]; then
  echo "Expected breaking candidate to block release (exit 2 or 3), got $candidate_status" >&2
  exit 1
fi

target/debug/api-compat check \
  --baseline .api-compat/baseline.json \
  --candidate-url http://127.0.0.1:3003 \
  --old-openapi examples/ecommerce-api/openapi/baseline.yaml \
  --new-openapi examples/ecommerce-api/openapi/compatible.yaml \
  --out reports/compatible

echo "Demo complete. Breaking report: reports/candidate/index.html"
echo "Compatible report: reports/compatible/index.html"
