#!/usr/bin/env bash
# Publish already-built documentation without restarting the agent.
set -euo pipefail
repository_directory="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
docs_directory="${FENIX_DOCS_DIRECTORY:-$HOME/.local/share/fenix-docs}"
release_name="$(date -u +%Y%m%dT%H%M%SZ)-$$"
release_directory="$docs_directory/releases/$release_name"
if [[ ! -f "$repository_directory/apps/docs/dist/index.html" ]]; then
    echo "Build documentation first: pnpm --dir apps/docs build" >&2
    exit 1
fi
install -d "$release_directory"
cp -a "$repository_directory/apps/docs/dist/." "$release_directory/"
ln -s "$release_directory" "$docs_directory/.current-$release_name"
mv -Tf "$docs_directory/.current-$release_name" "$docs_directory/current"
printf 'Published documentation: %s\n' "$release_directory"
