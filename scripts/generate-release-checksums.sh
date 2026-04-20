#!/usr/bin/env bash
set -euo pipefail

if [[ $# -eq 0 ]]; then
  echo "usage: generate-release-checksums.sh <artifact-path> [...]" >&2
  exit 1
fi

artifacts=()
for artifact in "$@"; do
  if [[ ! -f "$artifact" ]]; then
    echo "artifact not found: $artifact" >&2
    exit 1
  fi
  artifacts+=("$artifact")
done

checksum_dir=$(cd "$(dirname "${artifacts[0]}")" && pwd)
checksum_file="$checksum_dir/SHA256SUMS"
: > "$checksum_file"

for artifact in "${artifacts[@]}"; do
  artifact_dir=$(cd "$(dirname "$artifact")" && pwd)
  if [[ "$artifact_dir" != "$checksum_dir" ]]; then
    echo "all artifacts must live in the same directory" >&2
    exit 1
  fi

  artifact_name=$(basename "$artifact")
  checksum=$(shasum -a 256 "$artifact" | awk '{print $1}')
  printf '%s  %s\n' "$checksum" "$artifact_name" | tee -a "$checksum_file" > "$artifact.sha256"
done

echo "Created $checksum_file"
