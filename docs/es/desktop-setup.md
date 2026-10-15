# Configuración de escritorio en Windows y macOS

Fénix comenzó como una instalación Linux. El núcleo local incorpora adaptaciones Windows/macOS, detección de navegador y scripts de instalación desde el código. Esto no equivale a una release binaria de escritorio validada: el instalador público `install.sh` sigue orientado a Linux.

## Requisitos comunes

- Git y el toolchain Rust fijado en `rust-toolchain.toml`.
- Node compatible con Vite y pnpm 10.34.5.
- Herramientas de compilación nativas: Xcode Command Line Tools en macOS y Visual Studio Build Tools con C++/Windows SDK en Windows.
- Chromium, Google Chrome o Microsoft Edge si deseas navegador autónomo.
- Python 3 y los paquetes de extracción/OCR, solo para esas funciones documentales.

La aplicación abre en `http://127.0.0.1:4173`. Conserva correo y contraseña del propietario, sesiones de siete días y configuración de modelos. No instala un gateway multiusuario. Los assets web construidos se sirven desde el binario en estas instalaciones locales; el despliegue público conserva su frontend independiente.

## macOS

Desde un clon del repositorio:

```sh
bash scripts/install-macos.sh
```

También puedes usar `bash scripts/install-local.sh`, que selecciona el instalador macOS cuando corresponde. El script compila frontend y backend, copia el ejecutable y crea un LaunchAgent personal `com.crehlabs.fenix`. Funciona dentro de la sesión del usuario; no es un daemon root.

Instalación y datos:

```text
~/Library/Application Support/Fenix/
  bin/fenix
  web/
  data/
  logs/
~/Library/LaunchAgents/com.crehlabs.fenix.plist
```

La configuración registra PATH y, cuando están definidas al instalar, `FENIX_BROWSER_EXECUTABLE`, `FENIX_OCR_PYTHON` y `FENIX_SHELL`. No incluye automáticamente claves de proveedores en el plist. Antes de actualizar, detiene solo su LaunchAgent; conserva el directorio de datos.

```sh
launchctl print gui/$(id -u)/com.crehlabs.fenix
launchctl bootout gui/$(id -u)/com.crehlabs.fenix
"$HOME/Library/Application Support/Fenix/bin/fenix" owner EMAIL --password-stdin --data-dir "$HOME/Library/Application Support/Fenix/data"
```

Si una dependencia requiere OpenSSL o pkg-config, instala sus paquetes de desarrollo antes de compilar. La firma/notarización y un instalador `.dmg` no están incluidos.

## Windows

Desde PowerShell en el clon:

```powershell
.\scripts\install-windows.ps1
```

La política local debe permitir ejecutar el script. El instalador no desactiva globalmente esa política. Instala en `%LOCALAPPDATA%\Fenix`, limita permisos a la identidad actual y SYSTEM, conserva `data` y crea una tarea `FenixPersonal-<SID>` del usuario. Se inicia al iniciar sesión y puede iniciarse inmediatamente; no requiere una cuenta de servicio ni privilegios elevados.

La tarea utiliza Windows PowerShell y un wrapper local que define `FENIX_WEB_DIST` y registra la salida. No es un servicio Windows que continúe garantizado tras cerrar sesión. `-NoStart` instala y registra sin iniciarlo inmediatamente.

```powershell
& "$env:LOCALAPPDATA\Fenix\bin\fenix.exe" owner EMAIL --password-stdin --data-dir "$env:LOCALAPPDATA\Fenix\data"
Get-ScheduledTask -TaskName 'FenixPersonal-*'
```

No se genera todavía un `.msi` ni se descarga una release Windows desde el instalador público.

## Navegador y memoria

El valor `auto` detecta rutas habituales de Chromium, Chrome y Edge. Busca aplicaciones globales y del usuario en macOS y ubicaciones de instalación Windows, además de PATH. Se conserva una ruta explícita existente; la antigua ruta Linux `/usr/bin/chromium` se adapta en los otros sistemas.

`FENIX_BROWSER_EXECUTABLE` permite elegir un ejecutable instalado; una selección explícita inexistente produce un error y no se sustituye silenciosamente. Si no hay navegador con la configuración automática, el runtime puede abrirse y las operaciones de navegación explican la dependencia faltante.

La medición del árbol de procesos usa interfaces nativas mediante `sysinfo` en Linux, Windows y macOS. Devuelve un valor desconocido si el proceso no está disponible. La suma puede contar memoria compartida varias veces; no es una garantía ni un límite de RAM. No se añade una cola global por memoria en este cambio.

## Documentos

La extracción local busca `python3` o `python`, con `FENIX_OCR_PYTHON` como ruta opcional a Python 3 o un entorno virtual. La lectura del worker conserva verificación de hash e identidad del archivo, rechaza symlinks/reparse points y no depende de constantes `O_NOFOLLOW`/`O_NONBLOCK` inexistentes en Windows.

Las dependencias PDF y OCR se instalan aparte. Los formatos Office manejados directamente en Rust no necesitan esa extracción Python para las operaciones nativas. La disponibilidad de PaddleOCR y sus modelos debe comprobarse en cada plataforma; este cambio no demuestra OCR real en Windows/macOS.

## Validación y límites

Se añaden jobs CI para compilar el backend y sus tests en Windows/macOS, verificar detección/memoria y revisar sintaxis de instaladores. Su presencia no significa que se hayan ejecutado ni que haya una instalación completa validada.

El entorno de desarrollo actual es Linux. Las pruebas locales y la compilación cruzada se reportan por separado; quedan pendientes instalaciones reales, arranque al iniciar sesión, permisos de macOS, políticas Windows y recorridos completos con navegador/documentos en ambos sistemas.

Comprobaciones realizadas desde Linux en esta adaptación: compila el crate del navegador para Windows x64 y macOS ARM64; pasan diez pruebas unitarias del navegador, una prueba con Chromium real de navegación/sesión persistente, once pruebas de extracción, incluida ausencia simulada de flags POSIX y dobles OCR y seis contratos del instalador Linux. Se verifica sintaxis Bash y generación/lectura del plist, sin ejecutar launchd. La sintaxis PowerShell se comprueba en el job Windows, pendiente de ejecución.

El servidor Linux compila con sus tests, pasa la prueba Rust que invoca el extractor local y la documentación completa se construye correctamente. Esto no demuestra que los servicios nativos Windows/macOS hayan arrancado en equipos reales.
