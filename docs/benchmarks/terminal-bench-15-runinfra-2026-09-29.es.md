# Terminal-Bench 4.0: 15 tareas con GLM-5.3-Flash y Nemotron en RunInfra

[English](terminal-bench-15-runinfra-2026-09-29.md) · [Registro de intentos](terminal-bench-15-runinfra-2026-09-29.json)

Estado: completo. Ejecución original: **1/15** tareas aprobadas por el verificador oficial.
Tras correcciones generales del arnés, repeticiones etiquetadas de las mismas tareas:
**5/15**. Las repeticiones nunca sustituyen los resultados originales; usan binarios
congelados distintos y no son una ablación controlada.

## Resultados (verificador oficial; los conteos de pruebas son diagnósticos parciales)

| Tarea | Original (válido) | Tras los cambios |
| --- | --- | --- |
| `atrx-vep-crispr` | 0 (12/16) | aprobada (16/16) |
| `batched-eval-parity` | 0 (0/5) | aprobada (5/5) |
| `bun-sourcemap-leak` | 0 (31/36) | 0 (27/36) |
| `cargo-flight-dispatch` | 0 (9/27) | 0 (22/27) |
| `coq-block-bound` | 0 (2/3) | 0 (2/3) |
| `embedding-drift-monitor` | 0 (10/11) | aprobada (11/11) |
| `fin-saccr-rwa` | 0 (0/24) | aprobada (24/24) |
| `foodstuff-beta-activity` | 0 (0/13) | 0 (10/13) |
| `freight-dispatch-shift` | 0 (sin CLI) | 0 (101/232 puntos) |
| `glycan-ms2-elucidation` | 0 (2/12) | 0 (11/12) |
| `heat-pump-warranty` | 0 (0/20 casos) | 0 (15/20 casos) |
| `kv-live-surgery` | 0 (0/1) | 0 (2/6) |
| `music-harmony` | 0 (sin entrega) | 0 (3 infracciones) |
| `mvcc-lsm-compaction` | aprobada (15/15) | aprobada (15/15) |
| `nextjs-performance` | 0 (1/5) | 0 (1/5) |

## Método

Terminal-Bench 4.0 `v4.0.0` (`452bf305…`), Harbor 0.23.0 y el binario Rust `fenix eval` dentro
de cada contenedor oficial; un intento por ejecución; instrucciones, recursos, plazos y
verificadores oficiales sin cambios; ejecución secuencial en un host de 2 vCPU. La selección se
fijó antes de ver resultados: las primeras 15 tareas en orden lexicográfico con ≤2 CPU,
≤4096 MiB y sin GPU. Las tres tareas de FreeCAD no eran evaluables en la fecha (su imagen oficial
de verificación no se construye porque pip no puede reemplazar `vtk 9.2.6` de conda) y se
reemplazaron por las siguientes elegibles antes de conocer sus resultados.

Los roles reflejan producción: inteligente/maestro `glm-5-3-flash` y rápido/conversacional
`nemotron-3-5-lightning-30b`, ambos en RunInfra, con razonamiento predeterminado, contexto de
200.000 tokens y salida de 16.384. La evaluación nativa ahora configura el rol rápido
(`EvaluationConfig.fast`); la narración del progreso lo usó en cada intento. Las claves quedaron
en el host detrás de un proxy de registro.

## Hallazgos y cambios

La mayoría de los fallos originales fueron del arnés. El razonamiento agotaba a menudo el
límite de salida; un tope de tres recuperaciones por `output_length` durante toda la tarea
abortaba trabajo productivo, y el cliente rechazaba respuestas cuyo razonamiento de chat en
texto plano mencionaba una cabecera `Authorization:`. Cambios: presupuesto solo para
generaciones consecutivas sin acción, guía escalada, incorporación única del último
razonamiento en texto plano como notas no verificadas (8/9 acciones ejecutables al repetir
contextos grabados frente a 0/9 antes), recuperación de respuestas de chat solo con
razonamiento, enmascaramiento de la repetición de chat sin firma y permisos según la umask
para archivos nuevos del espacio de trabajo. Ver
[evaluación nativa](../es/native-evaluation.md).

Las incidencias de infraestructura se separaron de los resultados del agente: la `umask 077`
heredada hizo ilegibles los archivos de una tarea para un verificador sin privilegios (se repitió
con el binario original), una construcción de verificación sin espacio en disco (intento
archivado) y una superposición accidental de 30 minutos (la tarea original afectada se repitió
de forma secuencial).

Uso en 37 intentos: 1.613 solicitudes a GLM y 835 a Nemotron, $6.67 de costo
informado por el proveedor (7 respuestas `503 limiter_unavailable` del proveedor en todo el tráfico, 2 al inicio y archivadas como infraestructura).
