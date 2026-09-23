# Contratos persistentes por proyecto

Fénix conserva requisitos acordados entre tareas mediante `project_spec`, una
herramienta bajo demanda sobre SQLite. No crea otro planificador ni obliga a generar
documentos para cada acción. La conversación general no tiene un contrato implícito.

- Cada requisito conserva ID estable, texto, revisión, procedencia y tarea autora.
  El contrato tiene una versión creciente. Las eliminaciones conservan historial.
- `get` devuelve requisitos paginados; `changes` lista propuestas e historial;
  `get` con `change_id` muestra el antes/después y los conflictos de una propuesta.
- `propose` registra operaciones `add`, `modify` o `remove`, con ID, fuente, motivo
  y `expected_version`. Proponer no cambia el contrato vigente. Una repetición
  idéntica de la misma tarea reutiliza su propuesta pendiente.
- Los subagentes proponen; el maestro puede `apply` o `discard` dentro del proyecto
  asignado. Auditores y revisores solo leen. Las actualizaciones son transaccionales,
  versionadas y publican eventos en el sistema existente.
- Si dos propuestas afectan al mismo requisito, deben reconciliarse explícitamente.
  Cambios independientes pueden consolidarse después de releer la versión vigente,
  siempre que los requisitos afectados no hayan cambiado.

Antes de integrar un worktree, el motor comprueba que los cambios de requisitos
declarados por el trabajador no estén obsoletos, descartados o en disputa. Si lo
están, bloquea la integración y conserva los archivos. Una modificación explícita
posterior de la misma tarea sustituye sus deltas anteriores sobre esos IDs.

La comprobación compara IDs y revisiones; no entiende todas las contradicciones
entre textos ni descubre cambios no declarados. Tampoco revierte merges anteriores.
El maestro resuelve esos casos con aclaraciones, revisiones del plan y la auditoría
existente. En tareas sin Git se comprueban versiones y conflictos al consolidar.

El contrato refleja **comportamiento acordado, no implementación verificada**.
Consolidarlo no completa tareas, concede permisos ni reemplaza las pruebas. La
petición original del usuario y sus cambios explícitos conservan autoridad; una
inferencia no se transforma automáticamente en un requisito confirmado.

El contexto recibe solo versión, cantidad e instrucciones de consulta. Los textos
se cargan bajo demanda. Las acciones pequeñas sin cambios de comportamiento no
necesitan propuesta. No se añaden interfaz, embeddings ni llamadas obligatorias
al modelo. [Contrato técnico en inglés](../en/project-contracts.md).
