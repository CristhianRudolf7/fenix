#!/usr/bin/env bash
# Build and install the personal application as a user LaunchAgent.
set -euo pipefail
[[ "$(uname -s)" == Darwin ]] || { echo 'This installer requires macOS.' >&2; exit 1; }
for dependency in cargo pnpm python3 launchctl; do
  command -v "$dependency" >/dev/null || { echo "Missing dependency: $dependency" >&2; exit 1; }
done
repository_directory="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
install_directory="$HOME/Library/Application Support/Fenix"
data_directory="$install_directory/data"
plist_file="$HOME/Library/LaunchAgents/com.crehlabs.fenix.plist"
cd "$repository_directory"
pnpm --dir apps/web install --frozen-lockfile
pnpm --dir apps/web build
cargo build --release --locked -p fenix-server --bin fenix
mkdir -p "$install_directory/bin" "$install_directory/web" "$install_directory/logs" "$data_directory" "$HOME/Library/LaunchAgents"
chmod 700 "$install_directory" "$data_directory" "$install_directory/logs"
launchctl bootout "gui/$(id -u)/com.crehlabs.fenix" 2>/dev/null || true
install -m 755 target/release/fenix "$install_directory/bin/fenix"
cp -R apps/web/dist/. "$install_directory/web/"
python3 - "$install_directory" "$plist_file" "$PATH" <<'PY'
import os, pathlib, plistlib, sys
root, destination, search_path = sys.argv[1:]
root = pathlib.Path(root)
job = {
    'Label': 'com.crehlabs.fenix',
    'ProgramArguments': [str(root/'bin/fenix'), '--data-dir', str(root/'data')],
    'WorkingDirectory': str(root),
    'EnvironmentVariables': {'FENIX_WEB_DIST':str(root/'web'), 'PATH':search_path},
    'RunAtLoad':True, 'KeepAlive':{'SuccessfulExit':False},
    'ThrottleInterval':5, 'ExitTimeOut':20,
    'StandardOutPath':str(root/'logs/server.log'),
    'StandardErrorPath':str(root/'logs/server-error.log'),
}
for name in ('FENIX_BROWSER_EXECUTABLE', 'FENIX_OCR_PYTHON', 'FENIX_SHELL'):
    if os.environ.get(name): job['EnvironmentVariables'][name] = os.environ[name]
with open(destination, 'wb') as file: plistlib.dump(job, file)
PY
chmod 600 "$plist_file"
plutil -lint "$plist_file"
launchctl bootstrap "gui/$(id -u)" "$plist_file"
echo "Installed at $install_directory. Open http://127.0.0.1:4173"
printf 'Configure the owner if needed: "%s/bin/fenix" owner EMAIL --password-stdin --data-dir "%s"\n' "$install_directory" "$data_directory"
echo 'Browser and PDF/OCR dependencies are optional and installed separately.'
