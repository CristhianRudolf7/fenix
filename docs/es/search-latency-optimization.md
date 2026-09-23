# Optimización de búsquedas — 20 de septiembre de 2026

[English — full report](../en/search-latency-optimization.md)

Las búsquedas repetidas ahora localizan las páginas pertinentes del índice y las
comprueban directamente en internet. En los cuatro casos individuales medidos,
pasaron de **8,40–13,32 segundos a 0,23–2,13 segundos**. Diez búsquedas conocidas
simultáneas pasaron de **28,49 a 4,77 segundos para el grupo completo**.

## Comparación real

Se ejecutó el mismo programa antes y después, con un índice temporal vacío al
inicio de cada ejecución. Primero hizo diez consultas simultáneas nuevas, después
cuatro repetidas individuales, diez repetidas simultáneas, una consulta nueva
relacionada y un control explícito sin red. Las búsquedas en vivo conservaron
`fresh: true`. No se usaron modelo, Chromium ni proveedores de búsqueda.

| Consulta repetida | Antes | Después | HTTP antes → después |
| --- | ---: | ---: | ---: |
| React useEffect cleanup | 10.040 s | 0.281 s | 15 → 1 |
| Python asyncio TaskGroup | 13.316 s | 1.353 s | 12 → 1 |
| SQLite transactions isolation | 8.398 s | 0.227 s | 5 → 1 |
| WHO dengue symptoms | 9.448 s | 2.134 s | 9 → 2 |

| Medida | Antes | Después |
| --- | ---: | ---: |
| Mediana de cuatro consultas repetidas | 9,744 s | 0,817 s |
| Diez consultas nuevas simultáneas, grupo completo | 23,731 s | 18,898 s |
| HTTP del grupo nuevo | 88 | 87 |
| Diez consultas conocidas simultáneas, grupo completo | 28,491 s | 4,770 s |
| HTTP del grupo conocido | 91 | 17 |
| Consulta nueva relacionada, Python pathlib | 10,120 s | 7,916 s |
| Control sin red | 159 ms | 21 ms |
| Pico de RAM del proceso de prueba | 69,3 MiB | 59,0 MiB |

Las diez consultas desde cero conservaron las mismas URLs y su orden. Todos los
pasajes de las búsquedas en vivo optimizadas se obtuvieron o revalidaron por red,
o compartieron una descarga en curso; ninguno se devolvió como caché. La mejora
no consiste en presentar una copia antigua como si acabara de comprobarse.

La regresión adicional de 24 consultas también conservó exactamente los conjuntos
de URLs anteriores: 22 casos con fuentes pertinentes, uno parcial de Context7 y
una receta sin resultados. Sus grupos de diez consultas nuevas tardaron 44,43 y
38,22 segundos, con picos de 68,3 y 77,7 MiB. Son búsquedas desde cero y no deben
confundirse con el grupo de consultas conocidas que tardó 4,77 segundos.

## Cambios

- Consulta del índice antes de rastrear, con revalidación directa y descubrimiento
  alternativo si una página falla o cambia. Las consultas de actualidad siguen
  buscando enlaces nuevos.
- Caché para navegar directorios, conexiones reutilizables y menos trabajo
  repetido al comparar texto y guardar enlaces.
- Enlaces observados pendientes priorizados para el rastreador existente, hasta
  256 en cola. Priorizar no significa que ya se hayan leído.
- Las lecturas del agente aplazan el siguiente rastreo de esas páginas para evitar
  descargas duplicadas. Se mantienen los límites, reintentos y diez operaciones
  simultáneas.
- Compartir dominio ya no basta para visitar un enlace sin relación con el tema.
- Skill actualizado a 1.5.0, sin añadir servicios de pago ni proveedores externos.

El conjunto de clientes HTTP está limitado a 32 orígenes. Cada petición sigue
validando DNS y rechazando direcciones privadas. Las pausas de cada sitio se
respetan; la optimización reduce peticiones y trabajo, no esas restricciones.

## Alcance de la medición

Es una muestra pequeña y elegida de temas técnicos y científicos, con una ejecución
por versión. La red varía y hubo compilaciones en la misma máquina. Las búsquedas
nuevas, los temas ausentes y los sitios bloqueados pueden tardar bastante más.
La RAM corresponde al proceso de prueba, no a todo Fénix con modelo y navegador.
No se midió CPU de forma aislada ni un índice lleno en funcionamiento continuo.

El rastreador de fondo estuvo apagado durante la comparación controlada, para que
su tráfico no distorsionara los grupos. En Fénix se usa el trabajador existente;
las bibliotecas deben iniciarlo mediante `Web::run()`.

## Evidencia

- [Antes](../benchmarks/search-latency-before-2026-09-20.json).
- [Después](../benchmarks/search-latency-after-2026-09-20.json).
- [Regresión con temas aleatorios](../benchmarks/retrieval-optimized-random-2026-09-20.json).
- [Regresión con temas complementarios](../benchmarks/retrieval-optimized-extra-2026-09-20.json).

Los informes conservan tiempos, URLs, procedencia, hashes y longitudes, sin copiar
los pasajes completos. Los contadores individuales de HTTP se superponen durante
la concurrencia; la comparación utiliza los totales de cada grupo. Los comandos
para repetir las pruebas están en el informe en inglés.

Pasaron **71 pruebas del motor y cuatro de integración con el agente**, Clippy,
formato, validación del skill y comprobación de compilación del servidor. Se
comprobaron cambios y fallos de fuentes, caché de navegación, noticias, reintentos,
prioridad del rastreo y límites de memoria de conexiones y cola. La integración
usa un modelo simulado; las búsquedas reales prueban directamente el motor.

La instalación que ya estaba abierta no se sustituyó ni reinició; necesita cargar
una compilación con estos cambios para utilizar esta versión optimizada.
