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

El 28 de septiembre de 2026 se actualizaron explícitamente las configuraciones
guardadas de producción y desarrollo local a 200.000 tokens, con compactación a
160.000. Cambiar solo el valor predeterminado del código no migra una base de
datos existente: al desplegar, verifica también `settings.coding` en el endpoint
autenticado de estado y actualiza su configuración. Las evaluaciones nuevas
heredan estos valores cuando omiten `coding`. Los resultados históricos y las
pruebas unitarias que fuerzan contextos pequeños para comprobar la compactación
conservan sus valores originales.

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

### Roles rápido e inteligente

`settings` configura el rol inteligente (maestro, trabajadores y ejecución). El
objeto opcional `fast` usa el esquema existente de ajustes en vivo (`provider`,
`model`, `base_url`, `reasoning_effort`) para configurar el rol conversacional
rápido mediante la ruta normal `configure_live`, incluida su comprobación de
conexión antes de empezar a medir el tiempo. Usa el mismo archivo de credencial de
evaluación y, con `proxy_upstream`, el mismo túnel local; `{trial}` se sustituye en
ambos endpoints. La instrucción del benchmark sigue llegando directamente al
maestro, por lo que no se ejercita el enrutamiento del chat. El rol rápido solo lo
llaman funciones existentes, como la narración del progreso. `report.json` registra
`fast_settings`, `model_roles` (incluido `fast_exercised`) y conteos y tokens
`fast_*` separados; la comprobación de conexión se excluye y debe consultarse en la
contabilidad del proveedor. Sin `fast` se conserva el comportamiento anterior de un
solo rol.

`harbor/provider_proxy.py` registra tráfico de chat compatible con OpenAI hacia
RunInfra. Repite `--model` para cada rol permitido (sin esa opción solo acepta el
modelo Nemotron de la auditoría original). `--key-env-file RUTA --key-variable
NOMBRE` lee una credencial del host una sola vez, sin copiarla al directorio de la
ejecución. Los metadatos de cada solicitud registran el modelo y el uso informado
por el proveedor, incluido su costo.

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

Las generaciones sin una acción ejecutable son la excepción al conteo de por vida.
Son los cortes por límite de salida y las generaciones de chat que terminan con
razonamiento en texto plano pero sin texto ni llamadas de herramientas
(`cause: empty_output`); una respuesta sin razonamiento, texto ni llamadas sigue
siendo inválida. Un resultado de herramienta ejecutado correctamente después cierra
ese episodio y restablece su contador (`recovery_budget_restored`), de modo que solo
las generaciones consecutivas sin acción agotan el presupuesto. Desde la segunda
consecutiva, el mensaje de recuperación pide una sola llamada de herramienta para
el paso concreto más pequeño, como un script breve que calcule, en lugar de un
diseño completo o de cálculos a mano. Si un proveedor de chat devuelve como texto
plano el razonamiento o el texto de esa generación, solo el más reciente se
incorpora a la siguiente solicitud como notas no verificadas desde las que
continuar, y se retira cuando el modelo produce una acción ejecutable; no se
incorporan cargas de acciones demasiado grandes y nunca se descifra un
razonamiento opaco o cifrado (`recovery_strategy` registra su tamaño). La
repetición de Chat Completions (texto, llamadas de herramientas y razonamiento en
texto plano) no está firmada, así que el texto con forma de credencial se enmascara
como el contenido normal en lugar de hacer fallar la solicitud; la repetición
firmada u opaca, incluido `reasoning_details` del chat, sigue rechazándose sin
cambios. No cambian el límite de salida, el esfuerzo de razonamiento ni los
parámetros del proveedor. Efecto medido en 15 tareas de Terminal-Bench 4.0 con
GLM-5.3-Flash: [registro](../benchmarks/terminal-bench-15-runinfra-2026-09-29.es.md).

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

### Jev y credenciales de OpenAI con cuotas independientes

La configuración de evaluación acepta `decisions: {"enabled":true,"shadow":false}` junto con `proxy_upstream`. Jev usa el mismo túnel local y una credencial ficticia; las claves reales permanecen en el proxy del host, fuera del contenedor.

`openai_decision_proxy.py` distribuye solicitudes entre grupos de cuota confirmados, respeta los encabezados de OpenAI y las pausas 429. Varias claves de una misma organización deben compartir grupo: no se presupone una cuota independiente por clave. No se duplican solicitudes en carrera ni efectos de herramientas. Las métricas deben incluir esperas del proxy y llamadas a Jev.

