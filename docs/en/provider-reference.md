# Provider reference

[Español](../es/provider-reference.md)

Verified against official provider documentation on 2026-09-10. This reference describes the inference endpoints selected by Fénix, their API-key transport, and the meaning of project scope. Model availability and tool support depend on the selected model and account.

## Supported endpoints

`Bearer` means `Authorization: Bearer <api-key>`. The environment variable names below are Fénix defaults; a credential profile can reference a different environment variable. Base URLs include the API prefix, but exclude the inference operation.

| Provider ID | Protocol | Default API root | Key transport | Default environment variable |
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
| [`bedrock`](https://docs.aws.amazon.com/bedrock/latest/userguide/api-keys-use.html) | Converse | Explicit regional URL required | Bearer | `AWS_BEARER_TOKEN_BEDROCK` |
| [`azure`](https://learn.microsoft.com/en-us/azure/foundry/foundry-models/concepts/endpoints) | Chat Completions | Explicit resource URL required | Bearer | `AZURE_API_KEY` |
| [`vertex`](https://docs.cloud.google.com/gemini-enterprise-agent-platform/reference/express-mode/api-reference) | GenerateContent, express mode | `https://aiplatform.googleapis.com/v1/publishers/google` | `x-goog-api-key` | `VERTEX_API_KEY` |
| [`ollama`](https://docs.ollama.com/api/openai-compatibility) | Chat Completions | `http://localhost:11434/v1` | Optional Bearer | `OLLAMA_API_KEY` |

The client appends `/responses`, `/chat/completions`, `/messages`, `/models/{model}:generateContent`, or `/model/{model}/converse` according to the protocol. The Vertex root already includes `publishers/google`, producing the express-mode model path. For xAI, the selected Chat Completions route remains documented separately from its newer Responses interface. [xAI Chat Completions](https://docs.x.ai/developers/rest-api-reference/inference/chat-completions).

## Static keys and project keys

A project key is still a static API key on the wire. Its scope can be established when the provider issues it, through the selected resource endpoint, or through a documented request header. Fénix also has its own project scope for choosing which credential a task may use. A Fénix project name does not create a cloud project or change the permissions granted by a provider.

| Scope mechanism | Fénix behavior and provider requirements |
| --- | --- |
| Fénix project | Select a separate credential profile for each project and provider. Use a default profile only for requests without a project. |
| OpenAI project / organization | Send optional `OpenAI-Project` and `OpenAI-Organization` headers. The credential must already have access to that scope. |
| Anthropic workspace | Send optional `anthropic-workspace-id`. Identity-linked keys that span workspaces require this header; keys already bound to one workspace can omit it. |
| Gemini Cloud project | Select a key created for the intended Google Cloud project. No synthetic project header is added. |
| Mistral workspace | Select the key issued for the workspace. The workspace association is established when the key is created. |
| Together project | Select the key issued inside the intended Together project. The provider enforces the key's project scope. |
| Alibaba workspace | Select a key and endpoint for the same workspace and region. A workspace-specific endpoint can provide native isolation. |
| Azure resource / Bedrock access | Select the resource or regional endpoint and its authorized key. A Fénix project name does not narrow the provider key's permissions. |
| Other providers | Select a distinct key/profile per Fénix project. This integration does not claim a native project-selection header where none is configured in the catalog. |

Sources for native key scope: [OpenAI authentication](https://developers.openai.com/api/reference/overview), [Anthropic workspaces](https://platform.claude.com/docs/en/manage-claude/authentication), [Gemini keys](https://ai.google.dev/gemini-api/docs/api-key), [Mistral keys](https://docs.mistral.ai/admin/identity-access/api-keys), and [Together authentication](https://docs.together.ai/docs/api-keys-authentication).

## Deployment details

- **Anthropic:** Fénix sends `anthropic-version: 2023-06-01` with the Messages API. The documented `x-api-key` header remains supported alongside the newer bearer authentication form. A multi-workspace credential needs its native workspace ID configured. [Authentication](https://platform.claude.com/docs/en/manage-claude/authentication).
- **Gemini:** obtain a current authorization key from Google AI Studio. Google's documentation announces rejection of older standard keys in September 2026; authorization keys use the same `x-goog-api-key` header. [Key types and migration](https://ai.google.dev/gemini-api/docs/api-key).
- **Alibaba:** the default shared DashScope endpoint is for Singapore and remains supported. For production, configure a matching workspace endpoint such as `https://<workspace-id>.ap-southeast-1.maas.aliyuncs.com/compatible-mode/v1`. Keys are region-specific. The catalog uses the general inference endpoint; token-plan endpoints are separate products. [Endpoint and region reference](https://docs.modelstudio.console.alibabacloud.com/en/model-studio/base-url).
- **Bedrock:** supply an API root such as `https://bedrock-runtime.us-east-1.amazonaws.com` and an actual Bedrock bearer API key. An AWS access-key ID and secret pair cannot be substituted for this key; SigV4 is a different authentication mechanism. This integration uses Converse with an accessible model ID or inference profile. [Using a Bedrock API key](https://docs.aws.amazon.com/bedrock/latest/userguide/api-keys-use.html).
- **Azure / Microsoft Foundry:** supply `https://<resource>.openai.azure.com/openai/v1` and the resource API key. Use the deployment name as `model`. Current v1 documentation explicitly sends resource keys as Bearer credentials and does not require an `api-version` query parameter. This implementation targets that v1 route. Legacy `/openai/deployments/...` and the former `/models` inference route have different contracts. [Foundry endpoints](https://learn.microsoft.com/en-us/azure/foundry/foundry-models/concepts/endpoints).
- **Vertex:** this API-key integration targets express mode only. Its global endpoint omits `/projects/.../locations/...`; it is not a standard Vertex IAM/OAuth integration. Use an express-mode key accepted by that service. Fénix sends the key in the HTTP header rather than a URL query parameter. [Express API](https://docs.cloud.google.com/gemini-enterprise-agent-platform/reference/express-mode/api-reference), [Google API-key headers](https://docs.cloud.google.com/docs/authentication/api-keys-use).
- **Ollama:** the local server ignores API keys and provides no credential-based project isolation by itself. An optional key is useful when a separately managed proxy requires authentication. Local access without a key must be explicit in Fénix configuration. [OpenAI compatibility](https://docs.ollama.com/api/openai-compatibility).

These adapters use existing keys for inference. They do not create provider accounts, issue project keys, provision deployments, rotate keys, or implement OAuth, workload federation, and IAM signing.

## Verification boundary

Automated local HTTP tests validate the library's request paths, headers, request/response conversion, credential isolation, and failure handling. Such tests do not prove that a remote account has credit, model access, valid permissions, or a usable key. A live inference check requires a configured key and model for each provider; only an actual successful response establishes that account's end-to-end operation.
