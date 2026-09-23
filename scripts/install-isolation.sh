#!/bin/bash
# Install files only; configuration, account migration and activation are explicit.
set -euo pipefail
[[ $(id -u) == 0 ]] || { echo 'Run this installer as root.' >&2; exit 1; }
if systemctl is-active --quiet fenix-execution.service; then
    echo 'Stop the execution service and its controllers before replacing installed code.' >&2
    exit 1
fi
fenix_source=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
for command in docker systemctl iptables iptables-restore mount umount mkfs.ext4; do
    command -v "$command" >/dev/null || { echo "Missing dependency: $command" >&2; exit 1; }
done
/usr/bin/python3 -c 'from cryptography.hazmat.primitives.ciphers.aead import AESGCM'
install -d -m 0755 /opt/fenix/isolation /opt/fenix/isolation/fenix_execution
install -d -m 0700 /etc/fenix /var/lib/fenix /var/lib/fenix/users /var/lib/fenix/volumes /var/lib/fenix-execution
for file in execution-service.py fenix_isolation.py tenant-container.py tenant-network-guard.py tenant-storage.py tenant-preflight.py sandbox-gate.py prepare-gvisor.py gvisor-preflight.py microvm-gate.py prepare-microvm.py microvm-preflight.py; do
    install -m 0644 "$fenix_source/scripts/$file" "/opt/fenix/isolation/$file"
done
for file in "$fenix_source"/scripts/fenix_execution/*.py; do
    install -m 0644 "$file" /opt/fenix/isolation/fenix_execution/
done
install -m 0644 "$fenix_source/packaging/linux/seccomp-browser.json" /opt/fenix/isolation/
install -m 0644 "$fenix_source/packaging/linux/LICENSE.moby" /opt/fenix/isolation/
# Retained for explicitly selected runc browser execution. gVisor does not need
# the host's Chromium-specific AppArmor profile or apparmor_parser to be present.
install -d -m 0755 /etc/apparmor.d
install -m 0644 "$fenix_source/packaging/linux/fenix-browser.apparmor" /etc/apparmor.d/fenix-browser-v1
install -m 0644 "$fenix_source/packaging/linux/fenix-execution.service" "$fenix_source/packaging/linux/fenix-controller@.service" /etc/systemd/system/
install -m 0600 "$fenix_source/packaging/linux/execution.example.json" "$fenix_source/packaging/linux/execution.microvm.example.json" "$fenix_source/packaging/linux/integrations.example.json" /etc/fenix/
printf '%s\n' 'Files installed. Prepare gVisor using docs/en/self-hosted-gvisor.md and review /etc/fenix/*.example.json before activation.'
printf '%s\n' 'No service, existing account, volume or AppArmor profile has been activated or restarted.'
