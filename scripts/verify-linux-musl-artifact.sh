#!/usr/bin/env bash
set -euo pipefail

target=${1:?usage: verify-linux-musl-artifact.sh <target-triple> <artifact-path>}
artifact=${2:?usage: verify-linux-musl-artifact.sh <target-triple> <artifact-path>}

if [[ ! -f "$artifact" ]]; then
  echo "artifact not found: $artifact" >&2
  exit 1
fi

file_output=$(file "$artifact")
echo "$file_output"

case "$target" in
  x86_64-unknown-linux-musl)
    [[ "$file_output" == *"x86-64"* ]] || {
      echo "expected an x86-64 binary for $target" >&2
      exit 1
    }
    ;;
  aarch64-unknown-linux-musl)
    [[ "$file_output" == *"ARM aarch64"* ]] || {
      echo "expected an ARM aarch64 binary for $target" >&2
      exit 1
    }
    ;;
  *)
    echo "unsupported target triple: $target" >&2
    exit 1
    ;;
esac

[[ "$file_output" == *"ELF 64-bit"* ]] || {
  echo "expected an ELF artifact for $target" >&2
  exit 1
}

[[ "$file_output" == *"statically linked"* ]] || {
  echo "expected a statically linked musl artifact for $target" >&2
  exit 1
}

[[ "$file_output" != *"dynamically linked"* ]] || {
  echo "expected no dynamic linker dependency for $target" >&2
  exit 1
}
