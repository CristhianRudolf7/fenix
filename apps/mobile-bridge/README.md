# Puente móvil directo con Rust — diseño pendiente

Android e iOS alojarán el núcleo Rust dentro de su proceso y compartirán un contrato de bindings. Esta carpeta documenta la dirección de arquitectura; todavía no implementa la ABI ni genera bindings.

- El host entrega las rutas privadas del sistema y abre una sola instancia de Runtime por directorio de datos.
- Las llamadas tipadas configuran modelos, envían mensajes, consultan datos y controlan tareas. Los errores no cruzan como punteros o excepciones Rust.
- El núcleo publica eventos mediante suscripciones cancelables. El host actualiza la interfaz en el hilo de UI correspondiente.
- Los identificadores internos sirven para correlacionar eventos; no se muestran como instrucciones técnicas al usuario.
- El host coordina pausa, persistencia y cierre. La suspensión del sistema no equivale a poder ejecutar indefinidamente en segundo plano.
- La selección entre bindings generados y JNI/FFI se resolverá al implementar; no se mantiene un servidor HTTP local exclusivamente para comunicar la app con Rust.

La estructura inicial no certifica soporte móvil completo ni instala una aplicación.
