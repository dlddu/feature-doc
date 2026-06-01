#!/usr/bin/env bash
# End-to-end: build → kind load → apply → port-forward → smoke + playwright → cleanup.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
CLUSTER_NAME="${CLUSTER_NAME:-featuredoc}"
IMAGE="${IMAGE:-featuredoc:dev}"
MOCK_IMAGE="${MOCK_IMAGE:-featuredoc-mock:dev}"
KEEP_CLUSTER="${KEEP_CLUSTER:-0}"
LOCAL_PORT="${LOCAL_PORT:-8080}"
MOCK_PORT="${MOCK_PORT:-8081}"

PF_PID=""
MOCK_PF_PID=""

cleanup() {
  for pid in "${PF_PID}" "${MOCK_PF_PID}"; do
    if [ -n "${pid}" ] && kill -0 "${pid}" 2>/dev/null; then
      echo "[cleanup] stop port-forward (pid ${pid})"
      kill "${pid}" 2>/dev/null || true
      wait "${pid}" 2>/dev/null || true
    fi
  done
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

echo "[2/7] docker build → ${IMAGE} (+ mock ${MOCK_IMAGE})"
docker build -t "${IMAGE}" "${ROOT}"
docker build -t "${MOCK_IMAGE}" -f "${ROOT}/deploy/e2e/mock-github.Dockerfile" "${ROOT}"

echo "[3/7] kind load docker-image (app + mock)"
kind load docker-image "${IMAGE}" --name "${CLUSTER_NAME}"
kind load docker-image "${MOCK_IMAGE}" --name "${CLUSTER_NAME}"

echo "[4/7] kubectl apply -k (e2e overlay)"
kubectl apply -k "${ROOT}/deploy/e2e/"

echo "[5/7] wait for rollout"
kubectl rollout status deployment/mock-github --timeout=180s
kubectl rollout status deployment/featuredoc --timeout=180s

echo "[6/7] port-forward svc/featuredoc ${LOCAL_PORT}:8080 + svc/mock-github ${MOCK_PORT}:80"
kubectl port-forward svc/featuredoc "${LOCAL_PORT}:8080" >/tmp/featuredoc-pf.log 2>&1 &
PF_PID=$!
kubectl port-forward svc/mock-github "${MOCK_PORT}:80" >/tmp/mock-github-pf.log 2>&1 &
MOCK_PF_PID=$!
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
