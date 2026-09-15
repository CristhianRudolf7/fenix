# Diez búsquedas simultáneas: implementación y medición real

[English — complete reference](../en/web-search-concurrency.md)

Fénix admite ahora **10 búsquedas o lecturas directas simultáneas**, con un límite
adicional de **10 descargas HTTP** compartido entre agentes y rastreador. La
undécima llamada espera un cupo. Se sustituyó el bloqueo global de HTTP por
coordinación por origen: esperar a una web deja avanzar a las demás.
`max_concurrent_searches` vale 10 por defecto y admite valores entre 1 y 10. Los
archivos anteriores que no contienen el campo adoptan automáticamente 10.

Cada búsqueda lee hasta tres fuentes en paralelo. Si varios lectores piden una
misma URL mientras se descarga, comparten la operación y su resultado, incluso
con la caché de artículos desactivada. También se combinan las peticiones de
robots.txt. Cancelar libera capacidad; esperar un cupo tiene un límite de 30
segundos, separado del presupuesto de lectura de 35 segundos. El rastreador cede
prioridad antes de empezar otra URL. El ejecutor del agente lanza juntas las
llamadas de respuestas que sólo contienen herramientas web y entrega al modelo
los resultados en orden. [Contrato completo](web-search.md#diez-búsquedas-simultáneas).

## Medición del 14 de septiembre de 2026

Se repitieron los diez temas y once fases de la [medición anterior](web-search-benchmark.md):
110 llamadas en un proceso aislado, sin modelo. El rastreador indexó 10 páginas
de 10 orígenes en 60 segundos, descubrió 883 URLs e intentó 28 solicitudes HTTP.
Inicialmente hubo resultados para cinco temas. Abrir explícitamente diez fuentes
conocidas añadió siete páginas; tres artículos de Wikipedia volvieron a exceder
512 KiB. Las consultas se midieron con 17 páginas de 11 orígenes y el rastreador
detenido. Siete temas devolvieron resultados; esto no mide precisión ni cobertura
general de internet.

Compilación dev con Rust 1.98.1, sin símbolos ni compilación incremental, dos
hilos Tokio. Sólo las fases `fresh_reads_*` desactivan la caché de artículos.
Conservan robots, pausas por sitio y revalidación HTTP 304.

| Condición: lote de diez | Tiempo total, s | Pico RSS, MiB | CPU, s | Intentos HTTP | Lecturas fallidas |
| --- | ---: | ---: | ---: | ---: | ---: |
| Sólo índice, diez simultáneas | 0,0301 | 25,93 | 0,03 | 0 | 0 |
| Lecturas con caché, secuenciales | 0,0534 | 26,43 | 0,03 | 0 | 0 |
| Lecturas con caché, diez simultáneas | 0,0306 | 26,68 | 0,03 | 0 | 0 |
| Lecturas HTTP, secuenciales | 26,3262 | 30,68 | 1,28 | 15 | 0 |
| Lecturas HTTP, cuatro simultáneas | 15,8827 | 32,77 | 0,82 | 12 | 0 |
| Lecturas HTTP, diez simultáneas | 17,8689 | 36,74 | 0,90 | 12 | 0 |

**Las diez consultas simultáneas con HTTP tardaron 17,87 segundos en total**,
frente a 35,01 segundos y un fallo de lectura antes del cambio. Se completaron
las 15 lecturas solicitadas: dos cuerpos nuevos, diez revalidaciones y tres
resultados compartidos. Fueron necesarios 12 intentos HTTP. Las diez consultas
con caché tardaron 30,64 ms juntas, frente a 18,85 ms antes; no se demuestra una
mejora en búsquedas puramente locales.

Cuatro simultáneas, en grupos de 4+4+2, tardaron 15,88 segundos y fueron más rápidas
que diez en esta muestra. Más concurrencia no elimina las pausas por sitio ni
garantiza menos latencia. Clima fue la consulta más lenta con 17,87 segundos; IA
tardó 14,99 segundos y salud 0,17 segundos. Los temas vacíos no hicieron lecturas.
Las pruebas con servidores simulados verifican el solapamiento y los límites;
una sola pasada con internet real no permite aislar el efecto de la variabilidad
de red, contenido remoto y orden de las solicitudes.

El pico del proceso fue **37.624 KiB (36,74 MiB)**. No incluye el modelo ni todo
el servicio Fénix. Índice, WAL y configuración ocuparon **2.014.026 bytes
(1,92 MiB)**, con 1.105 URLs conocidas y 79 intentos HTTP en toda la ejecución.
RSS se muestreó cada 20 ms y VmHWM cada 100 ms; CPU tiene resolución de 10 ms.
La memoria retenida por el asignador y el orden de las fases afectan la comparación.
No se midieron bytes transferidos ni corpus grandes.

## Verificación y reproducción

Las pruebas HTTP comprueban diez descargas solapadas, la undécima en espera,
cancelaciones, descargas compartidas, relevo del lector cancelado, pausas por
origen, avance de otro sitio y prioridad de consultas sobre rastreo nuevo. La
integración con modelo simulado verifica lotes de diez, orden, registro único de
resultados, validación y denegaciones anteriores.

```sh
cargo test --locked -p fenix-web
cargo test --locked -p fenix-agent -p fenix-server --test web_search
CARGO_PROFILE_DEV_DEBUG=0 CARGO_INCREMENTAL=0 cargo build --locked -p fenix-web --example benchmark
target/debug/examples/benchmark target/verification/web-benchmark-parallel-10.json
```

La última orden usa internet real y elimina su índice temporal al terminar.
[Datos completos](../benchmarks/web-search-parallel-10-2026-09-14.json), con tiempos,
resultados y estado de cada lectura. Esta prueba mide concurrencia y recursos;
ampliar la cobertura útil de búsqueda sigue siendo un trabajo independiente.
