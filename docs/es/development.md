# Uso de la biblioteca y verificación

[English](../en/development.md)

## Responsabilidades

`fenix-core` contiene los contratos serializables de conversación.
`fenix-credentials` maneja secretos y la selección exacta por proyecto.
`fenix-providers` depende de ambos y contiene configuración TOML, endpoints,
HTTP y conversión de protocolos. El futuro agente, los canales, la memoria,
las herramientas y el planificador consumirán estas bibliotecas desde sus propios
módulos.

Esta entrega no incluye interfaz web, CLI interactiva, ejecución de herramientas
ni aprovisionamiento de cuentas. `ProviderClient::complete` realiza una solicitud
de inferencia sin streaming. Se utilizan claves existentes: las claves de cada
proyecto se crean en el proveedor y después se referencian desde Fénix.

## Claves estáticas

Registra las credenciales en `CredentialStore` y comparte un cliente entre tareas.
Las claves pueden venir de variables de entorno o de valores `ApiKey` inyectados
por un futuro gestor de secretos. No son serializables, se ocultan en `Debug` y su
memoria se sobrescribe al liberarlas. Los encabezados se marcan como sensibles;
los errores no incluyen cuerpos de respuesta ni credenciales. Esto no cifra los
secretos en disco: su persistencia corresponde a la aplicación que use la biblioteca.

```rust
use fenix_core::{ChatRequest, Message, ProviderId};
use fenix_credentials::{Credential, CredentialStore};
use fenix_providers::{ProviderClient, ProviderConfig};

async fn complete(model: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut credentials = CredentialStore::default();
    credentials.set_default(
        ProviderId::OpenAi,
        Credential::environment("OPENAI_API_KEY")?,
    );
    let client = ProviderClient::new(credentials)?;
    let request = ChatRequest::new(model, vec![Message::user("Say hello.")]);
    let response = client.complete(
        &ProviderConfig::new(ProviderId::OpenAi), None, &request,
    ).await?;
    println!("{}", response.message.text_content());
    Ok(())
}
```

El identificador de modelo siempre es explícito. La biblioteca no presupone que
la cuenta tenga acceso ni cambia de proveedor automáticamente. Para usar
herramientas hace falta un modelo que las admita. `temperature` es opcional porque
algunos modelos de razonamiento no aceptan ese parámetro; los límites de tokens
también dependen del modelo elegido.

## Claves por proyecto y configuración

Copia `config/providers.example.toml` a un archivo terminado en `.local.toml`,
ignorado por Git. Sustituye los ejemplos de recursos Azure y regiones Bedrock
cuando corresponda. Configura sólo las variables de los proveedores que vas a
usar. Las variables ausentes no impiden cargar el archivo: fallan al intentar usarlas.

```rust
use fenix_core::{ChatRequest, Message, ProviderId};
use fenix_providers::{ProviderClient, ProvidersConfig};

async fn complete_for_project(model: &str) -> Result<(), Box<dyn std::error::Error>> {
    let text = std::fs::read_to_string("config/providers.local.toml")?;
    let config = ProvidersConfig::from_toml(&text)?;
    let client = ProviderClient::new(config.credential_store()?)?;
    let project = Some("demo-project");
    let endpoint = config.provider(ProviderId::OpenAi, project)?;
    let request = ChatRequest::new(model, vec![Message::user("Say hello.")]);
    let response = client.complete(&endpoint, project, &request).await?;
    println!("{}", response.message.text_content());
    Ok(())
}
```

`None` selecciona una solicitud sin proyecto. `Some("demo-project")` exige la
credencial de ese proyecto para el proveedor elegido: nunca recurre a la clave
predeterminada ni a otro proyecto. Cada proyecto puede sobrescribir endpoints
mediante su lista `providers`. Debes pasar el mismo proyecto al resolver el
endpoint y al consultar el modelo. `ProviderClient` es una biblioteca para código
de confianza: la futura aplicación decidirá qué proyectos puede usar cada usuario
y quién puede cambiar los endpoints.

