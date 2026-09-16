# Evolución del arnés con Nemotron — auditoría final

[English](../../en/audits/nemotron-harness-evolution.md) · [Evidencia por intento](../../en/audits/nemotron-harness-evolution-results.csv)

**No se alcanzó el objetivo de adopción.** El primer candidato de contexto mejoró la confirmación de **0/20 a 2/20**, ambas repeticiones de una tarea. Tres ciclos posteriores no demostraron una mejora reproducible adicional respecto a ese candidato, por lo que se aplicó la regla de parada acordada. El PR experimental continúa en borrador y no se fusiona automáticamente.

El estudio del 15–16 de septiembre de 2026 ejecutó **70 intentos: 65 válidos y cinco exclusiones explícitas de transporte**. El gasto reportado por el proveedor fue **US$2.13128346**, incluidos **US$0.07315928** de intentos excluidos con inferencia. Incluye comprobaciones de conexión y resúmenes, no costos del equipo ni del desarrollo. Todas las llamadas tienen facturación numérica; los dos fallos de preparación sin llamadas no generaron gasto de inferencia.

## Protocolo

Referencia: main `574e390`; evaluador nativo Rust real; Terminal-Bench 2.1 en `7131e4375048a0e408a8fb404b5f499d726b695b`; Harbor 0.23.0. Todas las variantes usan RunInfra `nemotron-3-5-lightning-30b`, pensamiento predeterminado del proveedor, la misma configuración e instrucciones/verificadores oficiales. No se entregaron pruebas ocultas ni soluciones de referencia a agentes o skills. Los resultados del verificador se inspeccionaron después, solo para auditoría.

Capacidad de contexto configurada: 32.768 tokens; máximo de salida: 16.384; máximo de pasos del modelo: 128, con recuperación acotada. El límite nativo es 900 segundos, incluso cuando la tarea oficial permite más. Se ejecutan como máximo dos intentos en paralelo. Preparación y verificación se separan del tiempo nativo. La carga compartida de compilación/Docker limita comparar velocidades; el ciclo 4 empezó después de compilar. Son resultados de un subconjunto controlado, no una puntuación oficial con el presupuesto completo.

Desarrollo utiliza las mismas cinco tareas: `custom-memory-heap-crash`, `extract-elf`, `modernize-scientific-stack`, `schemelike-metacircular-eval`, `sanitize-git-repo`. Confirmación utiliza diez tareas nuevas preseleccionadas, con dos intentos por versión: A — `build-pov-ray`, `prove-plus-comm`, `regex-chess`, `sqlite-with-gcov`, `mailman`; B — `circuit-fibsqrt`, `make-mips-interpreter`, `write-compressor`, `sparql-university`, `raman-fitting`.

Se exigían **dos aciertos adicionales por cada diez tareas en promedio**, mejora positiva en ambas repeticiones y pruebas de regresión aprobadas. Las ganancias reales fueron **+1/10 y +1/10**. La reserva adicional de tareas nuevas no se utilizó: los candidatos posteriores no mejoraron el desarrollo del ciclo 1. La muestra pequeña, las tareas repetidas y las respuestas estocásticas no establecen superioridad estadística general.

## Resultados de desarrollo

| Variante / commit | Aciertos | Llamadas | Tokens entrada | Tokens salida | USD | Segundos nativos |
|---|---:|---:|---:|---:|---:|---:|
| main / `574e390` | 0/5 | 78 | 626,763 | 188,884 | 0.05044451 | 535.0 |
| cycle1 / `ea8b673` | 2/5 | 306 | 3,871,562 | 621,873 | 0.20582481 | 1,689.5 |
| cycle2 / `d95f8c1` | 1/5 | 158 | 1,906,808 | 253,367 | 0.09269777 | 748.0 |
| cycle3 / `f1efd24` | 1/5 | 328 | 4,293,553 | 633,872 | 0.21906277 | 2,149.7 |
| cycle4 / `4d4b534` | 2/5 | 402 | 5,304,397 | 768,083 | 0.26310238 | 2,499.1 |

Cada fila contiene cinco intentos válidos, incluidos los reemplazos declarados; el gasto excluido se informa por separado. Los segundos se suman: no son el tiempo de reloj de la ejecución concurrente. Los ciclos 1 y 4 aprobaron extracción y modernización. Los ciclos 2 y 3 aprobaron solo modernización. El ciclo 4 igualó la mejor puntuación con mayor gasto y tiempo nativo acumulado.

