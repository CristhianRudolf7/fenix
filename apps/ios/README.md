# Aplicación iOS — preparación

Carpeta reservada para el host iOS nativo. Todavía no contiene un proyecto Xcode ni una aplicación instalable.

El host resolverá su contenedor con las APIs Apple y pasará explícitamente `Library/Application Support/fenix` al núcleo Rust. No se fijarán rutas con UUID ni se dependerá del HOME de escritorio. La compatibilidad actual con `FENIX_APP_HOME` es una base, no la aplicación completa.

La interfaz se comunicará directamente con Rust mediante un puente nativo compartido: comandos/resultados tipados, eventos, cancelación y cierre. No utilizará HTTP localhost como transporte interno. Véase `../mobile-bridge/README.md`.

Pendientes: proyecto Xcode, bindings Swift, Keychain, selección autorizada de documentos, adaptación de tareas al ciclo de vida y pruebas reales. No se incorporan shells externos ni navegador autónomo. La autorización del usuario no concede privilegios root.
