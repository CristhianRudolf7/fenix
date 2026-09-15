# Evolución del arnés con Nemotron

[English](../../en/audits/nemotron-harness-evolution.md)

## Protocolo

Referencia: commit `574e390` de main, evaluador nativo Rust real, Terminal-Bench 2.1 en `7131e4375048a0e408a8fb404b5f499d726b695b` y Harbor 0.23.0. Ambas variantes usan RunInfra `nemotron-3-5-lightning-30b`, pensamiento predeterminado del proveedor, la misma configuración y las instrucciones y verificadores oficiales. No se introducen pruebas ocultas ni soluciones de referencia en el contexto del agente. Se ejecutan dos tareas en paralelo; descarga, preparación y verificación se miden por separado del tiempo del agente.

Desarrollo: `custom-memory-heap-crash`, `extract-elf`, `modernize-scientific-stack`, `schemelike-metacircular-eval` y `sanitize-git-repo`. Estos resultados permiten corregir mecanismos generales, pero no demuestran mejora en tareas nuevas.

Confirmación: dos grupos preseleccionados de cinco tareas y dos repeticiones por variante. Grupo A: `build-pov-ray`, `prove-plus-comm`, `regex-chess`, `sqlite-with-gcov`, `mailman`. Grupo B: `circuit-fibsqrt`, `make-mips-interpreter`, `write-compressor`, `sparql-university`, `raman-fitting`. Se exige una media de dos aciertos adicionales por diez tareas, mejora positiva en ambas repeticiones y ausencia de regresiones en las pruebas. Tres ciclos consecutivos sin mejora reproducible detienen el bucle. Los fallos del modelo o de herramientas cuentan como fallos; las exclusiones de infraestructura deben declararse.

Se congelan los hashes del binario y de la configuración. El coste procede del proveedor; si falta, queda como desconocido. Las trazas privadas permanecen en `.fenix/evaluations/harness-evolution-20260915/`, ignorado por Git. La rama experimental se revisa por separado y no se fusiona automáticamente.

## Fundamento

