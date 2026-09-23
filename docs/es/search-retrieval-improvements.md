# Mejoras de búsqueda — 20 de septiembre de 2026

[English — full report](../en/search-retrieval-improvements.md)

El buscador ahora encuentra referencias reales en consultas de React, Next.js y
Tokio que antes devolvían gob.pe o NASA. Las herramientas del agente revalidan las
fuentes por HTTP por defecto. La cobertura sigue siendo la de un rastreador e
índice propios acotados; no equivale a tener toda internet indexada.

## Correcciones

- Selección de fuentes iniciales pertinentes, normalización de consultas y filtro
  de coincidencias parciales con proximidad entre términos.
- Lectura de páginas de hasta 2 MiB y texto acotado a 64.000 caracteres; conservación
  de indentación y líneas de código, con pasajes de hasta 6.000 caracteres.
- Hasta 2.000 enlaces observados por documento, acumulación de nombres de secciones
  y seguimiento de enlaces útiles aunque la cola de rastreo esté llena.
- Procedencia de enlaces para aislar búsquedas simultáneas; mayor prioridad de
  rutas específicas y al menos dos oleadas antes de parar por coincidencia.
- `fresh: true` por defecto, actualización de fragmentos y descarte de páginas que
  ya no coinciden. Reutilización de una lectura comprobada en la misma búsqueda.
- Skill actualizado para comprobar versiones, pertinencia e idioma, y afinar con
  un dominio cuando la primera consulta solo aporta una pista.

## Resultados medidos

Se usó el motor Rust real, sin APIs de búsqueda externas, sin Chromium y sin un
modelo. Las 24 consultas partieron de índices temporales vacíos. La primera tanda
se eligió con semilla 20260920 de una lista de 24 temas; la segunda contiene los
12 restantes. La lista favorece temas técnicos y fuentes conocidas. Se usaron ambas
tandas para mejorar el código: no son una evaluación ciega ni una muestra
representativa de toda internet.

**23 de 24 consultas devolvieron fuentes: 22 se clasificaron como pertinentes,
una como parcial y una quedó vacía.** Esta revisión de fuentes no mide la exactitud
de una respuesta final del modelo. Algunos resultados secundarios siguen siendo
páginas generales o traducciones a otros idiomas.

| Medida | Resultado |
| --- | --- |
| Mediana por consulta en frío | 12,92 s |
| Rango por consulta en frío | 10,17–42,86 s |
| Diez búsquedas simultáneas, tanda A | 50,87 s para el grupo completo |
| Diez búsquedas simultáneas, tanda B | 41,23 s para el grupo completo |
| Pico de RAM del proceso de prueba | 62,7 y 75,9 MiB |
| Intentos HTTP de los grupos simultáneos | 103 y 105, incluidos robots/redirecciones |
| Pruebas automatizadas | 60 del motor + 4 de integración con el agente |

En la tanda A, las nueve consultas con resultados acabaron antes de 32,66 s;
la receta sin resultados terminó a los 50,86 s. El tiempo de cada grupo no se
multiplica por diez. Se mantiene la separación entre peticiones al mismo origen.
La RAM excluye el modelo, navegador y servidor de Fénix; tampoco es un límite duro.
Las pruebas de documentación ocuparon entre 0,37 y 1,90 MiB de índice temporal por
caso. No se midió CPU de forma aislada ni un índice lleno en funcionamiento continuo.

