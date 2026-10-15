# Ejecución personal — 0.3.2

Estado: implementación en desarrollo; no confundir con la release pública 0.3.1.

## Qué cambia

Cada instalación pertenece a una cuenta del sistema. Los comandos, servidores LSP, navegador y extractor documental se ejecutan como procesos locales. Se retiran el servicio de ejecución aislada, sus scripts de aprovisionamiento y el helper Landlock/seccomp de comandos de solo lectura. Los permisos reales son los de la cuenta del sistema: la carpeta de trabajo no es una barrera de seguridad.

El transporte remoto antiguo se ha retirado. `fenix-execution` conserva únicamente la fachada de archivos locales y la comprobación de configuración heredada. Las variables `FENIX_CONTAINER=1`, `FENIX_EXECUTION_REQUIRED=1` o `FENIX_EXECUTION_SOCKET` provocan un error explícito. Retíralas y crea una instancia personal con su propio directorio de datos. No reutilices una instancia compartida como ejecución local sin revisar propietarios, credenciales y procesos. No se migran ni borran datos de usuarios automáticamente.

## Permisos por conversación

Los comandos ordinarios pueden ejecutarse sin una pregunta adicional en instalaciones nuevas. Se mantienen las preferencias existentes y la política opcional de preguntar por todos los comandos. Un detector local busca señales de borrado, modificación de discos o permisos, privilegios administrativos, código descargado, instrucciones codificadas y acceso a credenciales. Los comandos detectados solicitan permiso incluso con política de permitir. Se comprueban tanto ejecución de comandos como texto enviado a procesos y las acciones de workflows.

La pregunta explica el riesgo y la acción; se responde **sí** o **no**, sin escribir identificadores. Con varias preguntas pendientes, el usuario responde al mensaje correspondiente. El permiso se consume una sola vez y solo autoriza los argumentos mostrados. Una negativa queda registrada e impide repetir esa acción. Las tareas no interactivas se bloquean si requieren este permiso; no inventan una aprobación.

La detección usa reglas, sin llamadas al modelo. No analiza exhaustivamente scripts, paquetes o todas las variantes del shell, por lo que ausencia de señal no demuestra seguridad. Las aprobaciones no aíslan procesos.

## Documentos y navegador

El navegador utiliza el backend local ya existente, conserva su perfil y supervisión de procesos. Las funciones del antiguo broker de contenedores no están disponibles. La extracción PDF/PNG/JPEG ejecuta el worker incluido en un proceso Python local supervisado. `FENIX_OCR_PYTHON` puede señalar un intérprete o entorno virtual preparado por el propietario; por defecto utiliza `python3`. Las dependencias se declaran en `scripts/ocr/requirements.lock.txt` y `resources/skills/pdf-workflows/scripts/requirements.txt`; los modelos verificados se preparan con `scripts/ocr/fetch_models.py` en `/opt/fenix-ocr/models`. Instala por separado las dependencias del extractor y modelos OCR cuando se necesiten; no se descargan automáticamente al procesar documentos.

Se conservan los hashes, validación de archivos, checkpoints locales del navegador, redacción de secretos, historial, worktrees y las protecciones de vistas de documentos/diseños. Estas protecciones no son contenedores de ejecución ni aislamiento entre usuarios.

## Distribución y comprobaciones

El instalador público conserva la release 0.3.1 hasta compilar, validar y publicar los paquetes 0.3.2. No se ha convertido un despliegue compartido existente ni se ha reiniciado su servicio como parte de este cambio. Las pruebas locales de esta versión se registran separadamente de cualquier validación de producción.

Validación local del 11 de octubre de 2026: compilan backend, frontend y documentación. Pasan las comprobaciones del detector, aprobación afirmativa/negativa, rechazo de configuración aislada antigua, valores iniciales, extracción local y lectura de archivos en paralelo. Los diez tests del worker documental pasan; los casos OCR usan dobles de prueba. La suite general detectó además dos fallos de clasificación/búsqueda de skills, pendientes de revisión y sin cambios en ese subsistema durante esta migración. No se realizaron llamadas a proveedores ni validación de producción.

El motor de procesos se comparte ahora entre Linux, macOS y Windows; la compatibilidad completa del resto de la aplicación no se da por concluida. Véase [motor nativo](native-process-engine.md).
