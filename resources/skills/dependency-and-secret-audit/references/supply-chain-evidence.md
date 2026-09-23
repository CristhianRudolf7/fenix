# Supply-chain evidence

For CycloneDX/SPDX, record schema/version, generation tool, artifact digest, platform and inventory exclusions. Preserve `bom-ref` or `SPDXID` as the graph identity and package URL/ecosystem/version as package identity. Do not collapse distinct versions or assume every graph reference is a package URL. Traverse nested components and report dangling references or unsupported schema features instead of silently discarding them.

Match advisories to actual ecosystem, resolved version, distro/backport and configuration. Keyword search provides candidates; it does not establish applicability. Preserve pagination, database dates, failed lookups and unscanned components. An unavailable feed is unknown, not an empty vulnerability list. Document VEX claims with issuer, product/version, status, justification and date; verify them against the deployed artifact. Dependency centrality or distance from the root is not proof of reachable vulnerable code.

For signatures/provenance, derive the expected identity independently of the artifact being checked. Resolve a mutable tag to a digest once and verify/deploy that same digest. Check cryptographic signature, trusted issuer/key, exact signer/workflow/ref, attestation subject digest and predicate type. Then verify the expected source repository/commit, builder and relevant build inputs under the selected policy. Broad identity regexes such as `.*` defeat identity constraints. A parse error, absent attestation or mismatched subject must not become `verified: true`.

Use current verifier documentation for the installed version. Record signature verification and policy verification separately: a signed malicious build remains possible, and a signature alone does not establish a SLSA level. See [SLSA artifact verification](https://slsa.dev/spec/v1.2/verifying-artifacts). Do not bootstrap a verifier by executing an unverified download from the artifact producer.

For exposed secrets, record a nonsecret incident identifier and affected consumers without emitting the value. Avoid unsalted hashes of low-entropy secrets in public reports. Rotation must update consumers and revoke the old credential/session according to provider semantics; deleting a repository line or rewriting history does not revoke access.

## Español

Conserva identidades de componentes, versiones y cobertura del SBOM. Un fallo de consulta no significa ausencia de CVE. Verifica digest, firmante, emisor, repositorio y procedencia esperados por separado; una firma válida no certifica seguridad ni nivel SLSA. Mantén secretos fuera de informes y comprueba revocación real.
