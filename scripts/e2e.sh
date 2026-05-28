#!/usr/bin/env bash
# End-to-end: build → kind load → apply → port-forward → smoke + playwright → cleanup.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
CLUSTER_NAME="${CLUSTER_NAME:-featuredoc}"
IMAGE="${IMAGE:-featuredoc-hello:dev}"
KEEP_CLUSTER="${KEEP_CLUSTER:-0}"
LOCAL_PORT="${LOCAL_PORT:-8080}"

PF_PID=""

cleanup() {
  if [ -n "${PF_PID}" ] && kill -0 "${PF_PID}" 2>/dev/null; then
    echo "[cleanup] stop port-forward (pid ${PF_PID})"
    kill "${PF_PID}" 2>/dev/null || true
    wait "${PF_PID}" 2>/dev/null || true
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
  kind create cluster --name "${CLUSTER_NAME}" --config "${ROOT}/deploy/kind-cluster.yaml"
fi

echo "[2/7] docker build → ${IMAGE}"
docker build -t "${IMAGE}" "${ROOT}"

echo "[3/7] kind load docker-image"
kind load docker-image "${IMAGE}" --name "${CLUSTER_NAME}"

echo "[4/7] kubectl apply"
kubectl apply -f "${ROOT}/deploy/deployment.yaml"
kubectl apply -f "${ROOT}/deploy/service.yaml"

echo "[5/7] wait for rollout"
kubectl rollout status deployment/featuredoc-hello --timeout=180s

echo "[6/7] port-forward svc/featuredoc-hello ${LOCAL_PORT}:8080"
kubectl port-forward svc/featuredoc-hello "${LOCAL_PORT}:8080" >/tmp/featuredoc-pf.log 2>&1 &
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
  if [ ! -d node_modules ]; then npm install; fi
  npx playwright install --with-deps chromium >/dev/null
  BASE_URL="http://localhost:${LOCAL_PORT}" npm test
)

echo "all green."
