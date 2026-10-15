# Disponibilidad de modelos y bloqueo de envío

El servidor personal inicia un trabajador de salud separado de los workers de evaluación. No se inicia automáticamente al abrir Runtime ni al usar `Runtime::start` en una evaluación: el host servidor llama a `start_model_health` explícitamente.

## Estado por conexión

Se conserva un estado para los roles inteligente y rápido. Una huella incluye proveedor, endpoint, modelo, credencial, proyecto/organización y opciones de razonamiento/procesamiento. El hash pertenece a la conexión concreta: una respuesta de una conexión anterior no valida una clave nueva. Las huellas no se entregan a la interfaz.

Estados: sin configurar, sin comprobar, comprobando, disponible o no disponible. Para permitir enviar desde la web deben estar disponibles ambos roles, porque el interlocutor y el maestro participan en el flujo.

Las configuraciones guardadas correctamente ya realizaron una solicitud real y registran esa validación. Al reiniciar se reutiliza una validación vigente; por defecto dura seis horas. Al caducar, el envío permanece bloqueado hasta comprobar nuevamente. El trabajador revisa vencimientos cada treinta segundos; no llama a proveedores por cada consulta HTTP de estado.

## Reintentos y consumo

Una comprobación envía una petición breve, con un máximo de salida de 1.024 tokens y plazo de treinta segundos por rol, ampliado a tres minutos cuando se configura Flex. Esto puede consumir tokens y no garantiza comprobar todas las capacidades del modelo, como herramientas o streaming.

La ejecución es single-flight. Los intentos manuales respetan treinta segundos de separación; errores temporales usan espera creciente y los de autenticación/configuración/cuota esperan quince minutos para la comprobación automática. Una configuración nueva puede verificarse inmediatamente mediante su flujo de configuración.

Las respuestas reales del maestro y del interlocutor actualizan la disponibilidad en los puntos de inferencia principales. Los errores se clasifican sin exponer cuerpos del proveedor ni secretos. Los sondeos exitosos registran uso y se incluyen en las métricas de consumo. Una validación previa no garantiza que la siguiente solicitud tenga éxito si el servicio cambia de estado.

## Interfaz y API

`GET /api/model-health` consulta el estado sin inferencia. `POST /api/model-health` solicita una comprobación con cooldown. `/api/status` incluye el estado para aprovechar el sondeo existente de la interfaz.

El chat muestra un aviso persistente y desactiva enviar mientras no pueda utilizarse el modelo. Conserva el borrador y permite comprobar la conexión o revisar la configuración. El servidor también rechaza nuevos mensajes web cuando falta disponibilidad, de modo que pulsar Enter o invocar el endpoint no evita el control. No se cancelan automáticamente trabajos previos por mostrar un aviso.

## Alcance de validación

Las pruebas utilizan proveedores HTTP locales simulados y fixtures de interfaz: no certifican disponibilidad, saldo ni compatibilidad de proveedores reales. La instalación móvil todavía necesita implementar el host y conectar este estado mediante el puente nativo.

Comprobación local de esta implementación: pasan catorce pruebas HTTP/backend (diez existentes y cuatro de disponibilidad) y dos pruebas de navegador sobre chat y recuperación de conexión. También pasa la compilación TypeScript/Vite. Los escenarios de proveedor usan HTTP local simulado; no se hicieron llamadas a proveedores reales ni se desplegaron los cambios.
