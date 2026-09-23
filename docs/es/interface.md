# Interfaz e identidad visual

Fénix utiliza por defecto un fondo marfil (`#F7F6F2`), superficies blancas, texto carbón, acciones en terracota y acentos verde petróleo. El modo oscuro conserva esa identidad con superficies oscuras y acentos más claros. El tema y el idioma se recuerdan tanto en el login como dentro de la aplicación.

El login combina una ilustración local con el formulario de correo y contraseña. En móvil, la ilustración se reduce a una cabecera compacta. Se puede mostrar u ocultar la contraseña. El botón para cerrar sesión permanece en la cabecera al navegar; la autenticación existente no cambia y no se añade un registro de cuentas.

El chat mantiene una sola conversación, distingue visualmente tus mensajes, muestra avances reales y conserva los controles de dictado y Live. Las sugerencias completan el campo de texto para que puedas revisarlas antes de enviar. El menú se puede contraer en escritorio y recuerda esa elección; al abrir una vista previa se oculta como antes. En pantallas pequeñas, la vista previa ocupa la pantalla. Memoria, proyectos, skills, tareas, configuración y diseños comparten colores y controles.

Se respetan la preferencia de reducir movimiento, el foco visible y los nombres accesibles de los controles, incluso con el menú contraído. No se añaden porcentajes de avance ficticios ni actividad simulada. La disponibilidad del micrófono y Live sigue dependiendo del navegador.

La identidad está en `apps/web/src/styles/identity.css`; los iconos vectoriales compartidos están en `components/icons.tsx`. La ilustración `apps/web/src/assets/phoenix-wing.webp`, de aproximadamente 109 KiB, se generó con el generador integrado y se convirtió a WebP sin cambiar su composición. Se sirve desde el gateway Rust, sin proveedores externos de imágenes o tipografías. El prompt completo está en la [documentación en inglés](../en/interface.md).

Las pruebas de navegador admiten puertos alternativos mediante `FENIX_TEST_APP_PORT`, `FENIX_TEST_GATEWAY_PORT`, `FENIX_TEST_RUNTIME_PORT` y `FENIX_TEST_SEARCH_PORT`. Si hay otra ejecución en el mismo repositorio, también se debe separar la carpeta de resultados con `--output`. `FENIX_TEST_EXPECT_TIMEOUT` ajusta la espera de las pruebas en máquinas lentas sin cambiar el comportamiento de producción.

Para compilar mientras otra tarea modifica la interfaz, se puede construir una copia estable del frontend y establecer `FENIX_WEB_DIST` con la ruta absoluta de su carpeta `dist` al compilar Rust. Por defecto se mantiene `apps/web/dist`.

`FENIX_TEST_BINARY` permite ejecutar las pruebas contra un binario guardado, sin que otra compilación lo reemplace durante la validación.


## Rutas y estados de carga

React Router gestiona `/chat`, `/projects`, `/memory`, `/skills`, `/design`, `/browser`, `/schedules`, `/activity` y `/settings`. Los enlaces directos, las recargas y el historial conservan la sección seleccionada; el login mantiene la URL solicitada. Las rutas de aplicación no interceptan APIs ni archivos inexistentes.

La pantalla Navegador en `/browser` permite iniciar sesión, conservar perfiles, habilitar al agente, elegir pestañas, ajustar tamaños y usar la bóveda cifrada. Las entradas del propietario usan las API del navegador sin pasar por la conversación. Los estados de carga muestran una llama, una barra indeterminada y líneas de contenido, con texto accesible y soporte para movimiento reducido.

## Visualización de Markdown

El historial del chat, las respuestas progresivas, las instrucciones de skills, las explicaciones de revisión, las memorias, los contextos de proyectos y las vistas de documentos simples comparten un renderizador Markdown. Admite títulos, énfasis, listas, listas de tareas, citas, enlaces, tablas y bloques de código. El contenido guardado y los editores conservan el Markdown original; los informes JSON y las diferencias de código permanecen literales.

Marked interpreta el texto y DOMPurify sanea el resultado. El HTML se muestra como texto, se eliminan protocolos inseguros de enlaces y las imágenes Markdown no generan solicitudes externas. El código y las tablas se desplazan dentro de sus contenedores en pantallas pequeñas. No se realizan llamadas adicionales al modelo.
