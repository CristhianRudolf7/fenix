# Comparación de arneses: tareas 10–12 de Terminal-Bench

[English](three-harness-tasks-10-12-luna-low-2026-09-26.md) · [Datos medidos](three-harness-tasks-10-12-luna-low-2026-09-26.json)

GPT-6 Luna low, servicio estándar, sin Flex. Un intento por combinación. Resultados: fenix 2/3, codex 1/3, opencode 1/3.

Cambios evaluados: pruebas diferenciales con estado inicial independiente; revisión de bloqueos por dudas secundarias; comprobación vigente de ejemplos ejecutables declarados; conservación del diagnóstico de dependencias. Validación local: 251 pruebas aprobadas, tres opcionales omitidas y compilación correcta.

| Tarea | Arnés | Pruebas | Tiempo (s) | USD estimados | Tokens entrada | Tokens salida |
|---|---|---:|---:|---:|---:|---:|
| 10. polyglot-rust-c | fenix | 1/1 | 161.79 | 0.01780715 | 376,236 | 7,568 |
| 10. polyglot-rust-c | codex | 0/1 | 25.85 | 0.00354923 | 19,079 | 2,329 |
| 10. polyglot-rust-c | opencode | 0/1 | 97.88 | 0.00731708 | 135,373 | 6,705 |
| 11. git-multibranch | fenix | 1/1 | 163.68 | 0.02848899 | 485,799 | 4,536 |
| 11. git-multibranch | codex | 1/1 | 62.27 | 0.00907058 | 202,864 | 5,452 |
| 11. git-multibranch | opencode | 1/1 | 96.90 | 0.00778408 | 220,814 | 4,592 |
| 12. configure-git-webserver | fenix | 0/1 | 34.02 | 0.00783213 | 99,699 | 777 |
| 12. configure-git-webserver | codex | 0/1 | 27.20 | 0.00397572 | 39,688 | 2,409 |
| 12. configure-git-webserver | opencode | 0/1 | 54.30 | 0.00481098 | 76,775 | 3,550 |

≥ significa mínimo registrado cuando falta consumo de alguna solicitud. Los tokens de esos casos también pueden ser parciales. Las esperas por cuota y errores del transporte se conservan en el JSON; no se atribuyen automáticamente al arnés.

El tiempo corresponde a ejecución del agente: excluye descarga/preparación del contenedor, instalación del arnés y verificador externo. Incluye esperas del proveedor. Los tokens de entrada se acumulan entre llamadas; no representan una sola ventana de contexto. El coste de una tarea abandonada no es un coste por tarea resuelta.

