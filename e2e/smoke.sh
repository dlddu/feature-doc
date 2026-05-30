#!/usr/bin/env bash
# HTTP smoke test against a stub-mode deployment:
#   - GET /hello returns the expected JSON probe
#   - GET / returns the SPA shell
#   - the credential flow never exposes an LLM key's plaintext (AC4.3)
set -euo pipefail

BASE_URL="${BASE_URL:-http://localhost:8080}"
EXPECTED_MESSAGE='Hello from FeatureDoc backend'

tmp="$(mktemp)"
jar="$(mktemp)"
trap 'rm -f "${tmp}" "${jar}"' EXIT

echo "smoke: GET ${BASE_URL}/hello"
hello_body="$(curl -fsS "${BASE_URL}/hello")"
echo "  body: ${hello_body}"
case "${hello_body}" in
  *'"message"'*"${EXPECTED_MESSAGE}"*) ;;
  *) echo "  fail: missing or unexpected message"; exit 1 ;;
esac

echo "smoke: GET ${BASE_URL}/ → expect 200 + SPA shell"
status="$(curl -fsS -o "${tmp}" -w '%{http_code}' "${BASE_URL}/")"
if [ "${status}" != "200" ]; then
  echo "  fail: expected 200, got ${status}"
  exit 1
fi
if ! grep -q '<div id="root">' "${tmp}"; then
  echo "  fail: index.html missing #root"
  exit 1
fi

echo "smoke: credential flow (stub) — an LLM key's plaintext must never be exposed"
SENTINEL='sk-ant-api03-PLAINTEXTSENTINEL0001'

# Log in (stub) and capture the session cookie.
curl -fsS -L -c "${jar}" -b "${jar}" -o /dev/null "${BASE_URL}/api/auth/login?as=smoke"

# Register a key whose plaintext carries a unique sentinel.
register="$(curl -fsS -b "${jar}" -X POST -H 'content-type: application/json' \
  -d "{\"provider\":\"anthropic\",\"key\":\"${SENTINEL}\"}" "${BASE_URL}/api/llm-keys")"
echo "  register: ${register}"
case "${register}" in
  *'"masked"'*'sk-ant-'*) ;;
  *) echo "  fail: register did not return a masked identifier"; exit 1 ;;
esac

# The sentinel plaintext must not appear in any credential-bearing response.
for ep in /api/llm-keys /api/me /api/audit; do
  body="$(curl -fsS -b "${jar}" "${BASE_URL}${ep}")"
  case "${body}" in
    *"${SENTINEL}"*) echo "  fail: ${ep} leaked the key plaintext"; exit 1 ;;
  esac
done
case "${register}" in
  *"${SENTINEL}"*) echo "  fail: register response leaked the key plaintext"; exit 1 ;;
esac
echo "  ok: identifiers only, no plaintext exposed"

echo "smoke: ok"
