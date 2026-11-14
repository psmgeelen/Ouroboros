#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
file="$root/.github/workflows/ci.yml"
if [ ! -f "$file" ]; then
  exit 0
fi
if grep -n "terraspace up" "$file"; then
  echo "ci.yml must not deploy with terraspace up" >&2
  exit 1
fi
if grep -n "docker push" "$file"; then
  echo "ci.yml must not docker push" >&2
  exit 1
fi
echo "ci.yml deploy guard passed"
