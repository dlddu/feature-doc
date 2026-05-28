#!/usr/bin/env bash
# HTTP smoke test: GET /hello returns expected JSON, GET / returns SPA shell.
set -euo pipefail

BASE_URL="${BASE_URL:-http://localhost:8080}"
EXPECTED_MESSAGE='Hello from FeatureDoc backend'

echo "smoke: GET ${BASE_URL}/hello"
hello_body="$(curl -fsS "${BASE_URL}/hello")"
echo "  body: ${hello_body}"
case "${hello_body}" in
  *'"message"'*"${EXPECTED_MESSAGE}"*) ;;
  *) echo "  fail: missing or unexpected message"; exit 1 ;;
esac

echo "smoke: GET ${BASE_URL}/ → expect 200 + SPA shell"
tmp="$(mktemp)"
trap 'rm -f "${tmp}"' EXIT
status="$(curl -fsS -o "${tmp}" -w '%{http_code}' "${BASE_URL}/")"
if [ "${status}" != "200" ]; then
  echo "  fail: expected 200, got ${status}"
  exit 1
fi
if ! grep -q '<div id="root">' "${tmp}"; then
  echo "  fail: index.html missing #root"
  exit 1
fi

echo "smoke: ok"
