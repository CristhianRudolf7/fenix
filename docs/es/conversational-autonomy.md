# Autonomía por conversación y ejecución basada en evidencia

[English](../en/conversational-autonomy.md)

El usuario decide mediante el mismo chat cuánto delegar: «hazlo todo tú solo» para esta tarea, «para este proyecto consúltame» o «de ahora en adelante encárgate tú». No hay nuevos botones de permisos ni otro agente.

La preferencia se guarda en la base de datos de la cuenta mediante `autonomy_manage`, con una cita exacta del mensaje del usuario o su corrección a la tarea. La prioridad es tarea, proyecto asignado, preferencia general y, finalmente, modo asistido. El ámbito duradero debe ser explícito. Los subagentes heredan la preferencia, pero no pueden cambiarla. Las siguientes llamadas reciben la política actual sin repetir acciones ya ejecutadas. El interlocutor debe tramitar el cambio y no afirmar que está guardado antes de que el motor lo confirme.

## Las dudas iniciales siempre se consultan

Si falta definir el objetivo, alcance, comportamiento esperado o criterios de aceptación, el agente pregunta antes de hacer cambios dependientes, incluso cuando el usuario haya pedido autonomía total. Primero revisa la instrucción, respuestas anteriores y evidencia disponible. `ask_user` usa por defecto la categoría de requisito, que la autonomía no puede desactivar. La respuesta sigue guardándose en el contrato existente. También se consultan contradicciones importantes que aparezcan después.

Las decisiones técnicas reversibles dentro del encargo usan la categoría de implementación: en modo asistido pueden consultarse; en autónomo el motor indica que deben resolverse investigando y registrando los supuestos relevantes. Una dependencia instalable que falta no debe presentarse como una duda del objetivo. El modelo existente interpreta la intención; el motor comprueba la procedencia, el ámbito y la política, sin otra llamada a un clasificador.

La autonomía no modifica permisos de comandos/archivos, autorizaciones pendientes, denegaciones, políticas de conectores, aislamiento, especificaciones, revisión de planes, requisitos de verificación ni auditorías de finalización. Una aprobación realmente exigida sigue siendo necesaria.

## Benchmarks sin interacción

La evaluación activa explícitamente el perfil sin usuario en su propia base de datos. No queda esperando una respuesta: resuelve detalles técnicos o registra un bloqueo verificable si falta un requisito o autorización indispensable. No inventa respuestas ni concede permisos. El cierre bloqueado debe citar la llamada que detectó la falta de interlocutor.

## Mejoras de las tareas 4–6

- Evitar repetir esquemas de herramientas ya disponibles y mejorar la selección del skill para fusionar JSON/CSV/Parquet.
- Reutilizar evidencias reales al cerrar si el modelo omitió redactarlas. Las comprobaciones pendientes, fallidas o desactualizadas siguen bloqueando el cierre.
- Informar archivo, bloque y líneas candidatas cuando un parche resulta ambiguo. No se elige una coincidencia al azar ni se escriben archivos durante una prevalidación fallida.
- Distinguir una dependencia ausente de una denegación real: la advertencia de descarga de `_apt` no convierte la ausencia de `dpkg-source` en falta de autorización.
- Revisar los lenguajes declarados por la compilación, incluidos `.pyx/.pxd` cuando corresponda, y probar operaciones representativas de las extensiones requeridas. Reutilizar ejemplos y pruebas existentes, sin inventar requisitos adicionales.

Los resultados anteriores del benchmark se conservan; estas correcciones no demuestran por sí solas una mejora de puntuación. Este cambio no incluye despliegue a producción.

## Mejoras tras las tareas 7–9

Las comparaciones deben partir de copias independientes del mismo estado y probar una operación válida que cambie datos. Dos rechazos que dejan archivos iguales no comprueban la operación principal. Los skills piden una prueba representativa, no muchas pruebas indiscriminadas.

En evaluaciones sin usuario, una consulta de requisitos necesita `blocking_reason` para justificar un bloqueo por falta de respuesta. Sin esa explicación, se indica revisar si es intención indispensable o un detalle de borde, avanzar en requisitos independientes y documentar decisiones reversibles. Las preguntas iniciales interactivas no necesitan ese campo. El motivo lo aporta el modelo: el motor controla la política, no demuestra semánticamente que toda duda sea imprescindible.

El cierre reconoce ejemplos ejecutables sencillos escritos entre backticks con rutas absolutas (compiladores, intérpretes y ejecutables con argumentos). Cada ruta reconocida exige una ejecución correcta después de la última edición registrada. Fallos, dependencias ausentes y errores ocultados no sirven. No ejecuta texto extraído ni pretende interpretar todo el shell: ejemplos relativos/complejos siguen usando el contrato explícito existente. Comprueba ejecución, no toda la semántica de las salidas. Las ediciones con herramientas de archivo invalidan la evidencia; no se infieren todas las mutaciones arbitrarias realizadas por shell. Se conservan permisos y auditorías.

`command_run` conserva el diagnóstico de dependencia ausente en vez de sobrescribirlo. El skill de servicios refuerza su vida después del turno y las comprobaciones del protocolo. Sin despliegue a producción.

## Mejoras posteriores a las tareas 10–12

Los ejemplos simples de compilación y ejecución exigen una cadena exitosa que genere y ejecute el mismo archivo. Se admiten compiladores sencillos y rutas `-o`; no es un intérprete de shell ni una certificación del binario. Otros sistemas de construcción conservan los contratos explícitos.

La inspección siempre informa directorio, usuario, proceso inicial, herramientas presentes/ausentes y fin de la comprobación. Busca también ejecutables administrativos y solo guarda en caché inspecciones completas. El catálogo de descubrimiento excluye esquemas ya cargados e indica que se llamen directamente las herramientas visibles.

Los skills exigen verificar el binario recién construido, probar despliegues con contenido nuevo y push exitoso, y entregar un servicio funcionando cuando eso sea lo solicitado. Un script o su sintaxis no equivalen al resultado. Se conservan las preguntas iniciales y permisos; no se añaden revisores ni llamadas extra al modelo.

El adaptador Harbor explica el entorno aislado de ejecución. Los tres arneses de comparación reciben el mismo contexto, sin pruebas ocultas ni ampliación de permisos en producción. Distinguir una duda realmente indispensable sigue siendo una decisión semántica del modelo.
