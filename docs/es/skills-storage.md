# Skills y almacenamiento privado multiplataforma

La gestión manual de skills utiliza `fenix-skills` y las operaciones privadas de `fenix-files`. Los módulos son independientes del servidor y del modelo: pueden probarse sin credenciales y sin consumir tokens. La integración mantiene el catálogo bajo demanda, estado activo/desactivado, personalizaciones, revisiones, recursos y traducciones guardadas.

## Gestión de skills

- Instalación por una carpeta temporal oculta; el catálogo descubre la skill cuando se publica el directorio completo. Nunca se sobrescribe una instalación existente.
- Exportación acotada de Markdown, recursos y marcas de ejecutable, sin ejecutar scripts. Se rechazan enlaces, archivos no regulares, rutas ambiguas y nombres incompatibles con Windows.
- Lectura y edición con comprobación de la revisión; los cambios obsoletos no se guardan. La edición bilingüe conserva el control previo de metadatos y los documentos traducidos por hash de la versión inglesa.
- Eliminación mediante retirada del directorio y registro durable de la eliminación, para que las skills predeterminadas no reaparezcan durante la siguiente actualización.
- Archivo `.skills.lock` para coordinar instalaciones, mantenimiento, traducciones y ediciones. No se bloquea el directorio mediante una operación exclusiva de Unix.
- En Windows, `.fenix-executables.json` conserva las marcas de ejecutable del paquete; no se interpretan como permisos POSIX ni se ejecuta código por importarlo. En Unix se conservan los bits de ejecución.
- Los accesos se realizan mediante directorios abiertos y rutas relativas comprobadas, sin `/proc/self/fd`. Los cambios directos externos al agente no se convierten en transacciones: la comprobación de contenido y la sustitución siguen siendo controles optimistas.

## Datos y permisos

Las instalaciones nuevas sin `--data-dir` usan:

| Sistema | Ubicación predeterminada |
| --- | --- |
| Linux | `$XDG_DATA_HOME/fenix`, o `~/.local/share/fenix` |
| macOS | `~/Library/Application Support/fenix` |
| Windows | `%LOCALAPPDATA%\fenix`, o la carpeta equivalente bajo el perfil |

Una carpeta `.fenix` existente en el directorio desde el que se inicia Fénix conserva prioridad para compatibilidad. No se trasladan bases de datos automáticamente. `--data-dir` y `--skills-dir` conservan prioridad y los instaladores que ya pasan rutas explícitas siguen usándolas. La biblioteca general de skills conserva `~/.fenix/skills`, con la carpeta personal resuelta por el sistema.

Los directorios privados usan permisos `0700` en Unix y los archivos privados `0600`. En Windows se aplica una DACL protegida que concede acceso al SID del usuario que ejecuta Fénix, utilizando las APIs nativas mediante bindings seguros; un error al establecerla no se ignora. Esto exige un sistema de archivos con soporte de ACL. No elimina las capacidades administrativas propias del sistema operativo.

La base SQLite se prepara antes de abrirla, se rechazan enlaces y se conserva WAL, las migraciones, conversación, memoria, tareas, documentos y preferencias. Un archivo `instance.lock` impide abrir dos instancias del runtime sobre el mismo directorio. Los bloqueos se liberan al cerrar el handle, incluso si el proceso termina. La configuración, traducciones y el historial de archivos utilizan guardados compatibles con cada plataforma; los modos Unix del historial no se aplican como ACL al restaurar en Windows.

## Verificación y límites

Las pruebas locales ejecutan operaciones reales en Linux: instalación/exportación, recursos, idiomas, ejecución como metadato, edición, eliminación, concurrencia, enlaces, permisos, bloqueo y recuperación de persistencia. Se comprueba además la compilación de los módulos portable de archivos y skills, con sus pruebas, para Windows y macOS Intel/ARM.

La matriz CI incluye ejecución nativa de archivos, skills, almacenamiento SQLite y procesos en los tres sistemas. Configurar esa matriz no equivale a haber ejecutado sus jobs. La ejecución nativa en Windows/macOS y las instalaciones completas permanecen pendientes hasta contar con sus resultados. El transporte remoto heredado se ha retirado. El almacenamiento privado del buscador, las conexiones y los perfiles del navegador también utiliza el motor portable. Los instaladores, las dependencias externas y la ejecución nativa completa en cada plataforma siguen pendientes; esta adaptación no anuncia soporte completo de toda la aplicación.

### Resultado de la comprobación local

Se aprobaron 53 pruebas dirigidas en Linux: 15 de archivos/skills portables, 20 de persistencia SQLite y 18 de integración del agente. Cubren también ediciones bilingües, conservación de personalizaciones y estado desactivado, historial de restauración, enlaces duros personales y aliases de las carpetas configuradas. Los aliases configurados se resuelven antes de proteger el almacenamiento; no habilitan enlaces dentro de paquetes de skills.

Clippy de los módulos nuevos y la compilación del servidor Linux terminaron correctamente. Los módulos de archivos y skills, con sus pruebas incluidas, compilan para Windows x64 y macOS Intel/ARM. La comprobación local no ejecutó SQLite ni el agente completo de forma nativa en Windows/macOS. No hubo despliegue ni llamadas reales a proveedores; las pruebas de traducción utilizaron respuestas simuladas locales.
