import { readFile, writeFile, mkdir, readdir, rm, copyFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

const directory = path.dirname(fileURLToPath(import.meta.url));
const repository = path.resolve(directory, "../../..");
const output = path.resolve(directory, "../src/content/docs");
const origin = "https://fenix.crehlabs.com";
const github = "https://github.com/CristhianRudolf7/fenix";
await copyFile(path.join(repository, "scripts/install.sh"), path.resolve(directory, "../public/install.sh"));
const pages = [];
function add(id, title, body, description, reference = false) {
  pages.push({ id, title, body, description, reference });
}
add(
  "inicio",
  "Un asistente. Muchas posibilidades.",
  `Fénix reúne conversación, proyectos y trabajo en segundo plano. Puedes pedir una investigación, preparar documentos o desarrollar software sin cambiar de conversación.

## Conversa y sigue trabajando
El interlocutor interpreta tus mensajes y comunica el progreso observable. El agente ejecutor realiza las tareas y puede delegar objetivos concretos a subagentes. La disponibilidad depende de los modelos y servicios configurados.

## Tu espacio de trabajo
- Una conversación continua con contexto por proyecto.
- Archivos y documentos privados por cuenta.
- Planes editables y trabajo en segundo plano.
- Skills que se descubren y cargan cuando hacen falta.
- Navegador, terminal y herramientas de documentos.

## Elige cómo empezar
[Instala desde el código](/docs/instalacion/) o [explora las capacidades actuales](/docs/capacidades/). Instala la versión personal con un solo comando desde [la guía de instalación](/docs/instalacion/).

## Un proyecto en evolución
La versión declarada es **0.3.0**. Tener una capacidad implementada no garantiza que resuelva cualquier tarea. Las guías, evaluaciones y limitaciones permiten distinguir lo disponible de lo que falta comprobar.`,
  "Conversación continua, herramientas y trabajo en segundo plano.",
);
add(
  "instalacion",
  "Instala Fénix",
  `## Instalador público
El script descarga una release precompilada, verifica SHA-256 y configura un servicio systemd individual conservando datos. Linux x86_64, glibc ≥ 2.39 y systemd de usuario son necesarios. La release 0.3.1 incluye los paquetes compilados. No instala automáticamente las dependencias opcionales ni el despliegue multiusuario.

Ejecuta sin sudo: \`curl -fsSL https://fenix.crehlabs.com/install.sh | bash\`.

Al terminar abre http://127.0.0.1:4173 y configura tu proveedor, modelo y clave desde la bienvenida. Las bibliotecas OpenSSL 3, zlib y zstd deben estar disponibles. El servicio es \`fenix-personal.service\`; puedes detenerlo con \`systemctl --user disable --now fenix-personal.service\` sin borrar los datos de \`~/.local/share/fenix-personal/\`. El acceso queda limitado a localhost; esta instalación no publica un servidor multiusuario. Consulta [la hoja de ruta](/docs/hoja-de-ruta/).

## Requisitos para desarrollar
La alternativa para desarrolladores parte del repositorio. Necesitas Linux, el Rust fijado en \`rust-toolchain.toml\`, una versión de Node compatible con Vite y pnpm 10.34.5.

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
`,
  "Instalación pública con paquetes compilados y alternativa desde el repositorio.",
);
add(
  "configuracion",
  "Configuración sin sorpresas",
  `## Dos formas de utilizar Fénix
**Instalación propia:** el propietario administra los modelos, las credenciales y los servicios locales. **Servicio alojado:** el operador configura los proveedores; el usuario final entra con su cuenta y no necesita conocer las claves del operador.

## Acceso y modelos son cosas distintas
La contraseña permite entrar a una cuenta de Fénix. La API key permite utilizar un proveedor de modelos. La autenticación de cuentas pertenece al gateway; Google OAuth requiere configuración adicional y capacidad de cuentas aprovisionada.

## Conexiones del agente
Fénix separa un modelo rápido de conversación y un modelo inteligente de ejecución. No hay un modelo de producción obligatorio en el código: la elección depende de la configuración y del acceso de la cuenta al proveedor. El proyecto admite múltiples proveedores mediante sus adaptadores.

El operador debe configurar secretos fuera de Git y comprobar el endpoint, el modelo, sus herramientas y la cuota disponible. Una prueba real de conexión puede consumir tokens y tener coste. Consulta [la referencia de proveedores](/docs/referencia/provider-reference/).

## Bienvenida propuesta
La próxima distribución simplificada abriría un asistente web para crear la cuenta del propietario, seleccionar las conexiones de modelos y verificar el acceso. Telegram y otras conexiones serían opcionales. Este asistente es una propuesta de distribución, no una promesa de que la interfaz actual ya permite completar todos esos pasos.

## Datos y entorno
La configuración inicial utiliza la hora de Perú. Los directorios personales, bases, sesiones y credenciales no deben publicarse con el repositorio. Los entornos aislados y el navegador requieren servicios adicionales: instalar la interfaz no los activa automáticamente.`,
  "Acceso, roles de modelos y credenciales en instalaciones propias y alojadas.",
);
for (const [id, title, filename, description] of [
  [
    "capacidades",
    "Capacidades actuales",
    "REQUIREMENTS.md",
    "Qué está implementado y cuáles son sus límites.",
  ],
  [
    "hoja-de-ruta",
    "Lo que viene después",
    "ROADMAP.md",
    "Propuestas, investigación y validaciones pendientes.",
  ],
  [
    "versiones",
    "Cambios y versiones",
    "CHANGELOG.md",
    "Registro de cambios del proyecto.",
  ],
])
  add(
    id,
    title,
    await readFile(path.join(repository, filename), "utf8"),
    description,
  );
add(
  "arquitectura",
  "Una base modular",
  `## Motor y canales
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
| \`apps/docs\` | Documentación Astro y Starlight, independiente de los modelos. |

## Trabajo concurrente
Los subagentes reciben un propósito exacto y cargan skills bajo demanda. Solo hay un nivel de delegación. Los worktrees permiten modificaciones simultáneas, con integración controlada; no resuelven solos todos los conflictos.

## Contexto y aislamiento
La ventana inicial del motor es de 200.000 tokens, con compactación a 160.000. Esto no aumenta la capacidad ni las cuotas del proveedor. Los permisos y los servicios aislados aplican las fronteras de acceso; una descripción en un prompt no sustituye al aislamiento.

## Frontend independiente
La aplicación y esta documentación se sirven como archivos estáticos. El backend mantiene la lógica y los datos. No se necesita un servidor Node para servir los archivos ya construidos.`,
  "Rust en el motor, frontend independiente y contexto por ámbito.",
);
add(
  "despliegue",
  "Publicación y servicios",
  `## Aplicación y documentación
La documentación ocupa la raíz de [fenix.crehlabs.com](${origin}). Las rutas \`/docs/\` son públicas y contienen estas guías. La vista móvil del entorno remoto existente conserva su acceso en [\`/movil.html\`](/movil.html); sus controles de autenticación siguen siendo independientes.

## Construir la documentación
\`\`\`sh
pnpm --dir apps/docs install --frozen-lockfile
pnpm --dir apps/docs build
pnpm --dir apps/docs preview
\`\`\`

Astro y Starlight transforman el contenido local en HTML y Pagefind crea el índice de búsqueda. No llama a modelos, no necesita claves y no recopila consultas de búsqueda en un servidor. Los archivos construidos quedan en \`apps/docs/dist\`, fuera de Git.

## Publicación estática
\`scripts/publish-docs.sh\` crea una versión y cambia atómicamente el enlace activo en \`~/.local/share/fenix-docs/current\`. Caddy sirve ese directorio mediante el servicio \`fenix-web.service\`. Revisa \`packaging/linux/Caddyfile.docs\` antes de adaptarlo a tu servidor: la ruta del entorno remoto y su puerto dependen del despliegue.

## Comprobaciones pendientes
Las pruebas locales no certifican todos los controles del despliegue. Sigue pendiente la comprobación real del firewall indicada en los requisitos y el README.`,
  "Documentación pública y servicios independientes.",
);
async function discover(directory, prefix = "") {
  for (const entry of await readdir(directory, { withFileTypes: true })) {
    const relative = path.posix.join(prefix, entry.name);
    if (entry.isDirectory())
      await discover(path.join(directory, entry.name), relative);
    else if (entry.name.endsWith(".md")) {
      const body = await readFile(path.join(directory, entry.name), "utf8");
      const title =
        body.match(/^# (.+)$/m)?.[1] || entry.name.replace(".md", "");
      add(
        `referencia/${relative.slice(0, -3)}`,
        title,
        body,
        "Referencia técnica conservada del repositorio; consulta su fecha y alcance.",
        true,
      );
    }
  }
}
await discover(path.join(repository, "docs/es"));

function resolveLink(href, source) {
  if (/^(?:[a-z][a-z0-9+.-]*:|\/|#)/i.test(href)) return href;
  const [file, anchor] = href.split("#");
  const target = path.posix.normalize(
    path.posix.join(path.posix.dirname(source), file),
  );
  const roots = {
    "README.md": "inicio",
    "README.en.md": "inicio",
    "REQUIREMENTS.md": "capacidades",
    "ROADMAP.md": "hoja-de-ruta",
    "CHANGELOG.md": "versiones",
  };
  const suffix = anchor ? "#" + anchor : "";
  if (roots[target]) return "/docs/" + roots[target] + "/" + suffix;
  if (/^docs\/(en|es)\/.*\.md$/.test(target)) {
    const id =
      "referencia/" + target.replace(/^docs\/(en|es)\//, "").slice(0, -3);
    if (pages.some((page) => page.id === id))
      return "/docs/" + id + "/" + suffix;
  }
  return github + "/blob/main/" + target + suffix;
}
await rm(output, { recursive: true, force: true });
for (const page of pages) {
  const source = page.reference
    ? "docs/es/" + page.id.replace("referencia/", "") + ".md"
    : {
        capacidades: "REQUIREMENTS.md",
        "hoja-de-ruta": "ROADMAP.md",
        versiones: "CHANGELOG.md",
      }[page.id] || "README.md";
  let body = page.body
    .replace(/^# .+\n/, "")
    .replace(/\[English\]\([^\n)]*\)/g, "");
  // Transform prose links without changing executable examples in fenced blocks.
  body = body
    .split(/(```[\s\S]*?```)/g)
    .map((part, index) =>
      index % 2
        ? part
        : part.replace(
            /\]\(([^\s)]+)\)/g,
            (_, href) => "](" + resolveLink(href, source) + ")",
          ),
    )
    .join("");
  if (page.reference)
    body =
      "> Esta referencia conserva la revisión original. Puede describir etapas anteriores; consulta [las capacidades actuales](/docs/capacidades/) para conocer el estado vigente.\n\n" +
      body;
  const destination = path.join(output, "docs", page.id + ".md");
  await mkdir(path.dirname(destination), { recursive: true });
  await writeFile(
    destination,
    "---\ntitle: " +
      JSON.stringify(page.title) +
      "\ndescription: " +
      JSON.stringify(page.description) +
      "\neditUrl: false\n---\n\n" +
      body,
  );
}
const references = pages
  .filter((page) => page.reference)
  .sort((a, b) => a.title.localeCompare(b.title, "es"));
await writeFile(
  path.join(output, "docs/referencias.md"),
  "---\ntitle: Referencias técnicas\ndescription: Guías y evaluaciones originales con su fecha, alcance y límites.\neditUrl: false\n---\n\n" +
    references
      .map((page) => "- [" + page.title + "](/docs/" + page.id + "/)")
      .join("\n") +
    "\n",
);
console.log("Prepared " + (pages.length + 1) + " Starlight content pages.");
