# Fénix

[**Documentación y guías → fenix.crehlabs.com**](https://fenix.crehlabs.com) · [Leer en inglés](README.en.md)

**Versión en desarrollo: 0.3.2.** Asistente de inteligencia artificial con conversación continua, ejecución en segundo plano y herramientas de programación, documentos y navegador. La distribución se orienta a instalaciones personales con procesos locales; el sandbox multiusuario se retiró de esta versión. La disponibilidad depende de los servicios y credenciales configurados; los resultados históricos no garantizan éxito en cualquier tarea.

## Capacidades

- Chat persistente, proyectos y tareas concurrentes; interlocutor rápido y modelo ejecutor configurables. Telegram opcional y conexiones adicionales según su configuración.
- Subagentes con objetivo específico, un nivel de delegación, seguimiento y maestro disponible durante su ejecución.
- Planes editables, hitos, dependencias, preguntas y entregas asíncronas. Autonomía elegida por texto y verificaciones según requisitos y riesgos.
- Herramientas de archivos y terminal interactiva, procesos persistentes, búsqueda regex/glob, parches, LSP configurable y worktrees de Git.
- Memoria por ámbito con procedencia, entidades, relaciones, revisiones y recuperación; compactación con checkpoints y búsqueda de resultados conservados.
- Catálogo de skills bajo demanda, edición personal con guardado explícito, traducciones persistidas y aprendizaje selectivo con revisiones recuperables. El backend conserva publicación de skills; la interfaz actual muestra el catálogo personal.
- Navegador Chromium con perfiles, sesiones, control del propietario y gestión de inactividad/capacidad. Hibernación en disco cuando el ejecutor compatible está configurado.
- Búsqueda propia con índice local, rastreador y lectura ligera; no ofrece cobertura ilimitada de internet.
- Biblioteca privada, extracción, fragmentos, citas y búsqueda híbrida con embeddings opcionales. Herramientas nativas para DOCX, XLSX y PPTX; PDF/OCR mediante componentes separados.
- Propuestas visuales en carrusel del chat, modales y panel de resultados con diagramas, documentos y multimedia. Explorador de archivos limitado a la cuenta.
- Cron, recordatorios y flujos programados con conectores y navegador.
- Perfil, consumo registrado y acceso personal con correo y contraseña; sesiones de siete días exactos.
- Evaluación nativa y adaptadores de benchmarks con informes de tareas, tiempos, consumo y resultados verificables.

## Tecnología y organización

El backend utiliza Rust y Axum. El frontend utiliza Vite, React, TypeScript, Tailwind y pnpm. En el despliegue estático, Caddy/systemd sirve la interfaz y reenvía las APIs al backend; actualizar la interfaz no exige recompilar Rust.

| Carpeta | Función |
|---|---|
| `apps/server` | Servidor personal, autenticación y APIs. |
| `apps/web` | Interfaz y pruebas de navegador. |
| `apps/docs` | Documentación Astro + Starlight + TypeScript, búsqueda local y guías. |
| `crates` | Componentes Rust del motor, proveedores, almacenamiento, navegador y documentos. |
| `resources` | Instrucciones internas, skills y recursos visuales. |
| `scripts` | Instalación, infraestructura, generación y evaluación. |
| `packaging/linux` | Unidades systemd y ejemplos del despliegue local. |
| `docs/es` | Guías en español; las versiones inglesas existentes se conservan en `docs/en`. |
| `tests` | Pruebas transversales y herramientas. |

Las instrucciones internas, skills y fixtures mantienen su idioma operativo. Los Markdown de la raíz se escriben en español; este archivo tiene su versión inglesa en `README.en.md`. La documentación de otras carpetas conserva sus versiones actuales.

## Instalación pública

El instalador `scripts/install.sh` instala una aplicación individual para Linux x86_64, glibc ≥ 2.39 y una sesión systemd de usuario, sin Cargo, Node ni pnpm. Separa la aplicación de los datos y no modifica los servicios del despliegue alojado.

La release **0.3.1** distribuye el binario y la interfaz compilados. En Ubuntu 24.04 o posterior (x86_64), con una sesión systemd de usuario:

```sh
curl -fsSL https://fenix.crehlabs.com/install.sh | bash
```

Para comprobar el servicio: `systemctl --user status fenix-personal.service`. Para detenerlo sin borrar datos: `systemctl --user disable --now fenix-personal.service`. Los datos y skills propios se conservan en `~/.local/share/fenix-personal/`.

Para fijar una versión: `curl -fsSL https://fenix.crehlabs.com/install.sh | FENIX_VERSION=v0.3.1 bash`. No hay que ejecutar con sudo. Al terminar abre `http://127.0.0.1:4173` para elegir proveedor, modelo y clave. Las bibliotecas de ejecución OpenSSL 3, zlib y zstd deben estar instaladas (normalmente lo están en una instalación de escritorio). Usa `FENIX_PORT` si el puerto 4173 está ocupado. Las dependencias opcionales del navegador y documentos requieren preparación adicional.

## Desarrollo y ejecución individual

Instala el Rust fijado en `rust-toolchain.toml`, Node compatible con Vite y pnpm **10.34.5**. Desde el repositorio:

```sh
pnpm --dir apps/web install --frozen-lockfile
pnpm --dir apps/web build
cargo run --locked -p fenix-server --bin fenix
```

El servidor individual escucha por defecto en `127.0.0.1:4173`. Las instalaciones nuevas guardan los datos en la carpeta de datos de aplicaciones de cada sistema; una `.fenix/` local existente se conserva. Admite `--data-dir`, `--skills-dir` y `--port`. La configuración de proveedores y del interlocutor pertenece a esa instalación. El valor inicial del código no confirma acceso real al proveedor.

Para desarrollo del frontend puede ejecutarse `pnpm --dir apps/web dev`: Vite utiliza el puerto 5173 y reenvía `/api` al servidor local. El servidor personal exige login con correo y contraseña, sin gateway multiusuario.

El instalador `scripts/install-local.sh` conserva una modalidad individual con systemd. Para frontend/backend separados, seguir [despliegue](docs/es/frontend-deployment.md). Las guías de aislamiento multiusuario corresponden a versiones anteriores; no son una modalidad admitida por 0.3.2.

## Documentación

- [Requisitos actuales](REQUIREMENTS.md), [hoja de ruta](ROADMAP.md) e [historial de cambios](CHANGELOG.md).
- [Arquitectura](docs/es/architecture.md), [desarrollo](docs/es/development.md) y [uso local](docs/es/local-operation.md).
- [Planificación](docs/es/planning.md), [autonomía](docs/es/conversational-autonomy.md) y [arnés de programación](docs/es/coding-harness.md).
- [Memoria](docs/es/adaptive-memory.md), [continuidad](docs/es/harness-continuity.md) y [skills](docs/es/skills-and-documents.md).
- [Navegador](docs/es/browser.md), [ciclo de vida](docs/es/browser-lifecycle.md) y [búsqueda](docs/es/web-search.md).
- [Biblioteca de documentos](docs/es/document-library.md), [resultados visuales](docs/es/design.md) y [archivos](docs/es/workspace-viewer.md).
- [Conexiones y flujos](docs/es/workflows-and-connectors.md), [Dictado](docs/es/dictation.md), [perfil](docs/es/account-profile.md) y [acceso personal](docs/es/remote-access.md).
- [Evaluación](docs/es/native-evaluation.md) e [informes históricos](docs/benchmarks/).

## Verificación y límites

```sh
cargo fmt --all --check
cargo test --workspace --all-targets --locked
cargo test --workspace --doc --locked
pnpm --dir apps/web check
pnpm --dir apps/web test
```

Las pruebas de navegador, OCR, infraestructura y benchmarks necesitan sus dependencias específicas. Algunas usan proveedores simulados: no constituyen mediciones de calidad de los modelos ni pruebas completas de producción.

**Pendiente:** verificar el firewall en la infraestructura real, incluidos destinos permitidos/bloqueados, DNS y detención al perder protección. Véanse [fronteras y validación](docs/es/security-boundaries.md).

La ventana inicial del motor es 200.000 tokens con compactación a 160.000; no amplía la capacidad ni las cuotas del proveedor. La restauración de archivos no revierte cualquier efecto externo. El navegador y los formatos de documentos tienen límites de compatibilidad documentados. Mantén claves, bases de datos, sesiones y artefactos personales fuera de Git.

## Sitio de documentación

La documentación pública está en **[fenix.crehlabs.com](https://fenix.crehlabs.com)**. Usa **Astro + Starlight + TypeScript + pnpm**, con búsqueda local Pagefind, navegación por guías, referencias técnicas, resaltado de código, modo claro/oscuro y diseño responsivo. Se construye desde los requisitos, la hoja de ruta, el registro de cambios y las guías españolas existentes. Las referencias históricas conservan su alcance; los requisitos vigentes prevalecen.

```sh
pnpm --dir apps/docs install --frozen-lockfile
pnpm --dir apps/docs build
pnpm --dir apps/docs preview
```

La publicación usa `scripts/publish-docs.sh` y el ejemplo `packaging/linux/Caddyfile.docs`, adaptado al despliegue actual. Caddy sirve archivos estáticos mediante systemd; construir esta web no llama a modelos ni requiere API keys. La documentación ocupa la raíz del dominio y las rutas `/docs/`. El acceso móvil existente se conserva en `/movil.html`. El instalador público descarga la release compilada desde GitHub; la documentación incluye el comando de instalación.

## Ejecución personal en 0.3.2

El [motor nativo de procesos](docs/es/native-process-engine.md) comparte ejecución y terminales entre Linux, macOS y Windows. El soporte completo de la aplicación y sus instaladores para Windows/macOS sigue pendiente; no debe confundirse con la portabilidad del motor.

La versión 0.3.2 en desarrollo utiliza comandos locales con los permisos de la cuenta del sistema. Retira el sandbox de ejecución multiusuario, sus instaladores y el modo de comandos `read_only`. No exige Docker, gVisor ni microVM. La release pública 0.3.1 sigue siendo el paquete distribuido hasta publicar 0.3.2.

Un detector determinista solicita aprobación para comandos potencialmente peligrosos, incluso si la política general permite ejecutar comandos. La pregunta aparece en el chat y se responde **sí** o **no**. Cada permiso es de un solo uso y corresponde a la acción exacta; rechazarla impide repetirla sin autorización. La detección no garantiza reconocer código malicioso arbitrario y no constituye un sandbox. Las instalaciones existentes conservan sus preferencias de aprobación.

Se conservan las protecciones de contenido de la interfaz, el historial y los worktrees. PDF e imágenes se extraen con un proceso Python local y dependencias opcionales instaladas por el propietario. Véase [ejecución local](docs/es/local-execution.md).

### Acceso al sistema de archivos en instalación personal

El workspace es la carpeta predeterminada para trabajar y crear proyectos, no una frontera de acceso. Las herramientas del agente admiten rutas absolutas, rutas relativas externas y enlaces a archivos, con los permisos del usuario que ejecuta Fénix. La búsqueda puede partir de una carpeta externa y el análisis LSP puede abrir archivos externos. Los comandos de riesgo siguen requiriendo aprobación textual; se conservan la ocultación de secretos en resultados y las comprobaciones contra cambios inesperados al editar. Los límites de tamaño de las herramientas de texto no son permisos del sistema: para archivos grandes o binarios se utilizan las herramientas apropiadas o comandos. Este cambio no concede permisos de administrador ni publica archivos externos mediante el visor web.

### Motor de archivos multiplataforma

La lectura, creación, edición, borrado, resolución de rutas y listado se concentran en `fenix-files`, con adaptación Unix/Windows. El chat y el explorador ya no requieren `/proc` para listar carpetas. Se mantienen las comprobaciones de contenido, los permisos del sistema y la separación entre acceso personal y archivos privados. Véase [el alcance y la validación del motor](docs/es/native-file-engine.md); esta adaptación no equivale a portar toda la aplicación ni a validar instalaciones completas en Windows/macOS.

### Skills y almacenamiento portable

`fenix-skills` gestiona instalación, exportación, edición y eliminación sin dependencias de `/proc`. El almacenamiento privado utiliza bloqueos por archivo, permisos Unix o ACL de Windows y conserva SQLite, migraciones y traducciones. Véase [rutas, operaciones y validación](docs/es/skills-storage.md). La aplicación completa sigue pendiente de validación nativa en Windows/macOS.

## Acceso del propietario en la versión en desarrollo

Antes de arrancar por primera vez, configura el único propietario con `fenix owner CORREO --password-stdin [--data-dir RUTA]`. La contraseña se recibe por entrada estándar, no por argumentos. Con Cargo: `cargo run -p fenix-server --bin fenix -- owner CORREO --password-stdin`. El login dura siete días exactos, sobrevive a reinicios y no se renueva con actividad. No hay registro público ni gateway multiusuario. Esta funcionalidad aún no está incluida en la release pública 0.3.1. Véase [acceso personal](docs/es/remote-access.md).

La adaptación del [núcleo portable](docs/es/platform-core.md) centraliza el cierre del agente y el almacenamiento privado, y retira el transporte de ejecución remoto. La compilación cruzada del núcleo no equivale a disponer de aplicaciones móviles o instaladores completos para todos los sistemas.

## Estudiar el código

Explora el [atlas interactivo del motor](https://fenix.crehlabs.com/atlas/): procesos generales, módulos del backend, búsqueda de funciones y enlaces al código. El inventario se regenera con la documentación; las explicaciones describen el árbol de desarrollo y pueden diferir de la release instalada.

La configuración desde código para Windows y macOS se describe en [la guía de escritorio](docs/es/desktop-setup.md). Los instaladores y la CI incorporados no sustituyen la validación real en esos sistemas; el instalador público conserva su alcance Linux.
