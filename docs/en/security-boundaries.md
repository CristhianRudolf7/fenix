# Secret, destination and instruction boundaries

These protections run in the engine and execution service, without extra LLM calls.
They complement account isolation and approvals; they are not a replacement for them.

## Secrets at the model and persistence boundaries

`fenix-core::safety::Redactor` is shared by the runtime and provider client. Registered
credentials are masked in plain text, base64, URL-safe base64, hexadecimal and common
percent-encoded forms. Recognizable provider tokens, labeled passwords/authorization,
private-key blocks and sensitive JSON fields are also masked. This is deterministic
detection, not a guarantee that every unknown or transformed secret can be recognized.

Every provider request passes through the redactor, including background memory and
planning requests. Authentication headers are built separately from the credential
store; masking a prompt does not remove the API authentication. Provider previews
buffer incomplete tokens and sensitive lines/blocks so splitting a credential across
chunks does not expose it. Buffered sensitive terminal output over 64 KiB stops the
process with an explicit incomplete-log error rather than releasing the remainder.

Opaque provider replay may contain signatures or encrypted reasoning. If redaction
would change it, the client rejects it with a static error instead of corrupting or
forwarding it. Such a response does not execute tools. Existing runtime redaction
continues to cover event journals, previews and errors. Memory admission additionally
masks recognizable secrets even for direct storage callers. This does not retroactively
rewrite all historical files or databases and is not encryption at rest.

## Outbound destinations

The host execution configuration supports `network_hosts` globally and per account:

```json
{
  "network_hosts": {
    "development": ["github.com", "registry.npmjs.org"],
    "web": ["example.org"]
  },
  "accounts": {
    "<existing-account-id>": {
      "network_hosts": {"web": ["example.org"]}
    }
  }
}
```

Merge this fragment into the existing operator-owned configuration; it is not a complete
service configuration. Account lists intersect global lists. An omitted profile keeps
the existing public HTTP(S) policy; an empty list denies all destinations. Names must
be exact lowercase ASCII hosts, with no wildcard, URL, port or path. Changes apply to
new runners after restarting the execution service. A task cannot set or widen these
lists; offline execution remains offline.

Before starting a restricted runner, the service resolves and freezes public IPv4
destinations. It rejects private/reserved addresses and configured host public IPs.
The host firewall allows only those IPs on TCP 80/443, still rejects host/private
destinations first, and removes DNS egress. Static host mappings and a disabled DNS
forwarder prevent the guest from bypassing the policy through Docker's resolver.
The monitor verifies the same frozen policy; missing or changed protection stops work.
This applies to the existing OCI, gVisor and microVM Docker adapters.

This is **IP destination isolation**, not an HTTP path or TLS/SNI authorization proxy:
multiple services sharing an approved CDN IP remain indistinguishable to the firewall.
For untrusted helpers needing strict remote operations, use offline execution and the
existing credential broker with fixed provider/action endpoints. Explicitly list hosts
required by redirects and dependencies; the system does not grant them automatically.

The native web reader separately accepts `allowed_hosts` in the owner's protected
`web-search.json`. `null` preserves public-web access; `[]` denies network reads. Each
request, robots lookup and redirect is checked before DNS or a socket is opened.
Existing public-IP validation and DNS pinning still apply. This is an outbound policy,
not deletion of previously indexed documents. Browser/shell traffic is governed by the
host execution policy, not by this reader-specific setting.

## Untrusted content and persistent instructions

Explicit authority/permission override and credential-export directives are flagged.
Simple immediately negated rules such as “Never bypass approvals” are accepted.
These patterns are advisory signals: quotations can match and adversarial paraphrases
can evade them. Files remain readable as reference data. No matching phrase grants
permissions, bypasses account boundaries or authorizes a tool.

- Agent file writes, edits and patch moves into `AGENTS.md`, `SOUL.md`, `IDENTITY.md`,
  `SYSTEM.md`, `USER.md`, `MEMORY.md`, `HEARTBEAT.md`, `TOOLS.md` and `SKILL.md` reject
  risky directives before applying changes. Multi-file patches preflight all changes.
- Repository and owner instruction files are screened again when loaded. Risky sources
  are replaced in the instruction context with a fixed notice; originals and hashes
  remain available for inspection. This also catches files written through shell tools.
- Skill tool results explicitly identify their reference-only trust and risk signal.
  Skill text cannot expand the task's tool permissions or execution profile.
- Risky memories remain unpinned candidates, even after repeated observations or an
  attempted explicit promotion. They cannot supersede a confirmed fact. Automatic
  context packs exclude risky records, including legacy records and entity cards.
  Original research text can still be inspected as ordinary reference material.
- A mixed-source compaction checkpoint cannot serve as evidence for a confirmed memory.
  The agent must use an original owner message; exact quotes and task/project scope
  are still validated. Scoped summaries are screened before instruction-context use;
  active-turn summaries retain their unverified-data role and checkpoint references.

There is no new model-based security reviewer or automatic rewriting of instructions.
Legitimate security examples belong in reference documents, with a short safe instruction
linking to them. Detection does not establish semantic truth: existing approvals,
provenance, scoped storage, sandboxing and the broker remain the enforcement boundaries.

## Validation

Local tests cover encoded and split secrets, provider authentication separation,
opaque replay rejection, memory promotion, protected file/patch preflight, host-policy
intersection, DNS pinning, redirect denial and firewall rule construction. Infrastructure
tests requiring live Docker/gVisor/Kata are opt-in. No production firewall was changed
by this implementation, and passing local tests is not a live sandbox penetration test.
