# Resúmenes de YouTube a partir de subtítulos

[English](../en/youtube-summary.md)

La skill propia `youtube-summary` permite pedir: «Resume este video de YouTube y destaca sus conclusiones: <enlace>». Un helper obtiene los subtítulos disponibles y el modelo de Fénix redacta el resumen en el idioma del usuario, con enlaces a los momentos relevantes.

## Implementación

Revisamos el README y el código de subtítulos de [pytube](https://github.com/pytube/pytube) como referencia. Elegimos [youtube-transcript-api](https://github.com/jdepoix/youtube-transcript-api) 1.2.4 como dependencia especializada. La skill, el helper, la validación de enlaces, los límites de red, la selección de idioma, el formato de evidencia y las pruebas son código de Fénix. No se copió la implementación de esos repositorios. Los commits y el alcance de revisión están en la [documentación técnica en inglés](../en/youtube-summary.md).

Admite enlaces normales, cortos, Shorts, directos y videos incrustados. Prefiere español y después inglés, con subtítulos manuales dentro del idioma elegido; declara cuándo usa automáticos u otro idioma. Conserva las marcas de tiempo y lee la pista completa, o indica qué parte pudo analizar. Las skills de transcripción de audio y edición de video mantienen sus funciones actuales.

No requiere GPU, descargar el video ni contratar un servicio de transcripción. Siguen existiendo los costos habituales del modelo y la infraestructura. Los subtítulos pueden tener errores y no describen necesariamente las imágenes. Si YouTube bloquea el acceso, exige iniciar sesión o no ofrece subtítulos, Fénix lo comunica y puede trabajar con una transcripción visible autorizada o un archivo SRT/VTT/texto aportado. No inventa el contenido desde el título ni importa cookies o proxies para evitar restricciones.

## Activación y comprobaciones

La skill se integra en el instalador existente y ejecuta el helper dentro del entorno aislado del usuario. `Dockerfile.workspace` incluye sus dependencias fijadas; `packaging/linux/Dockerfile.youtube` permite añadirlas a una imagen existente. El [procedimiento de construcción](../en/youtube-summary.md#packaging-and-operation) incluye cómo obtener la identidad inmutable de la imagen. Hace falta el despliegue habitual de aplicación e imagen para activarlo en una instancia en ejecución; no se cambió el servicio de producción.

Las pruebas cubren extracción, idiomas, enlaces, límites, errores, descubrimiento de la skill y recursos. La extracción real funcionó con dos videos públicos: seis segmentos en inglés y 316 en español. Esto no garantiza acceso desde cualquier servidor ni evalúa la calidad completa del resumen del modelo. Consulta el [registro de validación](../research/youtube-summary-validation-2026-09-21.json); no incluye los subtítulos descargados.

Validación: **16 pruebas Python**, repetidas correctamente dentro del contenedor restringido, y **33 pruebas nativas de skills** superadas. El descubrimiento incluye ocho solicitudes de YouTube y cuatro tareas vecinas.
