# Comparación de cuatro repositorios y ampliación de Fénix

> Revisión histórica de 156 skills. El catálogo actual contiene 167 tras la [revisión de diez repositorios](ten-repository-skills-review.md); las mediciones siguientes corresponden a la revisión anterior.

[English](../en/repository-skills-comparison.md) · [Matriz de las 50 entradas](../skills/repository-expansion-matrix.md) · [Evidencia verificable](../skills/repository-expansion-review.json)

Revisión del **19 de septiembre de 2026**, usando la lista corregida: `Imbad0202/academic-research-skills`, `greensock/gsap-skills`, `earthtojake/text-to-cad` y `huggingface/skills`. VoltAgent queda fuera de esta revisión. Este trabajo amplía la [comparación anterior con Superpowers](superpowers-comparison.md).

## Conclusión y alcance

Sí había funcionalidades útiles que incorporar. La biblioteca anterior tenía 145 skills y cubría investigación general, interfaces, entrenamiento, datasets y despliegue, pero faltaban procedimientos específicos para revisiones sistemáticas, auditoría de evidencias, animación, CAD, fabricación, robótica, operaciones del Hub e inferencia en navegador.

Se añadieron **11 skills propios y se reforzaron nueve existentes: Fénix pasa a 156**. Los 20 skills afectados incluyen seis auxiliares ejecutables, dos ejemplos reutilizables y 21 referencias. Hay mejoras concretas verificadas: comprobar vínculos entre afirmaciones y fuentes, recalcular recuentos de selección bibliográfica, generar y medir geometría exportada, revisar invariantes de DXF/URDF y comprobar animaciones en un navegador real.

Esto no demuestra todavía que el agente resuelva mejor cualquier tarea ni que supere globalmente a los cuatro proyectos. No se hizo una evaluación A/B con el modelo ejecutando tareas completas. El número de skills describe cobertura; por sí solo no mide calidad.

Se inventariaron las 50 rutas `SKILL.md`, sus tamaños y metadatos. Se revisaron README, licencias, arquitectura, registros de modos y secciones relevantes de los procedimientos y referencias. No se auditaron todas las líneas de los 5.274 archivos versionados ni se ejecutaron los programas completos de origen. No se incorporaron sus textos de skills, instaladores, hooks, servidor de visualización ni scripts de entrenamiento o despliegue: las instrucciones y el código añadidos son propios.

## Versiones y compatibilidad

