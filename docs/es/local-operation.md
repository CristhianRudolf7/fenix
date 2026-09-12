# Uso local

[English](../en/local-operation.md)

## Inicio y configuración

Ejecuta `cargo run --locked -p fenix-server --bin fenix` desde el clon y abre
http://127.0.0.1:4173. Los datos se guardan en `.fenix/`, relativa al directorio de
ejecución. Puedes indicar `--data-dir`, `--skills-dir` y `--port`; `--help` los lista.
El servicio escucha únicamente en loopback IPv4.

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
comparte por accidente la clave sin proyecto.

## Conversación y paneles

Los mensajes nuevos se encolan. Responde `aprobar <id>` o `rechazar <id>` a una
aprobación; un sí/no claro también sirve para la única solicitud pendiente. Las
aclaraciones usan `responder <id> <respuesta>`. Los demás mensajes esperan su turno.
`cancelar` detiene la tarea activa y `cancelar <task_id>` también admite tareas en
cola. Los controles son textuales y cancelar no revierte efectos anteriores.

La web gestiona memoria, skills, programaciones, actividad e instrucciones. Sólo
se recuerdan datos explícitos. Para instalar un skill manual, crea
`.fenix/skills/<nombre>/SKILL.md` con `description:` y `version:` en el frontmatter.
Actualiza el panel para descubrirlo. Las plantillas no sobrescriben archivos
personales existentes.

Las fechas sin offset usan horario peruano. Cron acepta cinco campos, o seis/siete
con segundos y año opcional. `0 9 * * 1-5` representa días laborables a las 09:00.
Un recordatorio entrega texto; una tarea programada utiliza al agente. Reactivar
una programación sólo habilita ejecuciones futuras. Pide su cancelación por chat
indicando nombre o ID.

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
automáticamente. El servicio se activa con la sesión de usuario. No se configura
para funcionar después de cerrar sesión. El acceso Fénix inicia el servicio y abre
la web. Una actualización mediante el instalador detiene el servicio existente;
las tareas pendientes se marcarán interrumpidas al reiniciar.

Consulta `systemctl --user status fenix` o `journalctl --user -u fenix` para
diagnósticos. Detén con `systemctl --user stop fenix`. Para desinstalar, deshabilita
con `systemctl --user disable --now fenix`, elimina ejecutable/acceso/unidad y
ejecuta `systemctl --user daemon-reload`. Los datos personales se conservan; bórralos
por separado sólo si deseas perder historial y claves.

## Datos y reinicios

El directorio usa permisos `0700` y la base de datos `0600`. Las claves no tienen
cifrado con una clave maestra independiente. Respalda el directorio completo con
el servicio detenido para incluir correctamente el estado de SQLite/WAL. Los datos
de ejecución, skills personales y `apuntes.md` permanecen ignorados por Git.

Cerrar la pestaña no detiene el servicio. Al apagarlo se cancelan procesos
gestionados. El reinicio conserva historial y programaciones, pero marca las tareas
anteriores sin terminar como interrumpidas; no repite sus efectos ni recupera
ejecuciones vencidas. No hay rollback automático ni restauración de procesos.
