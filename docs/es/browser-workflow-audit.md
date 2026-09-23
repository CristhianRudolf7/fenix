# Auditoría de navegación autenticada

[English](../en/browser-workflow-audit.md)

Verificado el 2026-09-20 con Chromium 153.0.8010.47. El recorrido final pasó con
**46 llamadas a herramientas, 2.457.600 bytes descargados y una sola solicitud de
descarga en 125,24 segundos**, con compactación del contexto durante la tarea.
El tiempo indicado excluye la compilación. Las decisiones del modelo son programadas.
La [revisión de implementación](browser-automation-review.md) explica los cambios.

La prueba `agent_authenticated_multistep_workflow_survives_wait_timeouts` usa el
motor real del agente, sus herramientas, Chromium local, perfiles persistentes y
archivos descargados. Las decisiones del modelo están programadas para la prueba;
la universidad, las credenciales y el informe son ficticios. Comprueba la ejecución,
no la capacidad de un modelo real para planificar una web desconocida.

## Cobertura

- Rechazo de contraseñas introducidas por el agente y acceso separado del propietario.
- Inicio de sesión persistente tras cerrar y volver a abrir Chromium.
- Búsqueda asíncrona con caracteres Unicode y lectura de ocho páginas enlazadas.
- Compactación del contexto durante el recorrido, conservando la sesión del
  navegador y continuando sin saltarse ni repetir acciones.
- Descarga autenticada de 99 segundos: una espera corta y tres de 30 segundos
  devuelven estado pendiente con bytes crecientes; otra confirma la finalización
  sin volver a iniciar la descarga.
- Comparación de todos los bytes, una única solicitud de descarga y enlace al
  archivo disponible después del cierre del navegador.
- La contraseña del propietario y la cookie no aparecen en el historial del agente.

## Fallo encontrado y corrección

La primera ejecución se detuvo con `Recovery budget exhausted for no_progress`:
el agente interpretaba varios clics en «Siguiente» como falta de progreso, aunque
las observaciones mostraran páginas diferentes.

La detección ahora considera los cambios observados por perfil, pestaña y marco.
Los identificadores nuevos de elementos y los nombres de capturas no cuentan como
progreso. Un clic exitoso por sí solo tampoco demuestra un cambio. Las pruebas
unitarias comprueban tanto el avance real como las repeticiones sin cambios.

`browser_inspect` también devuelve los identificadores, tamaños y enlaces de los
archivos descargados. El modelo simulado utiliza el enlace recibido de la
herramienta para entregar el archivo, sin inventar una ruta a partir de su ID.

## Validación adicional

**23 casos aprobados**: 6 de estado del agente, 5 de resultados abreviados,
5 del navegador y Chromium, 1 de flujo autenticado, 1 de API del propietario y
5 de interfaz y acceso por cuentas.

Pasaron seis pruebas de estado de ejecución, cinco de resultados abreviados, dos
del registro limitado de descargas, una de pérdida de eventos/cancelación/reinicio,
una de bóveda con pestaña explícita, la regresión de sesiones y tamaños y una de API
del propietario, privacidad y archivos. La interfaz compila con las dos
configuraciones de TypeScript.

Las pruebas de interfaz usan el paquete de producción con API simulada; comprueban
escritura Unicode, composición, teclas, pestañas, tamaños, envío privado a la bóveda,
limpieza de contraseñas, habilitación del agente, enlaces, cierre del sondeo, URL actual y una autorización
pendiente invalidada al cambiar de pestaña. También se verifican la ruta `/browser`
en el gateway, el acceso y cierre de sesión y el diseño del login en teléfono.
No equivalen a probar una cuenta real ni las decisiones de un modelo real.

## Límites comprobados

La instancia inspeccionada permite 128 turnos del modelo y cuatro horas de ejecución
activa. El navegador tiene límites de dos perfiles abiertos, cuatro pestañas por
perfil, diez minutos de inactividad y un presupuesto blando de 2048 MiB. Esto no
equivale a una prueba de cuatro horas ni garantiza acceso continuo a cualquier web.

El agente no puede introducir una contraseña recibida por chat. La vista `/browser`
ya está restaurada en el código, con controles de inicio de sesión, perfiles,
autorización del agente y bóveda cifrada. La instancia inspeccionada no tenía
perfiles personales. Las pruebas del proyecto son independientes del despliegue.

No se utilizaron cuentas reales, no se verificaron MFA/CAPTCHA y no se desplegó una
nueva aplicación en la instancia activa. Los comandos para repetir las pruebas
están en la [versión inglesa](../en/browser-workflow-audit.md#reproduce).
