# Perfil de cuenta y consumo registrado

[English](../en/account-profile.md)

**Ver mi perfil** abre `/profile`, con secciones General, Consumo y Contraseña. Respeta la paleta de Fénix, los modos claro/oscuro y el menú lateral izquierdo en móvil. Está disponible incluso antes de configurar un modelo.

Se puede editar el nombre visible. El correo de acceso es de solo lectura porque todavía no existe un flujo de verificación para cambiarlo. Para modificar la contraseña se exige la actual y otra diferente de al menos ocho bytes; hay protección contra intentos repetidos. El cambio invalida todas las sesiones anteriores y pide volver a ingresar, sin cambiar la duración exacta de siete días ni detener las tareas del agente. No se añade registro de usuarios.

Consumo ofrece períodos de 7, 30 y 90 días, totales de entrada/salida, gráfico diario y tabla accesible. Usa la hora de Lima y únicamente la base de datos del usuario autenticado. Distingue ausencia de actividad de errores al cargar. No cuenta de nuevo tokens copiados en resultados de herramientas ni duplica decisiones recuperadas de caché.

Son **tokens registrados, no una factura completa**. Se incluyen tareas, resúmenes, enrutamiento, aprendizaje, algunas operaciones de memoria, embeddings de la biblioteca y las nuevas conversaciones/narraciones del interlocutor. Puede faltar voz histórica, comprobaciones de conexión, solicitudes fallidas y otras operaciones sin registro. No se inventan costos ni se llama a un modelo para generar el gráfico.

Se comprueban aislamiento, cálculo, cambio de contraseña e invalidación de sesiones, además de navegación, guardado, períodos, estados vacíos/errores y vista móvil en Playwright. Frontend y backend conservan servicios systemd separados.
