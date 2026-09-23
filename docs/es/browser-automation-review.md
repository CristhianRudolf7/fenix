# Revisión de automatización del navegador

[English and pinned sources](../en/browser-automation-review.md)

Revisión del 19–20 de septiembre de 2026. Se revisaron código y documentación de
cuatro proyectos populares; no se importó su código ni se añadió un proveedor remoto.

Las tareas largas necesitan sesiones persistentes, pestañas identificadas,
progreso observable, esperas limitadas y pruebas de finalización. Fénix ya dispone
de registro de tareas, compactación del contexto y perfiles persistentes. Se
corrigieron los puntos concretos que faltaban:

- Vista `/browser` restaurada: inicio de sesión manual, bóveda cifrada, perfiles,
  pestañas, redimensionado y autorización del agente. Escritura Unicode, pegado y
  composición; las acciones y la bóveda usan la pestaña mostrada.
- Registro limitado a 128 descargas independiente de los últimos 100 eventos.
  Las esperas devuelven bytes y estado pendiente/completado/cancelado. El archivo
  completado permite verificar el resultado incluso después de reiniciar Chromium.
- La actividad se cuenta al finalizar la operación; una descarga que aumenta sus
  bytes evita el cierre por inactividad. Los eventos repetidos sin avance no lo
  evitan indefinidamente. Se mantiene el límite de memoria.
- El agente distingue avance de bytes de esperas estancadas. Los resultados
  abreviados conservan enlaces y progreso limitados; el registro completo sigue
  disponible mediante `task_inspect`.
- El skill explica cómo esperar al propietario, retomar después de una pausa y
  comprobar el archivo antes de entregarlo, sin repetir clics o envíos realizados.

No garantiza resolver CAPTCHA/MFA, navegar cualquier web ni emular otros motores.
Los perfiles personales siguen siendo estado compartido: tareas simultáneas deben
usar perfiles distintos o coordinar pestañas. El servicio serializa las operaciones.

Las mediciones y límites se documentan en la [auditoría](browser-workflow-audit.md).