`provider_project` y `organization` se refieren al ámbito nativo del proveedor.
OpenAI admite ambos; Anthropic admite su workspace mediante `provider_project`.
En los demás proveedores, el ámbito lo establece la clave, cuenta o recurso del
proveedor. Los encabezados no admitidos producen un error explícito. Consulta la
[referencia de proveedores](provider-reference.md).

## Herramientas y continuidad de conversaciones

Declara funciones mediante `ToolDefinition`, con nombre, descripción y un objeto
JSON Schema. La respuesta puede contener `ContentBlock::ToolCall` en vez de texto
final. La aplicación valida permisos y argumentos, ejecuta la operación, conserva
el `Message` del asistente sin modificarlo y añade un mensaje `Role::Tool` con
`ContentBlock::ToolResult` y los identificadores correspondientes.

Conserva `Message::provider_data` al guardar o reutilizar mensajes del asistente.
Contiene razonamiento y firmas que determinados proveedores necesitan para
continuar. Modificar el contenido visible sin retirar metadatos obsoletos, o
reutilizarlos en otro protocolo, produce un error. Cambiar de proveedor o modelo
puede exigir reconstruir el historial aunque ambos usen el mismo protocolo.

`FinishReason::Length` significa salida parcial; `ContentFilter` indica contenido
filtrado o rechazado. Los conteos desconocidos permanecen en `None`. La entrada
de Anthropic incluye los tokens de caché reportados, y la salida de Gemini incluye
los tokens de razonamiento reportados. Los conteos no equivalen a precios.

## Transporte

El cliente comparte conexiones y utiliza Rustls con validación de certificados.
Sólo permite HTTP sin TLS en loopback, para Ollama local y las pruebas. Desactiva
`ClientOptions::allow_http_loopback` si necesitas HTTPS en todos los casos. No se
siguen redirecciones y los endpoints no admiten credenciales, consultas ni fragmentos
en la URL.

Los timeouts y tamaños de solicitud/respuesta son configurables. No hay reintentos
automáticos de inferencia. Los errores distinguen autenticación, límites de uso,
timeouts, conexión, estado HTTP, respuesta inválida y tamaños excedidos.
`is_retryable()` orienta al futuro planificador; no autoriza repetir efectos de
herramientas. `RateLimited` conserva `Retry-After` cuando se expresa en segundos.
Descartar el futuro cancela la comunicación local, pero no revierte procesamiento
o consumo que el proveedor ya haya realizado.

## Pruebas locales

```sh
cargo test --workspace --all-targets --locked
cargo test --workspace --doc --locked
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
```

Las pruebas HTTP cubren las 17 integraciones con claves ficticias estáticas y por
proyecto, concurrencia, ámbitos nativos, ciclos de herramientas, timeouts,
redirecciones, respuestas inválidas y límites de tamaño, incluso sin Content-Length.
No realizan llamadas facturables a modelos.

## Prueba real opcional

Configura la variable de la clave del proveedor mediante tu gestor de secretos.
Indica `FENIX_LIVE_PROVIDER` y un modelo o despliegue habilitado en
`FENIX_LIVE_MODEL`. Después ejecuta:

```sh
cargo test -p fenix-providers --test live --locked -- --ignored
```

| Variable opcional | Función |
| --- | --- |
| `FENIX_LIVE_BASE_URL` | Recurso o región obligatorios para Azure y Bedrock; sobrescritura opcional para otros. |
| `FENIX_LIVE_PROVIDER_PROJECT` | Proyecto nativo de OpenAI o workspace de Anthropic. |
| `FENIX_LIVE_ORGANIZATION` | Organización nativa de OpenAI. |
| `FENIX_LIVE_PROJECT` | Probar el ámbito de un proyecto interno de Fénix. |
| `FENIX_LIVE_MAX_OUTPUT_TOKENS` | Presupuesto positivo de salida; puede aumentarse para modelos de razonamiento. |

La prueba envía una solicitud real y comprueba que haya texto visible. Usa la
variable de clave predeterminada del proveedor y no lee el TOML de ejemplo. Ejecútala
por separado para cada cuenta/modelo que necesites comprobar. Una región, endpoint,
cuota o permiso incorrectos pueden provocar errores. La prueba se omite en la batería
predeterminada: una compilación local exitosa no demuestra acceso a cuentas remotas.
