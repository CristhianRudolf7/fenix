# Comparación de arneses: tareas 4–6 de Terminal-Bench

[English](three-harness-next-three-luna-low-2026-09-26.md) · [Datos medidos](three-harness-next-three-luna-low-2026-09-26.json)

Después de corregir HOME para Git y orientar la verificación al requisito y al riesgo, se ejecutaron las siguientes tres tareas de la selección existente, con GPT-6 Luna low, sin Flex. OpenCode aprobó 3/3; Codex, 2/3; Fénix, 2/3 en las ejecuciones con verificador válido. **Fénix requirió una repetición por fallo de infraestructura del verificador de pMARS; su primer intento también mostró una detención innecesaria del agente.** Los resultados no permiten concluir que Fénix y Codex tengan igual fiabilidad general.

## Cambios y validación local

- HOME estable para comandos locales: cambiar cwd ya no oculta la identidad Git configurada. Se conserva el entorno permitido y el aislamiento del ejecutor remoto por usuario. No se inventan autores ni se copian identidades históricas.
- Prompt y cuatro skills: reutilizar pruebas/evidencias existentes; comprobar comportamiento requerido y riesgos concretos; añadir solo comprobaciones faltantes y detenerse cuando la evidencia relevante es suficiente. Sin revisor obligatorio ni contrato generado para cada herramienta.
- Migración de versiones conocidas de skills, preservando personalizaciones y estado desactivado.
- 12 pruebas de procesos, 5 de ejecución y 1 de migración de skills aprobadas; compilación del servidor y revisión de espacios correctas. La regresión de Git crea commits reales mediante pipe y PTY, con control negativo del comportamiento anterior.

No hubo despliegue. La corrección Git tiene regresión local; esta ronda no repite la tarea `fix-git` del benchmark.

## Medición

| Tarea | Arnés | Pruebas | Tiempo (s) | USD estimados | Entrada | Salida |
|---|---|---:|---:|---:|---:|---:|
| 4. multi-source-data-merger | fenix | 3/3 | 56.38 | 0.01082240 | 182,870 | 2,130 |
| 4. multi-source-data-merger | codex | 3/3 | 22.97 | 0.00396414 | 52,868 | 1,664 |
| 4. multi-source-data-merger | opencode | 3/3 | 47.55 | 0.00395757 | 71,278 | 2,205 |
| 5. build-cython-ext | fenix | 10/11 | 900.43 | ≥0.07228532 | 2,161,739 | 15,724 |
| 5. build-cython-ext | codex | 2/11 | 343.09 | 0.01044943 | 387,553 | 3,567 |
| 5. build-cython-ext | opencode | 11/11 | 521.87 | 0.02349096 | 1,018,581 | 7,576 |
| 6. build-pmars | fenix | 4/4 | 95.34 | 0.00840589 | 173,789 | 1,583 |
| 6. build-pmars | codex | 4/4 | 94.87 | 0.01350399 | 420,290 | 2,740 |
| 6. build-pmars | opencode | 4/4 | 250.53 | 0.01170876 | 368,692 | 3,078 |

Tiempo de ejecución del agente, incluido su arranque y los comandos que decide ejecutar; excluye preparación del contenedor, instalación del arnés y verificador externo. Los tokens de entrada son acumulados entre solicitudes, no el tamaño de una sola ventana. Coste estimado desde usage, con lectura/escritura de caché; no factura ni infraestructura. Cython de Fénix tiene una respuesta interrumpida sin usage: su coste y tokens son mínimos conocidos.

