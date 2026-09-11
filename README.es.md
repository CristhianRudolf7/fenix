# Fénix

[English](README.md)

Fénix es una base en Rust para un asistente personal de IA. La implementación actual
ofrece inferencia de modelos y credenciales API aisladas para 17 integraciones.
El motor del asistente, la memoria, el planificador, Telegram, la voz y la interfaz
de administración se describen en los requisitos y la hoja de ruta; se construirán
sobre estas bibliotecas.

## Implementado

- Claves estáticas y selección estricta de credenciales por proyecto para OpenAI,
  Anthropic, Gemini, OpenRouter, DeepSeek, xAI, Mistral, Alibaba/Qwen, Moonshot/Kimi,
  Z.ai, MiniMax, Groq, Together AI, Bedrock, Azure, Vertex en modo Express y Ollama.
- Inferencia de texto, definición de herramientas, llamadas y resultados estructurados,
  uso de tokens y metadatos para continuar conversaciones mediante cinco protocolos.
- Conexiones HTTP asíncronas compartidas, timeouts, cancelación al descartar el futuro
  de la solicitud, tamaños acotados y errores clasificados.
- Configuración TOML con referencias a variables de entorno en lugar de claves.

`complete()` envía una solicitud de inferencia sin streaming. La biblioteca entrega
las llamadas a herramientas al código que la utiliza; su ejecución corresponderá al
futuro motor del asistente.

## Organización

```text
crates/
  fenix-core/          Contratos de conversación e identificadores
  fenix-credentials/   Manejo de secretos y selección por proyecto
  fenix-providers/     Catálogo, configuración, protocolos, HTTP y pruebas
config/               Ejemplos de configuración sin secretos
docs/
  en/                 Documentación principal
  es/                 Documentación en español
```

Las bibliotecas funcionan independientemente del frontend y los canales de mensajes.
Los futuros agentes podrán compartir `ProviderClient`, mientras la persistencia
guarda conversaciones sin acceder a credenciales. No se necesita un proceso del
sistema operativo por agente.

## Desarrollo

Instala [Rust mediante rustup](https://rust-lang.org/tools/install/). La versión de
Rust se fija en `rust-toolchain.toml` y las dependencias en `Cargo.lock`.

```sh
cargo test --workspace --all-targets --locked
cargo test --workspace --doc --locked
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
```

Las pruebas predeterminadas utilizan servidores HTTP locales y credenciales ficticias.
La prueba real sólo se ejecuta al seleccionarla explícitamente; requiere una clave
y un modelo reales y puede generar consumo facturable. Las pruebas locales no
demuestran que una cuenta remota tenga acceso.

- [Uso de la biblioteca y verificación](docs/es/development.md)
- [Proveedores, autenticación y fuentes oficiales](docs/es/provider-reference.md)
- [Ejemplo de configuración](config/providers.example.toml)
- [Requisitos actuales](REQUIREMENTS.md)
- [Trabajo futuro](ROADMAP.md)
