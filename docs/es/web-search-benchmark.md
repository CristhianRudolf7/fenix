# Diez búsquedas y concurrencia — 2026-09-14

[English](../en/web-search-benchmark.md)

Esta es la medición histórica **anterior a implementar descargas concurrentes**. Se usó HTTP público, sin proveedor de búsqueda ni llamadas al modelo. Las descargas estaban serializadas y diez consultas sin caché produjeron un fallo de lectura. El estado actual está en la [implementación y nueva medición con límite de diez](web-search-concurrency.md).

## Método

Un proceso aislado, dos hilos Tokio, compilación dev sin símbolos de depuración, Rust 1.98.1. Diez consultas fijas en inglés repetidas en once condiciones: 110 llamadas a búsqueda. Primero se midió el índice vacío; luego se dejó rastrear 60 segundos con las semillas y límites predeterminados. Se detuvo el rastreo durante las mediciones de consultas.

El rastreador indexó 10 páginas, descubrió 883 URLs e intentó 29 solicitudes HTTP en ese minuto. Sólo cinco temas devolvieron resultados. Después se abrieron explícitamente diez URLs conocidas para medir la lectura con un corpus más útil: siete funcionaron, y tres páginas de Wikipedia excedieron 512 KiB. **Esta ampliación manual no demuestra descubrimiento autónomo de esos temas.** El corpus llegó a 17 páginas de 11 orígenes; siete temas devolvieron resultados.

Se compararon lotes secuenciales, grupos concurrentes de 4+4+2 y diez llamadas simultáneas a la misma instancia `Arc<Web>`. Las pruebas locales usan `read_pages=0`; las demás leen hasta tres resultados. Caché predeterminada de 15 minutos; para forzar HTTP se reabrió el mismo índice con `cache_minutes=0`. Se conservaron robots, ETags, límites por origen y presupuesto diario.

## Resultados por tema

| Tema / consulta | Resultados | Caché secuencial, ms | HTTP secuencial, s | HTTP con 10 simultáneas, s |
| --- | ---: | ---: | ---: | ---: |
| Astronomía: `solar system` | 5 | 4.88 | 0.574 | 35.008 * |
| Salud: `malaria` | 1 | 2.27 | 0.175 | 0.125 |
| Clima: `climate change` | 5 | 5.55 | 19.921 | 25.533 |
| Economía: `poverty` | 2 | 3.03 | 4.017 | 17.272 |
| Historia: `Roman Empire` | 0 | 0.93 | 0.002 | 0.006 |
| Biología: `photosynthesis` | 0 | 1.07 | 0.003 | 0.019 |
| Programación: `installation Rust` | 3 | 3.05 | 2.234 | 25.438 |
| Diseño web: `CSS` | 2 | 2.41 | 3.964 | 17.202 |
| Investigación IA: `attention transformer` | 1 | 2.19 | 0.063 | 14.995 |
| Matemáticas: `Pythagorean theorem` | 0 | 0.99 | 0.001 | 0.017 |

\* Astronomía: dos lecturas completadas y una agotó el presupuesto de 35 segundos. Los tres temas sin resultados no realizaron lecturas; sus tiempos pequeños no representan búsquedas útiles. Los resultados incluyen coincidencias parciales: por ejemplo, la consulta climática también devolvió un artículo de IA por la ampliación léxica. No se midió precisión con un conjunto etiquetado.

## Resultados por lote de diez

| Condición | Tiempo total, s | Pico RSS, MiB | CPU, s | Intentos HTTP | Lecturas fallidas |
| --- | ---: | ---: | ---: | ---: | ---: |
| `empty_index` | 0.0080 | 12.57 | 0.00 | 0 | 0 |
| `default_corpus` | 0.0479 | 22.82 | 0.04 | 0 | 0 |
| `index_sequential` | 0.0184 | 27.91 | 0.02 | 0 | 0 |
| `index_parallel_4` | 0.0149 | 27.91 | 0.02 | 0 | 0 |
| `index_parallel_10` | 0.0137 | 28.32 | 0.02 | 0 | 0 |
| `cached_reads_sequential` | 0.0277 | 28.96 | 0.03 | 0 | 0 |
| `cached_reads_parallel_4` | 0.0191 | 29.15 | 0.04 | 0 | 0 |
| `cached_reads_parallel_10` | 0.0189 | 29.29 | 0.03 | 0 | 0 |
| `fresh_reads_sequential` | 30.9567 | 32.86 | 1.10 | 15 | 0 |
| `fresh_reads_parallel_4` | 24.4349 | 36.78 | 0.99 | 15 | 0 |
| `fresh_reads_parallel_10` | 35.0081 | 38.85 | 1.22 | 15 | 1 |

## Interpretación y límites

- Las diez consultas con caché tardaron 18,85 ms juntas; una consulta individual esperó hasta 18,54 ms. En secuencia, cada consulta fue más rápida, pero el lote duró 27,73 ms. Concurrencia no equivale a menor latencia individual.
- Sin caché, el lote secuencial tardó 30,96 s; cuatro simultáneas, 24,43 s; diez, 35,01 s y un fallo de lectura entre 15 lecturas solicitadas. Las llamadas admiten concurrencia, pero un mutex comparte una sola vía HTTP, otro acceso serializa SQLite y sólo se analiza un HTML a la vez. Las esperas de red, políticas de sitios y orden de las solicitudes influyen; una sola pasada no prueba una aceleración estable.
- El pico del proceso fue 39.784 KiB (**38,85 MiB**). Índice, WAL y configuración ocuparon 2.026.285 bytes (**1,93 MiB**) y quedaron 1.105 URLs conocidas. Hubo 86 intentos HTTP en toda la ejecución.
- RSS se muestreó cada 20 ms por fase; además se observó VmHWM del proceso cada 100 ms. La memoria retenida por el asignador y el orden de las fases impiden atribuir toda diferencia de RAM sólo a la concurrencia. CPU se leyó de `/proc/self/stat`, con resolución de 10 ms: un cero no significa trabajo nulo.
- No se midieron el modelo, todo el servicio Fénix, bytes transferidos ni un corpus de miles de páginas. Compilar queda fuera de los tiempos. El índice temporal se eliminó al terminar.
- Para la implementación medida aquí, usar caché y limitar a cuatro consultas con lectura simultáneas era una opción prudente basada en esta muestra. Desde entonces se implementaron descargas concurrentes y combinación de solicitudes duplicadas; la cobertura y el filtrado siguen siendo aspectos independientes.

## Reproducir

```sh
CARGO_PROFILE_DEV_DEBUG=0 CARGO_INCREMENTAL=0 cargo build --locked -p fenix-web --example benchmark
target/debug/examples/benchmark target/verification/web-benchmark.json
```

[Datos completos](../benchmarks/web-search-2026-09-14.json). Las URLs de preparación, resultados, errores y tiempos individuales están conservados en JSON. La prueba usa internet real y los resultados pueden cambiar.
