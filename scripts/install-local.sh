#!/usr/bin/env bash
# Install for the current Linux user. No administrator privileges or keys needed.
set -euo pipefail

if [[ "$(uname -s)" == "Darwin" ]]; then
    exec bash "$(dirname -- "${BASH_SOURCE[0]}")/install-macos.sh" "$@"
fi
if [[ "$(uname -s)" != "Linux" ]]; then
    echo "Fenix currently supports Linux. / Fenix requiere Linux." >&2
    exit 1
fi
for dependency in cargo pnpm systemctl xdg-open; do
    if ! command -v "$dependency" >/dev/null 2>&1; then
        echo "Missing dependency / Dependencia faltante: $dependency" >&2
        exit 1
    fi
done
if ! systemctl --user show-environment >/dev/null 2>&1; then
    echo "A systemd user session is required. You can still run cargo run -p fenix-server." >&2
    exit 1
fi

fenix_repository="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
fenix_binary_directory="$HOME/.local/bin"
fenix_install_directory="$HOME/.local/share/fenix"
fenix_unit_directory="$HOME/.config/systemd/user"
fenix_application_directory="$HOME/.local/share/applications"

cd -- "$fenix_repository"
pnpm --dir apps/web install --frozen-lockfile
pnpm --dir apps/web build
cargo build --release --locked -p fenix-server --bin fenix
install -d -m 700 "$fenix_install_directory"
install -d "$fenix_binary_directory" "$fenix_unit_directory" "$fenix_application_directory"

if systemctl --user is-active --quiet fenix.service; then
    systemctl --user stop fenix.service
fi
install -d "$fenix_install_directory/web"
cp -a apps/web/dist/. "$fenix_install_directory/web/"
install -m 755 target/release/fenix "$fenix_binary_directory/fenix"
install -m 644 packaging/linux/fenix.service "$fenix_unit_directory/fenix.service"
install -m 644 packaging/linux/fenix.desktop "$fenix_application_directory/fenix.desktop"
systemctl --user daemon-reload
systemctl --user enable --now fenix.service

echo "Fenix installed. Open Fénix from your application menu or http://127.0.0.1:4173"
echo "Fenix instalado. Abre Fénix desde el menú de aplicaciones o http://127.0.0.1:4173"
echo "Data / Datos: $fenix_install_directory/.fenix"
