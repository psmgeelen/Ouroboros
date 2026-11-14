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

# Long body lines must be accepted (Cursor commit UI does not wrap bullets).
cat <<'EOF' | npx --no-install commitlint
chore: update CI/CD configuration and improve security policies

- Implemented advanced security policies for the relay service, including OWASP rules for SQL injection and other vulnerabilities.
EOF

echo "commitlint fixture passed"
