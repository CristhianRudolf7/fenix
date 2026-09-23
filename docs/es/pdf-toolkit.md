# Herramientas PDF y visor

[English](../en/pdf-toolkit.md) · [Revisión de repositorios](../research/pdf-repositories-2026-09-20.md)

Fénix incorpora una versión propia: skill `pdf-workflows`, motor Python y visor React. El motor permite inspeccionar, unir, seleccionar, reordenar, duplicar, eliminar y separar páginas; girar, recortar, añadir marcas de agua y números; cambiar metadatos, comprimir sin rasterizar, rellenar campos de texto AcroForm, convertir entre imágenes y PDF, reconstruir en escala de grises y redactar regiones sensibles.

Se ejecuta mediante `command_run` dentro del entorno del usuario. Conserva originales, comprueba hashes y rechaza sobrescrituras, enlaces y rutas fuera del espacio de trabajo. No necesita GPU, API de pago ni subida a un tercero. La skill elige el procedimiento y verifica el resultado; el aislamiento sigue correspondiendo al servicio de ejecución.

El panel derecho usa ahora un visor integrado con una barra compacta que solo permite navegar entre páginas y ajustar el zoom. La rotación, búsqueda, extracción de texto y edición se solicitan al agente en el chat. Acepta contraseña local para visualizar PDFs protegidos. Scripts y enlaces del PDF no se ejecutan; los recursos del visor se sirven desde Fénix. Los documentos escaneados siguen usando el OCR de CPU existente: el visor no ejecuta OCR ni muestra herramientas de edición.

La redacción crea un PDF nuevo de imágenes, eliminando los objetos originales. Pierde texto seleccionable, formularios, firmas y vectores. Se verifica que no conserve texto extraíble ni adjuntos, pero hay que revisar visualmente que las regiones cubran todos los datos sensibles. Recortar solo oculta contenido y no sirve para borrar secretos. Los originales y versiones anteriores permanecen.

No se promete manejo perfecto: no se implementaron edición arbitraria del texto existente, XFA, firmas digitales, cifrado, conversión Office/HTML, certificación PDF/A, reconstrucción de tablas ni capas OCR buscables. Las pruebas sintéticas verifican operaciones concretas, no todos los PDFs existentes.

La imagen ligera `packaging/linux/Dockerfile.pdf` instala dependencias sin OCR; `Dockerfile.ocr` reúne OCR y edición. La [documentación inglesa](../en/pdf-toolkit.md) contiene instalación, protocolo y límites completos. Para verlo en una instancia ya encendida hay que desplegar el servidor/agente y seleccionar la imagen actualizada. Compilar no cambia automáticamente los contenedores de los usuarios.
