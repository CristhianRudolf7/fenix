# Revisión de OpenClaw y mejoras incorporadas a Fénix

> Revisión histórica de 164 skills. El catálogo actual contiene 167 tras la [revisión de diez repositorios](ten-repository-skills-review.md); la evidencia siguiente corresponde a esta iteración anterior.

[English](../en/openclaw-skills-review.md) · [Decisiones por candidato](../skills/openclaw-review-matrix.md) · [Evidencia verificable](../skills/openclaw-review.json) · [Inventario del catálogo](../skills/openclaw-catalog-inventory.json)

## Resultado

Fénix pasa de **156 a 164 skills**. Se añadieron ocho procedimientos diferenciados, se mejoraron seis entradas existentes y se incorporaron cuatro auxiliares ejecutables propios y seis referencias. Las mejoras responden a necesidades concretas identificadas al comparar candidatos de OpenClaw con nuestra biblioteca real.

La aportación comprobada incluye comparación de feeds, detección de horas ambiguas, conciliación decimal de saldos y validación de subtítulos. Las instrucciones amplían la cobertura del agente, pero todavía no se midió su efecto en la tasa de éxito, coste o latencia de tareas completas ejecutadas por el modelo. La cantidad por sí sola no demuestra excelencia.

## Alcance real de la revisión

