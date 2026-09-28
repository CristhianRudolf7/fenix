---
name: dependency-and-secret-audit
description: Inspect dependency risk and accidental secret exposure using manifests, lockfiles and available scanners.
metadata:
  version: 1.2.0
  category: security
  tags: dependency, and, secret, audit
  aliases: dependencias vulnerables, secretos, token filtrado, supply chain, auditoría, SBOM, SLSA, Sigstore, signed build provenance, procedencia de artefactos
  required_tools: file_read,files_search,command_run
  requires: Authorized repository; vulnerability database access if advisory verification is needed
  effects: local-write
---
# Dependency And Secret Audit

## Scope

Use for supply-chain and credential exposure review. Do not validate discovered credentials by logging into external accounts.

## Inputs

Repository, included history/artifacts, package ecosystems and requested audit depth.

## Workflow

1. Inventory manifests, lockfiles, vendored code and build/download scripts. Identify direct versus transitive dependencies and versions actually resolved.
2. Use installed scanners where available, recording tool version and advisory date. Verify findings against primary advisories and reachable use, not package name alone.
3. Inspect likely secret exposure locations without printing values. Report file/line, secret type and a redacted fingerprint; treat uncertain matches as candidates.
4. Propose the narrowest compatible upgrade and check release notes. For exposed credentials, explain revocation/rotation and derived artifacts; deleting a line does not revoke a token.
5. Apply fixes only within the requested scope, preserve lockfile consistency and run affected checks.

For SBOM, artifact signatures or provenance, load [supply-chain evidence](references/supply-chain-evidence.md). Resolve component identities and verify the expected artifact, signer and source separately. Use vulnerability-management for cross-asset prioritization and remediation tracking.

## Focused follow-up

Keep dependency identity tied to the lockfile version, source and enabled features. A known advisory still needs applicability analysis; an unmaintained package alone is not a confirmed vulnerability. Record unavailable advisory data and avoid printing credential values. Use security-static-analysis for local scanner artifacts and security-finding-validation before claiming exploitability.

## Verification

Confirm the resolved dependency changed and inspect remaining advisory findings. Clearly state whether history, containers or generated bundles were excluded.

## Limits

Do not upload private source to a scanner without permission, blindly force major upgrades, or claim regex scans prove absence of secrets.