Tras varias rondas de lectura, Jev puede recomendar una reparación respaldada por evidencia, una comprobación concreta, más lectura o abstenerse. No autoriza acciones ni certifica éxito. El flujo de programación prioriza corregir el error observado y comprobarlo, completando compilación, instalación y uso desde el entorno solicitado antes de ampliar las pruebas.

### Requisitos de cierre con evidencia de herramientas

El contrato conserva comprobaciones obligatorias separadas de las anotaciones. Las solicitudes explícitas de suite reconocidas por el motor generan una comprobación pendiente; `task_contract(action="require")` permite registrar comandos inmutables citando literalmente la tarea. Solo sus resultados observados pueden aprobarlos. Que falte pytest, no se recojan pruebas, se ejecute una prueba filtrada o funcione un ejemplo no sustituye la suite. Las exclusiones reconocidas deben aparecer en la instrucción.

El cierre como completado se rechaza si quedan comprobaciones pendientes, fallidas o desactualizadas. Los procesos asíncronos conservan su comando y revisión. Las ediciones mediante herramientas y compilaciones/instalaciones reconocidas invalidan evidencia anterior. El contexto compacto conserva estados e identificadores. Los trabajadores de revisión y auditoría mantienen sus validadores específicos.

No es una prueba semántica de todos los requisitos: reconocer instrucciones, salidas de runners y modificaciones del shell usa heurísticas limitadas. Scripts arbitrarios, directorios incorrectos en suites inferidas, escrituras no reconocidas o tests debilitados requieren revisión. Los comandos registrados explícitamente pueden restringir el directorio. El evaluador oficial sigue siendo independiente.

Para evaluar sin Jev, omite `decisions` y `jev_key_file`: la pasarela rechaza su ruta sin cargar su clave.

Evaluación oficial única (2026-09-25), GPT-6 Luna low, sin Jev: recompensa **0**, **2/11 comprobaciones aprobadas**, **18,69 segundos** de tarea nativa (72,05 segundos de trabajo Harbor completo). El modelo clonó y decidió cerrar con `failed` sin bloqueo concreto. No se activó el control de `completed`; los otros estados terminales aún aceptan un texto de bloqueo sin demostrar que la recuperación esté agotada. Las cinco solicitudes al proveedor, incluida conexión inicial, devolvieron HTTP 200; no hubo fallo de cuota ni compactación. No se realizó otro intento. Validación local: 216 pruebas del agente aprobadas, 3 omitidas; 4 de pasarela aprobadas; compilación correcta. Véase [registro](../benchmarks/luna-low-completion-gate-2026-09-25.json). La recuperación sigue sin quedar resuelta.

### Cierres sin éxito respaldados por evidencia

Los ejecutores deben justificar `failed`/`blocked` con `stop_reason` y `blocker_call_id` de su propia tarea. El motor comprueba una denegación o fallo externo observado sin resolver, o tres contextos distintos de reparación seguidos del mismo chequeo fallido. Una explicación sin evidencia, una preparación exitosa, resultados de otra tarea o fallos ya resueltos no justifican abandonar. Las dependencias faltantes generan orientación para recuperarse.

El contrato conserva las observaciones tras compactar. Consultar varias veces un proceso terminado cuenta una sola vez. Repetir el mismo fallo sin cambios no es otra estrategia. Tras tres fallos idénticos sin reparación observada, o tres cierres rechazados sin progreso, el motor pausa con `runtime_stagnation`: informa del trabajo incompleto y de la necesidad de intervención del usuario/supervisor, sin declarar imposible la tarea. Cancelaciones, permisos y efectos inciertos conservan sus controles. Los subagentes ejecutores usan la misma validación; los revisores mantienen sus validadores específicos.

No se añade un modelo clasificador. Reconocer diagnósticos y reparaciones usa heurísticas limitadas: comandos equivalentes o escrituras arbitrarias del shell pueden no reconocerse, y la pertinencia de un fallo externo aún requiere criterio del agente. Nunca se sortean denegaciones ni decisiones pendientes. Las pruebas locales verifican recuperación después de un cierre rechazado, cancelación, evidencia por tarea, reparaciones diferentes, procesos asíncronos y estancamiento.

