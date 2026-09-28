# Comparación de arneses: tareas 13–15 de Terminal-Bench

[English](three-harness-tasks-13-15-luna-low-2026-09-26.md) · [Datos medidos](three-harness-tasks-13-15-luna-low-2026-09-26.json)

GPT-6 Luna low, servicio estándar, sin Flex. Un intento por combinación. Resultados: fenix 3/3, codex 3/3, opencode 3/3.

Cambios evaluados: vinculación de salida de compilación y ejecución; inspección concreta del entorno; menor descubrimiento redundante; evidencia nueva de despliegues y skills orientados al resultado ejecutado. Los tres adaptadores recibieron idéntico contexto general del entorno de ejecución. Validación local: 252 pruebas aprobadas, tres opcionales omitidas y compilación correcta.

| Tarea | Arnés | Pruebas | Tiempo (s) | USD estimados | Tokens entrada | Tokens salida |
|---|---|---:|---:|---:|---:|---:|
| 13. nginx-request-logging | fenix | 8/8 | 64.66 | 0.00994297 | 201,128 | 1,585 |
| 13. nginx-request-logging | codex | 8/8 | 49.03 | 0.00515203 | 85,830 | 1,795 |
| 13. nginx-request-logging | opencode | 8/8 | 58.96 | 0.00448368 | 103,985 | 1,618 |
| 14. openssl-selfsigned-cert | fenix | 6/6 | 35.57 | 0.00513034 | 87,690 | 1,209 |
| 14. openssl-selfsigned-cert | codex | 6/6 | 13.15 | 0.00318499 | 30,812 | 915 |
| 14. openssl-selfsigned-cert | opencode | 6/6 | 35.91 | 0.00320738 | 58,000 | 1,591 |
| 15. sqlite-db-truncate | fenix | 1/1 | 44.18 | ≥0.00658276 | 108,895 | 2,170 |
| 15. sqlite-db-truncate | codex | 1/1 | 26.24 | 0.00481085 | 69,088 | 1,957 |
| 15. sqlite-db-truncate | opencode | 1/1 | 59.91 | 0.00525387 | 101,696 | 3,916 |

≥ significa mínimo registrado cuando falta consumo de alguna solicitud. Los tokens de esos casos también pueden ser parciales. Las esperas por cuota y errores del transporte se conservan en el JSON; no se atribuyen automáticamente al arnés.

El tiempo corresponde a ejecución del agente: excluye descarga/preparación del contenedor, instalación del arnés y verificador externo. Incluye esperas del proveedor. Los tokens de entrada se acumulan entre llamadas; no representan una sola ventana de contexto. El coste de una tarea abandonada no es un coste por tarea resuelta.

