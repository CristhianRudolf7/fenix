# Auditoría selectiva de resultados y evidencia

Fénix separa que el ejecutor haya terminado de que el resultado esté verificado.
Las tareas pequeñas conservan su flujo directo. La revisión inicial de planes se
mantiene; esta auditoría comprueba resultados después de la ejecución.

- Los hitos que alimentan otros pasos pasan a `verifying`: sus dependencias esperan.
- Un trabajador de un plan con varios hitos también recibe auditoría al acumular 12
  resultados de herramientas sustantivas. Se excluyen descubrimiento de herramientas,
  carga de skills, consultas de estado de comandos y la llamada para terminar.
- Los planes con tres o más hitos, dependencias o trabajo sustancial observado reciben
  una auditoría final del resultado integrado. Para una tarea grande de un solo hito,
  el agente indica `audit_required=true`: tendrá una sola auditoría final.
- El mecanismo se aplica a planes; las instrucciones dirigen las tareas largas al
  planificador. No se añade un auditor a cada turno normal ni a cada herramienta.

El auditor es un subagente hermano, con contexto nuevo y el mismo modelo del maestro.
Trabaja en segundo plano y solo puede leer y preguntar. No modifica archivos, ejecuta
comandos, llama conectores, repara resultados ni crea otros agentes. Consulta la
petición original, modificaciones explícitas, respuestas anteriores y los artefactos
reales. El informe del ejecutor se trata como una afirmación pendiente de comprobar.

El registro durable distingue **tarea, requisito, artefacto y hecho** de sus estados
**pendiente, verificado, dudoso y refutado**. Conserva procedencia, autor, ámbito,
versión del plan y referencias a observaciones. Una hipótesis factual puede refutarse
con evidencia; eso no permite descartar un requisito ni una pregunta al usuario.
Los artefactos declarados por el ejecutor se registran automáticamente como pendientes.

`plan_evidence` permite consultar o añadir candidatos sin certificar su propio trabajo.
Si un candidato incluye una pregunta, se guarda antes de enviarla al chat. La respuesta
aclara la intención; no prueba que se haya ejecutado la tarea. El maestro y las ramas
independientes siguen disponibles mientras el auditor espera una aclaración.

Para aprobar, el auditor debe cubrir las entradas asignadas con observaciones propias.
El motor rechaza referencias inventadas, lecturas fallidas, informes desactualizados
y preguntas sin respuesta. Los archivos de texto leídos incluyen SHA-256; si cambian
antes de aceptar el informe, deben volver a leerse. Los originales y versiones
anteriores permanecen disponibles después de compactar el contexto.

Una auditoría incompleta bloquea el resultado correspondiente y comunica los hallazgos
en el canal original. No repite acciones automáticamente. Una corrección exige revisar
el plan; los informes de versiones anteriores no pueden aprobar la nueva. Reiniciar
el servicio conserva la evidencia y marca las auditorías inconclusas como interrumpidas.

La interfaz de tareas muestra el estado de verificación y el registro de evidencia.
El auditor inspecciona; las pruebas que requieren ejecución siguen a cargo del ejecutor.
La revisión representa el estado observado en ese momento, no una garantía permanente
ni una mejora de benchmarks ya demostrada.

La [documentación en inglés](../en/completion-audits.md) detalla los contratos y límites.