Un intento oficial, 2026-09-25 (Luna low, sin Jev): **9/11 comprobaciones aprobadas; recompensa 0; timeout externo de 900 segundos**. Recuperó dependencias y ejecutó la suite, sin abandonar. Quedaron fallos por `numpy.int` en Cython y compatibilidad con planarity. Las esperas por cuota sumaron **379,095 segundos**. También consumieron tiempo las ediciones ambiguas repetidas y 14 búsquedas de herramientas. No intentó un cierre injustificado: esta ejecución por sí sola no prueba causalidad del control nuevo. Después del intento se agruparon los parches fallidos por archivo/diagnóstico/revisión de reparación para que variaciones superficiales no evadan el detector. Este último ajuste solo tiene validación local: no se repitió el benchmark ni se desplegó. [Auditoría](../benchmarks/luna-low-stop-gate-2026-09-25.json).

El ajuste posterior aprobó las siete pruebas específicas del control de abandono, incluida la repetición de parches equivalentes. El resultado del benchmark corresponde exclusivamente a su binario congelado original.

### Comparaciones propias y cierre de tareas

El cierre registra comparaciones de archivos fallidas con `cmp`/`diff` aunque el usuario no haya pedido una suite. Se conservan en el contrato de la tarea al compactar o reiniciar. Un comando ajeno que termine bien, una lectura o volver a consultar el mismo proceso no borran el fallo. Repetir correctamente la comparación permite resolverlo; una edición posterior invalida la evidencia.

Si la prueba estaba mal construida, `task_contract resolve_check` exige justificar la corrección, citar el requerimiento original y aportar una comparación exitosa de la misma tarea, posterior al fallo y vigente. Conserva el resultado fallido y su reemplazo. No permite descartar suites solicitadas ni comprobaciones explícitas. Patrones comunes que ocultan fallos, como `cmp ... || true`, no sirven como evidencia. No añade otro modelo ni un revisor obligatorio.

Las instrucciones de programación exigen revisar formato, anchura de registros, rutas efectivas y datos iniciales equivalentes antes de comparar. El motor valida procedencia y ejecución; la pertinencia de una prueba corregida sigue requiriendo criterio del modelo. La detección de comandos y de errores ocultos es heurística, no un análisis completo de scripts o escrituras desde shell. Las regresiones locales reproducen el cierre contradictorio de la séptima tarea; no se repitió el benchmark para afirmar una mejora de puntuación.

## Aislamiento de evaluación y eficiencia del contexto (tareas 16–18)

La evaluación nativa desactiva el aprendizaje de skills antes de arrancar trabajadores. La política impide encolar aprendizaje, arrancar su trabajador o reactivarlo mediante herramientas, y aparece en report.json. No modifica el aprendizaje en producción ni los skills manuales. Así evita iniciar llamadas de aprendizaje que se cancelarían al cerrar la prueba; no trata el consumo desconocido como cero.

El contexto sustituye copias exactamente iguales de la solicitud por referencias, conservando el mensaje original, intenciones diferentes, dudas, alcance y registros durables. Las instrucciones permiten operaciones conocidas sin cargar un skill por formalidad y agrupar inspecciones independientes. No se añaden llamadas de resumen ni se descarta evidencia.

## Comprobaciones de requisitos y composición de herramientas

Antes de transformar datos desconocidos, se inspeccionan registros representativos, campos y unidades. Las instrucciones de cierre separan condiciones obligatorias y preferencias y piden comprobar el comportamiento pertinente, no solo existencia o formato. Los skills de pruebas y datos refuerzan esto sin otro modelo revisor. Los programas de herramientas exponen el esquema de pasos, usan nombres sin prefijos, aclaran referencias completas y rechazan programas inválidos antes de ejecutar efectos. Son mejoras de instrucciones y validación, no una garantía semántica general.

## Diagnóstico y reparación basados en evidencia (tareas 22–26)

La inspección del entorno incluye un resumen acotado y de solo lectura de Git cuando el repositorio está en la raíz del espacio de trabajo. No incluye contenido ni descubre repositorios superiores; la caché no representa un estado en vivo. Si la misma prueba falla tras una reparación observada, el contexto existente recomienda aislar un caso mínimo. Es orientación: no añade modelos, revisores, reglas de reintento ni permisos. La comprobación exitosa elimina el consejo. Los prompts y skills distinguen hipótesis de requisitos ausentes, escalan pruebas pertinentes y reutilizan datos. Se mantienen las preguntas legítimas y las comprobaciones de cierre. No constituye una prueba semántica de que el diagnóstico sea correcto.

## Procedencia de consultas y coherencia del resultado

