# Comparación de arneses: tareas 7–9 de Terminal-Bench

[English](three-harness-tasks-7-9-luna-low-2026-09-26.md) · [Datos medidos](three-harness-tasks-7-9-luna-low-2026-09-26.json)

GPT-6 Luna low, servicio estándar, sin Flex. Un intento por combinación. Resultados: fenix 2/3, codex 2/3, opencode 2/3.

| Tarea | Arnés | Pruebas | Tiempo (s) | USD estimados | Tokens entrada | Tokens salida |
|---|---|---:|---:|---:|---:|---:|
| 7. cobol-modernization | fenix | 3/3 | 69.71 | 0.00814693 | 184,469 | 3,299 |
| 7. cobol-modernization | codex | 3/3 | 31.50 | 0.00481439 | 69,319 | 2,468 |
| 7. cobol-modernization | opencode | 3/3 | 58.11 | 0.00632944 | 126,765 | 4,369 |
| 8. kv-store-grpc | fenix | 0/7 | 25.09 | 0.00384882 | 53,001 | 243 |
| 8. kv-store-grpc | codex | 5/7 | 46.52 | 0.00482404 | 121,769 | 1,661 |
| 8. kv-store-grpc | opencode | 7/7 | 69.57 | 0.00444282 | 113,637 | 2,059 |
| 9. polyglot-c-py | fenix | 1/1 | 77.68 | ≥0.00938776 | 221,072 | 4,375 |
| 9. polyglot-c-py | codex | 1/1 | 277.49 | ≥0.00526833 | 77,744 | 3,438 |
| 9. polyglot-c-py | opencode | 0/1 | 54.75 | 0.00370724 | 43,337 | 3,793 |

**≥ indica un mínimo registrado:** faltó el consumo de una solicitud de Fénix y otra de Codex en la tarea 9. Los tokens correspondientes también son parciales. Codex sufrió un timeout de lectura del proveedor de 240.158 s y reintentó; esa demora está incluida en sus 277.49 s. No es correcto atribuirla íntegramente al arnés. No hubo respuestas 429.

El tiempo corresponde a ejecución del agente: excluye descarga/preparación del contenedor, instalación del arnés y verificador externo. Incluye esperas del proveedor. Los tokens de entrada se acumulan entre llamadas; no representan una sola ventana de contexto. El coste de una tarea abandonada no es un coste por tarea resuelta.