No hubo llamadas a `skill_search` ni `skill_load` en los **25 intentos válidos de desarrollo**, aunque catálogo y esquemas estaban disponibles. Esto demuestra falta de uso en esta muestra, no que los textos de las skills sean ineficaces. Hubo una búsqueda en la confirmación de main, sin cargar una skill. Ampliar el catálogo por sí solo no demostró beneficio.

## Resultados de confirmación

| Métrica, 20 intentos válidos | Main | Ciclo 1 |
|---|---:|---:|
| Aciertos | 0 | 2 |
| Llamadas al proveedor | 707 | 1,387 |
| Tokens de entrada | 6,037,494 | 17,730,132 |
| Tokens de salida | 1,533,464 | 2,065,696 |
| Costo reportado, USD | 0.42910006 | 0.79789188 |
| Segundos nativos acumulados | 3,979.4 | 6,907.5 |

Solo pasó `prove-plus-comm`, en ambas repeticiones del ciclo 1. Eliminar fallos prematuros de compactación permitió avanzar más, pero la mayoría de las tareas siguieron fallando y aumentaron costo y tiempo.

| Resultado | Main | Ciclo 1 |
|---|---:|---:|
| Fallo de compactación sin cola | 9 | 0 |
| Recuperación sin progreso agotada | 8 | 2 |
| Límite de llamadas al modelo | 0 | 4 |
| Tiempo agotado | 1 | 1 |
| Turno completado, verificador fallido | 2 | 11 |
| Verificador aprobado | 0 | 2 |

## Intervenciones y límites observados

1. **Admisión de contexto y compactación sin cola (`ea8b673`).** Reservar margen útil de razonamiento sin apartar siempre toda la salida máxima; mantener el límite según capacidad estimada restante. Permitir un corte completo cuando no cabe una cola reciente y solicitar resúmenes breves. Desaparecieron los nueve fallos observados de cola vacía, pero la confirmación quedó bajo el objetivo.
2. **Selección de skills y procedimientos (`d95f8c1`).** Hasta dos sugerencias léxicas de metadatos, una skill de implementación y revisión de reparación/Git. Los textos siguen bajo demanda. Frecuencia ponderada, frecuencia inversa y normalización por longitud mejoran las pruebas de búsqueda. La migración por coincidencia exacta conserva ediciones personales y desactivaciones. Al no cargarse skills en desarrollo, estas ejecuciones no establecen la calidad de sus procedimientos.
3. **Evidencia de herramientas y cálculo de contexto (`f1efd24`).** Conservar inicio/final, metadatos de ejecución/salida/cursor, instrucciones del repositorio y acceso a resultados completos redactados. Contar la representación mayor entre contenido normalizado y datos del proveedor, sin sumarlas; conservar razonamiento oculto o firmado. Pasaron las regresiones, pero no mejoró la puntuación. La estimación por bytes no es un tokenizador ni la facturación.
4. **Evidencia antes de preguntar y escritura básica visible (`4d4b534`).** Consultar instrucciones originales y hechos locales antes de preguntar, manteniendo toda duda no resuelta. Mostrar esquemas de creación/edición básica sin cambiar permisos ni el descubrimiento avanzado. Distinguir un fallo del diagnóstico de un fallo del programa en la skill de entorno. Recuperó 2/5, sin superar el ciclo 1.

Los fallos del ciclo 4 muestran lo pendiente: C++ agotó el tiempo; Scheme agotó la recuperación sin progreso; Git realizó ediciones verificadas y declaró la tarea terminada, pero falló la eliminación y sustitución de secretos, aunque conservó archivos ajenos. El éxito de una herramienta no equivale al de la tarea. La búsqueda informó 531 archivos y un error omitido, con reglas de exclusión activas; el código también aplica un filtro de 1 MiB por archivo. Cero coincidencias no demuestra cobertura completa. Es una limitación de cobertura, no prueba de qué filtro causó el secreto restante.

Hipótesis posteriores, **no implementadas ni validadas en este estudio**: medir mejor el progreso ante comandos fallidos repetidos (nuevos IDs pueden parecer avance); explicitar cobertura/exclusiones de búsqueda; esperar algo más, de forma acotada, al iniciar comandos para reducir consultas de estado. La espera actual de 100 ms devolvió `running` en 11 de los primeros 19 resultados de comandos de extracción del ciclo 4. No se deben llamar redundantes a esas consultas: los procesos aún figuraban en ejecución.

## Correcciones de infraestructura y exclusiones