Una pregunta que no puede entregarse en modo no interactivo ya no crea por sí sola evidencia para abandonar. Las decisiones, datos privados o autorizaciones indispensables deben indicar su tipo y citar el encargo o una actualización del usuario. Las dudas técnicas vuelven a investigación autorizada. Las preguntas del chat y las políticas de aprobación se mantienen. Comprobaciones acotadas en español e inglés rechazan justificaciones que reconocen que la pregunta era innecesaria.

El cierre existente rechaza requisitos declarados sin verificar. Una incertidumbre admitida exige reconciliar el resultado; non_required_limitations permite explicar una limitación ajena al encargo, pero no omitir comprobaciones obligatorias ni requisitos pendientes. Aplica al maestro y trabajadores normales; los revisores especializados conservan sus validadores. No añade llamadas a modelos. Son controles de procedencia, estado y texto conservadores, no una prueba semántica general. Los prompts y skills existentes priorizan metadatos reales, comprobaciones independientes de convenciones importantes, cobertura de datos y representaciones visuales legibles.

## Diagnósticos durables y recuperación real (tareas 27–31)

El estado de cierre y verificación conserva extractos del principio y final de los errores con el resumidor existente. La salida completa sigue recuperable; el fragmento omitido no es evidencia revisada. Una edición file_edit verificada con hashes idénticos no cuenta como reparación, reinicio del control de bucles ni cambio que invalide una prueba vigente. Sin hashes suficientes se conserva el comportamiento conservador.

El aviso de proceso en silencio incluye hasta 1.200 bytes recientes del log redactado y los marca como repetidos, sin mover los cursores. Así una pregunta anterior sigue visible aunque no haya salida nueva. No clasifica preguntas, responde, cancela ni cambia tiempos automáticamente. Conserva propiedad de tarea, cancelación y plazos explícitos.

Se refuerzan skills existentes: instalación sin diálogos innecesarios, reinicio de servicios propios editados, entrada/salida mínima en otro ABI, primitivas numéricas probadas, aserciones reales y límites de caja negra. No añade modelos ni revisores. Las cinco pruebas midieron el binario anterior congelado; los cambios posteriores solo tienen validación local. Véase la [auditoría](../benchmarks/fenix-tasks-27-31-luna-low-2026-09-26.es.md).

## TerminalWorld, Terminal-Bench 4.0 y Gaia2

La evaluación utiliza el motor real de Rust. Se añadieron selección reproducible de
los datasets oficiales y una sesión interactiva opcional para Gaia2. Esa sesión
conserva el mismo contexto entre notificaciones, evita repetir mensajes con el
mismo ID y envía respuestas a las aclaraciones por el mecanismo normal del agente.
Las notificaciones se encolan como trabajo nuevo y no responden aclaraciones pendientes.
Una respuesta terminada no cierra el escenario: lo decide el controlador externo.

`python scripts/benchmarks/prepare.py BENCHMARK --revision COMMIT --output DIRECTORIO`
prepara una tarea con revisión inmutable y hashes. Admite `terminalworld`,
`terminal-bench-4.0` y `gaia2-cli`. El índice cero corresponde al primer registro
oficial de TerminalWorld Verified, al primer directorio ordenado de Terminal-Bench
o al primer identificador ordenado del split de Gaia2 (por defecto `execution`).
El directorio debe ser nuevo y privado; sus verificadores y soluciones nunca deben
copiarse al contexto, skills o espacio de trabajo del agente.

Para los dos benchmarks de terminal, `scripts/benchmarks/harbor/run_native.py`
ejecuta un intento mediante el adaptador existente, conservando instrucciones,
recursos, límites y verificadores oficiales. Terminal-Bench 4.0 utiliza un
verificador separado y permite hasta ocho horas; esto no implica que cada tarea
dure ese tiempo.

En un servidor limitado se puede indicar `--override-cpus N`; la desviación queda
registrada en el trabajo de Harbor. No cambia la tarea ni el verificador, pero sus
tiempos no son directamente comparables con la asignación oficial.

Para Gaia2 se integra la **variante oficial Gaia2-CLI**. `gaia2/build.py` construye
la imagen sobre una revisión fija de ARE y `gaia2/run_native.py` usa su controlador,
daemon y juez oficiales. Python solo conecta el entorno con `fenix eval`; no
sustituye el arnés. El proceso del agente no puede leer los archivos privados del
escenario, que pertenecen a otro usuario del contenedor. Las herramientas oficiales
median las acciones sobre aplicaciones. La conexión utiliza una red Docker aislada
del espacio de red del host, con un puerto de control publicado en loopback.

