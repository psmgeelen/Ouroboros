#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
dir="$root/apps/mobile/lib/partner"
if [ ! -d "$dir" ]; then
  exit 0
fi
if grep -RInE "void export|exportData|Share\.|Clipboard\." "$dir"; then
  echo "partner library must not export, share, or copy to the clipboard" >&2
  exit 1
fi
echo "partner export ban passed"
