# Planificación editable y conversación asíncrona

[English](../en/planning.md)

Los planes también tienen [revisión adaptativa](plan-reviews.md): revisión inicial
para trabajo complejo, especialistas bajo demanda y una respuesta basada en evidencia,
usando el modelo configurado y los trabajadores en segundo plano existentes.

Fénix conserva una conversación visible mientras atiende turnos independientes,
planifica y ejecuta hitos en tareas asíncronas de Rust. Una llamada lenta al modelo
o una pregunta pendiente no retiene el chat. Las instrucciones exigen comunicar
**toda duda detectada**, incluso pequeña, reutilizando las respuestas explícitas.
Detectar la ambigüedad depende del modelo; el motor guarda y dirige cada pregunta.

## Planes, dependencias y skills

El maestro utiliza `plan_manage create` para tareas largas. Cada hito incluye ID,
objetivo, resultado esperado, dependencias opcionales y sugerencias de skills.
SQLite rechaza ciclos, IDs repetidos y dependencias inexistentes. Cada plan tiene
un ámbito general o de proyecto fijo; la coordinación entre proyectos conserva
las reglas de delegación explícita existentes.

Los hitos listos se ejecutan como subagentes. Sus dependientes reciben resultados,
fuentes y referencias a efectos anteriores. Pueden consultar los registros de sus
predecesores y generaciones anteriores del mismo hito, sin leer ramas independientes
u otros proyectos. Todos ven el catálogo resumido de skills y cargan las habilitadas
bajo demanda. Se mantiene un nivel de delegación y hasta tres intentos ante fallos
transitorios del proveedor.

El planificador despacha y consolida resultados mediante código. No consulta al
modelo para sondear estados ni entregar avisos. Elaborar o interpretar un cambio
del plan sí consume tokens del modelo.

## Cambiar el plan en cualquier momento

Puedes describir el cambio en el chat. Para dirigirlo inmediatamente, incluso
con una llamada al modelo pendiente, utiliza:

```text
replanificar <id-tarea-o-plan> Cambia el formato de exportación y conserva el análisis completado
```

Al responder a un mensaje desde web o Telegram basta `replanificar <cambio>`.
Si todavía se está elaborando la tarea, se conserva su ID, se cancela la inferencia
sustituida y se reconstruye el contexto con la nueva instrucción y la evidencia.
Si ya hay un plan, se crea un turno de edición prioritario en la misma conversación,
con el ámbito y canal de entrega originales.

Un cambio en lenguaje natural pausa lo pendiente mientras se interpreta. Conserva
los hitos completados. Una revisión concreta invalida los hitos modificados y sus
dependientes; el trabajo que sigue siendo válido puede continuar. Cada revisión
exige la versión esperada. Un cambio posterior sustituye al turno de edición anterior.
Las respuestas antiguas no reactivan trabajo descartado. El motor comprueba la
generación del trabajador antes de despachar, aceptar resultados y producir efectos.
Espera a que terminen los efectos gestionados antes de declarar aplicada la revisión.
Los efectos ya realizados o confirmados por sistemas externos no se revierten;
sus registros se conservan para inspeccionar y corregir lo necesario.

La sección **Tareas → Planes** muestra hitos y revisiones. `cancelar <id-plan>`
detiene un plan; el ID de un hijo cancela sólo ese hijo. Las preguntas se responden
con `responder <id-solicitud> <respuesta>` o una respuesta nativa. Las aprobaciones
mantienen sus IDs y argumentos exactos. También existen `GET /api/plans`,
`GET /api/plans/{id}` y `POST /api/tasks/{id}/replan` con `{"text":"..."}`.

## Pub/sub y entrega

La migración 4 añade planes versionados, generaciones, preguntas, eventos SQLite y
cursores por suscriptor. Los eventos incluyen IDs y metadatos de tarea, sesión,
proyecto, plan, versión y destino cuando corresponda. El mensaje y su publicación
se guardan en una transacción. Los avisos en memoria sólo despiertan suscriptores;
la base es la fuente durable. Los consumidores confirman después de persistir su
trabajo y toleran repeticiones internas.

La web escucha `/api/events` mediante SSE y recupera novedades usando `Last-Event-ID`
o `?after=`. Actualiza historial y estado sin repetir inferencias. Las vistas
provisionales de texto OpenAI conservan `/api/progress`; los fragmentos de texto
no se acumulan en el bus durable.

Telegram entrega al bot y propietario capturados por la tarea. Cambiar de proyecto,
responder desde el otro canal vinculado o reconfigurar Telegram no redirige los
resultados pendientes. Los fallos se reintentan con espera creciente acotada. Una
respuesta de red incierta todavía puede duplicar un mensaje externo; la deduplicación
interna no garantiza entrega externa exactamente una vez.

## Límites y reinicios

Hay ocho plazas normales de despacho del maestro y una vía reservada para controles;
pueden mantenerse 64 turnos normales pendientes y uno de control. Se admiten 32
trabajos de subagentes vivos y cuatro inferencias simultáneas. Esperar una respuesta
libera capacidad de despacho y no consume tiempo activo. Los límites
predeterminados son cuatro horas por tarea principal, una hora por intento de hijo
y 128 pasos de modelo por tarea, acumulados entre reintentos y mensajes de
seguimiento. Son configurables; no se añade presupuesto acumulado de tokens.

Tras reiniciar, tareas, preguntas y planes sin terminar quedan interrumpidos.
Los eventos y mensajes no entregados siguen disponibles; los efectos no se reanudan
automáticamente. Para continuar un plan guardado, hay que revisarlo explícitamente
con su evidencia anterior. El contexto de cada turno excluye las secuencias de
herramientas todavía incompletas de otros turnos; al finalizar, cada secuencia
queda agrupada. El historial original y la auditoría se conservan.

Los turnos nuevos reciben referencias resumidas a tareas activas de su ámbito
autorizado. Así pueden dirigir cambios en lenguaje natural a una tarea que todavía
se elabora, sin importar su secuencia de herramientas incompleta.
