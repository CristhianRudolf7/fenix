# Verificación del buscador propio — 2026-09-19

[English — full report](../en/independent-search-audit.md)

Se probaron ocho fuentes originales conocidas: siete pudieron leerse, indexarse
y recuperarse. Reddit no permitió la lectura según robots.txt; se registró como
fallo y no se eludió la restricción. No se utilizaron proveedores de búsqueda,
metabuscadores, navegadores alojados ni llamadas a modelos.

La prueba partió de un índice vacío y URLs proporcionadas explícitamente, con el
rastreador de fondo detenido. Verifica ingestión y recuperación, **no cobertura
mundial ni descubrimiento desde una consulta sin fuentes conocidas**.

El lote tardó 10.60 segundos, con 16 solicitudes HTTP incluidas las políticas
robots. Las ocho consultas posteriores al índice hicieron **cero solicitudes HTTP**
y tardaron 1.23–6.59 ms cada una en este corpus pequeño. Es una medición puntual.

El [informe original](../benchmarks/independent-search-2026-09-19.json) conserva cada
fuente, duración y error. Las pruebas automatizadas comprueban filtros, fechas,
duplicados, RSS/Atom, migración y cancelación; el navegador Chromium real comprueba
interacciones y persistencia. La interfaz usa nuestra propia API y distingue fallos
y fechas de publicación frente a fechas de indexación.

Validación final: 32 pruebas del buscador, 6 de integración agente/API, 9 de skills,
3 del navegador local (incluido Chromium real), 4 de interfaz de búsqueda y 3 de
regresión de la aplicación. Compilaciones, formato y Clippy aprobados. Se ejecutaron
pruebas dirigidas; no se afirma haber repetido toda la suite del repositorio.

El rastreo de fondo conserva margen para lecturas explícitas. La instalación de
producción usa 2.500 solicitudes diarias y reserva aproximada del 20 %, sin reiniciar
el contador ni vaciar el índice. El valor predeterminado del repositorio sigue en 2.000.
