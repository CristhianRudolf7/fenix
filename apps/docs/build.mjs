import { readFile, writeFile, mkdir, readdir, cp, rm } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { marked } from 'marked';

const directory = path.dirname(fileURLToPath(import.meta.url));
const repository = path.resolve(directory, '../..');
const output = path.join(directory, 'dist');
const origin = 'https://fenix.crehlabs.com';
const github = 'https://github.com/CristhianRudolf7/fenix';
const escape = (value) => String(value).replace(/[&<>"']/g, (c) => ({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c]));
const slug = (value) => value.toLowerCase().normalize('NFD').replace(/[\u0300-\u036f]/g, '').replace(/[^a-z0-9 -]/g, '').trim().replace(/ +/g, '-');
const pages = [];
const group = [
  ['Empieza aquí', [['inicio','Introducción'],['instalacion','Instalación'],['configuracion','Configuración inicial']]],
  ['Conoce Fénix', [['capacidades','Capacidades actuales'],['arquitectura','Arquitectura'],['despliegue','Despliegue'],['hoja-de-ruta','Hoja de ruta'],['versiones','Cambios y versiones']]],
];
function add(id, title, body, description, reference = false) { pages.push({id,title,body,description,reference}); }
add('inicio', 'Un asistente. Muchas posibilidades.', `Fénix reúne conversación, proyectos y trabajo en segundo plano. Puedes pedir una investigación, preparar documentos o desarrollar software sin cambiar de conversación.

## Conversa y sigue trabajando
El interlocutor interpreta tus mensajes y comunica el progreso observable. El agente ejecutor realiza las tareas y puede delegar objetivos concretos a subagentes. La disponibilidad depende de los modelos y servicios configurados.

## Tu espacio de trabajo
- Una conversación continua con contexto por proyecto.
- Archivos y documentos privados por cuenta.
- Planes editables y trabajo en segundo plano.
- Skills que se descubren y cargan cuando hacen falta.
- Navegador, terminal y herramientas de documentos.

## Elige cómo empezar
[Instala desde el código](/docs/instalacion/) o [explora las capacidades actuales](/docs/capacidades/). La instalación pública con un solo comando está en la hoja de ruta; todavía no se distribuye un instalador en esta web.

## Un proyecto en evolución
La versión declarada es **0.3.0**. Tener una capacidad implementada no garantiza que resuelva cualquier tarea. Las guías, evaluaciones y limitaciones permiten distinguir lo disponible de lo que falta comprobar.`, 'Conversación continua, herramientas y trabajo en segundo plano.');
add('instalacion','Instala Fénix',`## Requisitos para desarrollar
Por ahora la instalación disponible parte del repositorio. Necesitas Linux, el Rust fijado en \`rust-toolchain.toml\`, una versión de Node compatible con Vite y pnpm 10.34.5.

## Obtén el código
\`\`\`sh
git clone ${github}.git
cd fenix
pnpm --dir apps/web install --frozen-lockfile
pnpm --dir apps/web build
cargo run --locked -p fenix-server --bin fenix
\`\`\`

Abre [http://127.0.0.1:4173](http://127.0.0.1:4173). La ejecución individual y el gateway con cuentas tienen configuración distinta: no asumas que el modo individual publica un servicio multiusuario.

## Instalación local con systemd
Desde el clon, \`bash scripts/install-local.sh\` compila e instala para el usuario actual. Requiere Cargo, pnpm, systemd, xdg-open y una sesión de systemd de usuario. Comprueba el script y evita ejecutarlo sobre otra instalación sin revisar sus rutas y su unidad \`fenix.service\`.

## Próximamente: un solo comando
La propuesta es descargar el binario Rust y el frontend ya compilados, comprobar su integridad, preparar dependencias y servicios y abrir una bienvenida web. **No hay todavía un comando público de instalación listo para copiar.**

El modelo rápido y el inteligente, las credenciales y los canales se configurarían desde esa bienvenida. Consulta [la hoja de ruta](/docs/hoja-de-ruta/).`, 'Instalación actual desde el repositorio y distribución prevista.');
add('configuracion','Configuración sin sorpresas',`## Dos formas de utilizar Fénix
**Instalación propia:** el propietario administra los modelos, las credenciales y los servicios locales. **Servicio alojado:** el operador configura los proveedores; el usuario final entra con su cuenta y no necesita conocer las claves del operador.

## Acceso y modelos son cosas distintas
La contraseña permite entrar a una cuenta de Fénix. La API key permite utilizar un proveedor de modelos. La autenticación de cuentas pertenece al gateway; Google OAuth requiere configuración adicional y capacidad de cuentas aprovisionada.

## Conexiones del agente
Fénix separa un modelo rápido de conversación y un modelo inteligente de ejecución. No hay un modelo de producción obligatorio en el código: la elección depende de la configuración y del acceso de la cuenta al proveedor. El proyecto admite múltiples proveedores mediante sus adaptadores.

El operador debe configurar secretos fuera de Git y comprobar el endpoint, el modelo, sus herramientas y la cuota disponible. Una prueba real de conexión puede consumir tokens y tener coste. Consulta [la referencia de proveedores](/docs/referencia/provider-reference/).

## Bienvenida propuesta
La próxima distribución simplificada abriría un asistente web para crear la cuenta del propietario, seleccionar las conexiones de modelos y verificar el acceso. Telegram y otras conexiones serían opcionales. Este asistente es una propuesta de distribución, no una promesa de que la interfaz actual ya permite completar todos esos pasos.

## Datos y entorno
La configuración inicial utiliza la hora de Perú. Los directorios personales, bases, sesiones y credenciales no deben publicarse con el repositorio. Los entornos aislados y el navegador requieren servicios adicionales: instalar la interfaz no los activa automáticamente.`, 'Acceso, roles de modelos y credenciales en instalaciones propias y alojadas.');
for (const [id, title, filename, description] of [
  ['capacidades','Capacidades actuales','REQUIREMENTS.md','Qué está implementado y cuáles son sus límites.'],
  ['hoja-de-ruta','Lo que viene después','ROADMAP.md','Propuestas, investigación y validaciones pendientes.'],
  ['versiones','Cambios y versiones','CHANGELOG.md','Registro de cambios del proyecto.'],
]) add(id, title, await readFile(path.join(repository,filename),'utf8'), description);
add('arquitectura','Una base modular',`## Motor y canales
El backend utiliza **Rust y Axum**. El interlocutor de conversación, el ejecutor, los subagentes y los canales comparten estado duradero con ámbitos de usuario y proyecto. La conversación visible puede ser continua sin mezclar automáticamente el contexto de proyectos distintos.

## Componentes
| Componente | Responsabilidad |
|---|---|
| \`fenix-core\` | Contratos de conversación y herramientas. |
| \`fenix-credentials\` | Resolución de credenciales por proveedor y proyecto. |
| \`fenix-providers\` | Transporte y adaptación de protocolos de modelos. |
| \`fenix-storage\` | Persistencia, memoria y registros de tareas. |
| \`fenix-agent\` | Ejecución, coordinación y herramientas. |
| \`fenix-web\` | Búsqueda propia y lectura ligera. |
| \`apps/server\` | Servidor, gateway y APIs. |
| \`apps/web\` | Aplicación Vite, React, TypeScript y Tailwind. |
| \`apps/docs\` | Documentación estática, independiente de los modelos. |

## Trabajo concurrente
Los subagentes reciben un propósito exacto y cargan skills bajo demanda. Solo hay un nivel de delegación. Los worktrees permiten modificaciones simultáneas, con integración controlada; no resuelven solos todos los conflictos.

## Contexto y aislamiento
La ventana inicial del motor es de 200.000 tokens, con compactación a 160.000. Esto no aumenta la capacidad ni las cuotas del proveedor. Los permisos y los servicios aislados aplican las fronteras de acceso; una descripción en un prompt no sustituye al aislamiento.

## Frontend independiente
La aplicación y esta documentación se sirven como archivos estáticos. El backend mantiene la lógica y los datos. No se necesita un servidor Node para servir los archivos ya construidos.`, 'Rust en el motor, frontend independiente y contexto por ámbito.');
add('despliegue','Publicación y servicios',`## Aplicación y documentación
La documentación ocupa la raíz de [fenix.crehlabs.com](${origin}). Las rutas \`/docs/\` son públicas y contienen estas guías. La vista móvil del entorno remoto existente conserva su acceso en [\`/movil.html\`](/movil.html); sus controles de autenticación siguen siendo independientes.

## Construir la documentación
\`\`\`sh
pnpm --dir apps/docs install --frozen-lockfile
pnpm --dir apps/docs build
pnpm --dir apps/docs preview
\`\`\`

El generador transforma contenido local en HTML y crea el índice de búsqueda. No llama a modelos, no necesita claves y no recopila consultas de búsqueda en un servidor. Los archivos construidos quedan en \`apps/docs/dist\`, fuera de Git.

## Publicación estática
\`scripts/publish-docs.sh\` crea una versión y cambia atómicamente el enlace activo en \`~/.local/share/fenix-docs/current\`. Caddy sirve ese directorio mediante el servicio \`fenix-web.service\`. Revisa \`packaging/linux/Caddyfile.docs\` antes de adaptarlo a tu servidor: la ruta del entorno remoto y su puerto dependen del despliegue.

## Comprobaciones pendientes
Las pruebas locales no certifican todos los controles del despliegue. Sigue pendiente la comprobación real del firewall indicada en los requisitos y el README.`, 'Documentación pública y servicios independientes.');
async function discover(directory, prefix = '') {
  for (const entry of await readdir(directory, {withFileTypes:true})) {
    const relative = path.posix.join(prefix,entry.name);
    if (entry.isDirectory()) await discover(path.join(directory,entry.name),relative);
    else if (entry.name.endsWith('.md')) {
      const body = await readFile(path.join(directory,entry.name),'utf8');
      const title = body.match(/^# (.+)$/m)?.[1] || entry.name.replace('.md','');
      add(`referencia/${relative.slice(0,-3)}`, title, body, 'Referencia técnica conservada del repositorio; consulta su fecha y alcance.', true);
    }
  }
}
await discover(path.join(repository,'docs/es'));
let currentSource = 'README.md';
const renderer = new marked.Renderer();
renderer.heading = ({tokens,depth}) => {
  const text = renderer.parser.parseInline(tokens);
  return `<h${depth} id="${slug(text.replace(/<[^>]+>/g,''))}">${text}</h${depth}>`;
};
renderer.link = ({href,title,tokens}) => {
  if (!href || /^(javascript|data|vbscript):/i.test(href)) return renderer.parser.parseInline(tokens);
  let url = href;
  if (!/^(?:[a-z][a-z0-9+.-]*:|\/|#)/i.test(href)) {
    const [file,anchor] = href.split('#');
    const target = path.posix.normalize(path.posix.join(path.posix.dirname(currentSource),file));
    const roots = {'README.md':'inicio','README.en.md':'inicio','REQUIREMENTS.md':'capacidades','ROADMAP.md':'hoja-de-ruta','CHANGELOG.md':'versiones'};
    if (roots[target]) url=`/docs/${roots[target]}/${anchor ? '#'+anchor:''}`;
    else if (/^docs\/(en|es)\/.*\.md$/.test(target)) {
      const id = 'referencia/' + target.replace(/^docs\/(en|es)\//,'').slice(0,-3);
      url = pages.some(p=>p.id===id) ? `/docs/${id}/${anchor ? '#'+anchor:''}` : `${github}/blob/main/${target}${anchor ? '#'+anchor:''}`;
    } else url = `${github}/blob/main/${target}${anchor ? '#'+anchor:''}`;
  }
  return `<a href="${escape(url)}"${/^https?:/.test(url)?' rel="noreferrer"':''}>${renderer.parser.parseInline(tokens)}</a>`;
};
marked.setOptions({renderer, gfm:true});
const logo = '<svg viewBox="0 0 40 40" aria-hidden="true"><path d="M20 35C9 29 5 18 5 7c5 5 9 9 15 11C26 16 30 12 35 7c0 11-4 22-15 28Z" fill="currentColor"/><path d="m20 4 5 11-5 6-5-6Z" fill="currentColor"/><path d="m8 30 12-7 12 7-12-3Z" fill="currentColor"/></svg>';
const nav = group.map(([name,links])=>`<section><h2>${name}</h2>${links.map(([id,label])=>`<a href="/docs/${id}/" data-page="${id}">${label}</a>`).join('')}</section>`).join('');
function shell(title, content, id = '', home = false) {
 return `<!doctype html><html lang="es"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><meta name="theme-color" content="#f7f6f2"><meta name="description" content="Fénix: documentación de un asistente de IA con conversación continua, herramientas y trabajo en segundo plano."><meta property="og:title" content="${escape(title)} · Fénix"><meta property="og:description" content="Conversa. Crea. Avanza. Explora las guías de Fénix."><meta property="og:image" content="${origin}/docs-assets/social.png"><link rel="canonical" href="${origin}${home?'/':'/docs/'+id+'/'}"><link rel="icon" href="/docs-assets/favicon.svg" type="image/svg+xml"><link rel="stylesheet" href="/docs-assets/site.css"><title>${escape(title)} · Fénix</title><script src="/docs-assets/site.js" defer></script></head><body data-page="${escape(id)}"><a class="skip" href="#content">Saltar al contenido</a><header><a class="brand" href="/">${logo}<span>Fénix</span><span class="brand-tag">DOCUMENTACIÓN</span></a><nav aria-label="Principal"><a href="/docs/inicio/">Guías</a><a href="/docs/hoja-de-ruta/">Hoja de ruta</a><a href="${github}">GitHub ↗</a></nav><button class="search-open" aria-label="Buscar en la documentación">⌕ <span>Buscar</span><kbd>Ctrl K</kbd></button><button class="theme" aria-label="Cambiar tema">◐</button><button class="menu" aria-label="Abrir menú" aria-expanded="false">☰</button></header>${home?content:`<div class="docs-layout"><aside class="sidebar"><div class="sidebar-top">GUÍA DE FÉNIX <span>0.3.0</span></div>${nav}<section><h2>Profundiza</h2><a href="/docs/referencias/">Referencias técnicas</a><a href="${github}/tree/main/docs/es">Archivos en GitHub ↗</a></section><div class="sidebar-note">Una base abierta.<br>Un proyecto en evolución.</div></aside><main id="content" class="article"><div class="eyebrow">${id.startsWith('referencia/')?'REFERENCIA TÉCNICA':'DOCUMENTACIÓN'} <span>FÉNIX / 0.3.0</span></div>${content}<footer class="article-footer"><a href="${github}">Explorar el código ↗</a><span>CREH Labs · Fénix</span></footer></main><aside class="outline"><span>EN ESTA PÁGINA</span><div id="outline-links"></div></aside></div>`}<dialog class="search-dialog"><div class="search-header"><label for="search-input">Buscar documentación</label><button data-close-search aria-label="Cerrar búsqueda">✕</button></div><input id="search-input" type="search" placeholder="Prueba «memoria», «documentos», «instalación»…" autocomplete="off"><div id="search-results" aria-live="polite"></div><div class="search-footer">Búsqueda local · sin enviar tus consultas</div></dialog></body></html>`;
}
const home = `<main id="content"><section class="hero"><div class="hero-copy reveal"><div class="pill"><span></span> ASISTENTE DE IA · CÓDIGO PÚBLICO</div><h1>Conversa.<br>Crea. <em>Avanza.</em></h1><p>Un espacio para tus ideas, tus proyectos y el trabajo que quieres resolver. Conoce cómo funciona Fénix y construye con él.</p><div class="hero-actions"><a class="button primary" href="/docs/inicio/">Explorar documentación <span>↗</span></a><a class="button secondary" href="${github}">Ver en GitHub <span>→</span></a></div><div class="hero-detail"><span>Rust en el motor</span><i></i><span>Tu contexto, conectado</span><i></i><span>Versión 0.3.0</span></div></div><div class="hero-visual reveal" aria-label="Conversación, planificación y documentos conectados"><div class="orbit orbit-one"></div><div class="orbit orbit-two"></div><div class="core">${logo}<span>FÉNIX</span></div><div class="float-card card-chat"><div class="card-icon">↗</div><span>UNA CONVERSACIÓN</span><strong>De la idea a la acción</strong><div class="lines"><b></b><b></b></div></div><div class="float-card card-plan"><div class="card-icon green">✓</div><span>TRABAJO EN SEGUNDO PLANO</span><strong>Tu proyecto sigue avanzando</strong><div class="steps"><i></i><i></i><i></i><i></i></div></div><div class="float-card card-doc"><div class="card-icon">▤</div><span>TU ESPACIO</span><strong>Documentos y conocimiento</strong><div class="mini-tags"><span>Archivos</span><span>Memoria</span></div></div></div></section><section class="start-section"><div class="section-heading reveal"><div><span class="eyebrow">EMPIEZA POR AQUÍ</span><h2>Menos vueltas.<br>Más posibilidades.</h2></div><p>Guías claras para instalar, entender y ampliar tu asistente.</p></div><div class="guide-cards"><a class="guide-card reveal" href="/docs/instalacion/"><span class="guide-number">01 / EMPEZAR</span><div class="guide-symbol">↳</div><h3>Hazlo tuyo</h3><p>Instalación actual, requisitos y configuración de tu entorno.</p><span class="guide-link">Instalar Fénix ↗</span></a><a class="guide-card reveal" href="/docs/capacidades/"><span class="guide-number">02 / DESCUBRIR</span><div class="guide-symbol">✳</div><h3>Todo, conectado</h3><p>Conversación, subagentes, documentos, navegador y programación.</p><span class="guide-link">Explorar capacidades ↗</span></a><a class="guide-card reveal" href="/docs/arquitectura/"><span class="guide-number">03 / CONSTRUIR</span><div class="guide-symbol">⌘</div><h3>Una base abierta</h3><p>Entiende el motor, los límites y cómo se organiza el proyecto.</p><span class="guide-link">Conocer la arquitectura ↗</span></a></div></section><section class="closing reveal"><div><span class="eyebrow">CONSTRUIMOS EN ABIERTO</span><h2>Lo siguiente empieza<br>con una buena idea.</h2><p>Explora las propuestas y validaciones pendientes. El futuro también se documenta.</p></div><a class="button secondary" href="/docs/hoja-de-ruta/">Ver la hoja de ruta ↗</a></section></main><footer class="site-footer"><a class="brand" href="/">${logo}<span>Fénix</span></a><span>Un proyecto de CREH Labs</span><a href="${github}">Código en GitHub ↗</a></footer>`;
await rm(output,{recursive:true,force:true});await mkdir(output,{recursive:true});
await writeFile(path.join(output,'index.html'),shell('Conversa. Crea. Avanza.', home,'',true));
const search = [];
for (const page of pages) {
 currentSource = page.reference ? 'docs/es/' + page.id.replace('referencia/','') + '.md' : ({capacidades:'REQUIREMENTS.md','hoja-de-ruta':'ROADMAP.md',versiones:'CHANGELOG.md'}[page.id] || 'README.md');
 const text = page.body.replace(/^# .+\n/,'').replace(/\[English\]\([^\n)]*\)/g,'');
 const notice = page.reference ? '<div class="notice">Esta referencia conserva la revisión original del repositorio. Puede describir etapas anteriores; consulta <a href="/docs/capacidades/">los requisitos actuales</a> para conocer el estado vigente.</div>' : '';
 const dir = path.join(output,'docs',page.id);await mkdir(dir,{recursive:true});
 await writeFile(path.join(dir,'index.html'),shell(page.title,`<h1>${escape(page.title)}</h1><p class="lead">${escape(page.description)}</p>${notice}<div class="prose">${marked.parse(text)}</div>`,page.id));
 search.push({title:page.title,url:`/docs/${page.id}/`,description:page.description,text:page.body.replace(/```[\s\S]*?```/g,'').replace(/[#*`|]/g,'')});
}
const references = pages.filter(p=>p.reference).sort((a,b)=>a.title.localeCompare(b.title,'es'));
await mkdir(path.join(output,'docs/referencias'),{recursive:true});
await writeFile(path.join(output,'docs/referencias/index.html'),shell('Referencias técnicas',`<h1>Referencias técnicas</h1><p class="lead">Guías y evaluaciones originales. Las fechas, el alcance y los límites de cada revisión importan.</p><div class="reference-grid">${references.map(p=>`<a href="/docs/${p.id}/"><strong>${escape(p.title)}</strong><span>Leer referencia ↗</span></a>`).join('')}</div>`,'referencias'));
await cp(path.join(directory,'assets'),path.join(output,'docs-assets'),{recursive:true});
await writeFile(path.join(output,'docs-assets/search.json'),JSON.stringify(search));
await writeFile(path.join(output,'robots.txt'),`User-agent: *\nAllow: /docs/\nAllow: /$\nDisallow: /api/\nDisallow: /movil.html\nSitemap: ${origin}/sitemap.xml\n`);
await writeFile(path.join(output,'sitemap.xml'),`<?xml version="1.0" encoding="UTF-8"?><urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9"><url><loc>${origin}/</loc></url>${[...pages,{id:'referencias'}].map(p=>`<url><loc>${origin}/docs/${p.id}/</loc></url>`).join('')}</urlset>`);
await writeFile(path.join(output,'404.html'),shell('Página no encontrada','<h1>Esta página no está aquí.</h1><p class="lead">Vuelve a las guías o utiliza la búsqueda para encontrar lo que necesitas.</p><a class="button primary" href="/docs/inicio/">Volver a la documentación →</a>','404'));
console.log(`Built ${pages.length + 2} documentation pages.`);
