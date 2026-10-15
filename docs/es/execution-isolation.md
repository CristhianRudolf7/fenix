> Documento histórico: el gateway multiusuario y el registro/acceso con Google se retiraron. Para la versión personal actual, consulta [acceso remoto](remote-access.md).

# Aislamiento de ejecución y seguridad de cuentas

> Documento histórico del despliegue compartido anterior a 0.3.2. El sandbox y sus scripts se retiraron de la distribución personal. Véase [ejecución local actual](local-execution.md); las políticas aquí descritas no están activas en 0.3.2.


El backend predeterminado ahora es [gVisor con systrap](self-hosted-gvisor.md), con herramientas persistentes y cuotas mensuales, sin KVM ni virtualización anidada. OCI y Kata requieren selección explícita. Se comprueba la identidad del runtime antes de liberar el comando.

Implementación: 2026-09-19. [English: referencia completa y migración](../en/execution-isolation.md) · [Plan original](user-isolation-plan.md).

La implementación ya está en el código. Las cuentas existentes requieren una migración explícita: actualizar el repositorio no modifica sus contenedores, identidades, volúmenes ni servicios activos.

## Qué cambió

El controlador de cada cuenta conserva conversaciones, configuración y credenciales. Terminales, comandos, servidores de lenguaje y navegadores se ejecutan por defecto en sandboxes gVisor separados. Los procesos reciben únicamente el proyecto seleccionado y los recursos de las skills cargadas, estos últimos en modo de lectura. No reciben la base de datos del controlador, sus variables, otros proyectos ni los sockets de Docker o del servicio de ejecución.

La biblioteca Rust `fenix-execution` se comunica con el servicio protegido de `scripts/fenix_execution/`. En modo multiusuario, la ausencia de identidad, revisión de autorización, políticas o servicio impide ejecutar: no existe una alternativa automática que lance el comando en el host. El modo local individual sigue disponible y no se presenta como aislamiento multiusuario.

Cada cuenta tiene UID/GID propios y un sistema de archivos de capacidad fija. Hay límites por proceso y cuenta para RAM, swap, CPU, procesos, descriptores, temporales, logs y concurrencia. La reserva de controladores, ejecuciones y host debe caber en la RAM física. Dos tareas distintas no pueden modificar simultáneamente montajes de proyectos que se superpongan; para ese caso se utilizan worktrees independientes.

Los archivos se abren mediante descriptores, rechazando enlaces y archivos especiales. Los montajes parten de inodos ya fijados y se preparan en un directorio exclusivo de root, evitando sustituciones de rutas antes de que Docker los utilice. La edición comprueba el contenido anterior y reemplaza mediante rename atómico; un parche de varios archivos no constituye una transacción del sistema de archivos.

## Red y navegador

La inspección y los servidores de lenguaje funcionan sin red. El desarrollo y el navegador admiten web pública por TCP 80/443 y resolutores DNS definidos. Se bloquean destinos privados, metadata, direcciones del host y conexiones entre cuentas; IPv6 queda deshabilitado. Hay que declarar también las IP públicas/NAT del host que no aparezcan en sus interfaces. Este perfil permite web pública: no es una lista exclusiva de registros de paquetes.

El cortafuegos se instala de forma transaccional y se verifica completo. La supervisión detiene ejecuciones ante cambios inválidos, pero su sondeo no garantiza una ventana de detección de cero segundos. No se publican puertos de ejecución ni CDP: las conexiones autorizadas llegan al loopback del contenedor correspondiente.

Chromium mantiene su sandbox interno. En el modo OCI/runc, las políticas específicas de seccomp y AppArmor permiten los espacios de nombres y chroot necesarios, manteniendo filtros y restricciones, usuario no-root y ausencia de capacidades del host. No se agrega `--no-sandbox`, `SYS_ADMIN` ni modo privilegiado. Las políticas parten de Moby y conservan su licencia y atribución; hay que revisarlas y repetir las pruebas cuando cambien el kernel, Docker o Chromium. El servicio elige el ejecutable aprobado de la imagen, evitando wrappers antiguos que desactiven el sandbox.

En gVisor, el perfil seccomp se aplica dentro del sandbox, además de los filtros propios del runtime en el host. El perfil AppArmor específico de Chromium/runc no se aplica a Sentry.

## Integraciones y copias cifradas

`integration_call` permite consultar e invocar operaciones configuradas por el operador. Cada conexión pertenece a un usuario, tiene vencimiento, destino HTTPS y método fijos, campos permitidos, recursos delimitados y cuotas persistentes. El agente no proporciona credenciales ni el ID de otra cuenta externa. Se comprueba DNS, se fija la IP de conexión conservando la validación TLS, se rechazan redirecciones y respuestas excesivas, y no se reintentan escrituras ambiguas. Se respeta la política de aprobación existente para comandos.

Es un intermediario propio para APIs JSON/HTTPS; no instala Composio ni implementa automáticamente todos los conectores u OAuth. Las credenciales deben tener permisos mínimos y corresponder a APIs confiables. Filtrar reflejos de secretos es una protección adicional, no sustituye la autorización del recurso. Una operación externa ya aceptada puede completarse aunque posteriormente se revoque la conexión.

