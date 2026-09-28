# Comparación diagnóstica de tres arneses — 26/09/2026

[English](three-harness-terminal-luna-low-2026-09-26.md) · [Datos medidos](three-harness-terminal-luna-low-2026-09-26.json)

Las tres primeras tareas de nuestra selección de Terminal-Bench 2.1 se ejecutaron en contenedores oficiales nuevos, con GPT-6 Luna low sin Flex. Fénix aprobó 2/3; Codex 0.157.1 y OpenCode 1.18.32 aprobaron 1/3 cada uno. No se modificó el comportamiento de Fénix ni se desplegó nada.

Los tiempos corresponden a la fase de ejecución del agente, incluido su arranque, y excluyen instalación, preparación y verificador. Costes estimados a partir de los tokens de OpenAI, incluidas lecturas y escrituras de caché; no son una factura ni incluyen infraestructura. Todas las respuestas confirmaron el mismo modelo y nivel de pensamiento. No hubo errores 429 ni respuestas exitosas sin contabilidad de tokens.

| Task | Harness | Pass | Execution (s) | Estimated USD | Input tokens | Output tokens |
|---|---|---:|---:|---:|---:|---:|
| cancel-async-tasks | Fénix | 1 | 38.82 | 0.003814 | 46,458 | 1,025 |
| cancel-async-tasks | Codex | 0 | 13.93 | 0.002918 | 29,079 | 741 |
| cancel-async-tasks | OpenCode | 0 | 39.70 | 0.003192 | 54,360 | 1,998 |
| fix-git | Fénix | 0 | 55.79 | 0.004766 | 86,245 | 832 |
| fix-git | Codex | 1 | 24.76 | 0.005387 | 98,502 | 1,817 |
| fix-git | OpenCode | 1 | 45.22 | 0.003987 | 93,614 | 1,676 |
| db-wal-recovery | Fénix | 1 | 44.51 | 0.005379 | 114,384 | 1,617 |
| db-wal-recovery | Codex | 0 | 38.33 | 0.006013 | 147,579 | 2,912 |
| db-wal-recovery | OpenCode | 0 | 49.67 | 0.004034 | 96,299 | 2,371 |

## Auditoría por tarea

1. **Cancelación asíncrona.** Fénix utilizó TaskGroup y semáforo y aprobó 6/6 comprobaciones. Codex usó gather y solo comprobó la importación; OpenCode también usó gather, hizo una prueba propia y la corrigió, pero ambos fallaron la limpieza al cancelar con trabajos en espera (5/6). Para Fénix: añadir pruebas acotadas de SIGINT y limpieza asíncrona con trabajos por encima y por debajo del límite; su prueba propia aún no cubría todos estos casos.
2. **Recuperación Git.** Fénix encontró el commit pero pidió nombre/correo y se detuvo; 0/2. La imagen ya tenía identidad Git, pero el motor sustituye HOME por el directorio de trabajo y deja de ver esa configuración. Codex y OpenCode recuperaron el commit, resolvieron el conflicto y aprobaron 2/2; Codex tuvo que corregir una edición fallida. Para Fénix: HOME estable por usuario y acceso explícito a la identidad Git autorizada, preservando el aislamiento; diagnosticar antes de preguntar por datos que ya existen.
3. **Recuperación SQLite.** Fénix detectó XOR 0x42, conservó el WAL antes de modificarlo, lo reparó y verificó datos exactos e integridad; 7/7. Codex y OpenCode abrieron el original en escritura antes de copiarlo, perdieron el WAL y generaron registros supuestos, reconociéndolo en sus respuestas finales; 5/7. Ambos omitieron las actualizaciones reales de las filas 1 y 2. Para Fénix: respaldar DB/WAL/SHM antes de cualquier conexión, incluso de solo lectura, y operar sobre una copia. Aquí el respaldo ocurrió después de la consulta inicial de solo lectura.

## Límites y preparación

Es una muestra diagnóstica de tres tareas, no una clasificación general. Solo hay un intento válido por combinación. Las dos ejecuciones preliminares de Codex 0.154.0 se excluyeron por problemas de preparación: carpeta temporal que bloqueaba herramientas auxiliares y falta de metadatos de Luna. Ambas habían aprobado; se conservaron y no se seleccionó el mejor resultado. Costaron $0.007494315 adicionales. El informe final usa Codex 0.157.1 con su distribución completa.

OpenCode hizo una solicitud auxiliar por tarea con otro modelo; el proxy la rechazó antes de llegar al proveedor. No se cobró ni contribuyó una respuesta de otro modelo, pero es una limitación de configuración a tener en cuenta. Se preservaron todos los resultados y trazas en la carpeta privada indicada en el JSON. Los agentes nunca recibieron los tests ocultos ni las soluciones de referencia.

[Tarifas oficiales utilizadas](https://developers.openai.com/api/docs/pricing): por millón, entrada normal $0.10; lectura de caché $0.01; escritura de caché $0.125; salida $0.50.
