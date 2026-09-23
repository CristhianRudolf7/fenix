#!/usr/bin/env bash
# Publish a built frontend without compiling Rust or restarting the agent.
set -euo pipefail
fenix_dist="${1:-apps/web/dist}"
fenix_web="${2:-$HOME/.local/share/fenix-web}"
[[ -f "$fenix_dist/index.html" && -d "$fenix_dist/assets" ]] || {
    echo "Build the frontend first with pnpm --dir apps/web build." >&2; exit 1;
}
mkdir -p "$fenix_web/releases"
fenix_web="$(cd -- "$fenix_web" && pwd)"
fenix_release="$(mktemp -d "$fenix_web/releases/$(date -u +%Y%m%dT%H%M%SZ)-XXXXXX")"
cp -a "$fenix_dist/." "$fenix_release/"
# Keep old hashed chunks accessible to tabs opened before the update.
if [[ -d "$fenix_web/current/assets" ]]; then
    cp -an "$fenix_web/current/assets/." "$fenix_release/assets/"
fi
fenix_link="$fenix_web/.current-$$"
trap 'rm -f "$fenix_link"' EXIT
ln -s "$fenix_release" "$fenix_link"
mv -Tf "$fenix_link" "$fenix_web/current"
echo "Published frontend: $fenix_release"
