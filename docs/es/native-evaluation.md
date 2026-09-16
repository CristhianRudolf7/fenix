# Evaluación nativa y recuperación de ejecución

[English](../en/native-evaluation.md)

Fénix incorpora `fenix eval` en el ejecutable Rust. Utiliza el agente real con
base de datos y skills aislados por intento. Terminal-Bench mantiene sus
contenedores, instrucciones y verificador oficial. Que el agente termine no
significa que haya aprobado la tarea.

## Uso

```bash
cargo build --locked -p fenix-server
./target/debug/fenix eval /ruta/config.json /ruta/intento-nuevo
```

El directorio de salida no debe existir. Para evaluar Terminal-Bench, se ejecuta
**dentro del contenedor oficial de la tarea**. El [ejemplo de configuración](../en/native-evaluation.md)
incluye modelo, proveedor, endpoint, archivo de credencial, instrucción exacta,
revisión del conjunto de datos y tiempos. Se usan los ajustes normales; no se
amplían los límites ni se responden preguntas automáticamente. La comprobación
inicial de conexión ocurre antes de medir la ejecución de la tarea.

El evaluador inicia los trabajadores del motor, sin rastreo web público en
segundo plano. Las herramientas web siguen disponibles con un índice local
nuevo; esta diferencia debe registrarse al comparar con el servidor interactivo.

El resultado incluye:

- `manifest.json`: hash SHA-256 del ejecutable y revisión del benchmark.
- `progress.json`: estados de tareas y planes durante la ejecución.
- `report.json`: eventos, llamadas, tokens registrados, tiempo, errores,
  recuperaciones y procesos.
- `ready.json`: señal de que terminó la inferencia y puede empezar el verificador.
- `runtime/` y `skills/`: datos y habilidades aislados, con checkpoints y logs.

La recompensa y el costo quedan en `null`: la recompensa la calcula el
verificador oficial; el costo debe provenir del registro del proveedor. Los
contadores nativos incluyen respuestas de ejecución y compactación activa,
pero no constituyen una factura completa de llamadas auxiliares, reintentos,
comprobación de conexión o caché. Los tokens desconocidos se señalan como tales.

## Integración con Harbor

El adaptador `native_fenix:NativeFenixAgent`, ubicado en
`scripts/benchmarks/harbor/native_fenix.py`, transfiere el binario y ejecuta el
modo nativo. Harbor utiliza Python en el host; el contenedor ya no necesita un
controlador HTTP escrito en Python. Se configura con `FENIX_EVAL_BUNDLE`,
`FENIX_EVAL_CONFIG` y `PYTHONPATH`, como explica la guía en inglés.

El adaptador requiere un proxy de registro accesible desde el contenedor y
proporciona una credencial desechable. No introduce la clave real del proveedor
en el entorno donde el agente ejecuta comandos. Un marcador `{trial}` opcional
en el endpoint se sustituye por el identificador del intento. El campo opcional
`proxy_upstream` abre un túnel TCP nativo en Rust hacia el proxy del host; mantiene
HTTP limitado a loopback y se cierra antes del verificador.

Con `--keep-services`, el proceso Rust conserva los servicios persistentes
cuando el intento llega a un estado terminal, pero detiene los trabajadores de
inferencia antes del verificador. Después se termina con SIGTERM o eliminando
el contenedor. Si vence el tiempo o se necesita una respuesta del usuario,
se detiene la ejecución sin inventar esa respuesta. Un intento terminal puede
contener tareas fallidas: se deben consultar los eventos y la recompensa oficial.

## Las seis mejoras

1. **Gestor de contexto:** reserva espacio para la salida y el razonamiento,
   aplica un presupuesto conjunto a las salidas de herramientas y conserva los
   resultados completos como artefactos paginables. Usa la capacidad configurada
   del modelo; la estimación por bytes no es un tokenizador exacto.
2. **Recuperación:** guarda el checkpoint antes de elegir el corte, incluso con
   un último lote demasiado grande. Los resúmenes usan el techo de salida
   configurado, sujeto al espacio disponible. Si fallan, conserva extractos
   acotados y una referencia recuperable. Una generación truncada no ejecuta
   acciones parciales; se solicita una acción menor. Los filtros del proveedor
   no se evaden mediante reintentos.
3. **Supervisor de procesos:** espera hasta 30 segundos antes de un efecto
   dependiente. Si el proceso sigue ocupado, devuelve sus identificadores y
   `executed: false`. Los comandos rápidos pueden devolver inmediatamente su
   resultado. Los servicios persistentes conservan trabajos ordinarios de shell
   en segundo plano; se recomienda ejecutarlos en primer plano. No se garantiza
   supervisión de demonios que se separan con doble fork, `setsid` o terminación
   explícita del shell. `command_status ready_port` comprueba disponibilidad TCP,
   no funcionamiento de la aplicación ni identidad del propietario del puerto.
4. **Bucles:** distingue resultados repetidos sin avance de cambios observados
   en procesos, estado o salida. Tres repeticiones sin progreso detienen el bucle;
   siguen vigentes los límites de pasos y tiempo.
5. **Contrato de tarea:** conserva objetivo original, ámbito, políticas,
   autorizaciones, respuestas explícitas y referencias a evidencia. La herramienta
   `task_contract` permite anotar supuestos, restricciones y criterios con fuentes,
   sin convertir anotaciones del modelo en permisos. Las preguntas normalizadas
   idénticas reutilizan respuestas; no se presupone equivalencia semántica de
   preguntas distintas. La verificación debe ser proporcional a la tarea.
6. **Evaluación continua:** incorpora trazas nativas y regresiones de los fallos
   observados. Las pruebas locales se ejecutan en CI sin consumir API. Para medir
   una mejora de precisión hay que repetir el benchmark y añadir tareas nuevas;
   las pruebas unitarias no producen un nuevo porcentaje de Terminal-Bench.

Se permiten tres recuperaciones por causa y ocho en total por tarea, dentro de
sus límites generales. Se mantienen los identificadores y no se repiten efectos
ya completados. La recuperación ocurre durante la ejecución; no reanuda
automáticamente efectos del sistema después de reiniciar la aplicación.

Los extractos y resúmenes pueden omitir información, aunque los originales
redactados sigan recuperables. Si las instrucciones y el contrato por sí solos
no caben, se informa del problema y se conserva un checkpoint.

Si se inicia explícitamente mediante el cargador `ld-linux` empaquetado, `binary_sha256` calcula el hash del programa evaluado y no el del cargador. `binary_path` y `process_image_path` registran ambas identidades. Los manifiestos anteriores bajo ese cargador no contienen esta corrección: utiliza el protocolo externo del binario congelado para su hash y no reescribas los informes históricos.