| Consulta | Segundos | Revisión de la fuente | Fuente seleccionada |
| --- | ---: | --- | --- |
| IPCC climate change | 10.65 | Pertinente | [Página](https://www.ipcc.ch/) |
| Python asyncio TaskGroup | 16.88 | Pertinente | [Página](https://docs.python.org/3/library/asyncio-task.html) |
| receta peruana de ají de gallina | 42.86 | Vacío | — |
| Docker multi stage builds | 10.30 | Pertinente | [Página](https://docs.docker.com/get-started/docker-concepts/building-images/multi-stage-builds/) |
| React useEffect cleanup | 10.58 | Pertinente | [Página](https://react.dev/reference/react/useEffect) |
| Git rebase conflicts | 11.78 | Pertinente | [Página](https://git-scm.com/docs/git-rebase) |
| Busca documentación actualizada de React useEffect cleanup | 12.89 | Pertinente | [Página](https://react.dev/reference/react/useEffect) |
| NASA Mars atmosphere | 12.97 | Pertinente | [Página](https://science.nasa.gov/mars/) |
| Tokio timeout | 12.95 | Pertinente | [Página](https://docs.rs/tokio/latest/tokio/time/fn.timeout.html) |
| NASA solar eclipse | 14.24 | Pertinente | [Página](https://science.nasa.gov/science-research/heliophysics/nasa-science-soars-during-august-total-solar-eclipse/) |
| World Bank poverty | 12.65 | Pertinente | [Página](https://www.worldbank.org/ext/en/topic/poverty/measuring-poverty) |
| PostgreSQL window functions | 11.64 | Pertinente | [Página](https://www.postgresql.org/docs/current/functions-window.html) |
| Next.js proxy middleware | 11.08 | Pertinente | [Página](https://nextjs.org/docs/app/getting-started/proxy.md) |
| SQLite transactions isolation | 10.17 | Pertinente | [Página](https://www.sqlite.org/isolation.html) |
| Rust ownership borrowing | 27.04 | Pertinente | [Página](https://rust-book.cs.brown.edu/ch04-00-understanding-ownership.html) |
| Kubernetes deployment rolling update | 17.81 | Pertinente | [Página](https://kubernetes.io/docs/tasks/run-application/update-deployment-rolling/) |
| TypeScript generics | 12.87 | Pertinente | [Página](https://www.typescriptlang.org/docs/handbook/2/generics.html) |
| CSS grid layout | 14.19 | Pertinente | [Página](https://developer.mozilla.org/en-US/docs/Web/CSS/Guides/Grid_layout) |
| NASA Jupiter moons | 14.45 | Pertinente | [Página](https://science.nasa.gov/jupiter/jupiter-moons/) |
| WHO malaria prevention | 10.71 | Pertinente | [Página](https://www.who.int/activities/promoting-malaria-chemoprevention) |
| WHO dengue symptoms | 16.32 | Pertinente | [Página](https://www.who.int/health-topics/dengue-and-severe-dengue) |
| Context7 API context | 13.31 | Parcial | [Página](https://github.com/upstash/context7) |
| Python pathlib Path | 10.33 | Pertinente | [Página](https://docs.python.org/3/library/pathlib.html) |
| historia del imperio inca | 22.77 | Pertinente | [Página](https://es.wikipedia.org/wiki/Portal:Imperio_incaico) |

Context7 devolvió su README al buscar solo por texto: una pista parcial. Indicando
`domains: ["context7.com"]`, el control encontró la referencia del endpoint de
contexto en 10,38 s. La receta siguió fuera de cobertura. La consulta histórica
incluyó fuentes en español, pero ordenó antes versiones en otros idiomas.

También se repitieron 11 controles de documentación: consultas sin URLs, filtros
por dominio, URLs explícitas y caché desactivada. Las URLs explícitas son controles
asistidos y no cuentan como descubrimiento autónomo. El método Rust `read()`
conserva su caché por compatibilidad; el agente usa `read_with_fresh()`.

## Evidencia y comprobaciones

- [Primera tanda](../benchmarks/retrieval-random-2026-09-20.json).
- [Segunda tanda](../benchmarks/retrieval-holdout-2026-09-20.json).
- [Controles de documentación](../benchmarks/live-documentation-search-improved-2026-09-20.json).
- [Medición original](../benchmarks/live-documentation-search-2026-09-20.json).

Los informes conservan URLs, tiempos, errores, procedencia y longitudes/hashes de
los pasajes; el texto completo se retiró después de revisarlo. Los archivos
initial/intermediate corresponden a iteraciones previas. El archivo llamado
holdout contiene la tanda complementaria, que también se usó durante los ajustes.

Pasaron Clippy estricto, formato, validación del skill y compilación de comprobación
del servidor, además de las 64 pruebas. La integración del agente usa un proveedor
de modelo simulado; las búsquedas reales probaron el motor directamente. Los
comandos para repetirlas están en el [informe en inglés](../en/search-retrieval-improvements.md).

## Límites y activación

Todavía pueden faltar temas arbitrarios. Los bloqueos, sesiones obligatorias,
contenido que requiere JavaScript, páginas demasiado grandes y ambigüedades léxicas
no desaparecen. Una lectura reciente tampoco prueba veracidad ni exhaustividad.
No se añadieron proveedores de búsqueda ni se importaron sesiones privadas.

Los cambios están programados y probados en el proyecto. La instalación de Fénix
que ya estaba ejecutándose no se sustituyó ni reinició: requiere recompilar y
desplegar esa instalación para cargar esta versión. Se ajustaron los valores
antiguos predeterminados de la configuración local del workspace; los límites
personalizados de otras instalaciones y los skills personalizados se conservan.
