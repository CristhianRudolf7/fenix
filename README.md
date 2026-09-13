# Fénix

[English](README.en.md)

**Versión actual: 0.1.1**

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

- Web en español e inglés con bienvenida, chat, proveedores, Telegram, memoria,
  skills, programaciones, actividad y trazas. Fondo blanco por defecto y modo
  oscuro activado manualmente.
- Una tarea activa y una cola FIFO. Las respuestas a preguntas y aprobaciones
  pendientes desbloquean su tarea; los otros mensajes esperan su turno.
- **17 herramientas** para archivos, comandos, memoria, historial, skills,
  programaciones, inspección, aclaraciones y aprobaciones.
- Comandos desde cualquier directorio accesible al usuario de Linux. Las
  herramientas específicas de archivos conservan la carpeta configurada.
- Aprobaciones por texto para acciones concretas, comprobadas por el ejecutor.
  La configuración permite autorizar comandos o cambios en archivos.
- Historial y tareas en SQLite, IDs, procedencia, consumo de tokens, resultados
  acotados y salidas recuperables por páginas. Compactación básica con resumen,
  historial original conservado y parejas de llamadas/resultados completas.
- Memoria explícita con CRUD, detección exacta de duplicados y búsqueda léxica
  Unicode mediante FTS5. Sin extracción automática ni embeddings.
- Cuatro skills manuales: `memory-and-reminders`, `local-file-work`,
  `conversation-recap` y `local-troubleshooting`.
- Recordatorios únicos, intervalos y cron en `America/Lima`. Los recordatorios
  entregan texto; las tareas programadas ingresan a la misma cola. Se comprueban
  programaciones activas idénticas y ejecuciones solapadas de una programación.
- Telegram opcional: vinculación de un propietario y mensajes privados de texto.
- Integración de 17 proveedores con claves estáticas y credenciales por proyecto.

Los datos están en `.fenix/`, excluido de Git. Las claves se guardan en una base de
datos local con permisos privados, sin una clave maestra de cifrado independiente.
La configuración nunca las devuelve a la web y las claves conocidas se redactan
en las salidas del agente. El servidor sólo escucha en loopback y rechaza otros
orígenes y hosts del navegador.

Las tareas interrumpidas quedan registradas y no se reanudan automáticamente.
La web muestra texto progresivo con OpenAI Responses y la actividad de las
herramientas. Los demás adaptadores y Telegram conservan respuestas completas.
Al reconectar el navegador se recupera la vista provisional sin repetir la inferencia. Búsqueda web, navegador, heartbeat, voz, subagentes, memoria avanzada y
restauración de acciones permanecen en [ROADMAP.md](ROADMAP.md).

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

- [Arquitectura](docs/es/architecture.md)
- [Instalación y uso local](docs/es/local-operation.md)
- [Registro de verificación](docs/es/verification.md)
- [Uso de las bibliotecas](docs/es/development.md)
- [Proveedores y autenticación](docs/es/provider-reference.md)
- [Requisitos actuales](REQUIREMENTS.md)
- [Trabajo futuro](ROADMAP.md)
