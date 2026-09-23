# Voz del navegador y Live

La interfaz usa Vite, React, TypeScript, Tailwind y pnpm. Axum incluye `apps/web/dist` dentro del ejecutable: Node no se ejecuta en producción. Antes de Cargo, ejecuta `pnpm --dir apps/web install --frozen-lockfile` y `pnpm --dir apps/web build`. Para desarrollo, `pnpm --dir apps/web dev` conecta con el backend local del puerto 4173.

React controla la estructura de la aplicación, el chat y Live. Los controladores de administración y navegador se trasladaron a TypeScript y se conservaron en límites DOM aislados; no se reescribieron todos sus formularios como componentes declarativos. El tipado admite parámetros implícitos de esos controladores durante la migración gradual.

## Funcionamiento

El micrófono dicta al borrador: puedes revisarlo y enviarlo al maestro. Live usa un modelo de conversación independiente, configurado en el engranaje junto a su botón. Se propone Nemotron con un endpoint compatible con RunInfra y una clave independiente, que se verifica antes de guardar. No cambia el modelo del maestro.

El interlocutor ve conversación reciente, proyectos, tareas, subagentes y progreso observable. No recibe herramientas de ejecución ni razonamiento interno. Ante una instrucción expresa puede enviar tus palabras exactas al maestro o dirigir un cambio a una tarea. Debe preguntar si el destino es ambiguo. Esa clasificación depende del modelo; permanecen las aprobaciones habituales del maestro. Las observaciones automáticas no pueden delegar ni modificar tareas, aunque el modelo lo solicite.

Los eventos se agrupan. El navegador consulta cada 20 segundos cuando hay actividad y el servidor evita narraciones repetidas; no hay llamada al modelo por cada herramienta. Puede guardar silencio si no hay novedades útiles. La conversación escrita y Live comparten historial y las conversaciones del usuario quedan disponibles para el contexto posterior. Narrar un estado no lo convierte en evidencia de que se completó una acción.

## Voz y cierre

STT y TTS son del navegador. Su disponibilidad depende del navegador/sistema; el reconocimiento puede enviar audio al proveedor del navegador y no garantiza funcionamiento sin internet. Si no existe soporte, el chat escrito continúa disponible. Los errores y permisos denegados se muestran al usuario.

El micrófono solicita cancelación de eco. Se pausa el reconocimiento durante la voz sintetizada; un detector de energía intenta interrumpir el audio cuando hablas, y existe un control explícito para interrumpirlo. Es una solución inicial: eco del altavoz, primeras sílabas, suspensión y dispositivos reales necesitan comprobación manual. No garantiza telefonía dúplex ni latencia uniforme.

Colgar libera el micrófono, detiene reconocimiento/síntesis y cierra únicamente Live. El maestro, comandos, planes y subagentes continúan. Cambiar de pantalla dentro de la web mantiene la llamada; cerrar la página termina la voz pero no las tareas.

Las pruebas automatizadas usan modelos y voz simulados. Comprueban dictado sin envío, historial, delegación exacta e idempotente, navegación y aislamiento del cierre. No certifican calidad acústica ni latencia real de Nemotron. [Documentación principal en inglés](../en/live.md).

## Interlocutor compartido entre canales

El chat web, Telegram y Live utilizan la misma conexión de conversación (Nemotron por defecto), configurada en los ajustes de conversación/Live. La conexión del maestro es independiente: utiliza Astra low con Flex para ejecutar tareas. Si faltan las credenciales del interlocutor, no se cambia silenciosamente al maestro.

El interlocutor responde conversaciones y consultas de estado observado, y transmite las instrucciones exactas del usuario al maestro. Las aprobaciones y los controles textuales mantienen sus manejadores deterministas. Las respuestas de Telegram se entregan a la cuenta de origen; las tareas conservan el canal y las referencias explícitas. Cerrar Live no cancela el chat ni las tareas en segundo plano. Los resultados del maestro permanecen en el historial compartido.

Los ajustes de ejecución de OpenAI admiten `flex_processing: true`, que envía explícitamente `service_tier: "flex"` en las solicitudes Responses, incluidos los subagentes. No se utiliza precio estándar como alternativa automática.
