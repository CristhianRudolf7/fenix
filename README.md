# Fénix

[English](README.en.md)

**Versión actual: 0.3.0**

Asistente personal local para Linux. Un servicio Rust ofrece una web y Telegram
opcional con **una sola conversación continua**, herramientas locales, memoria
explícita, skills manuales, recordatorios y tareas programadas. No necesita un
servidor Node durante el uso habitual.

## Iniciar

Instala Rust mediante [rustup](https://rust-lang.org/tools/install/) y ejecuta desde
el repositorio:

```sh
cargo run --locked -p fenix-server --bin fenix
```

Abre **http://127.0.0.1:4173**. La bienvenida pide una API key y comprueba una llamada
real antes de habilitar el chat. El valor actual es **OpenAI `gpt-5.6-luna`,
pensamiento bajo**. Puedes elegir cualquiera de los 17 proveedores con un modelo
compatible con herramientas. Telegram es opcional y se puede configurar después.

Para instalar un acceso en el menú de aplicaciones y un servicio de usuario en
Linux con systemd:

```sh
bash scripts/install-local.sh
```

Se instala en tu carpeta personal, sin `sudo`. Después abre **Fénix** desde el menú.
Cerrar la pestaña no detiene el servicio. La computadora y el servicio deben estar
encendidos para atender Telegram y ejecutar programaciones.

## Funcionalidades disponibles

- [Arnés de programación](docs/es/coding-harness.md): búsqueda regex/glob, parches de varios archivos, reglas `AGENTS.md` del repositorio, LSP configurable y compactación de tareas activas con checkpoints recuperables.

- Web en español e inglés con bienvenida, chat, proyectos, proveedores, Telegram, memoria,
  skills, programaciones, actividad y trazas. Fondo blanco por defecto y modo
  oscuro activado manualmente.
- Turnos de conversación concurrentes, subagentes y planes editables con hitos y
  dependencias. Las preguntas y la planificación no retienen el chat.
  [Planes, revisiones y pub/sub](docs/es/planning.md) conservan avances y canales de entrega.
- [Revisión adaptativa de planes](docs/es/plan-reviews.md): revisión inicial de planes
  complejos, especialistas bajo demanda y resolución con evidencia. Usa el mismo modelo
  configurado; el chat permanece disponible y las rondas de corrección están acotadas.
- Herramientas del principal para trabajo local, memoria, proyectos y delegación.
  Los hijos tienen un catálogo restringido, sin historial de la conversación ni
  delegación, con informe de cierre obligatorio e inspección de su tarea y evidencia de plan autorizada.
- Comandos desde cualquier directorio accesible al usuario de Linux. Las
  herramientas específicas de archivos conservan la carpeta configurada.
- Aprobaciones por texto para acciones concretas, comprobadas por el ejecutor.
  La configuración permite autorizar comandos o cambios en archivos.
- Historial y tareas en SQLite, IDs, procedencia, consumo de tokens, resultados
  acotados y salidas recuperables por páginas. Compactación básica con resumen,
  historial original conservado y parejas de llamadas/resultados completas.
- Proyectos con nombres, alias, contexto y carpeta opcional, sin abrir nuevos
  chats. Enrutamiento antes de recuperar memoria, conversación general separada
  y contexto fijo por ejecución; los informes tardíos no cambian el proyecto activo.
- [Memoria adaptable por proyecto](docs/es/adaptive-memory.md): hechos con procedencia, candidatos, vigencia e historial; entidades, alias y relaciones con esquemas versionados. Recuperación léxica/temporal, contexto seleccionado, extracción en segundo plano y embeddings opcionales.
- [Búsqueda web propia y lectura de fuentes](docs/es/web-search.md): rastreador en
  segundo plano, índice público en disco y lectura HTTP de resultados. Sin
  proveedor externo de búsqueda; cobertura inicial pequeña que crece con enlaces.
- [Navegador interactivo local](docs/es/browser.md): control en Rust de Chromium,
  perfiles persistentes, contraseñas cifradas, control manual y pruebas responsivas.
- Cinco skills manuales: `memory-and-reminders`, `local-file-work`,
  `conversation-recap`, `local-troubleshooting` y `browser-navigation`.
- Recordatorios únicos, intervalos y cron en `America/Lima`. Los recordatorios
  entregan texto; las tareas programadas ingresan a la misma cola. Se comprueban
  programaciones activas idénticas y ejecuciones solapadas de una programación.
- Telegram opcional: vinculación de un propietario y mensajes privados de texto.
- Integración de 17 proveedores con claves estáticas y credenciales por proyecto.

Los datos están en `.fenix/`, excluido de Git. Las claves de proveedores de modelos se guardan en una base de
datos local con permisos privados, sin una clave maestra de cifrado independiente.
La configuración nunca las devuelve a la web y las claves conocidas se redactan
en las salidas del agente. El servidor sólo escucha en loopback y rechaza otros
orígenes y hosts del navegador.

Las tareas interrumpidas quedan registradas y no se reanudan automáticamente.
La web muestra texto progresivo con OpenAI Responses y la actividad de las
herramientas. Los demás adaptadores y Telegram conservan respuestas completas.
Al reconectar el navegador se recupera la vista provisional sin repetir la inferencia. Heartbeat, voz y
restauración de acciones permanecen en [ROADMAP.md](ROADMAP.md).

## Trabajo en segundo plano y skills compartidas

Cada subagente recibe propósito preciso y resultado esperado, con contexto y
restricciones opcionales; no existe un campo obligatorio de “información
necesaria”. El motor limita la delegación a un nivel, cuatro inferencias simultáneas de hijos,
tres intentos totales ante fallos reintentables y límites configurables de 128
pasos de modelo por tarea y una hora por intento, de forma predeterminada.
Los reintentos conservan el contexto del hijo y los resultados previos,
sin volver a ejecutar acciones ya completadas. Los bloqueos, cancelaciones y
fallos no reintentables pueden detenerse antes. Un reinicio interrumpe las tareas
inconclusas; no las reanuda automáticamente.

**Tareas** muestra propósito, entrega esperada, estado, IDs, intentos y resultados.
Responde en la conversación con `aprobar <request_id>`, `rechazar <request_id>` o
`responder <request_id> <respuesta>`; cancela con `cancelar <task_id>`. Cada pendiente
tiene su ID y la espera de un hijo no bloquea al principal. No se puede cambiar el
modelo ni los permisos mientras haya tareas activas. El historial del hijo queda
fuera de la conversación principal; la memoria explícita sigue accesible mediante
herramientas. El programa serializa cambios de archivos, memoria y programaciones,
y la duración completa de sus comandos, dentro del mismo proceso. Los comandos
conservan los permisos del usuario de Linux; no están aislados en un sandbox.

Las skills se guardan en **`~/.fenix/skills/<skill-name>/SKILL.md`**, compartidas
entre directorios de ejecución. `--skills-dir PATH` permite elegir otra ruta;
`--data-dir` sólo cambia los datos de ejecución. Principal e hijos ven el catálogo
completo de metadatos breves y el estado de habilitación; las instrucciones sólo
se cargan mediante `skill_load`. Las deshabilitadas siguen visibles pero no se
pueden cargar. Las cuatro predeterminadas son plantillas, no un máximo de skills.

Al iniciar, se copian los recursos de `<data-dir>/skills` a la biblioteca elegida
antes de instalar los defaults. No se sustituyen destinos existentes, se omiten
enlaces simbólicos y se conservan los originales. La migración automática admite
32 MiB por archivo, 512 MiB en total, 10.000 entradas y 16 niveles; si excede esos
límites, selecciona la carpeta anterior con `--skills-dir` o copia manualmente.

## Organización y comprobaciones

`apps/server` contiene el ejecutable y la API; `apps/web`, la interfaz. Los crates
separan contratos, credenciales, proveedores, persistencia y motor del agente. Las
plantillas están en `resources/skills/`; cada instalación usa sus copias personales.
La documentación principal se mantiene en inglés con versión en español.

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo build --locked -p fenix-server
npm --prefix apps/web ci
cd apps/web
npx playwright install chromium
npm test
```

Las pruebas habituales usan claves ficticias y servidores HTTP locales. Las del
navegador levantan el servidor Rust real en un directorio temporal. Node y
Playwright sólo se necesitan para desarrollo. La prueba real opcional se ejecuta
con `python3 scripts/smoke-live.py`, desde la raíz y con `OPENAI_API_KEY` definido
en el entorno. Realiza llamadas facturables a Luna, elimina sus datos temporales y
guarda un informe sin secretos en `target/`. Las pruebas simuladas de 17 proveedores
no demuestran acceso real a las 17 cuentas.

- [Proyectos en una conversación](docs/es/projects.md)
- [Arquitectura](docs/es/architecture.md)
- [Instalación y uso local](docs/es/local-operation.md)
- [Registro de verificación](docs/es/verification.md)
- [Uso de las bibliotecas](docs/es/development.md)
- [Proveedores y autenticación](docs/es/provider-reference.md)
- [Requisitos actuales](REQUIREMENTS.md)
- [Trabajo futuro](ROADMAP.md)

Las sesiones de terminal interactivas, servicios persistentes, registros incrementales, límites configurables, seguimiento de subagentes y herramientas bajo demanda se describen en la [guía del motor de ejecución](docs/es/execution-harness.md).

[Local browser / Navegador local — sessions, controls and responsive tests](docs/es/browser.md)

[Evaluación nativa y recuperación del motor](docs/es/native-evaluation.md)

Consulta [skills y recuperación](docs/es/harness-recovery.md) para procedimientos bajo demanda, comandos restringidos a lectura y evaluación reproducible.
