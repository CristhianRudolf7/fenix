> La versión en desarrollo usa un único propietario y login en el propio servidor. Configúralo antes del primer arranque siguiendo [acceso personal](remote-access.md).

# Uso local

[Ejecución interactiva, límites actualizados y herramientas bajo demanda](execution-harness.md)

[English](../en/local-operation.md)

## Inicio y configuración

Primero instala Node 22.12+ y pnpm 10.34.5, y compila la interfaz con
`pnpm --dir apps/web install --frozen-lockfile` y `pnpm --dir apps/web build`.
Consulta [dictado y conversación escrita](dictation.md) para el micrófono del navegador y la configuración del modelo rápido.

Ejecuta `cargo run --locked -p fenix-server --bin fenix` desde el clon y abre
http://127.0.0.1:4173. Los datos se guardan en `.fenix/`, relativa al directorio de
ejecución. Puedes indicar `--data-dir`, `--skills-dir` y `--port`; `--help` los lista.
Las skills usan `~/.fenix/skills`, independientemente de `--data-dir`. El servicio
escucha únicamente en loopback IPv4.

La bienvenida pide proveedor, modelo y API key. Las opciones avanzadas incluyen
endpoint, cabeceras de proyecto/organización, ámbito de la clave, carpeta de
archivos y políticas de acciones. Luna con pensamiento bajo viene seleccionado;
OpenAI no es obligatorio. La bienvenida actual exige un valor de clave incluso
para Ollama local, aunque ese servidor pueda ignorarlo. Bedrock y Azure necesitan
un endpoint explícito. El modelo elegido debe admitir texto y herramientas.

La comprobación realiza una llamada real que puede tener coste. Una clave inválida
no completa la configuración. Los cambios de proveedor/modelo conservan el chat y
esperan a que terminen las tareas activas y en cola. Una clave vacía conserva la
guardada para ese proveedor y ámbito exactos; el proyecto llamado `default` no
comparte por accidente la clave sin proyecto. Las tareas activas incluyen hijos
en segundo plano: el motor rechaza cambios de modelo o permisos mientras trabajen.

## Conversación y paneles

Los mensajes nuevos se encolan. Responde `aprobar <id>` o `rechazar <id>` a una
aprobación; un sí/no claro también sirve para la única solicitud pendiente. Las
aclaraciones usan `responder <id> <respuesta>`. Los demás mensajes esperan su turno.
`cancelar` detiene la tarea activa y `cancelar <task_id>` también admite tareas en
cola o subagentes. Los controles son textuales y cancelar no revierte efectos
anteriores. Puede haber varias solicitudes simultáneas: usa su ID exacto. Un hijo
que espera aprobación no bloquea al principal.

La web gestiona proyectos, memoria, skills, programaciones, tareas e instrucciones. Sólo
se recuerdan datos explícitos. Para instalar un skill manual, crea
`~/.fenix/skills/<nombre>/SKILL.md` con `description:` y `version:` en el frontmatter.
Actualiza el panel para descubrirlo. Las plantillas no sobrescriben archivos
personales existentes. Principal e hijos ven todos los metadatos breves, incluidas
las skills deshabilitadas; sólo las habilitadas cargan instrucciones bajo demanda.
Las cuatro predeterminadas no son un máximo del catálogo. `--skills-dir` permite
mantener una biblioteca específica para una instalación.

Al iniciar se copian los recursos de `<data-dir>/skills` a la biblioteca elegida,
sin sustituir destinos, seguir enlaces simbólicos ni borrar originales. Los límites
de migración son 32 MiB por archivo, 512 MiB totales, 10.000 entradas y 16 niveles.
Si se exceden, los originales quedan intactos: usa la ruta anterior con `--skills-dir`
o copia manualmente los recursos restantes.

El catálogo varía según rol y ámbito. El principal delega con propósito preciso
y resultado esperado; el contexto y las restricciones son opcionales. Los hijos
no pueden leer la conversación principal ni volver a delegar; sólo inspeccionan
su propia tarea y evidencia de planes autorizada. Mantienen contexto independiente y devuelven informes
al principal. **Tareas** muestra propósito, entrega esperada, IDs, estado, intentos,
resultados y pendientes textuales, actualizados mientras la vista está abierta.

El motor impone un nivel de delegación y cuatro inferencias simultáneas de hijos. Cada hijo tiene
hasta tres intentos totales por fallos reintentables. Por defecto dispone de 128
pasos de modelo por tarea y una hora por intento, configurables desde Ajustes.
Se conserva el contexto y los resultados sin repetir acciones. Un
bloqueo, cancelación o error no reintentable puede finalizar antes. El programa
serializa cambios de archivos, memoria, programaciones y comandos hasta su cierre;
no coordina procesos externos ni otras instancias. Los comandos mantienen acceso
según los permisos del usuario de Linux y no están aislados en un sandbox.