| Repositorio | Revisión examinada | Entradas | Licencia raíz | Consecuencia |
| --- | --- | ---: | --- | --- |
| [academic-research-skills](https://github.com/Imbad0202/academic-research-skills/tree/1515a2192a6051f9a793cfd964bd05e3db20607c) | `1515a2192a605` | 4 | CC BY-NC 4.0 | Cuatro entradas grandes con 27 modos y orquestación ligada al entorno de origen. Conviene adaptar procedimientos concretos. |
| [gsap-skills](https://github.com/greensock/gsap-skills/tree/aed9cfd3277740755f6bfc1155c7aa645403b760) | `aed9cfd327774` | 8 | MIT | Aporta especialización útil, aunque algunas afirmaciones técnicas requieren corrección. |
| [text-to-cad](https://github.com/earthtojake/text-to-cad/tree/4ebbd2cc6172e4c5d53560a5948df214192aefac) | `4ebbd2cc6172e` | 12 | MIT | Once skills de producto y una guía interna de estilo. Gran parte de la capacidad procede de programas Python/JS. |
| [huggingface/skills](https://github.com/huggingface/skills/tree/abc20ae526d8b4c0e4dff89f904adce28a4a0eb6) | `abc20ae526d8b` | 26 | Apache 2.0 | Veinticinco skills principales y `hf-mcp`; algunos necesitan servicios, credenciales, paquetes o herramientas específicas. |

Fénix admite hasta 32.000 bytes por entrada cargada. Las cuatro entradas académicas superan ese límite: entre 37.392 y 52.683 bytes. También lo superan `hf-cli`, con 33.105, y `huggingface-lora-space-builder`, con 33.828. Parte de los metadatos no coincide con la convención de cadenas de Fénix; la guía interna `smui` ni siquiera lleva cabecera de skill. Copiar carpetas directamente no daría una biblioteca uniformemente utilizable.

La licencia no comercial del repositorio académico también desaconseja incorporar sus textos a esta biblioteca propia. En Hugging Face existe una licencia Apache en la raíz, pero algunas cabeceras remiten a un `LICENSE.txt` ausente en la versión examinada. Se registra la inconsistencia sin inferir permisos adicionales. La licencia del texto de los skills tampoco sustituye la de GSAP, de un modelo o de otra dependencia.

## Investigación académica

Los cuatro bloques de origen son `deep-research`, `academic-paper`, `academic-paper-reviewer` y `academic-pipeline`. Su registro contiene ocho modos de investigación, once de escritura, seis de revisión y dos de orquestación/reanudación. Lo valioso es separar actividades con criterios diferentes: explorar bibliografía, seleccionar estudios siguiendo un protocolo, escribir con resultados reales, revisar un manuscrito y responder observaciones.

Fénix ya disponía de `literature-review`, `scientific-writing`, `peer-review`, `bibliography-management` y `experiment-and-benchmark-design`. Se mantuvieron esas entradas y se añadieron dos capacidades diferenciadas:

- [systematic-review](../../resources/skills/systematic-review/SKILL.md): protocolo, búsquedas reproducibles, duplicados, decisiones de selección, razones de exclusión y textos no recuperados. Distingue publicaciones de estudios y recalcula totales desde registros identificables. El auxiliar no decide elegibilidad ni inventa revisores independientes. Cumplir una suma no acredita cumplimiento de [PRISMA 2020](https://www.prisma-statement.org/prisma-2020).
- [research-evidence-audit](../../resources/skills/research-evidence-audit/SKILL.md): vincula afirmaciones con versiones concretas del manuscrito y las fuentes, mediante hashes, líneas y citas. Conserva estados como respaldada, contradicha, exagerada o pendiente. El auxiliar verifica integridad estructural; no demuestra automáticamente que una cita respalde el argumento o que los datos sean auténticos.

La escritura incorpora un recorrido desde investigación hasta manuscrito y registro de revisiones. La revisión por pares añade criterios, evidencia, gravedad y respuesta a observaciones. La bibliografía diferencia una referencia correctamente identificada de una fuente que realmente respalda una frase. El diseño de experimentos conserva tarea, partición, versión, prompts, métrica y resultados.

No se trasladaron equipos fijos de agentes, hooks del entorno de origen ni aprobaciones reiteradas. Fénix conserva sus planes, historial y permisos. Tampoco se reprodujo el motor completo del pipeline académico ni se validó la calidad de una revisión sistemática sobre un corpus real durante este trabajo.

## GSAP y animación web

Las ocho entradas cubren núcleo, frameworks, React, rendimiento, plugins, ScrollTrigger, timelines y utilidades. El skill de interfaces existente cubría estructura y accesibilidad, pero no definía bien quién controla una animación, cuándo se recalculan posiciones ni cómo se limpia al desmontar un componente.

Se creó [web-animation](../../resources/skills/web-animation/SKILL.md), con referencias específicas de ciclo de vida y scroll/plugins. Incluye selectores limitados al componente, secuencias con un propietario claro, ejecución en navegador, limpieza, preferencias de movimiento reducido, orden de pinning y medición real del rendimiento. El ejemplo `scoped-reveal.mjs` recibe una instancia existente de GSAP y un elemento raíz; conserva contenido estático legible y devuelve una función de limpieza que puede ejecutarse más de una vez.

Se corrigieron dos puntos del material de origen:

1. Su guía de ScrollTrigger indica que una prioridad `refreshPriority` menor se recalcula antes. La [documentación oficial](https://gsap.com/docs/v3/Plugins/ScrollTrigger/static.sort()/) establece que los valores mayores van primero. Nuestra referencia usa ese orden y una prueba con GSAP real lo confirma.
2. La guía de React atribuye a `contextSafe` la capacidad de volver inofensivos los callbacks después del desmontaje. Su función es asociar animaciones creadas posteriormente al contexto; todavía hay que retirar listeners y gestionar trabajo asíncrono pendiente. Nuestra guía sigue la [documentación de integración con React](https://gsap.com/resources/React/).

Pasaron once comprobaciones en Chromium: aislamiento de selectores, visibilidad final, restauración de estilos, limpieza repetida, independencia entre componentes, movimiento reducido inicial y cambiante, rechazo de parámetros inválidos, orden de prioridad y contenido visible sin JavaScript. Esta evidencia cubre el ejemplo y la afirmación sobre prioridades; no todos los plugins, frameworks o animaciones posibles. La dependencia usada para las pruebas quedó aislada fuera del producto Fénix.

## CAD, fabricación y robótica

`text-to-cad` contiene un paquete `cadgen`, un visor y utilidades de exportación e integración. Sus instrucciones presuponen esos programas. Por eso se implementaron capacidades propias con dependencias y límites visibles:

| Funcionalidad de origen | Versión de Fénix | Límite actual |
| --- | --- | --- |
| `cad`, `step-parts`, `cad-viewer` | [parametric-cad](../../resources/skills/parametric-cad/SKILL.md), ejemplo editable en build123d, inspección STEP/mallas y procedencia de componentes | No se creó un visor servidor, un descargador universal de catálogos ni un generador de cualquier pieza. |
| `dxf`, `sendcutsend` | [cad-drawing-and-fabrication](../../resources/skills/cad-drawing-and-fabrication/SKILL.md), comprobación de unidades, capas, entidades y dimensiones | No compra piezas ni certifica cualquier perfil de corte, tolerancia o plegado. |
| `urdf`, `srdf`, `sdf` | [robot-description](../../resources/skills/robot-description/SKILL.md), comprobación de topología, límites, ciclos e inercias de URDF y guía por formato | La planificación SRDF y simulación SDF requieren herramientas nativas. No se verifican automáticamente mallas y comportamiento físico. |
| `dfam-check`, `gcode`, `bambu-labs` | [3d-printing-preparation](../../resources/skills/3d-printing-preparation/SKILL.md), requisitos de malla, perfiles reales de laminado y entrega a máquina | No hay intérprete universal de G-code ni controlador de impresora; no se realizó una impresión física. |
| Guía interna `smui` | Las convenciones de interfaz ya tienen destino en los skills de diseño | Se excluyeron reglas de apariencia particulares del visor ajeno. |

La prueba de geometría construye una placa de 40 × 30 × 4 mm con dos agujeros pasantes de 5 mm, exporta STEP y STL y vuelve a abrirlos. Compara dimensiones y volumen contra una fórmula independiente: `40*30*4 - 2*pi*(2.5**2)*4`. También se comprueban mallas abiertas y discrepancias de unidades/dimensiones. Los casos DXF incluyen archivos guardados y entrada corrupta. Los casos URDF incluyen árboles inválidos, límites invertidos, ejes incorrectos, ciclos y cálculos de inercia no finitos o no positivos.

Una geometría válida no demuestra resistencia, tolerancia de montaje o fabricabilidad. Una matriz de inercia positiva tampoco demuestra que corresponda a la pieza real. Las comprobaciones físicas y de colisiones siguen siendo necesarias según el trabajo. La [documentación de MoveIt](https://moveit.picknik.ai/main/doc/examples/urdf_srdf/urdf_srdf_tutorial.html) distingue la descripción geométrica de la semántica de planificación; el auxiliar propio no reemplaza MoveIt o Gazebo.

## Hugging Face

Se añadieron cuatro destinos: [huggingface-hub](../../resources/skills/huggingface-hub/SKILL.md), [model-selection](../../resources/skills/model-selection/SKILL.md), [ml-demo-apps](../../resources/skills/ml-demo-apps/SKILL.md) y [browser-ml-inference](../../resources/skills/browser-ml-inference/SKILL.md). Entrenamiento, preparación de datos, despliegue y evaluación siguen en los skills existentes, reforzados con referencias.

| Grupo de origen | Aportación incorporada | Adaptación o dependencia |
| --- | --- | --- |
| CLI, datasets, papers/publicación, constructor de herramientas y MCP | Tipo de repositorio, revisiones, archivos seleccionados, config/split/paginación, procedencia e identificadores de Jobs | Usa CLI/API/herramientas disponibles. No presupone `hf_jobs` ni sube automáticamente sesiones del agente. |
| Selección del mejor modelo, `hf-mem`, modelos locales | Comparación por tarea, artefacto GGUF exacto y separación de pesos, caché KV y sobrecoste de ejecución | El estimador propio recibe dimensiones explícitas. No reproduce la inspección remota de cabeceras ni todas las arquitecturas que contempla el proyecto externo. |
| LLM trainer, TRL, visión, sentence transformers y Trackio | Objetivos, etiquetas, negativos, transformaciones de imágenes, modelos/adaptadores guardados y visibilidad del seguimiento | No fuerza entrenamiento remoto ni publicación. No se entrenó ni descargó un modelo durante esta revisión. |
| Gradio, Spaces, LoRA y ZeroGPU | Resultados reales, estado por usuario, colas/cancelación, compatibilidad del adaptador y ejecución GPU | El alojamiento necesita acceso externo. Los [requisitos de ZeroGPU](https://huggingface.co/docs/hub/spaces-zerogpu) y sus cuotas se verifican al usarlo. |
| Seis skills de AWS/SageMaker | Cuenta/región, entorno, roles, cuotas, contenedor compatible, modalidad de servicio y limpieza por identificador | Se integran en despliegue, entorno y acceso. No se copiaron sus scripts ni se eligió una imagen solo por preferencia de proveedor. |
| Evaluaciones comunitarias | Tareas, prompts, versiones, dirección de métricas y comparación reproducible | No presupone una ruta privada `~/code/community-evals` ni el skill ausente `hugging-face-jobs`. |
| Transformers.js | Compatibilidad ONNX/procesador, workers, WebGPU/WASM, descargas, caché y liberación de recursos | Se incorporó el procedimiento; no se ejecutó un modelo ONNX en navegador. Se consulta la [documentación de la versión usada](https://huggingface.co/docs/transformers.js/en/index). |

Se evitó convertir aproximaciones en garantías. Una de las rutas de recomendación de origen estima memoria desde tamaños de parámetros sin establecer el consumo total de ejecución. Nuestra guía exige metadatos reales y separa pesos, caché y sobrecoste. `hf-mem` ya aporta inspección remota más sofisticada y soporte experimental de caché: el pequeño estimador propio no se presenta como sustituto completo.

También se descartaron instrucciones de envío inmediato a cómputo remoto y publicación obligatoria cuando el usuario solo pidió preparación local. La selección de datos para subir no incluye por defecto carpetas enteras de sesiones. La visibilidad de paneles alojados debe corresponder al alcance del trabajo.

Se realizaron tres lecturas públicas acotadas: metadatos de un modelo con revisión resuelta, descubrimiento de particiones de un dataset y dos filas de una configuración/partición concreta. Las tres respondieron correctamente. Esto verifica esas peticiones, no un entrenamiento, despliegue ni una ejecución completa del modelo de Fénix. La [API de filas](https://huggingface.co/docs/dataset-viewer/en/rows) se trata como consulta paginada; una vista previa no equivale a descargar el corpus.

## Pruebas, resultados y migración

El [conjunto de 65 casos](../skills/repository-expansion-cases.json) contiene 44 consultas bilingües de capacidades nuevas, nueve de skills existentes modificados y doce de tareas vecinas. Se redactó después de crear los skills y quedó fijado antes de la primera ejecución nativa; es un conjunto de regresión de desarrollo, no una evaluación independiente.

La línea base desactiva los once skills nuevos y restaura las nueve entradas originales mediante copias exactas. Compara ambos catálogos dentro del runtime actual; no ejecuta una versión histórica completa de la aplicación.

| Casos | Línea base: esperado primero / entre los tres primeros | Resultado nuevo: esperado primero / entre los tres primeros |
| --- | ---: | ---: |
| 44 de capacidades nuevas | No disponibles por construcción | 44 / 44 |
| 9 de skills existentes | 6 / 9 | 9 / 9 |
| 12 de tareas vecinas | 11 / 11 | 11 / 11 |

Ningún skill añadido se convierte en primer resultado de una tarea vecina. La consulta «Repair a regression in this Rust parser.» no recupera `test-driven-repair` entre los tres primeros en ninguna fase; `software-implementation` sigue primero. La primera ejecución detectó ese fallo previo y se corrigió la aserción para exigir que en esos casos se conserve el ganador anterior. El fallo permanece publicado: la recuperación esperada es **64/65**, no 65/65. La ausencia deliberada de skills nuevos en la línea base no permite anunciar una mejora de éxito del 0 al 100 %.

Comprobaciones finales superadas:

- **20 pruebas nativas de skills en Rust**, incluidas regresiones previas, búsqueda comparada, carga de recursos y migración. Clippy también pasó con advertencias tratadas como errores.
- **22 pruebas Python**, con geometría, mallas y dibujos reales, entradas inválidas y resultados numéricos independientes.
- **11 comprobaciones de animación en navegador**, con GSAP 3.15.0 y Chromium 153.0.8010.47.
- **Tres lecturas públicas del Hub**, sin credenciales, escrituras, trabajos remotos ni descarga de pesos.

El primer intento nativo encontró una caché compartida de Cargo con un catálogo generado antiguo de 145 entradas. Se regeneró antes de la ejecución válida de 156 entradas. Se distingue ese problema de compilación del fallo previo de recuperación. No se ejecutó toda la batería de la aplicación ni una evaluación A/B del modelo para este cambio.

Las nueve entradas anteriores se conservan como fixtures y sus hashes exactos se registran para la migración. Las pruebas confirman que se actualizan originales conocidos y se preservan textos personalizados y estados desactivados. Los auxiliares y referencias se cargan por la ruta nativa de recursos; desactivar un skill impide también cargar sus recursos.

Las 156 entradas pasaron la validación de formato; la mayor ocupa 6.539 bytes. También se comprobaron límites de archivos y paquetes, 49 hashes de artefactos, nueve hashes de migración, ambos catálogos y 517 enlaces locales. Esto verifica coherencia de la entrega, no la calidad semántica de cada instrucción.

Comandos para reproducir las pruebas desde la raíz del repositorio:

```bash
cargo test -p fenix-agent --lib skills:: --locked -- --nocapture
cargo clippy -p fenix-agent --lib --tests --locked -- -D warnings
PYTHONDONTWRITEBYTECODE=1 python3 tests/skills/test_repository_helpers.py
node tests/skills/test_web_animation.mjs /path/to/isolated/node_modules/gsap
```

Se usaron Python 3.13.5, build123d 0.11.1, trimesh 5.0.0, NumPy 2.5.2 y ezdxf 1.4.4. La prueba web utiliza Playwright del workspace, `/usr/bin/chromium` y GSAP 3.15.0 instalado por separado. El manifiesto registra versiones, hashes de pruebas, configuración de compilación, casos individuales y las peticiones públicas exactas. Las respuestas remotas pueden cambiar; la evidencia describe esta ejecución.

## Coste, activación y siguiente mejora útil

Las once entradas nuevas ocupan **36.347 bytes UTF-8**. Las nueve modificadas pasan de **16.576 a 19.537 bytes**. Los recursos complementarios de las veinte suman **64.927 bytes**. Son tamaños de archivos, no tokens ni coste por turno. La carga es bajo demanda y las dependencias externas siguen siendo requisitos explícitos.

La ampliación está incorporada al código y al catálogo que se integra en el binario. Se aplicará al compilar y arrancar la versión actualizada de Fénix; esta tarea no desplegó ni reinició un servicio activo. Tampoco modificó la instalación separada de skills de Codex. Un skill personalizado conserva su contenido y puede necesitar una integración deliberada de las mejoras.

La cantidad añadida responde a diferencias reales de procedimiento y descubrimiento. Para seguir mejorando conviene medir tareas completas con entradas independientes, artefactos evaluados, herramientas realmente disponibles y coste/latencia. Reproducir todo el visor CAD, conectar impresoras, inspeccionar automáticamente cabeceras de modelos o ejecutar ML alojado requiere implementación y pruebas propias; añadir más Markdown no proporciona esas integraciones.
