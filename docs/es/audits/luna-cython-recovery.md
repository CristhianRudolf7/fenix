# Auditoría de recuperación Cython con Luna — 23 de septiembre de 2026

[English / informe completo](../../en/audits/luna-cython-recovery.md)

La última ronda falló: **7/11 comprobaciones oficiales**, recompensa 0 y timeout
oficial de 900 segundos. Se detuvieron las pruebas por petición del usuario.
No se desplegó ni se subió a GitHub.

| Último intento | Comprobaciones | Tiempo nativo | Tokens entrada / salida | Consultas command_status |
|---|---:|---:|---:|---:|
| Luna low | 8/11 | 11m00s | 798.942 / 8.315 | 21 |
| Luna max | 7/11 | 14m32s | 751.059 / 31.636 | 2 |

El tiempo nativo excluye preparación y comprobación inicial del proveedor.
Max alcanzó el límite oficial de 15 minutos. Ninguno aprobó la tarea completa.
No es una comparación aislada del pensamiento: max utilizó un arnés posterior,
espera por eventos y contexto configurado de 128k; low utilizó 32k.

Low se detuvo por compactación del contexto. Max compiló e instaló las tres
extensiones, pero dedicó demasiado tiempo a planificación/revisión, inspección y
compilaciones. Corrigió fractions.gcd, pero dejó usos incompatibles de NumPy int y
float. Fallaron el comportamiento de ccomplexity, la equivalencia de invariantes,
el ejemplo y la suite original (18 pruebas fallidas).

La espera por eventos está implementada en Rust, con avisos tras 120 segundos o
60 segundos sin salida, sin cancelar por silencio. Se conservaron sesiones
interactivas, lecturas inmediatas, permisos y cancelación. También se corrigieron
el cierre explícito, compactación, falsos positivos de credenciales, transmisión
del requerimiento al subagente dentro del mismo ámbito y detección de bloqueos en
la evaluación. El informe inglés conserva todos los intentos y las validaciones.

Los resultados muestran menos consultas de estado; no demuestran que max sea
mejor ni que la reducción se deba únicamente al nuevo mecanismo. No se cambió
la tarea ni su evaluador y no se proporcionaron pruebas ocultas al agente.
