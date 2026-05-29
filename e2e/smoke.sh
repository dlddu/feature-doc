#!/usr/bin/env bash
# HTTP smoke test:
#   1. GET /hello returns expected JSON (readiness contract).
#   2. GET / returns the SPA shell.
#   3. The S01 credential flow works end to end (mock mode): sign in, register
#      an LLM key, list it — and assert the plaintext key is NEVER echoed back
#      (AC4.3: credentials are masked, never exposed).
set -euo pipefail

BASE_URL="${BASE_URL:-http://localhost:8080}"
EXPECTED_MESSAGE='Hello from FeatureDoc backend'

# A distinctive plaintext so we can grep responses (here) and pod logs
# (scripts/e2e.sh) to prove it never leaks. Valid for the mock validator
# (anthropic prefix, no "invalid", long enough).
SENTINEL='PLAINTEXTSENTINEL'
TEST_KEY="sk-ant-${SENTINEL}-abcd1234"

echo "smoke: GET ${BASE_URL}/hello"
hello_body="$(curl -fsS "${BASE_URL}/hello")"
echo "  body: ${hello_body}"
case "${hello_body}" in
  *'"message"'*"${EXPECTED_MESSAGE}"*) ;;
  *) echo "  fail: missing or unexpected message"; exit 1 ;;
esac

echo "smoke: GET ${BASE_URL}/ → expect 200 + SPA shell"
tmp="$(mktemp)"
jar="$(mktemp)"
trap 'rm -f "${tmp}" "${jar}"' EXIT
status="$(curl -fsS -o "${tmp}" -w '%{http_code}' "${BASE_URL}/")"
if [ "${status}" != "200" ]; then
  echo "  fail: expected 200, got ${status}"
  exit 1
fi
if ! grep -q '<div id="root">' "${tmp}"; then
  echo "  fail: index.html missing #root"
  exit 1
fi

echo "smoke: unauthenticated /api/me → expect 401"
status="$(curl -fsS -o /dev/null -w '%{http_code}' "${BASE_URL}/api/me" || true)"
if [ "${status}" != "401" ]; then
  echo "  fail: expected 401 for unauthenticated /api/me, got ${status}"
  exit 1
fi

echo "smoke: sign in (mock OAuth, following redirects)"
# -L follows the login → callback → / redirect chain, carrying cookies in jar.
curl -fsS -L -c "${jar}" -b "${jar}" -o /dev/null "${BASE_URL}/api/auth/login"
me_body="$(curl -fsS -b "${jar}" "${BASE_URL}/api/me")"
echo "  me: ${me_body}"
case "${me_body}" in
  *'"login"'*) ;;
  *) echo "  fail: session not established"; exit 1 ;;
esac

echo "smoke: register an LLM key"
reg_body="$(curl -fsS -b "${jar}" -H 'content-type: application/json' \
  -d "{\"provider\":\"anthropic\",\"key\":\"${TEST_KEY}\"}" \
  "${BASE_URL}/api/llm-keys")"
echo "  registered: ${reg_body}"
if printf '%s' "${reg_body}" | grep -q "${SENTINEL}"; then
  echo "  fail: registration response leaked the plaintext key"
  exit 1
fi

echo "smoke: list keys → masked only, no plaintext"
list_body="$(curl -fsS -b "${jar}" "${BASE_URL}/api/llm-keys")"
echo "  list: ${list_body}"
if printf '%s' "${list_body}" | grep -q "${SENTINEL}"; then
  echo "  fail: key listing leaked the plaintext key"
  exit 1
fi
case "${list_body}" in
  *'"masked"'*) ;;
  *) echo "  fail: listing missing masked identifier"; exit 1 ;;
esac

echo "smoke: ok"