Las fechas sin offset usan horario peruano. Cron acepta cinco campos, o seis/siete
con segundos y año opcional. `0 9 * * 1-5` representa días laborables a las 09:00.
Un recordatorio entrega texto; una tarea programada utiliza al agente. Reactivar
una programación sólo habilita ejecuciones futuras. Pide su cancelación por chat
indicando nombre o ID.

## Proyectos sin chats separados

Abre **Proyectos** para crear o editar nombre, alias únicos, descripción, contexto
y carpeta existente opcional. Menciona su nombre o alias en el mismo chat. La
conversación general permanece disponible; “conversación general” vuelve
explícitamente a ese ámbito. **Responder** en la web o responder a un mensaje de
Telegram aporta una referencia a su tarea. Esa referencia no equivale a aprobación.

El destino se resuelve antes de recuperar memoria o ejecutar herramientas. Las
referencias claras pueden utilizar reglas; otros mensajes requieren una llamada
al modelo configurado y pueden necesitar aclaración textual. Una petición compuesta
puede consultar A y delegar trabajo extenso de B. Cambiar de tema no cambia el
proyecto ni la carpeta de las ejecuciones activas.

En **Memoria**, el ámbito permite consultar información general o de un proyecto.
Una edición corriente no mueve recuerdos entre ámbitos. **Ver progreso** muestra
el estado registrado de las tareas, sin comprobar de nuevo recursos externos.

Guardar el formulario reemplaza explícitamente los valores como propietario e
incrementa la versión, sin bloqueo optimista. Vuelve a cargarlo antes de guardar si
otro trabajo cambió el contexto entretanto. Las actualizaciones del agente sí
exigen la versión esperada y rechazan escrituras obsoletas. El sondeo actualiza la
lista sin sustituir tus campos en edición. El historial antiguo se conserva como
general, sin asignarlo automáticamente a proyectos. Más en [la guía](projects.md).

## Telegram

Crea un bot con `@BotFather`, introduce su token en Configuración y activa Telegram.
Tras verificarlo, Fénix muestra `/start <código-aleatorio>`. Envíalo por privado al
bot para vincular al propietario. Web y Telegram utilizan el mismo historial y la
misma cola. No se procesan grupos, terceros, archivos, imágenes ni voz. No hace
falta servidor público ni webhook.

No uses otro poller o un webhook activo para el mismo bot. La web muestra el último
error de conexión. Las respuestas permanecen en el chat web aunque falle su envío
a Telegram. Un resultado de red ambiguo puede duplicar un mensaje saliente.

## Instalación desde el menú de Linux

Ejecuta `bash scripts/install-local.sh` en Linux con una sesión de usuario systemd.
El instalador compila la versión release y coloca el ejecutable en
`~/.local/bin/fenix`, el servicio en `~/.config/systemd/user/fenix.service`, el
acceso en `~/.local/share/applications/fenix.desktop` y los datos en
`~/.local/share/fenix/.fenix/`. No necesita `sudo`.

Es una instalación distinta de `.fenix/` en el clon; no copia claves
automáticamente. Comparte `~/.fenix/skills/` con los clones de desarrollo, aunque
cada base de datos conserva su habilitación de skills. El servicio se activa con la sesión de usuario. No se configura
para funcionar después de cerrar sesión. El acceso Fénix inicia el servicio y abre
la web. Una actualización mediante el instalador detiene el servicio existente;
las tareas con puntos seguros pueden continuar; las inciertas quedan interrumpidas.

Consulta `systemctl --user status fenix` o `journalctl --user -u fenix` para
diagnósticos. Detén con `systemctl --user stop fenix`. Para desinstalar, deshabilita
con `systemctl --user disable --now fenix`, elimina ejecutable/acceso/unidad y
ejecuta `systemctl --user daemon-reload`. Los datos personales se conservan; bórralos
por separado sólo si deseas perder historial y claves.

## Datos y reinicios

El directorio usa permisos `0700` y la base de datos `0600`. Las claves no tienen
cifrado con una clave maestra independiente. Respalda por separado la biblioteca
compartida de skills cuando esté fuera del directorio de datos. Respalda este último con
el servicio detenido para incluir correctamente el estado de SQLite/WAL. Los datos
de ejecución, skills personales y `apuntes.md` permanecen ignorados por Git.

Cerrar la pestaña no detiene el servicio. Al apagarlo se cancelan procesos
gestionados. El reinicio conserva historial y programaciones y puede continuar tareas
con puntos seguros; las inciertas quedan interrumpidas. No repite efectos guardados
ni ejecuciones vencidas. El historial de archivos permite restauración explícita,
pero no restaura procesos. Véanse [funcionamiento y límites](harness-continuity.md).

[Local browser / Navegador local](browser.md)