- Dos intentos originales de demostración en main (`3YsqxDh`, `74yYYdz`) asumían `/app`, pero el contenedor usaba `/workspace`. No hicieron llamadas al proveedor. El adaptador consulta `pwd`; ambos reemplazos fallaron.
- Una carrera entre el marcador final y la terminación del proceso impidió calificar tres informes completos: `write-compressor__FUsx9wm` del ciclo 1, `sanitize-git-repo__zeZS8N6` del ciclo 2 y `custom-memory-heap-crash__3jTn3wC` del ciclo 3. El adaptador vuelve a comprobar el marcador después de terminar el proceso. Los reemplazos con los mismos binarios fallaron el verificador. El gasto original sigue incluido. Esperar al usuario no es por sí solo una exclusión: una modernización pasó pese a ese estado.
- Los costos ausentes, nulos, booleanos o no finitos quedan desconocidos, no como costo cero. Pasaron cinco pruebas de transporte/facturación.
- Los arranques explícitos mediante glibc hacían que el manifiesto antiguo calculara el hash del cargador. El protocolo externo del binario congelado sí identifica Fénix y es la referencia de estas ejecuciones. Una corrección posterior solo de metadatos registra el hash del programa y ambas rutas; no se reescriben manifiestos históricos. No constituye un candidato nuevo del benchmark.

## Validación y procedencia

Main pasó 239 pruebas Rust y 13 de navegador; se omitieron cuatro dependientes del entorno. El ciclo 4 pasó **114 pruebas del agente**, incluidas 23 de ejecución y las seis de revisión de planes; una quedó omitida por el entorno. Una ejecución general anterior agotó dos tiempos de revisión bajo carga. Ambas pasaron posteriormente y las seis pasaron otra vez aisladas en 9,11 segundos, sin cambiar código ni límites. Pasaron Clippy del workspace, formato y compilación del ciclo 4. No cambió la web ni se repitieron sus pruebas.

La validación final del informe pasó las tres pruebas unitarias de evaluación, Clippy del workspace y una prueba nativa real mediante el cargador con dos respuestas simuladas locales (sin gasto de proveedor). La prueba verificó el hash del programa y lo distinguió del cargador. Estos cambios solo del informe son posteriores a los binarios congelados; no se les atribuye una mejora adicional de rendimiento. GitHub Actions estuvo bloqueado por facturación durante el estudio; las pruebas locales no deben presentarse como CI remoto aprobado.

| Variante | Fuente | SHA-256 del programa congelado |
|---|---|---|
| main | `574e390` | `a343bd8a6b1076162ed09928b31c9e3b87504a062799acde52a54df47a10b684` |
| cycle1 | `ea8b673` | `57b21e22b3093ff88d167632588490e80d30064c1fcb8cf08c17aab14cf8fdc5` |
| cycle2 | `d95f8c1` | `0700287d8716654d86fe5e676f9da3a957e3b8b733aa9d42c3472a2e0082b9e1` |
| cycle3 | `f1efd24` | `b468ee1ad82df3eb5c873cb9b434d0c810d83a736d17a2e029a4a669b61c1bec` |
| cycle4 | `4d4b534` | `43469b49f74a9b4f9b385616adb45132d105cd9a7ea88de9d1125e74199ff3c1` |

SHA-256 de configuración: `6e124d093eb1d00885682116d7fa635125295d9fb7ae1fefdd0ac86b8b5523ae`. Se conservaron los hashes de todos los binarios/configuraciones para verificarlos. Instrucciones, artefactos y trazas API privadas siguen en `.fenix/evaluations/harness-evolution-20260915/`, ignorado por Git; el CSV solo publica métricas y procedencia seleccionadas. Las correcciones posteriores del informe no cambian los binarios evaluados.

## Base de investigación

[AHE](https://arxiv.org/html/2604.25850v4) orientó hipótesis basadas en trazas y comprobación con tareas nuevas; [SkillsBench](https://arxiv.org/abs/2602.12670), [SkillRevise](https://arxiv.org/html/2606.01139v3) y [Group of Skills](https://arxiv.org/html/2605.06978v1), la selección y revisión prudente de procedimientos. [Context Compaction Theory](https://arxiv.org/abs/2608.01326) motiva comprobar conservación de información, no la heurística concreta por bytes. [Ask or Assume?](https://arxiv.org/html/2603.26233v1) estudia aclaración con un usuario simulado que conoce información ausente; aquí no se añadió ese oráculo. [SWE-RPG](https://arxiv.org/html/2608.09072v1) orientó distinguir fallos de requisitos, planificación y artefactos. Estas fuentes motivan experimentos; ninguna demuestra una mejora de Nemotron/Fénix.
