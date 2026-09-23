# Mail and payment-fraud evidence

Preserve the raw message and trusted gateway metadata. Separate display name, envelope sender, visible From, Reply-To and the actual domain, including internationalized/confusable forms. Read Received and Authentication-Results relative to the trusted receiving boundary; an attacker can insert earlier-looking headers. SPF authenticates an envelope path, DKIM a signed domain and DMARC alignment; none proves a payment request is legitimate. Forwarding and mailing-list transformations can explain failures, while a compromised legitimate account can pass all checks.

For QR codes, decode locally to text and inspect the destination without opening it. Preserve redirects/tracking parameters as evidence and do not automatically visit a live destination, submit a URL to a public scanner or enter credentials. A shortened link that cannot be safely resolved remains uncertain.

Business email compromise can contain no link or attachment. Compare payment-detail changes, new beneficiaries, invoice amounts and unusual urgency with an independently known vendor/channel. A familiar thread, voice or authenticated sender can be compromised. Recommend verification using an established contact; contact the third party only when authorized. Do not execute a transfer or change vendor records to test authenticity.

If account compromise is supported and relevant logs are available, inspect forwarding, inbox hiding rules, delegation, OAuth consent and session changes within scope. Preserve original evidence before remediation and separate suspicious indicators from confirmed compromise. Route attachment internals to security-artifact-triage and containment to incident-response.

## Español

Valida cabeceras desde el servidor confiable y distingue autenticación de legitimidad. Un fraude puede llegar desde una cuenta real y sin enlaces. Decodifica QR sin navegar y verifica cambios bancarios por canales ya conocidos; no contactes terceros ni alteres pagos sin autorización.
