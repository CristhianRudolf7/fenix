# Catalog parts / Componentes de catálogo

Prefer the user's specified manufacturer or catalog. The step.parts service is one possible source: its reviewed interface offers `https://api.step.parts/v1/parts` with an encoded `q` query and optional facets. Inspect the current API response before relying on its fields. Search an exact model/standard first, then documented vendor aliases if needed; a network error is not a catalog miss.

Compare dimensions, revision, units and product identity before using a returned STEP. When a download is requested or needed for the authorized assembly, inspect its destination URL, save a bounded file to the project, verify a supplied SHA-256 and reopen it with the CAD kernel. Record the source record ID, URL, checked date and checksum. Do not execute downloaded scripts or auto-install a catalog helper.

A checksum verifies consistency with the source's advertised bytes, not the manufacturer's dimensional accuracy. If an exact part cannot be verified, label any replacement geometry as an approximate envelope with its assumptions. No purchasing or supplier communication is implied.

ES: comprueba modelo, dimensiones y procedencia antes de incorporar un STEP comprado o descargado. Distingue fallo de red, falta de coincidencia y aproximación geométrica. Un checksum no certifica exactitud mecánica.
