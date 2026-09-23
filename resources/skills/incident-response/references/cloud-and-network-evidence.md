# Cloud and network evidence

For cloud incidents, identify account/project/tenant, region, resource IDs and the exact principal including role-assumption or federation chains. Preserve control-plane and available data-plane logs, grant/key changes, resource metadata and collection window before ephemeral resources disappear. Distinguish disabled logging, retention expiration, pagination gaps and no matching events. Record snapshot scope and consistency; a disk snapshot is not a RAM snapshot.

Containment changes identity and network state. Select the narrow authorized action with its expected interruption and verify it actually took effect. Revoking a long-lived key may leave temporary sessions active; changing a security group may not end every established connection. Preserve policy concurrency tokens and unrelated bindings. A clean health check proves service behavior, not that attacker access is gone. Retest access/session denial and intended recovery separately.

For supplied PCAP/flow data, record capture point, time/clock uncertainty, snap length, packet drops, truncation and filters. Distinguish display filters from capture filters and validate field names against the installed parser. Reassemble streams before interpreting application messages; separate retransmissions from repeated actions. NAT, proxies and address reuse complicate attribution. Encrypted payloads without suitable keys remain unavailable even when metadata is readable.

Regular beacons, unusual ports, domain entropy and reputation hits are leads that need host/process, DNS or identity corroboration and benign alternatives. Keep indicator source, confidence, observed validity interval and expiration. Shared CDN/cloud addresses should not trigger broad blocking by default. Do not replay captures or fetch extracted URLs merely to inspect them.

## Español

Conserva identidad, región, logs y límites de adquisición. Verifica contención y recuperación por separado; revocar claves no siempre elimina sesiones. En PCAP registra pérdidas, filtros y reloj, reconstruye flujos y evita atribuir ataques solo por periodicidad, puertos o reputación.
