# Consent and credential lifecycle

Inventory issuer/tenant, application/client/service-principal IDs, user/resource owner, delegated versus application permissions, granted scopes and effective resource access. A verified publisher or approved app name does not justify every requested permission. Correlate unexpected consent with audit events, login/session changes, mailbox rules and the user's intended workflow. Missing log retention is unknown; an unfamiliar publisher alone is not proof of compromise.

Use pagination and the provider's actual semantics when enumerating grants. Record omitted scopes or inaccessible accounts. Bind connected accounts to the calling user on the server, including retries, background jobs and caches; the model's account selection is input, not ownership proof.

For authorized rotation, map all consumers, supported overlap and revocation semantics first. Where supported, introduce the new credential, update consumers, verify real operations and revoke the old credential. When compromise requires immediate containment, choose the permitted interruption explicitly rather than automatically retaining a grace period. Never roll back by re-enabling a known-compromised key.

Check derived tokens, sessions, refresh grants, cached credentials and federated identities. A password/client-secret change may leave already issued tokens valid. Verify denial of the retired identity and continuity of permitted users under documented propagation times. Keep emergency access separate from the account being changed and record the remaining validity window without printing secrets.

## Español

Relaciona consentimiento, identidad, permisos y dueño real. Durante rotación actualiza consumidores y verifica revocación de claves, tokens y sesiones según el proveedor. Una clave nueva no elimina accesos derivados; no restaures credenciales comprometidas como rollback.