`workspace_checkpoint` crea, lista, restaura, clona y elimina copias de archivos del proyecto. Utiliza AES-GCM de `cryptography`, con clave fuera de las cuentas y metadatos autenticados de propietario, generación, imagen y política. No guarda RAM, sesiones, credenciales ni perfiles del navegador. Rechaza enlaces, archivos especiales y copias fuera de límites. Requiere detener las ejecuciones que puedan escribir en el proyecto; los servidores de lenguaje de solo lectura y los navegadores ajenos al proyecto pueden continuar. Es un archivo de proyecto, no un snapshot atómico del volumen: también deben detenerse escritores externos si se necesita consistencia de la aplicación. La restauración crea una carpeta nueva, comprueba la autorización actual y nunca revierte ni repite efectos externos.

Límites predeterminados: 32 MiB por copia, 128 MiB por cuenta, 1 GiB global, 32 copias por cuenta y retención de un minuto a siete días. El historial de ejecución se conserva siete días; la auditoría protegida rota en archivos de 4 MiB, con cuatro respaldos.

## Instalación y migración

Se requieren Linux, systemd, Docker con cgroup v2 y controlador systemd, seccomp, ext4, iptables y Python con `python3-cryptography`. La [guía inglesa](../en/execution-isolation.md#installation-and-migration) contiene la secuencia completa.

1. Construir el binario actualizado y la imagen de cuentas; fijar su digest.
2. Ejecutar `sudo scripts/install-isolation.sh`: solo instala archivos y ejemplos. No activa servicios ni modifica cuentas existentes.
3. Preparar y registrar gVisor siguiendo la [guía del runtime](self-hosted-gvisor.md#instalación-y-pruebas). Completar `/etc/fenix/execution.json` con UUID reales, UID/GID únicos, cuotas, rutas y direcciones del host. Mantener el propietario y los permisos 0600 del registro usado por el gateway; `registry_uid` identifica ese operador confiable y nunca puede ser una identidad de cuenta.
4. Configurar opcionalmente una clave aleatoria de 32 bytes para las copias y conexiones propias en el archivo de integraciones. No utilizar las credenciales de ejemplo.
5. En una ventana de mantenimiento, detener la cuenta antigua y sus trabajos, respaldar los datos y ejecutar `tenant-storage.py ... ACCOUNT_UUID migrate`. El padre de los directorios de cuenta debe pertenecer a root. La herramienta reserva capacidad real, conserva el directorio original en `.pre-isolation`, copia sin seguir enlaces y ajusta propietarios. Nunca reformatea una imagen existente ni sobrescribe el respaldo.
6. Para runc, activar el perfil de AppArmor instalado. Habilitar las unidades de montaje generadas e iniciar `fenix-execution.service`.
7. Recrear el controlador detenido con `tenant-container.py ... --execution-config ...` e iniciar la unidad de sistema `fenix-controller@ACCOUNT_UUID.service`. Retirar la unidad anterior para evitar trabajos o polling de Telegram duplicados.
8. Verificar una cuenta sintética y luego una cuenta real de prueba: login, archivos, terminal, LSP, navegador, descargas, cancelación, integraciones y restauración.

El servicio de ejecución controla Docker como root y es parte de la base de confianza. Sus montajes preparados deben ser visibles para dockerd; no se debe introducir un namespace de montajes privado mediante opciones de systemd. No es necesario ejecutar el gateway público como root.

## Revocación y recuperación

`fenix accounts disable|enable|passwd REGISTRY EMAIL` modifica el registro de forma atómica, bajo bloqueo, y avanza la revisión de autorización. Se ejecuta como el propietario establecido del registro; la contraseña entra por stdin. Las solicitudes y los streams SSE silenciosos comprueban la autorización actual. Deshabilitar una cuenta o cambiar su revisión revoca las ejecuciones y detiene el controlador antiguo. Volver a habilitarla no reactiva sesiones antiguas; hay que reprovisionar el controlador con la nueva revisión.

La cancelación, pérdida de conexión, vencimiento o revocación eliminan el contenedor completo, incluidos hijos separados. Si falla la API de Docker, el servicio puede detener el cgroup registrado directamente y conserva la información hasta confirmar la limpieza. Al reiniciar elimina recursos abandonados de su instancia. Un resultado de limpieza pendiente no se presenta como eliminación confirmada.

Ante un incidente: deshabilitar la cuenta, comprobar que sus procesos se detuvieron, conservar la auditoría acotada, rotar las credenciales afectadas y recuperar archivos en una carpeta nueva. No restaurar autoridad antigua ni repetir escrituras externas.

## Verificación y alcance

Las pruebas de `tests/isolation/` cubren autorización, rutas, cuotas, credenciales y copias cifradas. Las pruebas reales de Docker son optativas y utilizan cuentas sintéticas, volúmenes temporales y recursos etiquetados propios. Los comandos y resultados se detallan en la [guía inglesa](../en/execution-isolation.md#verification-and-limits) y el [registro de validación](../research/execution-isolation-validation-2026-09-19.json).

El [backend gVisor predeterminado](self-hosted-gvisor.md) utiliza un kernel de aplicación sin KVM. OCI comparte directamente el kernel del host; Kata sigue siendo opcional, requiere KVM y no se ha validado en hardware aquí. No se afirman snapshots de RAM, certificación frente a escapes, reinicios completos ni comparativas de rendimiento. Ejecutar la suite del backend elegido en el servidor de despliegue antes de admitir clientes.
