# Evaluación nativa de Gaia2, TerminalWorld y Terminal-Bench 4.0

[English](three-benchmarks-luna-low-2026-09-26.md) · [Datos y evidencias](three-benchmarks-luna-low-2026-09-26.json)

Una tarea de cada benchmark con el motor real de Fénix y **GPT-6 Luna low**, sin
Flex ni Jev. Hubo una ejecución del agente por tarea. Esta muestra comprueba la
integración y permite auditar fallos; no mide la precisión de todo el benchmark.

| Benchmark | Resultado | Tiempo del motor | Coste API estimado |
| --- | --- | ---: | ---: |
| Gaia2-CLI | Aprobado al recalificar la misma traza: 6/6 acciones coincidentes | 14 min 04 s | US$0,0591, incluido el juez |
| TerminalWorld | Verificador aprobado; ejecución funcional parcial | 1 min 45 s | US$0,0098 |
| Terminal-Bench 4.0 | No aprobado: faltó el informe requerido | 24 min 49 s | US$0,0806 |

Los tiempos incluyen esperas por la API: aproximadamente 629, 45 y 663 segundos,
respectivamente. Gaia2 incluye además la espera final del juez. Se compartieron
servidor y cuotas; no son mediciones con hardware dedicado. Se excluye la
preparación inicial de imágenes. Con preparación del verificador y limpieza,
TerminalWorld tomó 2 min 39 s y Terminal-Bench 32 min 06 s. El controlador de Gaia2
tomó 14 min 56 s antes de la recalificación. Las llamadas del agente no tuvieron
errores HTTP.

## Qué ocurrió

**Gaia2 — contactos, noticias y archivos.** Debía localizar una noticia, identificar
dos personas por varias pistas, enviarles el documento, moverlo, eliminar dos
fotos y contar otros archivos. Al principio buscó con poca eficiencia e intentó
pedir ayuda; después continuó investigando y completó las seis acciones esperadas.

El primer dictamen no era válido para medir al agente: la biblioteca del juez
impuso una temperatura incompatible con Luna y falló antes de llamar a la API.
Se corrigió esa compatibilidad y se calificó la misma traza guardada, sin repetir
acciones ni ejecutar otra vez al agente. La calificación aprobó. Un error posterior
al exportar el informe se recuperó desde el registro durable del juez, sin otra
llamada al modelo. Se conservaron todos los resultados originales y sus hashes.

Se utilizó la variante oficial **Gaia2-CLI**, con Luna low también como juez
configurable. No equivale al protocolo original de herramientas Python de ARE ni
al juez calibrado del leaderboard.

**TerminalWorld — ejecutar una CLI.** Debía lanzar un servidor con el proveedor
de prueba de SporeStack y guardar toda la salida. Fénix consultó la ayuda, ejecutó
el comando y guardó sus diagnósticos. Faltaba un token y el agente informó el
bloqueo; no financió una cuenta ni afirmó haber creado un servidor.

El verificador solo comprueba el archivo de salida y que hubo un intento de
lanzamiento. Su aprobado no demuestra un aprovisionamiento completo. Además,
el entorno pedía cuatro CPU y este servidor tiene dos: se registró explícitamente
ese ajuste, conservando instrucciones, plazo y verificador.

**Terminal-Bench 4.0 — análisis genómico.** Debía reconstruir variantes de ATRX,
anotarlas con VEP, elegir una variante que escapase de NMD dentro de un dominio
proteico y seleccionar una diana CRISPR. Se recuperó de errores iniciales de
lectura y coordenadas y llegó a ejecutar VEP y su plugin NMD. Finalmente interpretó
sus resultados como una contradicción entre requisitos, pidió una decisión y no
generó `mutation.report.json`. El verificador independiente devolvió 0. No hubo
agotamiento de contexto ni rechazo del proveedor.

El problema observado es pasar de una interpretación científica incierta a
declarar un conflicto de requisitos sin resolver esa interpretación por otra vía.
La prueba no demuestra que los requisitos sean inconsistentes. Conviene reforzar
la comprobación de formatos, coordenadas y criterios antes de escalar esos casos,
y conservar los resultados parciales identificándolos como incompletos. No se
añadieron soluciones específicas ni verificadores ocultos a las skills.

## Consumo y límites

| Ejecución | Solicitudes API | Tokens de entrada acumulados | Tokens de salida |
| --- | ---: | ---: | ---: |
| Gaia2: agente | 32 | 1.702.562 | 7.733 |
| Gaia2: juez | 1 | 2.649 | 51 |
| TerminalWorld | 14 | 226.921 | 1.613 |
| Terminal-Bench 4.0 | 53 | 2.756.576 | 29.901 |

Los tokens de entrada se acumulan entre llamadas; no representan una sola ventana
de contexto. La solicitud más grande tuvo 93.802 tokens. La salida incluye el
razonamiento. El coste se calculó con uso real registrado, caché y escrituras de
caché, aplicando las tarifas guardadas del 26 de septiembre. Es estimado, excluye
infraestructura e impuestos y no constituye una factura. Las comprobaciones
iniciales del juez están separadas en el JSON.

Las selecciones y versiones son reproducibles: primer registro de TerminalWorld
Verified, primer directorio ordenado de Terminal-Bench 4.0 y primer identificador
ordenado del split `execution` de Gaia2-CLI. Los tres usaron el mismo binario
congelado; las revisiones, hashes y ubicaciones de evidencia están en el JSON.

## Qué quedó implementado

- Preparación de datasets fijados por revisión y ejecución mediante el motor
  nativo, usando los verificadores externos.
- Transporte interactivo de Gaia2 que conserva estado, evita entregas duplicadas
  y distingue notificaciones de respuestas a preguntas.
- Separación de permisos entre agente, escenario y juez; se comprobó que el
  agente no puede leer archivos privados del benchmark.
- Correcciones posteriores a la ejecución: permitir aclaraciones cuando existe
  transporte interactivo, entregar los fallos terminales, validar el juez antes
  de ejecutar y exportar correctamente sus resultados. No se alteraron las trazas
  ya medidas ni se repitió al agente.
- Validación final: **22 pruebas locales aprobadas**, compilación de Rust y
  comprobaciones de Python. La prueba real de compatibilidad del juez también pasó.

La imagen actualizada quedó disponible como `fenix-eval-gaia2:87ebd38-fixed`.
No se desplegaron cambios a producción.
