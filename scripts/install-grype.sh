#!/bin/sh
# Install a pinned Grype binary without calling the GitHub releases API listing.
set -eu
version=0.121.3
os=$(uname -s | tr '[:upper:]' '[:lower:]')
arch=$(uname -m)
case "$arch" in
  x86_64) arch=amd64 ;;
  aarch64 | arm64) arch=arm64 ;;
esac
dest="${HOME}/.local/bin"
tmp=$(mktemp -d)
url="https://github.com/anchore/grype/releases/download/v${version}/grype_${version}_${os}_${arch}.tar.gz"
curl -fsSL "$url" -o "${tmp}/grype.tar.gz"
tar -xzf "${tmp}/grype.tar.gz" -C "$tmp" grype
mkdir -p "$dest"
mv "${tmp}/grype" "${dest}/grype"
chmod +x "${dest}/grype"
rm -rf "$tmp"
if [ -n "${GITHUB_PATH:-}" ]; then
  echo "$dest" >> "$GITHUB_PATH"
fi
echo "installed ${dest}/grype v${version}"
