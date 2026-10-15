# Aplicación Android — preparación

Carpeta reservada para el host Android nativo. Todavía no contiene una APK ni un host ejecutable.

El host pasará explícitamente `context.filesDir/fenix` al núcleo Rust. Los directorios de caché se obtendrán con las APIs Android y no se utilizarán para estado durable. Se corregirá la dependencia actual de HOME/XDG del fallback Rust para instalaciones Android nativas.

La interfaz se comunicará directamente con Rust mediante un puente nativo compartido: comandos y resultados tipados, suscripción a eventos, cancelación y cierre. No dependerá de un servidor HTTP localhost para comunicarse con el núcleo. El diseño común está en `../mobile-bridge/README.md`.

Pendientes: proyecto Gradle, puente JNI o bindings generados, permisos de archivos, Keystore, ejecución permitida en segundo plano y pruebas de ciclo de vida. El navegador autónomo no forma parte de Android.
