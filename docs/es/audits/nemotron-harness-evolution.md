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