[Tarifas oficiales](https://developers.openai.com/api/docs/pricing), por millón: entrada normal $0.10, lectura de caché $0.01, escritura $0.125 y salida $0.50. La entrada normal excluye ambas categorías de caché.

## Auditoría por tarea

### 4. Unificar usuarios desde JSON, CSV y Parquet

Normalizar campos y fechas, deduplicar con prioridad entre fuentes y generar Parquet e informe de conflictos. Los tres aprobaron 3/3.

- **Fénix:** implementó y comprobó IDs, prioridades, fechas y conflictos. Hizo descubrimiento redundante de herramientas, cargó un skill poco específico para ETL y necesitó tres llamadas a `task_finish`: las dos primeras omitieron el campo estructurado de evidencia. 12 solicitudes, frente a 5 de Codex y 11 de OpenCode.
- **Codex:** inspeccionó las fuentes y ejecutó un script coherente; fue el más rápido.
- **OpenCode:** fusionó los datos y releyó el resultado para comprobar estructura, registros y conflictos.
- **Mejora pendiente:** seleccionar mejor el skill y hacer inequívoco el cierre con evidencia existente. Las pruebas pertinentes no explican la mayor parte del exceso de llamadas.

### 5. Compilar pyknotid con NumPy 2.3.0

Construir e instalar tres extensiones Cython, ejecutar el ejemplo y las pruebas del repositorio indicadas.

- **Fénix:** 10/11 comprobaciones oficiales. Logró compilar, instalar y reparar incompatibilidades Python/NumPy/planarity; el ejemplo y la suite del repositorio pasaron. Quedó `np.int` en `ccomplexity.pyx`, que fallaba al ejecutar una función nativa aunque importaba correctamente. Las búsquedas/correcciones se centraron en `.py`. Los parches ambiguos produjeron reintentos y varios intentos de cierre parcial; el control de cierre lo hizo continuar. Agotó 900 s, de los que 368.366 s fueron esperas para respetar TPM.
- **Codex:** 2/11. Varias llamadas de su código auxiliar devolvieron solo stdout y omitieron el identificador del proceso pendiente; relanzó compilaciones mientras otras continuaban. Se observaron dos compilaciones concurrentes en un contenedor de una CPU. Terminó reconociendo que la instalación estaba incompleta. Esto describe la trayectoria observada, no demuestra un defecto universal de Codex.
- **OpenCode:** 11/11. Esperó las compilaciones, corrigió aliases también en `.pyx`, `fractions.gcd` y atributos de planarity, reinstaló y comprobó ejemplo y suite. Su espera por TPM fue 23.356 s.
- **Mejoras pendientes:** incluir todos los tipos de archivo implicados por el requisito; comprobar una operación representativa de cada extensión requerida, no solo importarla; recuperarse de parches ambiguos con cambios acotados; reducir contexto y llamadas repetidas. No puede atribuirse todo el fallo al límite del proveedor ni afirmarse que sin él habría aprobado.

### 6. Compilar pMARS desde Debian sin X11

Conservar las fuentes, instalar el ejecutable y verificar simulación y depurador.

- **Fénix, primer intento:** 47.30 s y $0.00419035 adicionales. Descargó las fuentes, pero faltaba `dpkg-source`. Confundió la advertencia de `_apt` con un bloqueo y pidió autorización, sin compilar. Después el verificador falló por 404 de paquetes Debian, antes de ejecutar tests. Su reward=0 no es una calificación funcional válida y se conserva como intento adicional.
- **Fénix, repetición por infraestructura:** mismo binario, skills, modelo y configuración; 4/4. Instaló dependencias, extrajo las fuentes, compiló sin X11 y agrupó pruebas de simulación, dependencias y depurador. Cerró con evidencia en una llamada. La variación respecto del primer intento evidencia autonomía inconsistente.
- **Codex:** 4/4. Descargó fuentes Debian, instaló las herramientas faltantes, modificó Makefile, compiló y comprobó resultado y depurador.
- **OpenCode:** 4/4. Usó fuentes Debian y verificó el resultado. Perdió 120 s con `pmars -h`, que quedó esperando hasta agotar el timeout; luego comprobó el depurador con entrada explícita.
- **Mejora pendiente:** distinguir dependencia ausente de denegación real y reparar el entorno dentro del alcance autorizado. Usar entrada y espera acotadas para comprobaciones interactivas.

## Reproducibilidad y límites

Codex 0.157.1, OpenCode 1.18.32, Fénix binario `9c12ff4046cf267bd0f29876cebf8650b7a3cf39622fd1f5e31b91242ee8a910`. Revisión del dataset `7131e4375048a0e408a8fb404b5f499d726b695b`. Un contenedor oficial nuevo por intento, 1 CPU, 2 GiB y plazo de agente de 900 s. Ejecuciones secuenciales con orden rotado, sin exponer tests ocultos al modelo. Todas las respuestas completas confirmaron GPT-6 Luna low y servicio default; no hubo 429. El JSON incluye costes, tokens, esperas, resultados brutos y ubicaciones privadas de las trazas.

La repetición de pMARS se decidió por ausencia del informe de pruebas, antes de conocer su resultado, y se conserva el intento original. No hubo cambios en Fénix durante las mediciones ni selección del mejor resultado. Son tres tareas diagnósticas nuevas, no un experimento controlado antes/después de las mejoras.
