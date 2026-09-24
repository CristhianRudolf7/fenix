# Luna max con ventana de 200k

[English / informe completo](../../en/audits/luna-cython-200k.md)

Se repitió build-cython-ext con Luna max, ventana de 200.000 tokens,
compactación a 160.000 y el mismo plazo oficial de 900 segundos.

**Resultado: 2/11 comprobaciones, recompensa 0; 14m19s de ejecución nativa.**
Solo pasaron la versión de NumPy y la clonación. No hubo compactaciones ni
agotamiento de contexto: la entrada máxima registrada fue de 90.864 tokens.
Se contabilizaron 1.771.277 tokens de entrada y 82.967 de salida acumulados.

El maestro falló porque la nueva regla de cierre explícito rechazó sus mensajes
de progreso durante la delegación y agotó sus recuperaciones. El subagente
recibió tres errores HTTP 429 por el límite de 200k tokens/minuto de la cuenta y
agotó sus intentos. Había trabajado en un entorno virtual auxiliar y no terminó
la instalación global requerida. No fue el timeout oficial lo que finalizó la ronda.

Limitación del experimento: el proxy de registro no reenviaba Retry-After, aunque
el cliente de Fénix sí lo interpreta. Por ello, los reintentos rápidos observados
no permiten afirmar cómo respondería el despliegue con las cabeceras originales.

La ronda max anterior con 128k aprobó 7/11; la última low con 32k aprobó 8/11.
Una sola ejecución no aísla causalmente el tamaño del contexto. No se inició otra
ronda ni se hicieron más correcciones después de este resultado.

Quedaron actualizados los valores predeterminados del código y formulario a
200k/160k, conservando configuraciones explícitas guardadas. Compilación y prueba
del presupuesto de contexto aprobadas. Sin despliegue ni push.
