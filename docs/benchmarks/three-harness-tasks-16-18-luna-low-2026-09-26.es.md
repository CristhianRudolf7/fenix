# Comparación de arneses: tareas 16–18 de Terminal-Bench

[English](three-harness-tasks-16-18-luna-low-2026-09-26.md) · [Datos medidos](three-harness-tasks-16-18-luna-low-2026-09-26.json)

GPT-6 Luna low, servicio estándar, sin Flex. Un intento por combinación. Resultados: fenix 1/3, codex 3/3, opencode 2/3.

Cambios evaluados: aprendizaje auxiliar desactivado antes de evaluar; referencias a solicitudes repetidas exactamente; carga opcional de skills para operaciones conocidas e inspecciones independientes agrupadas. Los tres adaptadores mantienen el mismo contexto general de ejecución. Validación local: 255 pruebas del agente y cuatro del adaptador aprobadas, tres opcionales omitidas y compilación correcta.

En las tres evaluaciones nativas no hubo solicitudes de extracción de skills; todas las llamadas registradas devolvieron consumo. Se eliminaron copias del encabezado y enrutamiento, pero el contrato persistente conserva otra copia del objetivo. Estos fallos muestran que reducir llamadas debe preservar inspecciones y comprobaciones pertinentes.

| Tarea | Arnés | Pruebas | Tiempo (s) | USD estimados | Tokens entrada | Tokens salida |
|---|---|---:|---:|---:|---:|---:|
| 16. large-scale-text-editing | fenix | 5/5 | 206.02 | 0.00919554 | 125,863 | 1,753 |
| 16. large-scale-text-editing | codex | 5/5 | 208.75 | 0.00544949 | 126,526 | 2,610 |
| 16. large-scale-text-editing | opencode | 5/5 | 196.68 | 0.00322518 | 52,883 | 1,930 |
| 17. log-summary-date-ranges | fenix | 1/2 | 32.98 | 0.00514585 | 75,338 | 1,101 |
| 17. log-summary-date-ranges | codex | 2/2 | 14.14 | 0.00369942 | 46,952 | 689 |
| 17. log-summary-date-ranges | opencode | 1/2 | 22.78 | 0.00208226 | 20,106 | 408 |
| 18. constraints-scheduling | fenix | 2/3 | 34.98 | 0.00489940 | 96,511 | 658 |
| 18. constraints-scheduling | codex | 3/3 | 15.07 | 0.00352835 | 33,258 | 1,050 |
| 18. constraints-scheduling | opencode | 3/3 | 39.57 | 0.00382673 | 63,082 | 1,936 |

≥ significa mínimo registrado cuando falta consumo de alguna solicitud. Los tokens de esos casos también pueden ser parciales. Las esperas por cuota y errores del transporte se conservan en el JSON; no se atribuyen automáticamente al arnés.

El tiempo corresponde a ejecución del agente: excluye descarga/preparación del contenedor, instalación del arnés y verificador externo. Incluye esperas del proveedor. Los tokens de entrada se acumulan entre llamadas; no representan una sola ventana de contexto. El coste de una tarea abandonada no es un coste por tarea resuelta.

