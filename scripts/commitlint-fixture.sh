#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$root"

if [ ! -d node_modules/@commitlint ]; then
  npm install
fi

echo "update stuff" | npx --no-install commitlint && {
  echo "commitlint accepted an invalid message" >&2
  exit 1
} || true

# commitlint exits non-zero for invalid messages. Confirm that happened.
if echo "update stuff" | npx --no-install commitlint; then
  echo "commitlint accepted an invalid message" >&2
  exit 1
fi

echo "chore: scaffold the ouroboros repository" | npx --no-install commitlint
echo "commitlint fixture passed"
