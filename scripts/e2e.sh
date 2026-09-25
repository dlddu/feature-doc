#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
CLUSTER_NAME="${CLUSTER_NAME:-featuredoc}"
IMAGE="${IMAGE:-featuredoc:dev}"
KEEP_CLUSTER="${KEEP_CLUSTER:-0}"
LOCAL_PORT="${LOCAL_PORT:-8080}"
SKIP_BUILD="${SKIP_BUILD:-0}"
SKIP_PLAYWRIGHT_INSTALL="${SKIP_PLAYWRIGHT_INSTALL:-0}"

PF_PID=""
PF_LOG="${PF_LOG:-/tmp/featuredoc-pf.log}"

# The forward is supervised rather than one-shot. `kubectl port-forward svc/…` binds to
# one pod behind the Service and exits when that pod goes away, so any spec that edits
# API env kills it for good. Respawning against whatever pod is ready now is what makes
# an API rollout survivable.
supervise_port_forward() {
  local child=''
  trap 'kill "${child}" 2>/dev/null || true; exit 0' TERM INT
  while :; do
    kubectl port-forward svc/featuredoc "${LOCAL_PORT}:8080" >>"${PF_LOG}" 2>&1 &
    child=$!
    wait "${child}" 2>/dev/null || true
    echo "[pf] forward exited — re-establishing against the current pod" >>"${PF_LOG}"
    sleep 1
  done
}

cleanup() {
  if [ -n "${PF_PID}" ] && kill -0 "${PF_PID}" 2>/dev/null; then
    echo "[cleanup] stop port-forward supervisor (pid ${PF_PID})"
    kill "${PF_PID}" 2>/dev/null || true
    wait "${PF_PID}" 2>/dev/null || true
  fi
  # kind e2e only ever runs in CI, so this log is the only surviving record of a mid-run
  # pod swap. Print it rather than letting it go with the runner.
  if [ -s "${PF_LOG}" ]; then
    echo "[cleanup] port-forward log (${PF_LOG}):"
    sed 's/^/  | /' "${PF_LOG}"
  fi
  if [ "${KEEP_CLUSTER}" != "1" ]; then
    echo "[cleanup] kind delete cluster --name ${CLUSTER_NAME}"
    kind delete cluster --name "${CLUSTER_NAME}" >/dev/null 2>&1 || true
  fi
}
trap cleanup EXIT INT TERM

require() {
  command -v "$1" >/dev/null 2>&1 || { echo "missing required tool: $1" >&2; exit 1; }
}
require docker
require kind
require kubectl
require curl

echo "[1/7] kind create cluster (${CLUSTER_NAME})"
if ! kind get clusters 2>/dev/null | grep -qx "${CLUSTER_NAME}"; then
  kind create cluster --name "${CLUSTER_NAME}" --config "${ROOT}/deploy/e2e/kind-cluster.yaml"
fi

if [ "${SKIP_BUILD}" = "1" ]; then
  echo "[2/7] docker build — skipped (SKIP_BUILD=1, using prebuilt ${IMAGE})"
  docker image inspect "${IMAGE}" >/dev/null 2>&1 || {
    echo "SKIP_BUILD=1 but ${IMAGE} is not in the local docker image store" >&2
    exit 1
  }
else
  echo "[2/7] docker build → ${IMAGE}"
  docker build -t "${IMAGE}" "${ROOT}"
fi

echo "[3/7] kind load docker-image"
kind load docker-image "${IMAGE}" --name "${CLUSTER_NAME}"

echo "[4/7] kubectl apply -k (e2e overlay)"
kubectl apply -k "${ROOT}/deploy/e2e/"

echo "[5/7] wait for rollout (API + worker)"
kubectl rollout status deployment/featuredoc --timeout=180s
# Worth waiting on even though the overlay holds the worker at 0 replicas: a rollout
# that reports complete still proves the Deployment itself applied cleanly.
kubectl rollout status deployment/featuredoc-worker --timeout=180s

echo "[6/7] port-forward svc/featuredoc ${LOCAL_PORT}:8080 (supervised)"
: >"${PF_LOG}"
supervise_port_forward &
PF_PID=$!
for _ in $(seq 1 30); do
  if curl -fsS "http://localhost:${LOCAL_PORT}/hello" >/dev/null 2>&1; then
    break
  fi
  sleep 1
done

echo "[7/7] run e2e (smoke + playwright)"
BASE_URL="http://localhost:${LOCAL_PORT}" bash "${ROOT}/e2e/smoke.sh"
(
  cd "${ROOT}/e2e"
  if [ "${SKIP_PLAYWRIGHT_INSTALL}" != "1" ]; then
    if [ ! -d node_modules ]; then npm install; fi
    npx playwright install --with-deps chromium >/dev/null
  fi
  BASE_URL="http://localhost:${LOCAL_PORT}" npm test
)

echo "all green."
