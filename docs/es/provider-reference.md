# Referencia de proveedores

[English](../en/provider-reference.md)

Verificado con documentación oficial el 2026-09-10. Esta referencia describe los endpoints de inferencia elegidos por Fénix, cómo transmiten las claves API y qué significa el ámbito por proyecto. La disponibilidad de modelos y herramientas depende del modelo y de la cuenta seleccionados.

## Endpoints compatibles

`Bearer` significa `Authorization: Bearer <api-key>`. Las variables de entorno siguientes son los valores predeterminados de Fénix; cada perfil de credenciales puede referenciar otra variable. Las URL incluyen el prefijo de la API y excluyen la operación de inferencia.

| ID de proveedor | Protocolo | Raíz predeterminada de la API | Transporte de la clave | Variable de entorno predeterminada |
| --- | --- | --- | --- | --- |
| [`openai`](https://developers.openai.com/api/reference/overview) | Responses | `https://api.openai.com/v1` | Bearer | `OPENAI_API_KEY` |
| [`anthropic`](https://platform.claude.com/docs/en/manage-claude/authentication) | Messages | `https://api.anthropic.com/v1` | `x-api-key` | `ANTHROPIC_API_KEY` |
| [`gemini`](https://ai.google.dev/api/generate-content) | GenerateContent | `https://generativelanguage.googleapis.com/v1beta` | `x-goog-api-key` | `GEMINI_API_KEY` |
| [`openrouter`](https://openrouter.ai/docs/api/reference/authentication) | Chat Completions | `https://openrouter.ai/api/v1` | Bearer | `OPENROUTER_API_KEY` |
| [`deepseek`](https://api-docs.deepseek.com/) | Chat Completions | `https://api.deepseek.com` | Bearer | `DEEPSEEK_API_KEY` |
| [`xai`](https://docs.x.ai/developers/quickstart) | Chat Completions | `https://api.x.ai/v1` | Bearer | `XAI_API_KEY` |
| [`mistral`](https://docs.mistral.ai/api) | Chat Completions | `https://api.mistral.ai/v1` | Bearer | `MISTRAL_API_KEY` |
| [`alibaba`](https://docs.modelstudio.console.alibabacloud.com/en/model-studio/base-url) | Chat Completions | `https://dashscope-intl.aliyuncs.com/compatible-mode/v1` | Bearer | `DASHSCOPE_API_KEY` |
| [`moonshot`](https://platform.moonshot.ai/docs/api/chat) | Chat Completions | `https://api.moonshot.ai/v1` | Bearer | `MOONSHOT_API_KEY` |
| [`zai`](https://docs.z.ai/api-reference/introduction) | Chat Completions | `https://api.z.ai/api/paas/v4` | Bearer | `ZAI_API_KEY` |
| [`minimax`](https://platform.minimax.io/docs/api-reference/text-chat-openai) | Chat Completions | `https://api.minimax.io/v1` | Bearer | `MINIMAX_API_KEY` |
| [`groq`](https://console.groq.com/docs/openai) | Chat Completions | `https://api.groq.com/openai/v1` | Bearer | `GROQ_API_KEY` |
| [`together`](https://docs.together.ai/docs/inference/openai-compatibility) | Chat Completions | `https://api.together.ai/v1` | Bearer | `TOGETHER_API_KEY` |
| [`bedrock`](https://docs.aws.amazon.com/bedrock/latest/userguide/api-keys-use.html) | Converse | Requiere URL regional explícita | Bearer | `AWS_BEARER_TOKEN_BEDROCK` |
| [`azure`](https://learn.microsoft.com/en-us/azure/foundry/foundry-models/concepts/endpoints) | Chat Completions | Requiere URL de recurso explícita | Bearer | `AZURE_API_KEY` |
| [`vertex`](https://docs.cloud.google.com/gemini-enterprise-agent-platform/reference/express-mode/api-reference) | GenerateContent, modo exprés | `https://aiplatform.googleapis.com/v1/publishers/google` | `x-goog-api-key` | `VERTEX_API_KEY` |
| [`ollama`](https://docs.ollama.com/api/openai-compatibility) | Chat Completions | `http://localhost:11434/v1` | Bearer opcional | `OLLAMA_API_KEY` |

El cliente añade `/responses`, `/chat/completions`, `/messages`, `/models/{model}:generateContent` o `/model/{model}/converse` según el protocolo. La raíz de Vertex ya incluye `publishers/google`, formando la ruta del modo exprés. En xAI, la ruta elegida de Chat Completions continúa documentada por separado de su interfaz más reciente Responses. [Chat Completions de xAI](https://docs.x.ai/developers/rest-api-reference/inference/chat-completions).

## Claves estáticas y claves por proyecto

Una clave por proyecto sigue siendo una clave API estática al transmitirse. Su ámbito puede establecerse al emitirla, por el recurso al que se envía o mediante una cabecera documentada. Además, Fénix tiene su propio ámbito por proyecto para elegir qué credencial puede utilizar una tarea. El nombre de un proyecto de Fénix no crea un proyecto en la nube ni cambia los permisos del proveedor.

| Mecanismo de ámbito | Comportamiento de Fénix y requisitos del proveedor |
| --- | --- |
| Proyecto de Fénix | Seleccionar un perfil de credenciales distinto para cada proyecto y proveedor. Usar el perfil predeterminado únicamente en solicitudes sin proyecto. |
| Proyecto / organización de OpenAI | Enviar las cabeceras opcionales `OpenAI-Project` y `OpenAI-Organization`. La credencial ya debe tener acceso a ese ámbito. |
| Workspace de Anthropic | Enviar `anthropic-workspace-id` cuando corresponda. Las claves vinculadas a una identidad con varios workspaces requieren esta cabecera; las limitadas a uno pueden omitirla. |
| Proyecto de Google Cloud para Gemini | Elegir una clave creada para el proyecto de Google Cloud correspondiente. No se añade una cabecera de proyecto inventada. |
| Workspace de Mistral | Elegir la clave emitida para ese workspace. Su asociación se establece al crearla. |
| Proyecto de Together | Elegir la clave emitida dentro del proyecto de Together correspondiente. El proveedor aplica su ámbito. |
| Workspace de Alibaba | Elegir una clave y un endpoint del mismo workspace y región. Un endpoint propio del workspace puede aportar aislamiento nativo. |
| Recurso de Azure / acceso de Bedrock | Elegir el recurso o endpoint regional y su clave autorizada. Un proyecto de Fénix no reduce los permisos que posee la clave. |
| Otros proveedores | Elegir una clave o perfil distinto por proyecto de Fénix. Esta integración no supone una cabecera nativa de selección de proyecto cuando el catálogo no la configura. |

Fuentes del ámbito nativo: [autenticación de OpenAI](https://developers.openai.com/api/reference/overview), [workspaces de Anthropic](https://platform.claude.com/docs/en/manage-claude/authentication), [claves de Gemini](https://ai.google.dev/gemini-api/docs/api-key), [claves de Mistral](https://docs.mistral.ai/admin/identity-access/api-keys) y [autenticación de Together](https://docs.together.ai/docs/api-keys-authentication).

## Detalles de despliegue

- **Anthropic:** Fénix envía `anthropic-version: 2023-06-01` con Messages. La cabecera documentada `x-api-key` continúa siendo válida junto con la autenticación Bearer más reciente. Una credencial de varios workspaces necesita el ID del workspace nativo. [Autenticación](https://platform.claude.com/docs/en/manage-claude/authentication).
- **Gemini:** obtener una clave de autorización actual de Google AI Studio. La documentación anuncia el rechazo de las antiguas claves estándar en septiembre de 2026; las claves de autorización utilizan la misma cabecera `x-goog-api-key`. [Tipos de clave y migración](https://ai.google.dev/gemini-api/docs/api-key).
- **Alibaba:** el endpoint compartido predeterminado es el de Singapur y continúa disponible. Para producción, configurar uno del workspace, por ejemplo `https://<workspace-id>.ap-southeast-1.maas.aliyuncs.com/compatible-mode/v1`, con la clave correspondiente. Las claves dependen de la región. El catálogo utiliza inferencia general; los endpoints de planes por tokens son productos separados. [Endpoints y regiones](https://docs.modelstudio.console.alibabacloud.com/en/model-studio/base-url).
- **Bedrock:** indicar una raíz como `https://bedrock-runtime.us-east-1.amazonaws.com` y una clave API Bearer de Bedrock. Un ID de clave de acceso de AWS y su secreto no sustituyen esa clave; SigV4 es otro mecanismo. La integración usa Converse con un modelo o perfil de inferencia accesible. [Uso de claves de Bedrock](https://docs.aws.amazon.com/bedrock/latest/userguide/api-keys-use.html).
- **Azure / Microsoft Foundry:** indicar `https://<resource>.openai.azure.com/openai/v1` y la clave del recurso. Usar el nombre del despliegue en `model`. La documentación v1 actual envía las claves como Bearer y no exige `api-version`. Esta implementación utiliza esa ruta v1. Las rutas anteriores `/openai/deployments/...` y `/models` tienen contratos distintos. [Endpoints de Foundry](https://learn.microsoft.com/en-us/azure/foundry/foundry-models/concepts/endpoints).
- **Vertex:** esta integración con clave API cubre solamente el modo exprés. El endpoint global omite `/projects/.../locations/...`; no implementa la autenticación IAM/OAuth del servicio estándar. Utilizar una clave del modo exprés aceptada por el servicio. Fénix la envía como cabecera HTTP, sin incluirla en la URL. [API exprés](https://docs.cloud.google.com/gemini-enterprise-agent-platform/reference/express-mode/api-reference), [cabeceras de claves de Google](https://docs.cloud.google.com/docs/authentication/api-keys-use).
- **Ollama:** el servidor local ignora las claves y no proporciona aislamiento entre proyectos mediante credenciales. Una clave opcional sirve cuando un proxy administrado por separado requiere autenticación. El acceso local sin clave debe configurarse explícitamente en Fénix. [Compatibilidad con OpenAI](https://docs.ollama.com/api/openai-compatibility).

Los adaptadores utilizan claves existentes para inferencia. No crean cuentas, emiten claves por proyecto, aprovisionan despliegues, rotan credenciales ni implementan OAuth, federación de identidades o firma IAM.

## Alcance de la verificación

Las pruebas HTTP locales comprueban rutas, cabeceras, conversión de solicitudes y respuestas, aislamiento de credenciales y tratamiento de errores. No demuestran que una cuenta remota tenga saldo, acceso al modelo, permisos correctos o una clave válida. Una comprobación real necesita una clave y un modelo configurados para cada proveedor; solamente una respuesta real satisfactoria confirma el funcionamiento completo de esa cuenta.