[Agentic Harness Engineering](https://arxiv.org/html/2604.25850v4) motiva registrar la predicción de cada cambio y comprobar regresiones por tarea. [SkillsBench](https://arxiv.org/abs/2602.12670) respalda estudiar procedimientos curados y concisos, pero también registra tareas perjudicadas: ampliar el catálogo no garantiza mejores resultados. Son hipótesis de investigación, no evidencia de mejora de esta implementación.

## Validación e infraestructura

La referencia pasó 239 pruebas Rust y 13 de navegador; cuatro pruebas Rust dependientes del entorno quedaron omitidas. Clippy, formato y documentación pasaron. Como `/tmp` estaba lleno, se utilizó un directorio temporal privado en el disco principal. GitHub Actions no comenzó por un bloqueo de facturación de la cuenta: no equivale a una aprobación de CI.

## Resultados

Referencia: **0/5**. Totales del proveedor: 78 llamadas, 626.763 tokens de entrada, 188.884 de salida, **US$0,05044451**. Tres fallos de compactación sin cola y dos agotamientos de recuperación por falta de progreso. Los segundos nativos no incluyen descarga ni preparación. La evaluación de desarrollo aparece abajo; falta la confirmación con tareas nuevas.
| Task | Main pass | Native seconds | Input tokens | Output tokens | USD |
|---|---:|---:|---:|---:|---:|
| custom-memory-heap-crash | 0 | 64.1 | 95111 | 24587 | 0.007225 |
| extract-elf | 0 | 208.1 | 203788 | 66755 | 0.016982 |
| modernize-scientific-stack | 0 | 48.1 | 66619 | 20857 | 0.005850 |
| sanitize-git-repo | 0 | 112.0 | 173451 | 40970 | 0.011685 |
| schemelike-metacircular-eval | 0 | 102.6 | 87794 | 35715 | 0.008702 |


## Ciclo 1: admisión de contexto

Antes de evaluar la variante: las dos primeras tareas de referencia fallaron (0/2). La extracción terminó con `No completed batch to compact`; la reparación C++ repitió lecturas después de compactar y agotó la recuperación. La variante reserva hasta 8.192 tokens para salida al decidir cuándo compactar; mantiene el máximo configurado y limita cada solicitud a la capacidad disponible. También admite un corte completo si no cabe una cola reciente y pide un resumen conciso. Predicción: menos compactaciones y lecturas repetidas, sin fallo de cola vacía. Riesgo: puede aumentar el truncamiento si queda menos espacio para salida. No se cambia proveedor, modelo, capacidad configurada ni máximo de salida.

Contraste bibliográfico: [ARC](https://arxiv.org/abs/2607.25066) separa observaciones archivadas del contexto activo, pero evalúa preguntas de contexto largo, no este benchmark de programación. [Context Compaction Theory](https://arxiv.org/abs/2608.01326) formaliza selección y resumen; no prescribe reservar 8.192 tokens. Ese valor es una hipótesis de ingeniería que se prueba aquí. El antecedente de 2025 [The Complexity Trap](https://arxiv.org/abs/2508.21433) motiva comparar el enmascarado con resúmenes costosos sin asumir que resumir es mejor.

## Resultados de desarrollo del ciclo 1

La variante resolvió **2/5**, frente a **0/5** de main. Totales: 306 llamadas al proveedor, 3.871.562 tokens de entrada, 621.873 de salida y **US$0,20582481**. El tiempo nativo acumulado fue 1.689,5 segundos frente a 535,0. Los intentos más largos cuestan más que fallos tempranos del arnés; no es una mejora global de velocidad.

| Task | Candidate pass | Native seconds | Input tokens | Output tokens | USD |
|---|---:|---:|---:|---:|---:|
| custom-memory-heap-crash | 0 | 576.1 | 1250914 | 187942 | 0.062188 |
| extract-elf | 1 | 454.2 | 903475 | 163516 | 0.052119 |
| modernize-scientific-stack | 1 | 14.8 | 80982 | 2585 | 0.002783 |
| sanitize-git-repo | 0 | 588.1 | 1400213 | 251774 | 0.079315 |
| schemelike-metacircular-eval | 0 | 56.3 | 235978 | 16056 | 0.009420 |

Fallos restantes: C++ compiló, pero falló en ejecución release y memoria; la tarea del intérprete terminó pidiendo una aclaración equivocada sin crear el archivo solicitado; la limpieza Git agotó la recuperación por falta de progreso. La revisión de código también encontró que `command_run` cuenta como efecto aunque su código de salida sea distinto de cero; esta limitación no se cambió en la variante congelada. No se incorporó contenido oculto del verificador a prompts ni skills.

El límite nativo configurado es de 900 segundos por tarea en ambas variantes, incluso cuando la tarea oficial permite más tiempo. Las puntuaciones corresponden a esta configuración y subconjunto, no a un resultado oficial con todo el presupuesto. El equipo es compartido, por lo que los tiempos son descriptivos. Los archivos congelados `source.json` vinculan cada hash de binario con el commit real de su código, separado del checkout del ejecutor.

## Corrección de transporte durante la confirmación

`prove-plus-comm` usa `/workspace`, no `/app`. El adaptador inicial falló antes de realizar ninguna solicitud al proveedor. Ahora obtiene el directorio oficial del contenedor mediante `pwd`, sin crear ni modificar archivos de la tarea. Pasaron dos pruebas del adaptador. Esos fallos de preparación sin inferencia requieren intentos de reemplazo y no deben contarse como fallos del modelo ni como evidencia de mejora. Se mantienen los hashes de binarios y los parámetros de inferencia.

## Auditoría de skills

El usuario incluyó explícitamente la mejora de skills en el alcance experimental. En las diez ejecuciones de desarrollo (cinco por versión), los registros nativos no contienen llamadas a `skill_search` ni `skill_load`. Las solicitudes de inferencia confirman que estaban disponibles ambos esquemas y el catálogo manual. Esto demuestra falta de uso en esta muestra, no que el contenido de los skills sea ineficaz. Ampliar el catálogo por sí solo no demuestra una mejora si no se carga ninguno.

Una versión posterior podrá probar criterios de selección más claros, recomendaciones relevantes y breves, o revisiones de procedimientos justificadas por la ejecución. Deberá preservar los skills editados por el usuario, la carga bajo demanda y datos nuevos para la confirmación. Los binarios y skills de la confirmación actual siguen congelados. [SkillRevise](https://arxiv.org/html/2606.01139v3) propone diagnosticar si un fallo es atribuible al skill antes de modificarlo y validar la revisión ejecutándolo. [Group of Skills](https://arxiv.org/html/2605.06978v1) motiva explicitar el procedimiento de entrada y sus comprobaciones de apoyo. Ninguno demuestra una mejora para Fénix o Nemotron; no se incorporarán soluciones específicas del benchmark a los skills.

## Candidato del ciclo 2: selección y procedimientos de skills

Preparado a partir de trazas de desarrollo mientras la confirmación del ciclo 1 sigue congelada: hasta dos sugerencias léxicas de metadatos por tarea, conservando los textos bajo demanda; un procedimiento de implementación centrado en el entregable ejecutable solicitado; mejoras en reparación de fallos de memoria y en la distinción entre archivos actuales e historial de Git. Hipótesis: más cargas pertinentes de skills, menos observaciones repetidas y menos entregables ausentes. Riesgos: sugerencias irrelevantes, costo adicional de contexto/herramientas y regresiones. Es una intervención conjunta: no permite aislar selección y contenido.

Los skills originales de reparación y Git se actualizan solo si su contenido coincide exactamente; se conservan ediciones personales y el estado desactivado. El validador genérico de Codex rechaza las extensiones existentes de Fénix `version` y `required_tools`, por lo que no se considera una validación aprobada; las pruebas del parser y de migración de Fénix cubren su formato nativo. Los binarios y configuraciones de la confirmación actual no cambian. Los resultados del benchmark para este candidato están pendientes.

Una prueba de recuperación detectó que palabras genéricas de la descripción desplazaban un término específico del contenido. El candidato ahora satura conjuntamente la frecuencia ponderada y aplica frecuencia inversa de documento y normalización por longitud. La recuperación es léxica y determinista, sin embeddings ni inferencia adicional. Las compilaciones locales concurrentes en el servidor compartido también pueden afectar la latencia; los tiempos son descriptivos, no una demostración de mayor velocidad.

Validación del candidato: 60 pruebas unitarias del agente aprobadas (una dependiente del entorno omitida), 22 pruebas de integración del motor aprobadas y Clippy del workspace sin advertencias. Las cinco pruebas finales de skills, incluidas consultas Unicode acotadas y skills desactivados o personalizados, pasaron tras el último ajuste. La interfaz no cambió y sus pruebas no se repitieron para este candidato.

## Auditoría de aclaraciones

Algunas ejecuciones de confirmación del ciclo 1 terminaron con preguntas en texto, sin el entregable solicitado; el motor marcó esos turnos como completados, pero el verificador rechazó el resultado. Una pregunta mediante herramienta produjo `requires_user_input`, también sin resolver en este benchmark desatendido. Son casos distintos de los fallos de preparación. Algunas preguntas se refieren a hechos disponibles en los archivos o a decisiones internas de implementación; las dudas reales pendientes sobre requisitos del usuario deben seguir comunicándose.

[Ask or Assume?](https://arxiv.org/html/2603.26233v1) evalúa requisitos ausentes con un usuario simulado que dispone de la información original omitida. Respalda las aclaraciones pertinentes, no la supresión de todas las preguntas, y no valida una solución específica para Nemotron. [SWE-RPG](https://arxiv.org/html/2608.09072v1) motiva diagnosticar requisitos, planificación y entrega de artefactos por separado del acierto final. Una intervención posterior deberá mejorar la investigación y distinguir intención del usuario de dudas técnicas, sin inventar requisitos ni proporcionar respuestas del verificador.

## Resultado de confirmación del ciclo 1

Terminaron los 40 intentos válidos: diez tareas, dos repeticiones por versión. La base resolvió **0/20 (0%)**; el ciclo 1 resolvió **2/20 (10%)**, las dos repeticiones de `prove-plus-comm`. Cada repetición pasó de 0/10 a 1/10. Es una mejora limitada que se repite, pero **no alcanza la meta predefinida de dos aciertos adicionales por cada diez tareas en promedio**. La investigación continúa; el avance de desarrollo de 2/5 no representa rendimiento general. Los dos fallos originales de preparación sin inferencia fueron excluidos y reemplazados; ambos reemplazos fallaron.

| Métrica, 20 intentos válidos por versión | Base | Ciclo 1 |
|---|---:|---:|
| Aciertos | 0 | 2 |
| Llamadas al proveedor | 707 | 1.334 |
| Tokens de entrada | 6.037.494 | 16.999.044 |
| Tokens de salida | 1.533.464 | 1.954.413 |
| Costo registrado del proveedor, USD | 0,42910006 | 0,76266231 |
| Segundos nativos acumulados | 3.979,4 | 6.603,2 |

Todas las llamadas registradas tienen costo numérico. Los segundos nativos se suman entre intentos; no son el tiempo de reloj del estudio concurrente. Preparación y verificación se miden aparte. Se comprobaron nuevamente los hashes de los binarios.

| Categoría de resultado | Base | Ciclo 1 |
|---|---:|---:|
| Fallo de compactación sin tramo reciente | 9 | 0 |
| Recuperación agotada por falta de avance | 8 | 2 |
| Límite de llamadas del agente | 0 | 4 |
| Tiempo agotado | 1 | 1 |
| Requiere respuesta del usuario | 0 | 1 |
| Turno completado, verificación fallida | 2 | 10 |
| Verificador oficial aprobado | 0 | 2 |

El intento que necesita respuesta cuenta como fallo aunque no tenga recompensa numérica del verificador; no se excluye como infraestructura. Eliminar interrupciones prematuras permitió trabajar más, pero no hizo correctos la mayoría de los resultados y elevó tiempo y costo totales. La evaluación del candidato de skills ya está en marcha sobre las cinco tareas originales de desarrollo.

## Candidato del ciclo 3: conservar evidencia y contar una vez la repetición del proveedor

La capa de procesos conserva el inicio y el final de una salida, pero el motor volvía a recortar ese resultado por el principio. Así podía ocultar errores finales del compilador o de las pruebas. El candidato conserva ambos extremos, metadatos de ejecución/salida/cursor, las instrucciones completas del repositorio y un enlace al registro íntegro redactado. Las pruebas cubren diagnósticos finales, recuperación del centro omitido, secretos y límites UTF-8.

La estimación del contexto también serializaba el contenido normalizado del asistente junto con los datos opacos del proveedor, aunque los adaptadores envían estos últimos en lugar de ambas representaciones. El candidato cuenta la representación mayor más un margen. Sigue contando y conservando el razonamiento oculto o firmado; sigue siendo una estimación por bytes, no la facturación. Predicción: menos compactaciones innecesarias y menos diagnósticos perdidos. No cambia modelo, tamaño configurado del contexto, límite de salida, permisos ni tiempo máximo. Es una intervención mecánica combinada aún no confirmada.

La iteración basada en trazas sigue [AHE](https://arxiv.org/abs/2604.25850); [Context Compaction Theory](https://arxiv.org/abs/2608.01326) motiva comprobar la conservación de información, no una heurística concreta. Ninguna fuente demuestra que este candidato mejore Nemotron. Primero se evaluará en desarrollo y solo después, si procede, en nuevas tareas reservadas.

Validación del ciclo 3: pasaron 64 pruebas unitarias del agente (una dependiente del entorno omitida), las 23 pruebas de integración y Clippy de todo el workspace sin advertencias. La nueva prueba confirma que el modelo recibe el diagnóstico final mientras el centro completo redactado permanece recuperable en el registro.

## Carrera al recoger el informe final: corrección pendiente

Una auditoría posterior encontró que el evaluador puede escribir `ready.json` y terminar entre las comprobaciones de disponibilidad y de proceso vivo. Dos ejecuciones tenían informes completos pero nunca recibieron calificación oficial: `write-compressor__FUsx9wm` del ciclo 1 y `sanitize-git-repo__zeZS8N6` del ciclo 2. El adaptador vuelve a comprobar el marcador final después de terminar el proceso; pasaron cuatro pruebas de transporte. Se reemplazarán estos dos intentos con sus binarios y configuraciones originales. Por tanto, la tabla anterior del ciclo 1 es provisional (contaba conservadoramente un intento sin calificar como fallo), no la comparación final aceptada. El gasto de los intentos excluidos se conservará por separado; esperar al usuario no constituye por sí mismo un fallo de infraestructura.

El resultado preliminar de desarrollo del ciclo 2 es 1/5, con Git pendiente de reemplazo. Sus cinco intentos iniciales consumieron 179 llamadas, 2.186.996 tokens de entrada, 294.779 de salida, US$0,10604281 y 856,8 segundos nativos acumulados. Ninguno buscó ni cargó una skill. La modernización pasó aunque el agente pidió aclaración. Estos datos preliminares no justifican confirmación ni adopción. El ciclo 3 está congelado en `f1efd24` y ejecuta las mismas cinco tareas de desarrollo.
