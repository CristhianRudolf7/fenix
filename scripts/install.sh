#!/usr/bin/env bash
# Install a published, prebuilt personal Linux application without build tools.
set -euo pipefail
umask 077
fail() { printf 'Fénix: %s\n' "$*" >&2; exit 1; }
case "${1:-}" in
    --help|-h) printf '%s\n' 'Usage: bash install.sh [--help]' 'Optional: FENIX_VERSION=vX.Y.Z FENIX_PORT=4173 FENIX_NO_OPEN=1' 'Requires Linux x86_64, glibc >= 2.39 and a systemd user session.'; exit 0 ;;
    '') ;;
    *) fail 'Unknown argument. Use --help.' ;;
esac
[[ $(uname -s) == Linux ]] || fail 'Actualmente se admite Linux.'
[[ $(uname -m) == x86_64 ]] || fail 'Actualmente se admite x86_64; no se instalará un binario incompatible.'
for tool in curl tar sha256sum systemctl getconf sort head awk grep mktemp install mv cp chmod readlink date mkdir ln sleep rm; do
    command -v "$tool" >/dev/null || fail "Falta la dependencia: $tool"
done
libc=$(getconf GNU_LIBC_VERSION 2>/dev/null) || fail 'Se requiere glibc; Alpine/musl no está soportado.'
[[ $libc == glibc\ * ]] || fail 'Se requiere glibc.'
[[ $(printf '%s\n' 2.39 "${libc#glibc }" | sort -V | head -1) == 2.39 ]] || fail 'Se requiere glibc 2.39 o posterior.'
systemctl --user show-environment >/dev/null 2>&1 || fail 'Se necesita una sesión systemd de usuario (sin sudo).'
port=${FENIX_PORT:-4173}
[[ $port =~ ^[0-9]{1,5}$ ]] && ((10#$port > 1023 && 10#$port < 65536)) || fail 'Puerto inválido; usa 1024–65535.'
version=${FENIX_VERSION:-latest}
[[ $version == latest || $version =~ ^v[0-9]+\.[0-9]+\.[0-9]+([.-][a-zA-Z0-9.-]+)?$ ]] || fail 'Versión inválida.'
base=https://github.com/CristhianRudolf7/fenix/releases
if [[ $version == latest ]]; then base+=/latest/download; else base+=/download/$version; fi
asset=fenix-linux-x86_64.tar.gz
work=$(mktemp -d)
trap 'rm -rf -- "$work"' EXIT
fetch() { curl --fail --silent --show-error --location --proto '=https' --proto-redir '=https' --tlsv1.2 --connect-timeout 15 --max-time 600 --retry 2 "$1" -o "$2"; }
printf '%s\n' 'Descargando Fénix…'
fetch "$base/$asset" "$work/$asset" || fail 'No existe un paquete compatible en esta versión publicada; no se modificó la instalación.'
fetch "$base/$asset.sha256" "$work/checksum" || fail 'No se pudo obtener la suma de verificación.'
read -r digest filename extra < "$work/checksum"
[[ $digest =~ ^[0-9a-fA-F]{64}$ && $filename == "$asset" && -z ${extra:-} ]] || fail 'Formato de suma de verificación inválido.'
printf '%s  %s\n' "$digest" "$asset" > "$work/verified.sha256"
(cd "$work" && sha256sum --check verified.sha256) || fail 'El paquete no superó la verificación de integridad.'
# Never extract links, special files, absolute paths or parent traversals.
while IFS= read -r member; do
    [[ $member == VERSION || $member == fenix || $member == web || $member == web/ || $member == web/* ]] || fail 'Ruta inesperada en el paquete.'
    [[ /$member/ != *'/../'* && /$member/ != *'/./'* ]] || fail 'Ruta insegura en el paquete.'
done < <(tar -tzf "$work/$asset")
if tar -tvzf "$work/$asset" | awk 'substr($0,1,1)!="-" && substr($0,1,1)!="d" {bad=1} END {exit !bad}'; then fail 'El paquete contiene enlaces o archivos especiales.'; fi
mkdir "$work/package"
tar -xzf "$work/$asset" -C "$work/package" --no-same-owner --no-same-permissions
[[ -f $work/package/fenix && -f $work/package/web/index.html && -f $work/package/VERSION ]] || fail 'Paquete incompleto.'
release_version=$(<"$work/package/VERSION")
[[ $release_version =~ ^[0-9]+\.[0-9]+\.[0-9]+([.-][a-zA-Z0-9.-]+)?$ ]] || fail 'Versión del paquete inválida.'
[[ $version == latest || $version == "v$release_version" ]] || fail 'La versión descargada no corresponde a la solicitada.'
chmod 700 "$work/package/fenix"
"$work/package/fenix" --help >/dev/null || fail 'El binario no es compatible con este sistema.'
root=$HOME/.local/share/fenix-personal
units=$HOME/.config/systemd/user
unit=$units/fenix-personal.service
# The personal installer never replaces the hosted deployment or its units.
if [[ -e $unit ]] && ! grep -q '# Managed by the Fenix public installer' "$unit"; then fail 'Ya existe una unidad no administrada por este instalador.'; fi
if command -v ss >/dev/null && ss -H -ltn "sport = :$port" | grep -q .; then
    if ! systemctl --user is-active --quiet fenix-personal.service || ! grep -q -- "--port $port " "$unit"; then
        fail "El puerto $port está ocupado; selecciona FENIX_PORT."
    fi
fi
install -d -m 700 "$root/releases" "$root/data" "$root/skills" "$root/config" "$units"
release=$root/releases/$(date -u +%Y%m%dT%H%M%SZ)-$$
mv "$work/package" "$release"
previous=$(readlink "$root/current" || true)
was_active=0
was_enabled=0
systemctl --user is-enabled --quiet fenix-personal.service && was_enabled=1
systemctl --user is-active --quiet fenix-personal.service && was_active=1
[[ ! -e $unit ]] || cp "$unit" "$work/previous.service"
ln -s "$release" "$root/.next-$$"
mv -Tf "$root/.next-$$" "$root/current"
cat > "$unit" <<UNIT
# Managed by the Fenix public installer
[Unit]
Description=Fenix personal application
After=network.target
[Service]
Type=simple
ExecStart=%h/.local/share/fenix-personal/current/fenix --port $port --data-dir %h/.local/share/fenix-personal/data --skills-dir %h/.local/share/fenix-personal/skills
WorkingDirectory=%h/.local/share/fenix-personal
Environment=FENIX_WEB_DIST=%h/.local/share/fenix-personal/current/web
Environment=TZ=America/Lima
EnvironmentFile=-%h/.local/share/fenix-personal/config/server.env
UMask=0077
Restart=on-failure
RestartSec=5
[Install]
WantedBy=default.target
UNIT
rollback() {
    systemctl --user stop fenix-personal.service || true
    if [[ -n $previous ]]; then ln -s "$previous" "$root/.rollback-$$"; mv -Tf "$root/.rollback-$$" "$root/current"; else rm -f "$root/current"; fi
    if [[ -f $work/previous.service ]]; then cp "$work/previous.service" "$unit"; else rm -f "$unit"; systemctl --user disable fenix-personal.service 2>/dev/null || true; fi
    systemctl --user daemon-reload || true
    if ((!was_enabled)); then systemctl --user disable fenix-personal.service 2>/dev/null || true; fi
    if ((was_active)); then systemctl --user start fenix-personal.service || true; fi
    fail 'No se pudo iniciar Fénix. Se restauró la instalación anterior; los datos se conservaron.'
}
systemctl --user daemon-reload || rollback
systemctl --user enable fenix-personal.service || rollback
systemctl --user restart fenix-personal.service || rollback
ready=0
for attempt in {1..30}; do
    if curl --fail --silent --max-time 2 "http://127.0.0.1:$port/" >/dev/null; then ready=1; break; fi
    systemctl --user is-active --quiet fenix-personal.service || break
    sleep 1
done
((ready)) || rollback
printf 'Fénix %s instalado: http://127.0.0.1:%s\nDatos: %s/data\n' "$release_version" "$port" "$root"
printf '%s\n' 'Servicio: systemctl --user status fenix-personal.service' 'El navegador, aislamiento y otras dependencias opcionales requieren configuración adicional.'
if [[ ${FENIX_NO_OPEN:-0} != 1 ]] && command -v xdg-open >/dev/null && [[ -n ${DISPLAY:-}${WAYLAND_DISPLAY:-} ]]; then
    xdg-open "http://127.0.0.1:$port/" >/dev/null 2>&1 || true
fi
