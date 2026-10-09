# Fénix

[**Documentación y guías → fenix.crehlabs.com**](https://fenix.crehlabs.com) · [Leer en inglés](README.en.md)

**Versión declarada: 0.3.0.** Asistente de inteligencia artificial con conversación continua, ejecución en segundo plano y herramientas de programación, documentos y navegador. El código incluye ejecución individual y una arquitectura compartida con cuentas y entornos aislados. La disponibilidad depende de los servicios y credenciales configurados; los resultados históricos no garantizan éxito en cualquier tarea.

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
- Perfil, consumo registrado, sesiones de siete días, acceso con contraseña y soporte de Google OAuth sujeto a configuración y capacidad reservada.
- Evaluación nativa y adaptadores de benchmarks con informes de tareas, tiempos, consumo y resultados verificables.

## Tecnología y organización

El backend utiliza Rust y Axum. El frontend utiliza Vite, React, TypeScript, Tailwind y pnpm. En el despliegue estático, Caddy/systemd sirve la interfaz y reenvía las APIs al backend; actualizar la interfaz no exige recompilar Rust.

| Carpeta | Función |
|---|---|
| `apps/server` | Servidor local, gateway, autenticación y APIs. |
| `apps/web` | Interfaz y pruebas de navegador. |
| `apps/docs` | Sitio público de documentación estática, búsqueda local y guías. |
| `crates` | Componentes Rust del motor, proveedores, almacenamiento, navegador y documentos. |
| `resources` | Instrucciones internas, skills y recursos visuales. |
| `scripts` | Instalación, infraestructura, generación y evaluación. |
| `packaging/linux` | Unidades systemd, contenedores y ejemplos del despliegue. |
| `docs/es` | Guías en español; las versiones inglesas existentes se conservan en `docs/en`. |
| `tests` | Pruebas transversales, herramientas y aislamiento. |

Las instrucciones internas, skills y fixtures mantienen su idioma operativo. Los Markdown de la raíz se escriben en español; este archivo tiene su versión inglesa en `README.en.md`. La documentación de otras carpetas conserva sus versiones actuales.

## Desarrollo y ejecución individual

Instala el Rust fijado en `rust-toolchain.toml`, Node compatible con Vite y pnpm **10.34.5**. Desde el repositorio:

```sh
pnpm --dir apps/web install --frozen-lockfile
pnpm --dir apps/web build
cargo run --locked -p fenix-server --bin fenix
```

El servidor individual escucha por defecto en `127.0.0.1:4173`. Los datos se guardan en `.fenix/`; admite `--data-dir`, `--skills-dir` y `--port`. La configuración de proveedores y del interlocutor pertenece a esa instalación. El valor inicial del código no impone un modelo al servicio compartido ni confirma acceso real al proveedor.

Para desarrollo del frontend puede ejecutarse `pnpm --dir apps/web dev`: Vite utiliza el puerto 5173 y reenvía `/api` al servidor local. La interfaz del servicio compartido requiere gateway y autenticación; no asumir que un servidor individual replica ese despliegue completo.

El instalador `scripts/install-local.sh` conserva una modalidad individual con systemd. Para frontend/backend separados y cuentas, seguir [despliegue](docs/es/frontend-deployment.md) y [aislamiento](docs/es/user-isolation.md); no sustituir las políticas del servidor compartido con el instalador individual.

## Documentación

- [Requisitos actuales](REQUIREMENTS.md), [hoja de ruta](ROADMAP.md) e [historial de cambios](CHANGELOG.md).
- [Arquitectura](docs/es/architecture.md), [desarrollo](docs/es/development.md) y [uso local](docs/es/local-operation.md).
- [Planificación](docs/es/planning.md), [autonomía](docs/es/conversational-autonomy.md) y [arnés de programación](docs/es/coding-harness.md).
- [Memoria](docs/es/adaptive-memory.md), [continuidad](docs/es/harness-continuity.md) y [skills](docs/es/skills-and-documents.md).
- [Navegador](docs/es/browser.md), [ciclo de vida](docs/es/browser-lifecycle.md) y [búsqueda](docs/es/web-search.md).
- [Biblioteca de documentos](docs/es/document-library.md), [resultados visuales](docs/es/design.md) y [archivos](docs/es/workspace-viewer.md).
- [Conexiones y flujos](docs/es/workflows-and-connectors.md), [Live](docs/es/live.md), [perfil](docs/es/account-profile.md) y [Google](docs/es/google-sign-in.md).
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

La documentación pública está en **[fenix.crehlabs.com](https://fenix.crehlabs.com)**. Tiene búsqueda local, navegación por guías, referencias técnicas, modo claro/oscuro y diseño responsivo. Se construye desde los requisitos, la hoja de ruta, el registro de cambios y las guías españolas existentes. Las referencias históricas conservan su alcance; los requisitos vigentes prevalecen.

```sh
pnpm --dir apps/docs install --frozen-lockfile
pnpm --dir apps/docs build
pnpm --dir apps/docs preview
```

La publicación usa `scripts/publish-docs.sh` y el ejemplo `packaging/linux/Caddyfile.docs`, adaptado al despliegue actual. Caddy sirve archivos estáticos mediante systemd; construir esta web no llama a modelos ni requiere API keys. La documentación ocupa la raíz del dominio y las rutas `/docs/`. El acceso móvil existente se conserva en `/movil.html`. El instalador público de un solo comando sigue pendiente.
