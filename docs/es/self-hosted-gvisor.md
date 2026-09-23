# Espacios de trabajo con gVisor en servidores propios

[English: referencia completa](../en/self-hosted-gvisor.md) · [Política de ejecución](execution-isolation.md)

Fénix utiliza por defecto **gVisor con `systrap`** en el servicio de ejecución multiusuario. No necesita `/dev/kvm` ni virtualización anidada. Puede funcionar en un servidor Linux compatible, incluida una VM habitual de la nube, sin cuenta ni suscripción de E2B. El servidor, almacenamiento, actualizaciones y respaldos siguen a cargo del operador.

gVisor proporciona un kernel de aplicación aislado; no es una máquina virtual de hardware. El controlador conserva credenciales, conversaciones y programación. Los comandos, terminales, scripts de skills, servidores de lenguaje y Chromium se ejecutan en sandboxes separados. Las operaciones nativas de archivos/documentos siguen en el controlador confiable de Fénix.

## Configuración y funcionamiento

El ejemplo principal `packaging/linux/execution.example.json` selecciona `backend: gvisor`, `runtime: fenix-runsc` y un manifiesto privado `gvisor_manifest`. Omitir `backend` ahora selecciona gVisor. Las instalaciones que conserven intencionadamente el modo anterior deben declarar `backend: oci` y `runtime: runc`.

Kata permanece como opción explícita `backend: microvm` para hosts con KVM. No es la opción predeterminada. Si gVisor falta, cambia o no supera las comprobaciones, se rechaza la ejecución; no hay cambio automático a runc, Kata ni ejecución en el host.

Cada ejecución recibe un sandbox nuevo. Los proyectos y las herramientas instaladas en el directorio privado de usuario sobreviven a su eliminación. Se conservan virtualenvs, prefijos npm, directorios Cargo y cachés. Los paquetes del sistema pertenecen a la imagen administrada por el operador; no se guarda una sesión de root ni un sistema operativo mutable completo.

Los montajes se mantienen compartidos para que las herramientas nativas de Fénix y el sandbox vean los cambios de archivos. Los recursos de solo lectura se protegen también en el host. El servidor de lenguaje recibe proyecto y herramientas de solo lectura, sin red; el navegador recibe su perfil y descargas. No se exponen secretos del controlador, sockets administrativos ni carpetas ajenas.

## Comprobaciones de seguridad

La preparación fija hashes de `runsc`, todos sus binarios auxiliares en `gvisor-bin/` y el programa de arranque. La versión candidata comprobada es `release-20260914.0`. Se exige la distribución completa; las versiones actuales no deben instalarse copiando únicamente `runsc`. Las sumas oficiales deben verificarse antes: el manifiesto identifica los archivos locales, no demuestra su procedencia.

La configuración exige systrap y la red aislada de gVisor. Desactiva acceso directo a archivos desde Sentry, sockets y FIFOs del host, dispositivos privilegiados, escrituras de paquetes crudos, depuración y cambios de opciones mediante anotaciones. Un proceso separado, Gofer, media el acceso a archivos. Se activa explícitamente el seccomp de la aplicación además de la protección propia del runtime.

Antes de liberar el comando, el servicio comprueba el registro de Docker, el marcador de gVisor, el ejecutable real de Sentry, su cgroup exacto y el seccomp del host. Después vuelve a comprobar autorización y plazo. Son comprobaciones de despliegue, no atestación criptográfica.

Los límites de CPU, memoria y procesos se aplican al cgroup real del host. Los cgroups visibles dentro de gVisor son virtuales. La memoria configurada incluye al runtime; la admisión añade una reserva conservadora de 128 MiB por ejecución. Se mantienen las cuotas por cuenta, disco, concurrencia y tiempo mensual. Los límites entre procesos dentro de un mismo sandbox no equivalen a contenedores host independientes.

La revocación y cancelación eliminan la ejecución completa. Si falla Docker, se puede detener su cgroup registrado y conservar la limpieza como pendiente. Continúan las restricciones de red, los túneles autenticados y la ausencia de puertos CDP públicos. Chromium conserva su sandbox interno; su perfil seccomp se ejecuta dentro de gVisor. El perfil AppArmor específico de runc no se aplica como si Sentry fuera Chromium.

La versión probada traduce incorrectamente un código de error de seccomp que glibc necesita para crear hilos. El programa confiable de arranque añade un filtro BPF que mantiene `clone3` bloqueado con ENOSYS, comprueba el resultado y conserva los demás filtros. Los relés internos reciben el mismo ajuste. Las pruebas comprueban hilos funcionales y la denegación de `clone3` y de espacios de nombres no autorizados. No se desactiva seccomp para resolver la incompatibilidad; revisar este ajuste al actualizar gVisor.

## Instalación y pruebas

Se requiere Linux 5.6+ en x86_64 o aarch64, systemd, Docker con cgroup v2/controlador systemd y las dependencias existentes de cortafuegos, almacenamiento y Python. Instalar primero una distribución completa y verificada de gVisor, conservando `gvisor-bin/` junto a `runsc`. Los binarios deben pertenecer a root y permitir lectura/ejecución a los procesos no privilegiados del runtime.

```sh
sudo scripts/install-isolation.sh
sudo python3 /opt/fenix/isolation/prepare-gvisor.py \
  --runsc /usr/local/bin/runsc --register-runtime
sudo systemctl reload docker
sudo python3 /opt/fenix/isolation/gvisor-preflight.py
```

La preparación no descarga binarios ni reinicia Docker. Conserva los demás ajustes y el runtime predeterminado, valida el JSON y respalda el anterior. Sin `--register-runtime`, escribe únicamente un manifiesto y un fragmento de configuración. No sobrescribe un manifiesto existente: detener las ejecuciones y preparar las actualizaciones explícitamente.

Adaptar el ejemplo de ejecución con identidades, cuotas y digest de imagen reales, siguiendo la [migración de cuentas](execution-isolation.md#instalación-y-migración). Actualizar el repositorio no modifica servicios activos ni migra usuarios.

```sh
python3 -m unittest discover -s tests/isolation -v
sudo env FENIX_GVISOR_TESTS=1 \
  FENIX_GVISOR_MANIFEST=/etc/fenix/gvisor/manifest.json \
  FENIX_TEST_IMAGE=fenix-tenant:local \
  python3 -m unittest discover -s tests/isolation -p test_gvisor_acceptance.py -v
```

La segunda suite ejecuta gVisor real con cuentas sintéticas y comprueba límites del host, permisos, persistencia, coherencia de archivos, terminal, transporte Rust, red, revocación, limpieza y navegador. Si se solicita y falta el runtime, falla. Debe repetirse en el servidor de despliegue y después de actualizar imágenes o runtime.

El [registro de validación](../research/self-hosted-gvisor-validation-2026-09-20.json) recoge 45 pruebas de política, 15 casos reales de gVisor y una regresión OCI superadas. Se probaron también Node.js con hilos y compilación C. Este host x86_64 no tiene `/dev/kvm`. Al terminar se retiraron el runtime temporal y su registro en Docker, restaurando la configuración anterior; no se migraron cuentas reales. El registro incluye fallos iniciales, correcciones, hashes y límites de cobertura.

La compatibilidad con aplicaciones concretas y la capacidad bajo carga requieren pruebas propias. Validar por separado la imagen de LibreOffice y multimedia. No se incorporan snapshots de RAM, restauración de procesos suspendidos, GPU, escritorios Windows ni almacenamiento replicado. Los archivos persistentes siguen siendo locales y necesitan respaldos.
