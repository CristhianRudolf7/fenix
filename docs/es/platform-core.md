# Núcleo portable: cierre, ejecución local y almacenamiento privado

## Alcance

Esta adaptación prepara tres componentes para Linux, Windows, macOS, Android e iOS. No anuncia una aplicación móvil completa ni sustituye pruebas en equipos reales. La selección de plataforma es determinista en Rust; no requiere llamadas a modelos.

## Cierre y ciclo de vida

`fenix-process::shutdown::Shutdown` recibe SIGINT/SIGTERM en Linux/macOS, eventos de consola en Windows y una solicitud explícita del host en Android/iOS. Un host nativo puede obtener `token()` y cancelarlo al detener el servicio o la aplicación. El servidor y las evaluaciones usan esa misma fuente para ejecutar el cierre del runtime y sus trabajadores. Esperar de nuevo después de una solicitud no deja el proceso bloqueado.

El sistema operativo puede terminar un proceso sin avisar; esta capa no evita un cierre forzoso. La integración de Kotlin/Swift y del controlador de servicios de Windows pertenece a los hosts/instaladores pendientes.

## Ejecución local y evaluaciones

Se eliminaron el cliente del servicio remoto, sus leases, relays, credenciales de ejecución y llamadas de control por socket. `fenix-execution` queda como fachada pequeña para archivos nativos y entorno permitido, además de rechazar configuraciones antiguas de sandbox: no contiene un protocolo de ejecución remota.

Comandos, servidores LSP y Chromium usan `fenix-process` directamente. Los controles de tareas, worktrees, aprobaciones, entrada/salida, cancelación y sesiones persistentes siguen en el agente. La extracción documental local deja de depender de la existencia de un servicio remoto; necesita sus dependencias locales y no está disponible como proceso externo en iOS.

La comunicación interactiva de benchmarks es independiente. `interaction_socket` conserva compatibilidad con rutas Unix; `interaction_endpoint` admite `tcp://127.0.0.1:PUERTO`, `tcp://[::1]:PUERTO` o `unix:///ruta/absoluta`. No se pueden configurar ambos campos. TCP acepta solamente direcciones IP de loopback, con plazo de conexión; es para un harness local de confianza, no una API pública. El protocolo de mensajes, respuestas y stop no cambia.

Los perfiles locales del navegador conservan cookies y datos persistidos. No se cierran formularios abiertos para simular un checkpoint completo de RAM. Los checkpoints del antiguo servicio remoto no se restauran como si fueran perfiles locales: requieren una decisión explícita de cierre/reapertura y no se repiten envíos automáticamente.

## Almacenamiento privado

El buscador usa `fenix-files` para configuración y base de datos. Abre SQLite con NOFOLLOW, conserva WAL y rechaza enlaces en los archivos privados. Las conexiones mantienen el formato cifrado AES-GCM y su clave existente; se adaptan lectura, creación y reemplazo atómico al motor privado. Los perfiles y el bloqueo del navegador también utilizan esa capa.

Unix aplica permisos de directorios/archivos; Windows utiliza la DACL del propietario. No se omiten errores de protección. Android/iOS utilizan archivos dentro del espacio autorizado por el sistema y por el host nativo. Esto no sustituye Android Keystore, Apple Keychain ni los permisos del selector de documentos, que siguen pendientes en las aplicaciones móviles.

## Validación

La compilación cruzada del núcleo de archivos, procesos y fachada de ejecución pasa para Windows x64, macOS ARM64, Android ARM64 e iOS ARM64; Linux se comprueba nativamente. Las pruebas de búsquedas y conexiones emplean servidores locales simulados: no son llamadas a modelos ni validación de proveedores reales. También se prueban privacidad, reapertura, cifrado, transporte local y cancelación.

En Linux pasaron 14 pruebas del gestor de procesos, siete de evaluación y diez del motor nativo, incluida una señal SIGTERM real. También pasó una prueba con Chromium real que comprueba navegación e inicio de sesión persistente. Las pruebas de almacenamiento privado verifican reapertura, cifrado y rechazo de enlaces.

La ejecución nativa fuera de Linux y las aplicaciones/servicios completos en los cinco sistemas todavía requieren validación. Estos resultados no equivalen a un despliegue ni a pruebas en dispositivos móviles.