[Precio oficial](https://developers.openai.com/api/docs/models/gpt-6-luna): por millón de tokens, entrada $0.10; lectura de caché $0.01; escritura de caché $0.125; salida $0.50. La entrada normal descuenta ambas categorías de caché. Estimación por uso, no factura; sin infraestructura ni impuestos.

## 16. Transformación de un millón de filas con macros de Vim restringidas

Crear tres registros distintos no vacíos y transformar input.csv en expected.csv usando solo los comandos permitidos. Los tres aprobaron las cinco comprobaciones oficiales.

- **Fénix:** inspeccionó entorno y muestras, corrigió una llamada inválida a tool_program con nombre de herramienta incorrecto, escribió las macros y ejecutó la transformación completa. Esperó al proceso correcto y observó código 0 de Vim y coincidencia byte a byte. Descubrió command_cancel, pero no canceló la transformación. No necesitó cargar skills por formalidad ni hubo aprendizaje posterior.
- **Codex:** inspeccionó muestras, escribió macros y verificó el resultado. Su traza incluye una espera inválida con identificador de ejemplo, inspección del proceso y otra invocación de Vim; después utilizó correctamente el identificador de sesión. Son pasos concretos de manejo de procesos, no prueba de diferencias de inteligencia del modelo.
- **OpenCode:** escapó incorrectamente el comodín de una expresión, inspeccionó la discrepancia, corrigió la macro y repitió Vim con éxito.
- **Oportunidad pendiente:** validar primero una muestra pequeña independiente y después la comparación completa obligatoria. Agrupar pasos independientes y llamar directamente con el esquema correcto; usar tool_program para un solo paso añade oportunidades de error. Se separan tiempo del agente y verificador externo. El resultado no mide de forma aislada el efecto de reducir contexto.

## 17. Conteo de severidades en logs por cinco rangos de fechas

Generar 15 filas de CSV con ERROR, WARNING e INFO para hoy, últimos siete/treinta días inclusivos, mes actual y total, usando la fecha de referencia indicada. Codex aprobó ambas comprobaciones; Fénix y OpenCode crearon el archivo, pero fallaron el conteo.

- **Fénix:** listó archivos, inspeccionó el entorno y contó palabras de severidad en todo el contenido sin leer primero registros representativos. Se recuperó de un error de Python al manejar el periodo total y comprobó encabezado, filas, orden y números no negativos. Esas comprobaciones de formato no validaban que cada coincidencia perteneciera al campo de severidad. Para ERROR de hoy obtuvo 414 en lugar de 370.
- **Codex:** leyó muestras, identificó el campo de severidad entre corchetes y generó el CSV correcto con cuatro llamadas a la API.
- **OpenCode:** contó todas las apariciones de palabras con el mismo enfoque demasiado amplio y obtuvo el mismo conteo incorrecto, también con cuatro llamadas.
- **Oportunidad pendiente:** agrupar descubrimiento de archivos y lectura de una muestra; comprobar un registro cuyo mensaje mencione otra severidad. Es una prueba pequeña ligada al requisito. El mismo número de llamadas produjo resultados distintos: ahorrar contexto no debe eliminar evidencia necesaria. Una ronda no pareada no permite atribuir causalmente el fallo a los cambios del prompt.

## 18. Reunión con conflictos de calendario y restricciones de disponibilidad

Leer tres calendarios inmutables y escoger la primera reunión de una hora que cumpla disponibilidad, conflictos y desempates por preferencias. Codex y OpenCode aprobaron las tres comprobaciones. Fénix aprobó dos, pero falló la restricción horaria obligatoria.

- **Fénix:** leyó los calendarios, se recuperó de otra llamada a tool_program con nombres de herramientas incorrectos y escribió el lunes 15 a las 09:00 UTC. Bob tenía prohibidas las reuniones antes de las 10:00. Esa instrucción sí estaba en la solicitud enviada al modelo: no se perdió por contexto ni compactación. La evidencia final comentaba conflictos entre reuniones, pero no comprobaba cada restricción de disponibilidad; crear el archivo no demostraba que el horario fuera válido.
- **Codex:** agrupó la lectura de los calendarios, escribió el miércoles 17 a las 11:00 UTC y mostró el archivo. Tres llamadas a la API y resultado oficial correcto, aunque sin una comprobación independiente exhaustiva visible.
- **OpenCode:** leyó los calendarios, mantuvo una lista de pasos, eligió el mismo horario correcto y ejecutó comprobaciones. Sin embargo, su comprobación de integridad incluía una alternativa siempre verdadera y la de conflictos mezclaba formato HHMM con minutos desde medianoche. Esas pruebas propias no eran evidencia fiable, aunque el resultado pasó el evaluador oficial.
- **Oportunidad pendiente:** extraer las restricciones obligatorias y comprobar el horario contra cada una y contra los intervalos ocupados antes de cerrar. Basta una comprobación determinista pequeña, sin otro modelo revisor. Separar preferencias de reglas, usar una sola unidad de tiempo y evitar aserciones siempre verdaderas. También aclarar los nombres admitidos por tool_program para que agrupar pasos no produzca reintentos.

## Protocolo y límites

Codex 0.157.1; OpenCode 1.18.32; Fenix SHA256 `8f42d79a3c295eba61395cc0f671421da3613bfa721ba59fe9461a5cdbd07528`; dataset `7131e4375048a0e408a8fb404b5f499d726b695b`.

Contenedor oficial nuevo por intento, 1 CPU, 2 GiB y 900 segundos de plazo del agente. Orden rotado: Fénix/Codex/OpenCode; Codex/OpenCode/Fénix; OpenCode/Fénix/Codex. Se congelaron binario, adaptadores y configuración. Sin cambios durante la medición, intentos adicionales de tarea ni exposición de tests ocultos. Los tres arneses mantienen el mismo contexto general del entorno que en la ronda anterior. La evaluación nativa desactiva ahora el aprendizaje de skills en segundo plano. Se respetaron las cuotas mediante el mismo proxy. El JSON conserva respuestas, tiempos, tokens, esperas y rutas privadas de las trazas.

No se desplegó ni se modificó el agente durante esta ronda. Tres tareas no permiten concluir superioridad general; las mejoras respecto de las tareas 13–15 no se midieron con una repetición pareada.
