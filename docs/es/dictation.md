# Dictado y conversación escrita

El micrófono del chat transcribe voz al borrador mediante el reconocimiento del navegador. Puedes revisar el texto antes de enviarlo; dictar no envía mensajes automáticamente. La compatibilidad y los permisos dependen del navegador. No existe modo de llamada Live ni síntesis de voz en la aplicación.

El interlocutor del chat web y Telegram conserva su modelo rápido independiente del maestro. La configuración se consulta y guarda mediante `/api/model-roles/fast`; las tareas, planes y subagentes siguen funcionando independientemente del dictado. El motor conserva la configuración y el consumo históricos del modelo rápido al actualizar instalaciones anteriores.

Las pruebas con reconocimiento y proveedores simulados verifican el comportamiento del código; no certifican calidad acústica real.
