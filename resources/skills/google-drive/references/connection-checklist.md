# Google Workspace connection checklist

Use this only when diagnosing a missing or wrong Google connection. It describes prerequisites, not an installed connector.

1. Discover the operations actually exposed by Fénix. A matching skill is not a matching tool. If there are no remote tools, stop remote operations and offer a supplied export or an owner-enabled browser session.
2. Identify the account and operation. Drive file access, document editing, email and calendars can require different APIs and OAuth scopes. Google Cloud IAM access is not automatically access to a person's Workspace files.
3. Prefer the application's normal OAuth flow. The owner completes sign-in, consent and any MFA directly; never paste passwords, access tokens or refresh tokens into chat, skill files or examples.
4. Request only scopes needed for the requested action. A read-only workflow should not silently obtain write or sharing access. A service account sees resources shared with it; do not assume domain-wide delegation.
5. Handle errors by their actual meaning: authentication failure needs reconnecting; forbidden access may reflect permissions or scopes; not found can also conceal inaccessible files. Retry transient failures with the provider's guidance, but inspect state before retrying a write.
6. Verify the account and a harmless read of the specified resource. Report connection readiness separately from successful completion of the requested edit.

Official reference: https://developers.google.com/workspace/drive/api/guides/about-sdk

## Español

Comprueba herramientas, cuenta, APIs y permisos reales. IAM de Google Cloud no equivale a acceso a archivos personales. El usuario autentica desde el flujo normal, sin entregar secretos al chat. La skill no configura OAuth ni amplía permisos por sí sola.