Solo las respuestas finales y preguntas de aclaración marcan cambios de turno para
Gaia2. El progreso permanece en la traza nativa. El informe distingue errores del
entorno, resultado oficial, tokens, espera por cuotas y coste del juez. En esta
primera integración se usa Luna low también como juez configurable; no es el juez
de referencia calibrado del leaderboard. No se deben comparar sus puntuaciones
como si fueran protocolos idénticos.

El arranque comprueba el juez con su biblioteca real antes de iniciar el agente.
El adaptador descarta la temperatura 0 incompatible que trae esa biblioteca para
respetar la temperatura 1 configurada para Luna. Las reglas de calificación siguen
siendo las oficiales. La comprobación inicial consume unos pocos tokens.

Si falla el transporte del juez, `gaia2/rejudge.py` permite volver a calificar una
traza guardada de **un solo turno**, sin ejecutar otra vez al agente ni repetir sus
acciones. Conserva el resultado original y guarda el nuevo dictamen por separado,
con hashes de los archivos de entrada.

La [primera auditoría de tres tareas](../benchmarks/three-benchmarks-luna-low-2026-09-26.es.md)
detalla resultados, consumo, cambios del adaptador y límites de interpretación.

La guía [en inglés](../en/native-evaluation.md#terminalworld-terminal-bench-40-and-gaia2)
incluye revisiones, comandos y detalles de reproducción. Estos cambios no despliegan
ni modifican las sesiones de producción.

## Evidencia para recuperación y estimaciones de cuota

El cierre de un subagente corresponde a su encargo completo, no a un diagnóstico
intermedio. Que el análisis siga incompleto no demuestra un bloqueo terminal.
Puede guardar avances relevantes con `task_contract` y consultar
`task_inspect` con `recent: true` para recuperar las últimas evidencias de ejecución.
Esta vista acotada excluye transcripciones del proveedor e inspecciones recursivas,
y conserva la autorización por tarea/proyecto. Se contrasta el historial con los
archivos actuales antes de repetir acciones. La paginación anterior y la búsqueda
léxica siguen disponibles.
El control de cierre explícito también se activa tras lecturas y descubrimiento
de herramientas: un diagnóstico en texto no puede quedar registrado como tarea
completada. La conversación sin herramientas y la delegación activa se conservan.

Las auditorías de hitos incompletas entran una vez por ejecutor en la revisión de
reparación existente. Siguen bloqueadas hasta que una revisión del plan resuelva
los hallazgos; no se aprueban ni repiten acciones automáticamente. El veredicto
`needs_user` y las preguntas pendientes del auditor conservan su pausa. Los errores
de referencias identifican la llamada y entrada exactas para corregir la cita.
Las instrucciones de programación comprueban reglas sobre entradas cambiantes y
restricciones relacionadas mediante casos pequeños derivados del requerimiento;
un ejemplo correcto o una implementación fallida no demuestran una regla general
ni que el objetivo sea imposible.

El intermediario de evaluación usa el tamaño del JSON como estimación de cuota,
no como contador exacto de tokens. Una estimación excesiva reserva como máximo una
cuota completa observada y espera disponibilidad; no inventa un fallo del proveedor.
Las cabeceras de OpenAI y Retry-After siguen siendo la autoridad. El registro distingue
los tokens estimados de los reservados para auditar las esperas.
Las esperas de admisión se serializan para que solicitudes pequeñas posteriores no
consuman continuamente la cuota que necesita una solicitud grande en espera. Las
llamadas ya admitidas siguen ejecutándose en paralelo. Las interrupciones del
entorno se registran aparte de los fallos del agente y se repiten en entornos
limpios; no se debe pausar un contenedor activo para vaciar el intermediario,
porque la comprobación de procesos necesita que el contenedor esté disponible.

Las revisiones distinguen el trabajo previsto de los entregables terminados: que
un hito pendiente aún no produzca sus archivos no invalida su planificación.
El coordinador recibe las referencias de evidencia de los trabajadores asignados
y las claves exactas `rol/id_hallazgo`. Los errores de validación identifican la
clave o evidencia inválida. Una revisión aplicada correctamente es su resultado
explícito y permite cerrar sin una herramienta de ejecución no disponible para ese
rol. Se conservan la auditoría final, las preguntas al usuario y el aislamiento.

El resumen inicial de auditoría indica explícitamente si evalúa un hito o la tarea
integrada y muestra su entrega asignada. Un registro pendiente solicita verificación;
no demuestra por sí mismo que el trabajo falló. La reparación posterior se comprueba
al terminar ese hito y de nuevo en el cierre integrado.
