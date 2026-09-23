# TLS and certificate evidence

Preserve the failing client's hostname, SNI, address family, port, trust store and time. Inspect the certificate and chain actually served on each affected route/load balancer; a valid file on disk or a successful issuance job does not prove deployment. Distinguish hostname/SAN mismatch, expiry, missing intermediate, untrusted root, protocol negotiation and proxy interception.

For an authorized security assessment, record scanner version, tested endpoints and failed/unsupported checks separately. Supported algorithms are not automatically negotiated by the failing client. HSTS is an HTTP/browser policy, not a TLS handshake property; check it on the intended HTTPS response and do not enable preload/subdomain coverage without considering the affected hosts. Revocation behavior varies by client/CA; confirm current policy rather than assuming every missing OCSP staple is a defect.

For renewal, identify the actual issuer/ACME account, challenge path, DNS/API ownership and consumer reload behavior. Prefer a provider-supported staging/dry-run route when available. Verify issuance, deployment, reload and a new handshake independently; inspect old connections and alternate frontends as needed. Keep private keys out of output and preserve an authorized rollback path. A schedule exists only if it was installed and observed, not because a skill describes it.

## Español

Comprueba el certificado realmente servido con SNI y confianza del cliente original. Separa emisión, despliegue, recarga y nueva conexión. No confundas HSTS con TLS ni un fallo de escáner con éxito. Verifica renovación sin mostrar claves ni desactivar la validación.
