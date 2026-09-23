# Google Cloud project and identity selection

Load for a Google Cloud deployment. Use the installed leaf-command help or current official documentation for concrete syntax; do not invent a dry-run flag or assume every service supports it.

Resolve these independently from existing configuration and the user's request:

| Identity or target | Why it matters |
| --- | --- |
| CLI account and impersonation | Principal issuing the deployment request |
| Project and region/zone | Resources actually changed; avoid inherited defaults |
| Build identity | Access to sources, images and build secrets |
| Runtime service identity | Permissions the deployed workload uses |
| Application Default Credentials | Credentials client libraries resolve; may differ from CLI login |
| Quota/billing project | Usage attribution; may differ from the resource project |

Prefer existing attached workload identities, federation or short-lived impersonation where supported. Do not create or print long-lived keys to solve an unrelated auth failure. For an application endpoint, check whether the contract requires an audience-bound ID token rather than an API access token. Authentication success does not establish resource authorization.

Use bounded projections and filters for discovery, noting when results are partial. Keep the selected project and location explicit in commands that accept them. A noninteractive flag controls prompting; it does not supply user authorization or make defaults correct.

Inspect the planned resource changes and exposure, then follow existing task authorization. After execution, check asynchronous operation status, deployed artifact/revision, runtime identity and an intended user request. For IAM edits, use identity-access-review to preserve policy conditions and concurrency state.

## Español

Distingue cuenta del CLI, credenciales de bibliotecas, identidad de compilación y servicio, proyecto de recursos y cuota. Fija proyecto y ubicación, verifica sintaxis real y comprueba la revisión desplegada. No amplíes permisos ni generes claves como solución implícita a un error.
