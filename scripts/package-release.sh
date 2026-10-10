#!/usr/bin/env bash
# Package only freshly built artifacts. Run from release CI after compilation.
set -euo pipefail
repository=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
output=${1:?Usage: package-release.sh OUTPUT_DIRECTORY}
mkdir -p "$output"
output=$(cd "$output" && pwd)
[[ -x $repository/target/release/fenix && -f $repository/apps/web/dist/index.html ]] || { echo 'Build the Rust application and frontend first.' >&2; exit 1; }
work=$(mktemp -d)
trap 'rm -rf -- "$work"' EXIT
install -m 755 "$repository/target/release/fenix" "$work/fenix"
cp -a "$repository/apps/web/dist" "$work/web"
tar -czf "$output/fenix-linux-x86_64.tar.gz" -C "$work" fenix web
(cd "$output" && sha256sum fenix-linux-x86_64.tar.gz > fenix-linux-x86_64.tar.gz.sha256)
