# Skill marketplace and Fénix Sentinel

Fénix Sentinel is an original Rust publication reviewer. It does not depend on or embed NVIDIA SkillSpector. It combines local heuristics with Nemotron, with no tools exposed to the reviewing model. Private skill creation, editing and normal use do not trigger these publication reviews.

## User workflow

The Skills page includes My skills, Marketplace and Publications. A user requests publication, selects a category, and receives a queued review. The marketplace only shows approved immutable snapshots. Users can search, filter categories, preview instructions and install a new private copy. Existing local folders are never overwritten. Withdrawing a publication cancels pending versions too. Already-installed copies remain with their recipients; withdrawal is not remote deletion.

A public version is a snapshot, not a live link to the author's directory. Editing local files never changes public content. Use “Review latest changes” to submit a new version. The previous approved version remains public until a replacement passes review or the owner withdraws it. Category and skill name are public; account email, source snapshots and full review reports are not exposed to other owners. Public bundle downloads require an authenticated account.

## Review and corrections

Sentinel inspects complete UTF-8 text bundles, including references and scripts. Current limits are 128 files, 128 KiB total, 32,000 bytes for SKILL.md, 12 path components, and 256 filesystem entries during export. Links, hidden paths, special files, binary/opaque content and archives are rejected rather than omitted or marked safe. This first version does not query a CVE database, unpack Office archives, provide YARA/AST analysis or establish signature trust.

Local rules cover instruction overrides, invisible control characters, possible credentials, remote/dynamic execution, unsafe deserialization, sensitive reads, external transfer, internal network access, host privileges, destructive commands, persistence, memory changes, unsafe output, model configuration, dependency installation and agent configuration access. They are conservative signals, not proofs of malicious intent. Suspected credentials block model transmission and require owner action.

Nemotron identifies legitimate capabilities with quotations and reviews all supplied material as untrusted data. If it identifies a problem, it proposes unique, anchored edits to a copy. Files cannot be added or removed. A second security review and a separate capability comparison assess the corrected copy. Deterministic checks preserve file paths, executable flags, frontmatter, Markdown headings, fenced code, inline code and links. Changes to executable code remain proposals requiring regression validation; the service never executes untrusted scripts or fabricates test success.

The private original is never rewritten. Approved prose corrections are published as a separate reviewed snapshot, with an owner-visible diff and report. Incomplete analysis, malformed model output, lost capabilities or uncertain functional preservation prevent publication. Semantic comparison cannot mathematically prove behavioral equivalence; no “zero vulnerabilities” guarantee is claimed. Unclear cases remain pending for owner review and a corrected resubmission, without an administrator bypass endpoint.

## Hosting and credentials

The authenticated gateway owns the shared SQLite database in `skill-exchange/market.sqlite` beside its account registry. Keep this directory outside tenant mounts, with mode 0700. Tenant identity comes exclusively from the authenticated session. Local single-user operation instead uses `skill-exchange.sqlite` in its runtime data directory and reuses the Nemotron conversation connection.

For hosted operation configure `FENIX_SENTINEL_CONFIG` with an absolute path to a host-only 0600 JSON file:

```json
{
  "model": "nemotron-3-5-lightning-30b",
  "base_url": "https://api.runinfra.ai/v1",
  "api_key": "<operator-provisioned key>"
}
```

If the environment variable is omitted, the gateway uses `skill-exchange/sentinel.json` beside its registry when present. There is no credential-management endpoint for tenants. The hosted reviewer uses the OpenAI-compatible adapter and accepts only a Nemotron model identifier. Missing credentials cause an incomplete review, never approval. Inputs and model responses have bounded sizes and deadlines. Provider diagnostics are not stored in user-visible reports. Reported token usage is retained for completed review passes. Provider charges may still occur if a request times out or is interrupted.

## Abuse resistance and recovery

The database atomically deduplicates owner + skill name + full bundle hash + policy version. The hash covers every file and executable flag. Repeated clicks return the existing review. Different-content submissions are limited to five per hour, twenty per day and two queued/reviewing versions per owner; the shared queue holds at most 64 jobs. All exchange routes have a per-account request limit of 120 per minute. These defaults are service policy, independent of agent task token budgets.

Two workers process the durable queue. One review uses at most four model calls (assessment, repair, reassessment, preservation check), with 150-second provider deadlines and a ten-minute job deadline. Retries are bounded to two worker attempts per content version. Retrying a failed review requires a one-minute cooldown. Interrupted work is recovered on restart; an incomplete attempt never publishes a result. A new version supersedes older pending work, and completion updates are conditional so cancelled work cannot publish late.

This queue is intended for one active gateway process per database. Run replicas against separate stores only if you supply a coordinated shared queue and lease design. UI controls are convenience; authorization, idempotency, request throttling, queue limits and publication checks live in the backend. Keep infrastructure-level connection/body/rate limits and existing tenant isolation in place.

## Validation

Automated tests use a local mocked Nemotron endpoint to check the actual review/repair protocol, evidence validation, incomplete responses, credential exclusion, immutable public snapshots, cross-owner access, deduplication, quotas, late completion after withdrawal and version ordering. Browser tests cover publication controls, filtering, reports and safe private installation. Mock tests establish workflow behavior; they do not establish Nemotron's real vulnerability-detection accuracy. Measure that separately on a labeled benign/adversarial corpus before making accuracy claims.

Two live smoke checks also passed with the configured `nemotron-3-5-lightning-30b`: a benign publication review and a four-call injection-removal flow. The latter removed the injected instruction while retaining the report capability and leaving the original unchanged. These synthetic checks demonstrate connectivity and the repair protocol, not general security accuracy.