[Precio oficial](https://developers.openai.com/api/docs/models/gpt-6-luna): por millón de tokens, entrada $0.10; lectura de caché $0.01; escritura de caché $0.125; salida $0.50. La entrada normal descuenta ambas categorías de caché. Estimación por uso, no factura; sin infraestructura ni impuestos.

## 7. Convertir COBOL a Python

Reproducir exactamente los archivos de cuentas, libros y transacciones para la misma entrada y estado inicial. Los tres aprobaron 3/3 comprobaciones oficiales.

- **Fénix:** leyó fuente y datos, implementó Python, compiló COBOL en una copia temporal y comparó archivos; corrigió la longitud del registro de entrada y comprobó sintaxis. Ambas ejecuciones observadas rechazaron la transacción. No probó una actualización válida de saldos, propietario e historial; además ejecutó Python después de COBOL sin restaurar esa copia. La igualdad obtenida no demuestra equivalencia general. 12 llamadas.
- **Codex:** implementó los registros de ancho fijo y la lógica, y reconoció que no ejecutó el programa contra los datos. 6 llamadas. Aprobó el verificador externo; parte de su rapidez corresponde a menor verificación propia.
- **OpenCode:** probó una transacción válida, detectó diferencias, corrigió la preparación de la comparación y el corte de entrada de 18 a 22 bytes. Repitió la comparación con igualdad byte a byte y restauró los datos originales. 15 llamadas.
- **Mejora para Fénix:** usar copias independientes con el mismo estado y comprobar un caso válido que cambie datos más un rechazo pertinente. Verificar que ocurrió el cambio esperado, no solo que coinciden dos salidas sin cambios. Son pruebas ligadas al requisito, no una suite extensa obligatoria.

## 8. Crear y mantener activo un servidor gRPC

Instalar las versiones indicadas de gRPC, definir dos RPC, generar módulos, implementar un diccionario y dejar el puerto 5328 atendiendo.

- **Fénix:** cargó un skill e inspeccionó el entorno. Preguntó qué devolver para una clave inexistente y convirtió ese caso no especificado en requisito indispensable. El entorno sin usuario respondió owner_input_unavailable y terminó bloqueado antes de implementar: 0/7. Es un bloqueo excesivo observado con la política nueva, no un fallo de cuota, permisos o acceso al modelo; no hubo comparación pareada que demuestre que la versión anterior aprobaba.
- **Codex:** implementó archivos y servicio. El primer arranque con nohup no sobrevivió; después una sesión en primer plano sí respondió SetVal/GetVal durante el turno. Tras terminar el agente, el verificador encontró cerrado el puerto: 5/7. El fallo observado está en la vida del servicio, no en la lógica RPC que sí llegó a funcionar.
- **OpenCode:** generó los módulos, comprobó valores positivos y negativos, lanzó un proceso separado con start_new_session y verificó RPC reales. Aprobó 7/7.
- **Mejora para Fénix:** distinguir intención indispensable ausente de comportamiento de borde no especificado. Si hay usuario, preguntar y avanzar en los requisitos independientes; sin usuario, documentar una convención reversible para detalles no esenciales. Evitar inventar requisitos bloqueantes y abandonar toda la tarea. Conservar aprobaciones y criterios obligatorios. Comprobar también que los servicios sobrevivan al cierre; Fénix no llegó a esa etapa en este intento.

## 9. Un único programa Fibonacci válido como C y Python

Crear un archivo que pueda interpretarse con Python y compilarse con GCC, produciendo los mismos números de Fibonacci.

- **Fénix:** implementó enteros grandes en C y Python, y reparó errores de delimitadores y escapes. Faltaba Python en el entorno inicial y la compilación C previa falló. Después del último parche pidió cerrar como completed mientras reconocía que faltaba verificar ambos modos. El verificador oficial instaló sus dependencias y aprobó 1/1. Aprobar no justifica ese cierre prematuro.
- **Codex:** Implementó precisión arbitraria, reintentó automáticamente tras un timeout de 240 segundos del proveedor, detectó el error inicial de delimitadores de GCC, lo corrigió y comprobó C para N=10 y N=100. Reconoció que no había Python y no afirmó haber probado ambos modos. El verificador oficial aprobó 1/1. No reparó esa dependencia para su comprobación propia.
- **OpenCode:** creó un borrador inválido y terminó reconociendo el fallo, sin ejecutar las comprobaciones anunciadas ni intentar repararlo. La compilación oficial falló: 0/1. No agotó cuota ni plazo.
- **Mejoras para Fénix:** repetir los comandos requeridos después de la última edición y exigir evidencia vigente de esas rutas de aceptación. No aceptar completed cuando el propio resultado dice que falta verificación. Reparar herramientas faltantes dentro de la autorización existente. La revisión del código encontró además que processes.rs reconoce missing_dependency, pero command_run en tools.rs lo sobrescribe como program_exit; así aparece el fallo de python3 registrado. Unificar el diagnóstico para que llegue la indicación útil al modelo. No se modificó el agente durante esta auditoría.

## Protocolo y límites

Codex 0.157.1; OpenCode 1.18.32; Fenix SHA256 `2591947b0b0958ffee78150a1ca89e3272e1f2cf59939e03807b8522d6e4506c`; dataset `7131e4375048a0e408a8fb404b5f499d726b695b`.

Contenedor oficial nuevo por intento, 1 CPU, 2 GiB y 900 segundos de plazo del agente. Orden rotado: Fénix/Codex/OpenCode; Codex/OpenCode/Fénix; OpenCode/Fénix/Codex. Se congelaron binario, adaptadores y configuración. Sin cambios durante la medición, intentos adicionales de tarea ni exposición de tests ocultos. Se respetaron las cuotas mediante el mismo proxy. El JSON conserva respuestas, tiempos, tokens, esperas y rutas privadas de las trazas.

No se desplegó ni se modificó el agente durante esta ronda. Tres tareas no permiten concluir superioridad general; las mejoras respecto de las tareas 4–6 no se midieron con una repetición pareada.
