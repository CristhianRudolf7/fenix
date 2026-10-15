# Categorías, idiomas y modelos de skills

[English](categories-and-languages.md)

El catálogo único está en `resources/skill-categories.json`. Para editarlo sin
recompilar, configura `FENIX_SKILL_CATEGORIES` con la ruta de un JSON externo.
Cada categoría tiene un `id` estable, nombres `labels.en` y `labels.es`, y `aliases`.
Los alias no distinguen mayúsculas y no pueden repetirse. Los skills aprendidos deben
usar categorías existentes; los manuales antiguos con categorías desconocidas se
agrupan en `other`. No se necesita una habilidad ni una llamada al modelo para
editar el catálogo. El marketplace obtiene la categoría del propio SKILL.md.

En producción aislada, el campo `skill_categories` de `/etc/fenix/execution.json`
permite montar el catálogo del operador en modo de solo lectura. El gateway usa
la misma ruta mediante `FENIX_SKILL_CATEGORIES` y necesita permiso para leer el
archivo y atravesar sus directorios superiores. Tras editar el contenido, reinicia
el gateway para actualizar las categorías de publicaciones existentes. Los índices
de skills detectan los cambios automáticamente. Si cambias la ruta, recrea el
controlador. El archivo debe pertenecer a root y no admitir escritura de otros usuarios.

Todas las tareas comparten el catálogo, incluida seguridad. Las categorías y los
metadatos históricos de especialización no restringen skills ni conceden permisos.
Se conservan el aislamiento, los permisos y los skills desactivados.

El agente siempre utiliza el `SKILL.md` original en inglés. Un proceso del servidor
prepara la traducción con el modelo rápido al crear, importar, actualizar o restaurar
un skill. Detecta también ediciones manuales durante su revisión periódica, diez
segundos después del lote anterior. Procesa hasta cuatro traducciones simultáneas
y conserva los reintentos tras fallos. No se ejecuta en los benchmarks nativos.

Cada versión guarda un Markdown completo en
`state/skill-localizations/<hash-del-original>/SKILL.es.md`, con título y descripción
en español. Se guarda de forma atómica; un cambio del original invalida la versión
anterior. El código protegido se conserva y el archivo inglés no se modifica.

Las tarjetas y la vista completa leen archivos guardados, sin llamar al modelo.
Si falta una versión, se indica su disponibilidad; no se traduce al abrirla ni se
muestra inglés silenciosamente. Abrir un skill desactivado no lo habilita. Al
exportar se incluye la versión española disponible; el marketplace lee ese archivo
y comprueba que corresponda al original revisado.

Hay dos roles reutilizables: modelo inteligente para las tareas y modelo rápido
(normalmente GPT-6 Luna con pensamiento bajo) para conversación y traducción. Se reutilizan las credenciales
existentes. `/api/model-roles` muestra ambos y `/api/model-roles/fast` configura el
rápido. El modo de llamada y sus rutas se han eliminado. El auditor de publicaciones
conserva su configuración del operador para que un usuario no pueda sustituirlo.

La conexión rápida guarda `reasoning_effort` sin modificar el modelo inteligente. La revisión pública usa la configuración privada de Sentinel del operador, con `provider` y `reasoning_effort` explícitos.

La validación conserva exactamente los fragmentos de código y su cantidad, además del orden de los bloques. Los identificadores en línea pueden cambiar de posición dentro de una frase traducida sin provocar un rechazo falso.
