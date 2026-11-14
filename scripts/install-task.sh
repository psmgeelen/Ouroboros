#!/bin/sh
# Install a pinned Task binary without calling the GitHub releases API.
set -eu
version=v3.44.0
os=$(uname -s | tr '[:upper:]' '[:lower:]')
arch=$(uname -m)
case "$arch" in
  x86_64) arch=amd64 ;;
  aarch64 | arm64) arch=arm64 ;;
esac
dest="${HOME}/.local/bin"
tmp=$(mktemp -d)
url="https://github.com/go-task/task/releases/download/${version}/task_${os}_${arch}.tar.gz"
curl -fsSL "$url" -o "${tmp}/task.tar.gz"
tar -xzf "${tmp}/task.tar.gz" -C "$tmp" task
mkdir -p "$dest"
mv "${tmp}/task" "${dest}/task"
chmod +x "${dest}/task"
rm -rf "$tmp"
if [ -n "${GITHUB_PATH:-}" ]; then
  echo "$dest" >> "$GITHUB_PATH"
fi
echo "installed ${dest}/task ${version}"
