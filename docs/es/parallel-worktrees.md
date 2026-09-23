# Worktrees para programación en paralelo

Fénix crea un worktree de Git antes de iniciar un subagente ejecutor en un
repositorio. Cada trabajador conserva su rama `fenix/worker-<task_id>`, archivos,
directorio de comandos y bloqueo de escritura. El maestro sigue disponible y
los trabajadores pueden editar, compilar y probar simultáneamente. Los revisores
y auditores conservan su acceso a las evidencias sin crear una copia editable.

## Funcionamiento

1. Configurar como workspace la raíz del repositorio, con un commit inicial,
   rama identificada y sin cambios pendientes ni operaciones Git incompletas.
   Fénix no guarda en stash, confirma ni descarta los borradores del usuario.
2. Registrar tarea, carpeta original, rama de destino, commit base y worktree.
   En modo alojado se guarda en
   `<user-root>/projects/.fenix-worktrees/<repository-key>/<task-id>`; en modo
   independiente, en `.fenix-worktrees/<repository-key>/<task-id>` junto al
   repositorio. Se conserva la asignación original de usuario y proyecto.
3. Dirigir archivos, instrucciones del repositorio, LSP y comandos relativos
   hacia esa carpeta. Los worktrees tienen bloqueos independientes; los servicios
   compartidos mantienen su sincronización. Esto no añade llamadas al modelo.
4. Antes de comunicar éxito, detener los procesos del trabajador, esperar sus
   escrituras y pedir aprobación si la política de archivos es `ask`. Guardar sus
   cambios en un commit local con mensaje en inglés.
5. Integrar de forma serializada, comprobando la vigencia del plan, la rama de
   destino y la ausencia de cambios pendientes. Detectar conflictos antes de
   modificar la carpeta principal. Los hitos dependientes arrancan después de la
   integración del resultado anterior.
6. Si falla la integración, comunicar `blocked`, conservar rama y archivos y
   explicar el motivo. Un conflicto detectado no modifica los archivos ni el
   índice de la carpeta principal. Los planes bloqueados requieren conciliación
   mediante una revisión; el maestro puede reintentar un trabajador independiente
   ya detenido con `worktree_manage`.

La herramienta `worktree_manage` permite consultar asignación, cambios y estado;
solo un maestro autorizado puede integrar. El resultado y el estado del
subagente incluyen los datos del worktree. Integrar otra vez un commit ya
incorporado no crea otro commit de integración.

## Límites

- Las carpetas sin Git mantienen el comportamiento anterior. Los repositorios
  dañados o configurados desde una subcarpeta producen un error explícito.
- Un worktree separa archivos; el aislamiento de seguridad corresponde al
  sandbox existente. Las rutas Git deben pertenecer al usuario y los comandos
  deben comenzar dentro de la carpeta asignada.
- Los contenedores de comandos montan el checkout del trabajador, sin exponer los
  metadatos Git compartidos. Allí los comandos Git directos pueden no funcionar;
  la herramienta nativa consulta el estado y el motor realiza commits y merges.
- Se requiere Git 2.38 o posterior. Los repositorios con filtros o drivers de
  merge que ejecutan programas externos quedan bloqueados para integración manual.
- No cambiar ramas ni modificar internamente Git desde un trabajador. No se hace
  push ni se eliminan ramas o carpetas automáticamente.
- La serialización cubre este runtime, no editores externos. Reservar la carpeta
  principal para integración durante el trabajo paralelo. Un merge sin conflictos
  no garantiza corrección: siguen siendo necesarias las pruebas y la auditoría
  selectiva existente.
- Cancelaciones, errores y reinicios conservan los archivos. No se reanudan tareas
  automáticamente después de reiniciar. Una creación interrumpida queda bloqueada
  para revisión en vez de sobrescribir la carpeta reservada.

Contrato detallado: [English](../en/parallel-worktrees.md).

Los cambios de requisitos declarados reciben una comprobación del
[contrato del proyecto](project-contracts.md) antes de integrarse. Detecta revisiones
obsoletas y solapamientos de IDs; no contradicciones semánticas arbitrarias ni cambios
que el trabajador no haya declarado.
