# Motor nativo de archivos

Fénix comenzó con acceso a archivos exclusivo de Linux. El módulo `fenix-files` separa ahora las operaciones de lectura, creación, edición, borrado y listado del transporte de ejecución. `fenix-execution::fs` conserva los nombres anteriores como fachada de compatibilidad: las herramientas, documentos, parches y comprobaciones de contenido siguen utilizando la misma interfaz.

## Comportamiento

- El workspace es el destino predeterminado, no una frontera de acceso. Las rutas externas se resuelven con los permisos del usuario del sistema.
- Se admiten nombres Unicode, rutas absolutas y rutas relativas con `..`. Los enlaces simbólicos y junctions existentes se resuelven antes del acceso personal. En Windows, una ruta de unidad debe ser absoluta (`C:\carpeta\archivo`); se rechaza `C:archivo` para evitar depender del directorio oculto de esa unidad. Las rutas UNC y las rutas extendidas se representan mediante los tipos nativos de Rust.
- Los archivos personales pueden tener enlaces duros. Las operaciones privadas del motor siguen rechazándolos. La sustitución atómica modifica la ruta editada; no reescribe las demás rutas que apuntaban al mismo inode.
- La creación no sobrescribe archivos existentes. La edición y el borrado exigen que el contenido coincida con el esperado. El motor no interpreta ni cambia los saltos de línea de los bytes recibidos.
- La edición escribe y sincroniza un temporal en la misma carpeta antes de sustituir el destino. Ante un error, no elimina el original para forzar la operación. En Windows, un archivo de solo lectura o un bloqueo incompatible devuelve un error recuperable.
- El listado del chat y del explorador utiliza un directorio abierto mediante un handle, sin construir rutas `/proc/self/fd`.
- Unix conserva acceso relativo a descriptores, rechazo de enlaces durante la apertura y modos de permisos. Los archivos del workspace siguen el umask; los privados usan modos restrictivos. Windows utiliza handles, comprobaciones de enlaces y permisos del sistema: los archivos nuevos heredan las ACL del directorio. Los modos POSIX no equivalen a ACL; las operaciones personales no conservan explícitamente entradas ACL personalizadas al sustituir archivos. El almacenamiento privado ahora aplica una DACL protegida mediante el submódulo `private`; véase [skills y almacenamiento](skills-storage.md).

Las comprobaciones de contenido evitan ediciones con una versión obsoleta, pero no constituyen una transacción con programas externos que escriben simultáneamente. Los límites de texto, la redacción de secretos, las aprobaciones de riesgo y el alcance del visor web se conservan.

## Validación y alcance

Se prueban operaciones reales en Linux: rutas externas, Unicode, saltos de línea, creación sin sobrescritura, edición y borrado con contenido esperado, enlaces duros, enlaces simbólicos, modos y listados. La matriz de CI ejecuta el motor en Linux, Windows y macOS; incluye casos propios de Windows para archivos de solo lectura y bloqueos de uso compartido.

La comprobación cruzada de compilación con las pruebas incluidas no sustituye la ejecución nativa en Windows/macOS. Tampoco convierte toda la aplicación en multiplataforma: siguen pendientes las validaciones nativas completas de instalación, servicios y dependencias externas. El transporte remoto heredado se ha retirado y el almacenamiento privado del buscador, las conexiones y los perfiles del navegador utiliza ahora el motor portable. La gestión de skills y el almacenamiento privado ya están adaptados en [su módulo portable](skills-storage.md). No se declara validado un instalador completo para Windows o macOS mediante estas pruebas.

### Resultado de esta comprobación local

- Nueve pruebas del motor y nueve pruebas de integración del agente aprobadas en Linux.
- Compilación cruzada del motor con pruebas incluidas aprobada para Windows x64, macOS Intel y macOS ARM.
- Clippy del motor con advertencias tratadas como errores aprobado; servidor Linux compilado correctamente mediante `cargo check`. El servidor conserva tres advertencias anteriores ajenas a esta adaptación.
- Sin ejecución nativa en Windows/macOS, sin despliegue y sin publicación de una release en esta comprobación.
