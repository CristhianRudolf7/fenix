# Cryptographic review boundaries

Identify what each primitive protects: password verification, token authenticity, confidentiality, content identification or transport trust. An algorithm name in source is a lead; a nonsecurity checksum has different consequences from a password hash or signature. Trace the actual library mode, parameters, callers and key source before assigning impact.

For authenticated encryption, verify nonce uniqueness under the same key across retries, parallel writers, process restart, backups and cloned instances. Check that authentication failure rejects plaintext use, that associated data binds the intended context/tenant, and that key identifiers cannot select another user's key. Do not replace a maintained protocol/library with custom cryptography during an unrelated fix.

For password storage, check a supported password KDF, salt generation, deployed cost and upgrade strategy against current guidance and workload limits. Distinguish random-token generation from deterministic simulation randomness. Review comparison and error behavior where secrets are verified; a generic text scan cannot establish constant-time execution.

Trace key generation, storage, access, rotation, recovery and destruction. Consider envelope-encryption key hierarchy and whether restored data still has a recoverable authorized key. Environment variables may be exposed by logs, dumps or child processes; their use alone does not establish secure storage.

For certificates or tokens, inspect chain/issuer/audience/name validation and failure paths instead of merely checking that encryption or a signature exists. Reproduce the concrete failure with harmless fixtures and verify normal operation. Document untested properties; do not invent numerical strength grades or false-positive rates.

## Español

Revisa el propósito y uso real de la criptografía, especialmente nonces, autenticación del cifrado y ciclo de vida de claves. Un nombre de algoritmo no demuestra un defecto y una variable de entorno no prueba almacenamiento seguro. Usa bibliotecas mantenidas y evidencia concreta, sin inventar cifras de seguridad.