[Precio oficial](https://developers.openai.com/api/docs/models/gpt-6-luna): por millón de tokens, entrada $0.10; lectura de caché $0.01; escritura de caché $0.125; salida $0.50. La entrada normal descuenta ambas categorías de caché. Estimación por uso, no factura; sin infraestructura ni impuestos.

## 10. Un archivo Fibonacci para Rust y C++

La tarea se llama polyglot-rust-c, pero el enunciado utiliza rustc y g++ (C++). Ambos deben calcular Fibonacci con f(0)=f(1)=1.

- **Fénix:** corrigió errores de comentarios y cadenas. El control de recuperación existente rechazó un cierre fallido temprano y continuó. Después, el nuevo control rechazó completed porque la prueba de Rust era anterior a la última edición; repitió Rust y cerró. Aprobó 1/1. Incluye 28,535 s de espera por cuota. Hubo búsquedas de herramientas de cierre/contrato ya disponibles: persiste algo de trabajo redundante.
- **Codex:** inspeccionó el directorio y terminó sin crear el archivo, indicando que no consiguió un polyglot fiable. Falló 0/1 por ausencia de fuente, sin agotar cuota ni plazo.
- **OpenCode:** hizo varios cambios de delimitadores antes de compilar; Rust seguía fallando y terminó reconociéndolo. Falló 0/1.
- **Mejora pendiente en Fénix:** vincular el ejecutable comprobado con la salida de la compilación. Su último rustc usó cwd=/app sin -o, pero ejecutó /app/polyglot/main de una compilación previa con salida explícita. El verificador sí reconstruyó correctamente la fuente final, por lo que el aprobado es válido; ejecutar comandos recientes no demuestra por sí solo la procedencia del binario. Conviene probar pronto un esqueleto mínimo y evitar búsquedas repetidas. La traza demuestra que actuó el nuevo control, no cuantifica su efecto mediante un experimento pareado.

## 11. Git por SSH y despliegues HTTPS separados para main/dev

Recibir pushes con contraseña y publicar ambas ramas mediante post-receive en menos de tres segundos. Los tres obtuvieron 1/1 oficial.

- **Fénix:** configuró servicios, encontró utilidades administrativas fuera del PATH mediante rutas absolutas y reparó la propiedad del repositorio tras fallar un push real. Pasó a directorios hermanos main/dev, evitando que publicar una rama borre la otra. Comprobó clonación/push autenticados, las dos respuestas HTTPS y un push de dev inferior a tres segundos. No se detuvo por dudas innecesarias. Persistieron búsquedas repetidas de la herramienta de cierre, aunque su esquema ya estaba disponible.
- **Codex:** probó pushes reales y detectó que el hook borraba su directorio temporal antes de moverlo; después encontró un problema de permisos de publicación. Corrigió ambos y comprobó las respuestas HTTPS. Reportó un push de aproximadamente 0,56 s.
- **OpenCode:** implementó servicios, hook y certificado; comprobó pushes y clonación, corrigiendo su configuración de contraseña/clave del host en la prueba. Ambas rutas devolvieron el contenido esperado.
- **Limitación del verificador:** dio aprobado aunque rechazara pushes por non-fast-forward, porque las pruebas previas del agente ya habían publicado los textos esperados. Su flujo shell/Expect compara contenido HTTP sin exigir de manera fiable que cada push de prueba termine bien. Se conserva la puntuación oficial, pero no se considera prueba independiente de un nuevo despliegue; las trazas de pushes exitosos de los agentes aportan evidencia complementaria.
- **Mejora pendiente en Fénix:** localizar ejecutables antes de reinstalar paquetes, evitar descubrimiento redundante y usar pruebas limpias/aisladas con contenido nuevo identificable. Verificar éxito del push y correspondencia del commit publicado, no solo texto HTTP que podría ser previo. Es el mismo principio de comprobar cambios reales de la tarea 7.

## 12. Publicar pushes de Git mediante un servidor web en el puerto 8080

Crear /git/server y publicar master para que una clonación, commit y push posteriores hagan accesible hello.html por HTTP. El enunciado excluye configurar el login. Los tres fallaron 0/1: seguía faltando Git y el servidor no respondió (HTTP 000). La imagen comienza deliberadamente con Ubuntu y curl; instalar Git formaba parte de preparar la solución, no es prueba de un fallo del verificador.

- **Fénix:** cargó dos skills pertinentes e inspeccionó el entorno. La inspección devolvió salida vacía con código 0, por lo que aportó poca información. Preguntó si /app era el servidor destino y pidió su nombre/IP. Tras rechazarse la primera pregunta, añadió una justificación de requisito y cerró bloqueado por falta de respuesta, sin configurar nada. La nueva política exige explicar el bloqueo, pero no demuestra que esa explicación sea correcta: esta ejecución revela esa limitación semántica pendiente. No se detuvo por cuota ni por agotar el plazo.
- **Codex:** creó un script de configuración y un README para un supuesto servidor externo. No instaló Git ni ejecutó el script; dejó esa ejecución al usuario. El script presupone systemd sin comprobar el entorno real.
- **OpenCode:** también entregó un script y documentación en lugar de configurar el entorno. Comprobó sintaxis de shell y permisos de ejecución, lo que no demuestra un despliegue funcional. No instaló Git ni arrancó el servicio.
- **Mejora pendiente en Fénix:** explicitar el entorno/destino de evaluación en el adaptador para los tres arneses, sin revelar pruebas ocultas ni relajar los requisitos de producción. Antes de convertir una duda sobre el destino en un bloqueo, inspeccionar evidencia local y avanzar con preparación independiente que esté dentro del alcance. Para una orden de configurar un servicio, exigir evidencia del servicio funcionando y del flujo push→HTTP; entregar un script basta únicamente cuando eso sea lo solicitado. Investigar también la salida vacía de la inspección del entorno. Conservar las preguntas iniciales y autorizaciones legítimas en tareas normales.

Los tres confundieron ejecutar la configuración con entregar instrucciones, pero Fénix se detuvo antes mediante su política de consultas. Son hallazgos pendientes: no se aplicaron cambios adicionales durante esta medición.

## Protocolo y límites

Codex 0.157.1; OpenCode 1.18.32; Fenix SHA256 `580f0588c8757a90304a327c9f3914a1d923adb860211351cc5d6054d3a17b32`; dataset `7131e4375048a0e408a8fb404b5f499d726b695b`.

Contenedor oficial nuevo por intento, 1 CPU, 2 GiB y 900 segundos de plazo del agente. Orden rotado: Fénix/Codex/OpenCode; Codex/OpenCode/Fénix; OpenCode/Fénix/Codex. Se congelaron binario, adaptadores y configuración. Sin cambios durante la medición, intentos adicionales de tarea ni exposición de tests ocultos. Se respetaron las cuotas mediante el mismo proxy. El JSON conserva respuestas, tiempos, tokens, esperas y rutas privadas de las trazas.

No se desplegó ni se modificó el agente durante esta ronda. Tres tareas no permiten concluir superioridad general; las mejoras respecto de las tareas 7–9 no se midieron con una repetición pareada.