Se examinó la [versión `4367aac4da2a` del catálogo de VoltAgent](https://github.com/VoltAgent/awesome-openclaw-skills/tree/4367aac4da2a6202b56cf6dfdbab91076bbb8a1d), obtenida el 19 de septiembre de 2026. Sus treinta archivos de categorías contienen **5.196 entradas interpretables y 5.193 URLs distintas**, además de una entrada con enlace mal formado. No son necesariamente 5.193 capacidades independientes. Algunas categorías mezclan temas: aparecen herramientas contables bajo voz y plataformas sociales de agentes bajo juegos.

Se inventariaron todas las categorías y se seleccionaron candidatos por carencias de procedimiento, debilidades concretas o mecanismos que Fénix ya tiene. **Se leyeron completas 27 entradas de skills; una descarga adicional devolvió 404.** No se afirma haber leído los miles de cuerpos ni auditado todos sus scripts. La [matriz](../skills/openclaw-review-matrix.md) separa las decisiones por candidato del análisis general por categoría.

Los cuerpos se obtuvieron de las descargas `clawskills.sh/skills-markdown/…` enlazadas por el catálogo. Se registraron URL, fecha, tamaño y SHA-256. Son instantáneas de un espejo, no commits verificados del autor. La consulta al archivo histórico `openclaw/skills` de GitHub devolvió 404 en este entorno. La licencia MIT del índice no demuestra la licencia ni seguridad de cada paquete enlazado. No se copiaron ni ejecutaron cuerpos, instaladores, scripts o activos de origen.

## Ocho skills nuevos

| Skill | Mejora concreta | Dependencia o límite |
| --- | --- | --- |
| [feed-monitoring](../../resources/skills/feed-monitoring/SKILL.md) | Detecta novedades tardías y artículos modificados; separa lectura y entrega | Incluye comparador local RSS 2.0/Atom. No instala un monitor permanente. |
| [web-data-extraction](../../resources/skills/web-data-extraction/SKILL.md) | Extracción estructurada con esquema, paginación y procedencia por registro | Usa herramientas nativas o exportaciones. No incorpora un servicio externo de scraping. |
| [desktop-automation](../../resources/skills/desktop-automation/SKILL.md) | Comprueba ventana, foco y controles antes de actuar; verifica archivos guardados | Requiere sesión gráfica y adaptador compatible. No instala un controlador de escritorio. |
| [smart-home-control](../../resources/skills/smart-home-control/SKILL.md) | Resuelve dispositivos exactos, unidades y estado posterior | Requiere un controlador autorizado. No se operaron dispositivos reales. |
| [audio-production](../../resources/skills/audio-production/SKILL.md) | Narración con herramientas disponibles, duración medida, uniones y mezcla verificadas | Se probó procesamiento FFmpeg; no un servicio TTS ni calidad de voz. |
| [weather-briefing](../../resources/skills/weather-briefing/SKILL.md) | Diferencia observación y pronóstico, fechas locales, unidades y alertas oficiales | Necesita fuentes actuales; no crea un proveedor meteorológico. |
| [game-development](../../resources/skills/game-development/SKILL.md) | Ciclo jugable, tiempos de simulación, colisiones, pausa y reinicio | Respeta el motor solicitado. No instala un motor ni publica juegos. |
| [wearable-data-analysis](../../resources/skills/wearable-data-analysis/SKILL.md) | Analiza sueño y actividad con identidad de sesiones, unidades y cobertura | Usa archivos proporcionados o una conexión existente. No diagnostica ni importa credenciales. |

Estas separaciones tienen una razón funcional. Extraer un dataset requiere reconciliar registros y cobertura; investigar una pregunta requiere respaldar afirmaciones. Seguir feeds necesita identidad persistente y estado de entrega. Controlar una aplicación nativa requiere evidencias de foco y sesión distintas de las de una página web. Un dispositivo físico exige comprobar capacidades y estado real. No se creó un skill distinto para cada proveedor o CLI.

## Seis mejoras sobre lo existente

- **calendar-scheduling 1.1.0:** horas repetidas o inexistentes por cambios de zona, semántica ICS y límites de sincronización CalDAV. El auxiliar devuelve las interpretaciones UTC posibles sin elegir una reunión. Sincronizar puede publicar cambios locales pendientes.
- **learning-and-tutoring 1.1.0:** tarjetas con fuentes, práctica de recuperación, distinción entre nota y tarjeta de Anki y preservación del historial. Preparar un archivo no equivale a importarlo.
- **audio-transcription 1.1.0:** manifiesto de segmentos, offsets de tiempo global, revisión de uniones y comprobación SRT. Dividir audio no garantiza ausencia de errores de memoria o transcripción.
- **invoice-and-expense-records 1.1.0:** movimientos pendientes y contabilizados, pagos parciales o agrupados y transferencias. Un auxiliar decimal verifica saldos por cuenta y moneda sin categorizar ni pagar.
- **skill-source-review 1.1.0:** distingue un índice o sello de escáner de una revisión del código, registra procedencia y señala recursos pendientes de lectura. Se mejoró su descripción en español tras detectar un fallo de búsqueda.
- **scheduled-workflows 1.1.0:** incorpora expresiones naturales de tareas recurrentes y solapamiento tras un fallo observado. El procedimiento de programación existente ya era adecuado.

Las seis versiones anteriores se conservaron como fixtures y se registraron sus hashes. Las pruebas verifican actualización de originales conocidos, preservación de personalizaciones y estado desactivado, carga exacta de recursos y bloqueo de recursos de un skill desactivado. Las referencias personalizadas existentes no se sobrescriben automáticamente.

## Qué se aprovechó y qué se descartó

En feeds, el [procedimiento de digest](https://clawskills.sh/skills-markdown/odysseus0/feed-digest.md) marca como leídas las entradas seleccionadas. Nuestra versión distingue selección, lectura y entrega confirmada, para no perder pendientes si falla el envío. El comparador reconoce además una publicación recién incorporada con una fecha anterior a la última vista: un cursor basado únicamente en fecha podría omitirla.

En calendario, la [entrada CalDAV](https://clawskills.sh/skills-markdown/asleep123/caldav-calendar.md) prescribe sincronizar antes de consultar y propone una política donde gana el remoto. Nuestra guía comprueba cambios pendientes y antigüedad de la caché. El auxiliar usa transiciones IANA reales, incluidas las de media hora y días civiles omitidos. Se contrastó con [zoneinfo de Python](https://docs.python.org/3/library/zoneinfo.html).

Las entradas de [escritorio Linux](https://clawskills.sh/skills-markdown/ouyangabel/linux-desktop.md) y [desktop guardian](https://clawskills.sh/skills-markdown/s3rous/desktop-guardian.md) aportan operaciones útiles, pero presuponen herramientas X11 o un monitor permanente con cierre automático de aplicaciones. Fénix comprueba compatibilidad y observaciones recientes; no añade ese comportamiento persistente.

En domótica se conservaron las comprobaciones útiles de identidad y capacidades. También se explicita una diferencia importante de la [API de Home Assistant](https://developers.home-assistant.io/docs/api/rest/): escribir el estado registrado no comunica una orden al dispositivo; para controlarlo se usa el servicio correspondiente. Una respuesta HTTP satisfactoria no demuestra por sí sola un cambio físico.

El [generador de audio](https://clawskills.sh/skills-markdown/udiedrichsen/audio-gen.md) presupone credenciales, una ruta personal y un ritmo fijo de habla. Nuestra versión verifica herramienta y sintaxis y mide muestra y resultado. No se adoptaron las promesas generales de memoria o unión perfecta del [wrapper de Whisper](https://clawskills.sh/skills-markdown/neal-collab/auto-whisper-safe.md).

La [guía Rust/Godot](https://clawskills.sh/skills-markdown/kjaylee/game-dev-rust-godot.md) incorpora preferencias de otro propietario, ubicaciones privadas de activos y un destino de publicación. Se retuvieron pruebas de jugabilidad y exportación, respetando el motor y alcance que solicite nuestro usuario.

También se leyeron candidatos de memoria y mejora automática. Fénix ya tiene memoria por ámbito con evidencia, historial persistente, planes y diagnóstico de ejecuciones. Otro WAL, una base paralela de aprendizajes, escritura obligatoria antes de cada respuesta o umbrales que conviertan errores en reglas globales introducirían responsabilidades duplicadas. No se añadió un bucle automático de reescritura de políticas.

## Pruebas y medición

Se fijaron [54 casos de desarrollo](../skills/openclaw-expansion-cases.json) antes de editar: 32 para novedades, diez para las entradas inicialmente modificadas y doce para tareas vecinas. La línea base desactiva las ocho novedades y restaura los seis padres anteriores en el mismo runtime. No es una comparación entre binarios históricos completos ni una evaluación independiente del modelo.

| Grupo | Base: primero / entre los tres primeros | Resultado final: primero / entre los tres primeros |
| --- | ---: | ---: |
| 32 consultas nuevas | No disponibles por construcción | 31 / 32 |
| 10 consultas de skills modificados | 3 / 4 | 8 / 10 |
| 12 consultas vecinas | 11 / 11 | 12 / 12 |

El resultado final es **54/54 entre los tres primeros y 51/54 en primera posición**. Ningún skill nuevo desplaza como primer resultado una tarea vecina. La primera ejecución detectó una consulta de revisión de skills y un fallo previo de tareas recurrentes; se ajustaron descripciones/alias sin cambiar las consultas. Son resultados de desarrollo ajustados con esas pruebas, no una predicción del éxito real. La regresión anterior de cuatro repositorios conserva su fallo previo documentado de reparación de parsers.

Pasaron:

- **23 pruebas nativas de skills en Rust**, incluidas regresiones anteriores, comparación de búsqueda, migración y activación de recursos.
- **23 pruebas Python**, con publicaciones tardías, IDs conflictivos, límites XML, cambios de horario, exactitud decimal, pendientes, monedas incompatibles, subtítulos y códigos de salida. Los archivos de entrada permanecen intactos.
- Una comprobación FFmpeg real: unión de tonos de uno y dos segundos, codificación MP3 y decodificación. El resultado mide 3,024 segundos por la codificación, mono a 48 kHz. No demuestra síntesis o calidad de voz.
- Validación de formato, herramientas nativas, tamaños, catálogos bilingües, hashes de migración y enlaces locales; formato Rust y Clippy sin advertencias.

Para reproducir: `cargo test -p fenix-agent --lib skills:: --locked -- --nocapture`, `cargo clippy -p fenix-agent --lib --tests --locked -- -D warnings` y `PYTHONDONTWRITEBYTECODE=1 python3 tests/skills/test_openclaw_helpers.py`. Los auxiliares usan Python 3.9+ y datos IANA, sin dependencias de red. El manifiesto conserva versiones, hashes y resultados comparados.

Las nuevas entradas ocupan **24.054 bytes UTF-8**. Las modificadas pasan de **10.632 a 12.600 bytes** y los recursos complementarios suman **23.472 bytes**. Se cargan bajo demanda; esos tamaños no son medidas de tokens ni de ahorro por turno.

## Activación y límites pendientes

La ampliación está integrada en el código y catálogo del binario. Se activará al arrancar una compilación actualizada de Fénix. No se desplegó ni reinició un servicio activo ni se modificó la instalación separada de skills de Codex.

No se conectaron ni probaron aplicaciones nativas, domótica, cuentas de wearables, calendarios externos, scraping alojado, TTS o motores de videojuegos. Las dependencias quedan explícitas. La entrega aporta utilidades locales comprobadas y procedimientos utilizables sin atribuirle conexiones inexistentes al agente.

Para medir una mejora general hacen falta tareas completas independientes, evaluadas por artefactos entregados, acciones correctas, finalización y coste/latencia. El inventario queda disponible para revisiones dirigidas según necesidades reales; no es una recomendación de instalar miles de entradas.
