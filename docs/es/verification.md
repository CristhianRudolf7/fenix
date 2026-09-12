# Verificación de la aplicación inicial

[English](../en/verification.md)

Comprobaciones realizadas en Linux durante la implementación inicial, el
2026-09-12. No se incluyen credenciales ni datos personales de ejecución.

## Pruebas automáticas

La ejecución final aprobó 75 pruebas unitarias/de integración y dos ejemplos de
documentación, además de los cuatro recorridos de Chromium. También pasaron formato
y Clippy de Rust, formato/sintaxis del frontend, sintaxis de shell y validación del
acceso de escritorio.

El workspace verifica los 17 adaptadores, credenciales predeterminadas y por
proyecto, cinco protocolos, metadatos de razonamiento, errores y límites HTTP.
Las nuevas pruebas cubren cola FIFO y orden del contexto, aprobaciones obligatorias
y concretas, rechazos, cancelación, aclaraciones, protección de claves, memoria
léxica con acentos, duplicados, correcciones, archivos, comandos, timeouts,
programaciones, reinicios sin reanudación, skills y compactación conservando
historial y parejas de llamadas/resultados.

También se comprueban vinculación del propietario en Telegram, rechazo de otros
remitentes/adjuntos y protección de host/origen de la API local. Son pruebas
locales; no equivalen a una conexión real a Telegram.

Los cuatro recorridos de Chromium utilizan el servidor Rust real con un modelo
simulado: bienvenida y validación de inferencia, chat con aprobación por texto y
cola, paneles de memoria/skills/programaciones/actividad, y temas/vista móvil.
Se comprobó fondo blanco inicial incluso con un sistema en modo oscuro, persistencia
del tema y ausencia de claves guardadas en las respuestas al navegador. Se
inspeccionaron capturas claras de escritorio y oscuras de móvil.

## Llamadas reales a Luna

Se encontró la clave autorizada en el proyecto vecino, se leyó sin mostrarla y se
utilizó con `gpt-5.6-luna`, pensamiento bajo. La comprobación mínima devolvió HTTP
200. La bienvenida de la aplicación también comprobó una inferencia real y el
script `scripts/smoke-live.py` completó cinco recorridos:

| Recorrido | Llamadas al modelo | Resultado |
| --- | --- | --- |
| Memoria explícita | 5 | Guardada, comprobada y recuperada con procedencia. |
| Recuperación léxica | 2 | Preferencia ficticia esperada encontrada. |
| Creación y lectura de archivo | 5 | Una aprobación textual y contenido final exacto verificado. |
| Comando fuera de la carpeta de archivos | 3 | Una aprobación, salida esperada y código de salida 0 comprobados. |
| Recordatorio recurrente | 3 | Registro, intervalo y próxima ejecución verificados. |

No hubo errores de herramientas en estos recorridos. El informe local está en
`target/verification/live-smoke.json`, ignorado por Git. Sus datos temporales,
archivos y clave se eliminaron al finalizar. La prueba real es optativa y CI no
realiza llamadas de pago.

Sólo se comprobó acceso remoto real a **OpenAI Luna**. Los demás proveedores tienen
pruebas locales de protocolo y autenticación; se deben verificar permisos, saldo y
modelos al configurarlos con sus respectivas claves. No se proporcionó token de
bot, por lo que Telegram no tuvo una prueba remota de conexión/envío. El instalador
de escritorio se incluye con comprobaciones estáticas/de shell; no se instaló ni
activó un servicio de usuario durante esta implementación.

Es una aplicación inicial funcional: no demuestra puntuaciones en benchmarks,
autonomía durante días, seguridad exhaustiva ni decisiones correctas de cualquier
modelo en todas las tareas.
