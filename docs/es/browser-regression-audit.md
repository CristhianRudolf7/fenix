# Auditoría de regresión del navegador

[English](../en/browser-regression-audit.md)

La [auditoría de navegación autenticada](browser-workflow-audit.md) cubre la
ejecución desde el agente, la continuidad de tareas y las descargas lentas.

Las pruebas ejecutan el controlador Rust de Fénix sobre Chromium real con sandbox.
Verifican el DOM, el estado de la aplicación, los archivos descargados y las
dimensiones de las capturas. No basta con que la herramienta responda «éxito».
Se utilizan perfiles temporales y datos sintéticos, sin tocar cuentas personales.

Verificado el 2026-09-19 con Chromium 153.0.8010.47 en Debian 13: **59/59 casos de
la matriz, 6/6 pruebas Rust y 8/8 pruebas focalizadas de interfaz aprobadas**.
La persistencia también comprueba el valor de un token en IndexedDB después
del reinicio, además de la existencia de la base. Clippy pasó sin advertencias.
El [informe JSON](../benchmarks/browser-regression-2026-09-19.json) contiene los
resultados y tiempos individuales. La aplicación recompilada también aprobó el
recorrido real de los controles y su preparación inicial (2/2): escritura manual,
redimensionado, toma de control, cierre y conservación del perfil.

## Cobertura

La matriz incluye 59 escenarios independientes: Unicode y emoji, escritura y
borrado, cursor y teclado, inputs de distintos tipos, fechas, colores, textarea,
contenteditable, restricciones de edición, un formulario controlado por React,
casillas, radios, opciones deshabilitadas, archivos, elementos tapados o recortados,
hover, doble clic, arrastre, Shadow DOM, controles tardíos, frames anidados y de
otro sitio, escalado CSS, diálogos JavaScript e historial normal y SPA.

Otro recorrido comprueba referencias caducadas, privacidad de las observaciones,
redirecciones, descargas, ventanas nuevas, pestañas, cancelación y limpieza del
perfil temporal. Verifica diseño y clics a 390×844 con DPR 3, 768×1024 con DPR 2,
1280×800 y 1920×1080, incluidas las dimensiones reales de los PNG y la emulación
táctil. La prueba existente conserva su cobertura de bóveda y persistencia de
cookies, localStorage e IndexedDB al cerrar y reabrir Chromium.

Una prueba opcional visita el formulario público de Selenium, introduce datos
sintéticos y verifica su envío. Las pruebas de interfaz comprueban teclado,
pegado, teclas muertas, eventos de composición sin duplicación y unidades de
scroll. La composición se simula: no equivale a probar físicamente todos los IME.

## Correcciones

El navegador valida que el campo sea editable y que acepte el valor solicitado.
Los tipos especiales usan el setter nativo y eventos input/change; el texto común
usa entrada nativa de Chromium. Las observaciones, esperas y acciones comparten
las reglas de visibilidad, deshabilitación y Shadow DOM.

Los frames conservan el aislamiento de Chromium; se utiliza el proceso correcto,
se transforman las coordenadas y se comprueba si la página padre tapa el destino.
Los diálogos permiten recuperar el control sin esperar el timeout. La acción
`dialog` acepta `text` para responder un prompt. Una acción interrumpida no debe
repetirse automáticamente: el clic que abrió el diálogo puede haberse ejecutado.
Una operación ya cancelada no puede cerrar el perfil activo.

Los controles manuales reciben composición y texto mediante un campo auxiliar,
lo envían una vez y vacían el campo. El scroll convierte líneas y páginas a píxeles del viewport remoto. El cambio
de foco del teclado no desplaza una captura alta ni altera el destino del clic.

## Ejecutar

Desde la raíz, con las dependencias instaladas y Chromium con sandbox disponible:

```sh
node apps/web/tests/fixtures/build-browser-controlled.mjs
cargo test -p fenix-browser
cargo test -p fenix-browser adversarial_browser_matrix -- --ignored --nocapture
cargo test -p fenix-browser browser_navigation_resources_and_responsive_layout -- --ignored --nocapture
cargo test -p fenix-browser real_chromium_navigation_responsive_and_persistent_login -- --ignored --nocapture
# Opcional: requiere acceso a internet.
cargo test -p fenix-browser public_selenium_form_smoke -- --ignored --nocapture
pnpm --dir apps/web exec playwright test browser-search.spec.js
```

Tras compilar el formulario React, todas las pruebas Rust del navegador pueden
ejecutarse en serie con `cargo test -p fenix-browser -- --include-ignored --nocapture --test-threads=1`.
La matriz guarda resultados y tiempos en `target/browser-adversarial-report.json`.
`FENIX_BROWSER_CASE=frame` selecciona solamente casos de frames y sobrescribe ese
informe. El RSS impreso es una observación dependiente del equipo y la carga,
no una garantía de consumo; puede contar memoria compartida varias veces.

## Alcance

Esto no garantiza todas las webs, cuentas universitarias o WhatsApp, reservas,
pagos, CAPTCHA, Shadow DOM cerrado, controles de canvas, dispositivos físicos ni
otros motores. Chromium sigue siendo el motor. Las correcciones pertenecen al
navegador local y sus controles; no cambian la planificación general de Fénix.
