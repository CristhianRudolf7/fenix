# Evaluación nativa y recuperación de ejecución

[English](../en/native-evaluation.md)

Fénix incorpora `fenix eval` en el ejecutable Rust. Utiliza el agente real con
base de datos y skills aislados por intento. Terminal-Bench mantiene sus
contenedores, instrucciones y verificador oficial. Que el agente termine no
significa que haya aprobado la tarea.

## Configuración del contexto

Las configuraciones nuevas usan una ventana de 200.000 tokens y compactación
a partir de 160.000. Se conservan los valores explícitos ya guardados; la capacidad
debe ser compatible con el modelo elegido. Esto no aumenta el tiempo del
benchmark ni garantiza que la tarea se complete.

## Delegación y evaluación bloqueada

El contrato durable del trabajador conserva la petición original del maestro y
sus actualizaciones solo si ambos tienen el mismo ámbito resuelto, general o de
proyecto. Su encargo sigue delimitando la ejecución; no se copia la conversación
completa. Cambiar de ámbito elimina esa referencia heredada. Si un plan no tiene
tareas ejecutándose o en cola y espera una respuesta del usuario, la evaluación
termina como `requires_user_input` tras el plazo configurado, sin inventar respuestas.

## Espera de comandos por eventos

Los comandos en primer plano esperan en Rust su finalización sin consultas del
modelo. Se devuelve un aviso tras 120 segundos o 60 segundos sin salida. El
silencio es solo una advertencia: ninguno de estos plazos cancela el proceso.
Siguen vigentes el timeout del comando, la cancelación de la tarea y el apagado.
Terminales interactivas y servicios persistentes devuelven el control rápidamente.
`yield_ms=0` permite continuar sin esperar y `command_status` con `wait_ms=0`
consulta inmediatamente. La salida se sigue recogiendo en el registro del proceso.
Es una notificación interna, no un webhook HTTP externo.

## Resultados explícitos de ejecución

Las tareas que ejecutan comandos o modifican archivos se cierran mediante
`task_finish`, exclusivo del maestro. El modelo indica `completed` con evidencia
observada, o `blocked`/`failed` con un impedimento concreto. Después de ejecutar,
el motor rechaza un mensaje final sin ese resultado y solicita continuar o
declararlo; esta recuperación utiliza los límites de recuperación existentes.
Tampoco permite cerrar mientras un comando en primer plano sigue ejecutándose.
El resumen se entrega directamente, sin otra llamada al modelo. La conversación
normal conserva su cierre habitual y los subagentes utilizan `subagent_finish`.

Este protocolo no verifica por sí solo la corrección: la evidencia declarada puede
ser incorrecta. Sigue siendo necesario el verificador oficial del benchmark.

## Uso

```bash
pnpm --dir apps/web install --frozen-lockfile
pnpm --dir apps/web build
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

## Evaluaciones supervisadas externamente

Para una evaluación cuyo ejecutor externo controla la cancelación, usa `settings.execution.externally_supervised: true`. Esta opción desactiva el plazo de la tarea principal, el máximo de pasos del modelo y el timeout HTTP total del proveedor. Por defecto permanece desactivada. En este modo, `timeout_seconds: 0` desactiva también el temporizador de la evaluación. Se conservan cancelación e informes. Siguen aplicándose los tiempos de conexión inicial, la capacidad del proveedor, controles de herramientas, políticas de recuperación y plazos oficiales del benchmark: no es cómputo ilimitado.

Configura el contexto según la capacidad verificada del proveedor y una salida menor que ese contexto; cada solicitud sigue ajustándose al espacio estimado restante. El máximo configurable de salida es 1.048.576 tokens. Cambiar varios ajustes a la vez no permite atribuir el resultado a uno solo. Conserva configuración, hash del binario, errores de infraestructura y costos conocidos.

### Cierre de turnos delegados y credenciales de evaluación
El maestro puede liberar el turno con un mensaje de progreso mientras sus propios subagentes o plan siguen activos. Se registra un resultado delegado; el trabajo de fondo conserva su estado y su ruta de reporte. Los comandos en primer plano impiden cerrar, y los trabajadores ajenos o terminados no justifican una delegación. Estos cierres parciales no generan aprendizaje de skills.
El proxy local de evaluación puede alternar dos credenciales que permanecen en el host. Reenvía las cabeceras de límites y respeta una pausa compartida ante HTTP 429; alternar no aumenta cuotas compartidas. Los reintentos de subagentes respetan la duración completa de Retry-After.

La paginación con task_inspect(after=0 y luego next_after) devuelve solo eventos, sin repetir el contrato completo. En reparaciones de paquetes estrechamente ligadas, conserva diagnóstico, reparación y verificación en un mismo trabajador y la auditoría final del trabajo sustancial.
