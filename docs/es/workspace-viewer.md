# Visor del espacio de trabajo

El chat comparte un panel derecho con pestañas para archivos del proyecto, resultados de Design y fuentes de la biblioteca. **Archivos** abre un explorador temporal: permite elegir proyecto, recorrer carpetas, buscar nombres en la carpeta actual y abrir un archivo. Al seleccionarlo, el modal se cierra. Se recuerda la última carpeta por sesión de cuenta y proyecto; los favoritos son referencias guardadas en la sesión del navegador.

En escritorio se colapsa la navegación principal; en móvil el visor ocupa la pantalla bajo las pestañas y se puede cerrar para volver al chat. Las pestañas se mantienen al navegar dentro de la aplicación, pero no tras recargar toda la página. Abrir nuevamente un archivo actualiza su pestaña.

La herramienta `workspace_open` permite al agente abrir archivos, diseños o documentos y especificar una página o diapositiva. Las escrituras y ediciones nativas, parches, cambios de Design e importaciones de biblioteca generan eventos que actualizan el panel sin otra llamada al modelo. Después de cambios mediante comandos, el agente debe llamar explícitamente a `workspace_open`; no se infieren archivos mediante un observador del disco. Los resultados de tareas en segundo plano pueden activar su pestaña.

Cada cuenta conserva su proceso, base de datos y carpetas existentes. Las rutas de listado, vista previa y descarga validan el espacio de trabajo del usuario y del proyecto. Se rechazan rutas absolutas, recorridos al directorio padre, enlaces simbólicos, archivos con enlaces duros, datos internos y rutas protegidas de credenciales. No se listan archivos ocultos ni se almacenan las respuestas privadas en caché.

Markdown se muestra con formato seguro; HTML usa el visor aislado; PDF incluye páginas y zoom; imágenes y video usan visualización nativa. Word, Excel y PowerPoint muestran contenido extraído, sin reproducir fielmente la maquetación ni recalcular fórmulas. Las presentaciones nativas de Design conservan su presentación visual. La diapositiva solicitada de un PPTX se extrae directamente. Los archivos no compatibles se pueden descargar.

Límites del visor: archivos de hasta 64 MiB, texto en pantalla de hasta 2 MiB, páginas de 100 archivos y lotes de 100 bloques extraídos. La búsqueda por nombre se limita a la carpeta actual. La edición se solicita al agente; el explorador es de lectura.
