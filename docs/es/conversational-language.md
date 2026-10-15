# Conversación natural

Fénix se presenta como un solo asistente en web, Telegram y voz. Al recibir trabajo responde «Perfecto, me encargo»; solo afirma avances o resultados cuando existe evidencia. Una negativa, una aclaración y un cambio de instrucciones reciben confirmaciones distintas.

La política compartida `resources/conversation-style.md` guía las respuestas del modelo conversacional y del ejecutor. Las preguntas identifican acciones o documentos por su nombre. Los identificadores se conservan para el funcionamiento interno y los detalles técnicos opcionales, sin exigir al usuario que los copie. Las aprobaciones conservan los detalles relevantes de la acción, sus comandos y destinos.

El modelo recibe un resumen acotado de las habilidades instaladas y activas, sin cargar todas sus instrucciones ni deducir las capacidades únicamente de la última conversación. La presencia de una habilidad no garantiza acceso a herramientas o cuentas conectadas.

Los avisos de planes, automatizaciones, cancelaciones y errores usan descripciones legibles; los diagnósticos completos se conservan internamente. El historial existente no se reescribe. Las respuestas generadas siguen estas instrucciones, sin una garantía absoluta ni un filtro que borre contenido técnico solicitado. Las confirmaciones automáticas se emiten después de aceptar correctamente la acción.

Las pruebas locales comprueban respuestas sin crear tareas duplicadas, negativas, resúmenes de habilidades y avisos de planes sin identificadores visibles. Utilizan un modelo simulado y no envían mensajes a usuarios de producción.

## Avances persistentes y cancelación

Con el interlocutor configurado, los borradores y comentarios de herramientas del maestro quedan en el registro interno. Un proceso independiente agrupa los avances por tarea y pide al interlocutor mensajes breves y sencillos. Se guardan en la conversación, llegan al canal correspondiente y permanecen al recargar. La narración tiene una frecuencia limitada por tarea; sus errores no detienen la ejecución. Los resultados finales y las preguntas conservan su mecanismo de entrega y las instrucciones compartidas de lenguaje natural.

Las copias visibles ocultan UUID; los argumentos y evidencias originales se conservan. La interfaz deja de pedir identificadores y reconoce la reconexión del progreso.

Cancelar ejecuta una acción real: primero se confirma que se solicitó detener el trabajo y se cierran sus preguntas pendientes; el motor confirma después que se detuvo. Una solicitud breve e inequívoca de cancelación no se interpreta como una respuesta a una aclaración. Si hay varios trabajos y no está claro cuál cancelar, se pide aclaración. Esta implementación no modifica el historial de producción ni supone desplegar los cambios.

## Enrutamiento estructurado

El interlocutor devuelve su respuesta y decisión mediante una sola llamada interna `conversation_decision`. Crear una programación se delega al ejecutor como trabajo nuevo. El motor rechaza llamadas múltiples, desconocidas o incompletas y valida permisos y preguntas pendientes antes de ejecutar. Se mantiene compatibilidad con proveedores que devuelven el mismo objeto como texto JSON. Una respuesta inválida puede reintentarse antes de despachar trabajo, evitando duplicarlo.