[Precio oficial](https://developers.openai.com/api/docs/models/gpt-6-luna): por millón de tokens, entrada $0.10; lectura de caché $0.01; escritura de caché $0.125; salida $0.50. La entrada normal descuenta ambas categorías de caché. Estimación por uso, no factura; sin infraestructura ni impuestos.

## 13. Nginx con registros y límite de solicitudes

Instalar y ejecutar Nginx en el puerto 8080, servir las páginas de inicio/404 solicitadas, registrar accesos y errores y configurar 10 solicitudes/s por IP. Los tres aprobaron las 8 comprobaciones oficiales.

- **Fénix:** instaló y configuró el servicio real; resolvió que nginx no estuviera en el PATH usando /usr/sbin/nginx. Lo inició como servicio persistente supervisado y comprobó sintaxis, respuestas 200/404 y registros. Corrigió dos llamadas rechazadas: un timeout de cero y un campo adicional inválido en task_finish. No se bloqueó preguntando al usuario y llamó al cierre directamente, sin redescubrimiento repetido.
- **Codex:** instaló y configuró el servicio; corrigió una escritura de configuración y comprobó sintaxis, HTTP, puerto y registros. Arrancó Nginx directamente porque el contenedor impedía el arranque automático del paquete.
- **OpenCode:** utilizó una lista de tareas y parches, arrancó Nginx, comprobó respuestas/procesos y repitió una petición con otro agente de usuario para revisar su registro.
- **Oportunidad pendiente en Fénix:** evitar argumentos inválidos y el primer fallo de PATH usando evidencia actualizada de ejecutables después de instalar. Aquí se demuestra el servicio funcionando, pero el enunciado explicita localhost: no demuestra resueltos todos los bloqueos previos por ambigüedad del destino. La prueba oficial del límite de solicitudes comprueba configuración, no una prueba completa de carga.

## 14. Certificado TLS autofirmado para desarrollo

Generar una clave RSA de 2048 bits, certificado de un año con el sujeto solicitado, PEM combinado, informe de verificación y comprobador Python. Los tres aprobaron el verificador oficial.

- **Fénix:** inspeccionó el entorno, generó los archivos, dejó clave y PEM con permisos 600 y ejecutó el script y comprobaciones OpenSSL. Revisó fechas/tamaño y comparó las huellas impresas de las claves públicas. La inspección ampliada devolvió disponibilidad concreta de herramientas. Cerró con evidencia, sin bloquearse por consultas.
- **Codex:** completó el trabajo con tres llamadas al modelo y un script que carga conjuntamente certificado y clave. Sin embargo, dejó el PEM combinado en 644 aunque también contiene la clave privada; el aprobado oficial no cubre esa exposición.
- **OpenCode:** inspeccionó herramientas, generó los archivos con ambos contenedores de clave privada en 600, ejecutó su comprobador y revisó clave e informe.
- **Oportunidad pendiente en Fénix:** mantener comprobaciones pertinentes reduciendo preparación innecesaria. Cuando se exige igualdad, una aserción directa evita que el modelo tenga que comparar dos hashes impresos. Los tres usan el método privado ssl._ssl._test_decode_cert de Python: funcionó en este entorno, pero tiene implicaciones de portabilidad. Proteger copias/archivos combinados que contienen secretos, no solo la clave original.

## 15. Recuperar filas de un archivo SQLite truncado

Recuperar registros word/value en JSON conservando el archivo dañado. Cada arnés reportó diez filas recuperadas y aprobó la comprobación oficial 1/1.

- **Fénix:** intentó leer SQLite sin escribir, inspeccionó bytes y punteros tras detectar la cabecera ausente, decodificó las celdas del árbol y corrigió el tratamiento de los decimales antes de escribir. Volvió a cargar el JSON y comprobó cantidad de filas y estructura. Terminó sin consultas ni bucles de abandono, conservando la base original.
- **Codex:** superó la ausencia de herramientas de diagnóstico usando Python, decodificó las celdas, comprobó longitudes y escribió JSON sin permitir valores numéricos no finitos. Usó seis llamadas al modelo.
- **OpenCode:** inspeccionó celdas y valores, corrigió un error en el límite de la cabecera de los registros, escribió diez filas y validó el JSON. Usó quince llamadas al modelo.
- **Límite de cobertura:** el test oficial exige más de seis coincidencias entre ocho filas de referencia; no certifica independientemente las diez recuperadas ni rechaza todas las filas extra. Los decodificadores adaptados a este archivo no están validados como recuperadores generales de SQLite.
- **Oportunidad pendiente en Fénix:** comprobar explícitamente longitudes, límites y tipos numéricos al recuperar registros dañados; agrupar inspecciones relacionadas conservando su procedencia. Después del trabajo exitoso se inició una llamada de aprendizaje de skills que terminó con broken pipe sin registrar consumo. El resultado funcional es válido, pero coste y tokens son mínimos registrados. El cierre de evaluación debería esperar o desactivar explícitamente ese aprendizaje y contabilizarlo por separado, sin omitirlo silenciosamente de la comparación.

## Protocolo y límites

Codex 0.157.1; OpenCode 1.18.32; Fenix SHA256 `f7086c387fc10d270084440a33728fb264f17ca1e0727cae0799c97da75700d4`; dataset `7131e4375048a0e408a8fb404b5f499d726b695b`.

Contenedor oficial nuevo por intento, 1 CPU, 2 GiB y 900 segundos de plazo del agente. Orden rotado: Fénix/Codex/OpenCode; Codex/OpenCode/Fénix; OpenCode/Fénix/Codex. Se congelaron binario, adaptadores y configuración. Sin cambios durante la medición, intentos adicionales de tarea ni exposición de tests ocultos. Se añadió idéntico contexto general del entorno para los tres arneses; este cambio de protocolo impide una comparación causal directa con la ronda anterior. Se respetaron las cuotas mediante el mismo proxy. El JSON conserva respuestas, tiempos, tokens, esperas y rutas privadas de las trazas.

No se desplegó ni se modificó el agente durante esta ronda. Tres tareas no permiten concluir superioridad general; las mejoras respecto de las tareas 10–12 no se midieron con una repetición pareada.
