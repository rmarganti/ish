#!/usr/bin/env bash
set -euo pipefail

platform=${1:?usage: smoke-test-alpine.sh <docker-platform> <artifact-path> [version-args...]}
artifact=${2:?usage: smoke-test-alpine.sh <docker-platform> <artifact-path> [version-args...]}
shift 2
args=("$@")
if [[ ${#args[@]} -eq 0 ]]; then
  args=(version)
fi

if ! command -v docker >/dev/null 2>&1; then
  echo "docker is required for the Alpine smoke test" >&2
  exit 1
fi

if [[ ! -f "$artifact" ]]; then
  echo "artifact not found: $artifact" >&2
  exit 1
fi

artifact_dir=$(cd "$(dirname "$artifact")" && pwd)
artifact_name=$(basename "$artifact")

docker run --rm \
  --platform "$platform" \
  -v "$artifact_dir:/artifacts:ro" \
  alpine:3.20 \
  "/artifacts/$artifact_name" "${args[@]}"
