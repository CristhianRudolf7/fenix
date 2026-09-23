# Token and session boundaries

For JWTs, configure expected token type, allowed algorithms, issuer, audience and required temporal claims from the application's trust model. Validate the signature before using claims and reject algorithm/key-type confusion. Resolve `kid` only within trusted key sources; token-supplied `jku`, `x5u` or inline keys must not choose a new trust root or unrestricted network destination. Handle unknown keys, cache refresh, rollover and provider outage without disabling verification. Use access and ID tokens for their intended audiences and purposes.

For OAuth/OIDC, bind state, redirect URI, PKCE verifier and nonce where applicable to the initiating session/provider. Validate exact permitted redirects and issuer before account linking; an equal email alone is insufficient. Consume codes once and prevent replay/mix-up across parallel logins. Do not expose tokens in URLs or treat client-supplied user/tenant fields as authority.

Define refresh rotation/reuse handling, logout, password/MFA recovery and revocation propagation together. Rotating a client secret need not revoke already issued access tokens. Test intended denial after expiry/revocation across normal HTTP, open WebSocket streams and queued jobs; document any designed validity window. Handle cookies with the app's CSRF, SameSite, Secure and HttpOnly requirements; do not assume CORS enforces authorization.

Test valid login, wrong issuer/audience, tampering, missing required claims, expired/future tokens, key rollover, reused callbacks and two simultaneous users. Use controlled fixtures or the provider's test tenant. The [JWT best current practices](https://www.rfc-editor.org/rfc/rfc8725) and [OAuth security BCP](https://www.rfc-editor.org/rfc/rfc9700) are starting points; confirm provider-specific behavior before coding.

## Español

Valida tipo de token, firma, emisor, audiencia y tiempos contra confianza configurada. No dejes que el token elija claves o URLs arbitrarias. Prueba mezcla de sesiones, rotación, reuso y revocación; cambiar un secreto no invalida necesariamente tokens ya emitidos.
