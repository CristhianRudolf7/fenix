# Browser memory and durable waiting

Fénix prefers its lightweight `web_read` tool for public pages whose text is
sufficient. JavaScript, signed-in pages, forms, screenshots and other interactions
still use isolated Chromium. This selection policy neither adds external services
nor silently replaces browser actions with HTTP requests.

## Task ownership

Profiles track their tasks. Completing or cancelling the last task closes its
browser and execution lease. Personal profiles remain on disk; temporary profiles
are deleted only after execution cleanup is confirmed. Other active tasks,
owner-controlled sessions, pending downloads and `integration-*` profiles prevent
automatic closure. Long-lived integrations such as WhatsApp keep their existing
connection behavior.

Waiting for a user answer marks the task as waiting without a model call. Once all
tasks using a browser are waiting, maintenance may suspend it after 60 seconds.
Owner-controlled browsers without tasks use the existing 600-second idle interval.
Maintenance checks every five seconds; active task pages are not killed simply
because they were idle.

## Saving and restoring

With compatible gVisor and Docker, the service saves **process state** to disk and
stops the sandbox. This aims to preserve unsent forms and JavaScript state, beyond
cookies and local storage. A profile alone cannot guarantee those in-memory
values. A stopped browser has no resident processes; the controller, broker, OS
file cache and small metadata still consume memory.

Suspension requires quiet networking, no pending download or dialog, and no
outstanding browser command. CDP and its relays disconnect before checkpointing.
The broker encrypts and authenticates the snapshot in bounded chunks, without
loading the whole image into RAM. Controller metadata records tab, viewport and
task ownership, without copying cookies or form values into the conversation.

Suspended browsers keep their exclusive profile ownership while releasing active
RAM/compute reservations. Resume uses global admission, rotates the attachment
capability, validates the account, image, runtime and writable-file fingerprints,
then reconnects. The agent must observe the page before acting; prior actions and
stale element references are not replayed. Successful resume consumes the snapshot.
Valid checkpoints can survive a broker restart.

Remote sites may expire authentication, CSRF tokens or server sessions while the
browser sleeps. Open connections or runtime incompatibilities may prevent saving.
Unsupported checkpointing or failed disk preflight keeps the original browser
active. If encryption fails after stopping it, the native snapshot is retained in
host-private Docker storage for recovery; this exceptional state is audited and
is not guaranteed to survive broker restart. It is never reported as successful
encrypted persistence.

## Global admission

New executions and resumes wait in the host queue. Admission combines account and
host reservations/concurrency limits with Linux `MemAvailable` and a host reserve.
Waiting starts no Chromium process and consumes no model tokens. Users rotate,
with one pending request per owner per sweep. Cancellation removes the ticket;
unused resume grants return to suspension. Waiting for capacity does not hold the
browser session-map lock, so existing sessions can continue or close.

Tickets are bounded, expire after a disconnected client, and live in memory. After
broker restart, clients must request admission again. This is not a durable task
queue. Capacity control limits concurrency; it does not reduce the memory needed
by arbitrary active pages. Account cgroups and isolation remain enforced.

## Configuration

`<data-dir>/browser.json` adds `hibernate_after_seconds` (default 60, range
10–86400). `idle_seconds` defaults to 600. Legacy `memory_budget_mib` remains
readable for compatibility, but no longer triggers destructive eviction of active
pages: host admission and safe suspension control resources. Standalone browsers
without an execution service cannot hibernate.

Optional host settings in `/etc/fenix/execution.json`:

| Setting | Default | Meaning |
| --- | --- | --- |
| `browser_checkpoints` | `true` | Allow checkpoints when runtime support exists |
| `browser_checkpoint_ttl_seconds` | 86400 | Retention; expired snapshots are deleted |
| `browser_checkpoint_max_bytes` | 2147483648 | Individual snapshot ceiling |
| `browser_checkpoint_account_bytes` | 4294967296 | Per-account snapshot ceiling |
| `browser_checkpoint_total_bytes` | 8589934592 | Combined snapshot ceiling |
| `host_memory_reserve_mib` | 1024 | RAM left for host services |
| `max_runner_memory_mib` | 4096 | Combined runner reservation ceiling |
| `max_executions` | 32 | Active execution ceiling |
| `max_queued_executions` | 128 | Total pending tickets |
| `max_queued_per_account` | 8 | Pending tickets per owner |
| `admission_wait_seconds` | 900 | Maximum ticket lifetime |

Explicit existing configuration overrides these defaults. The root-private
`checkpoint_key` is required. A separate SQLite manifest and encrypted directory
beside execution records hold browser snapshots. Free-disk and quota checks run
before checkpointing; encryption temporarily needs additional disk space.

## Deployment and validation boundary

Docker must expose checkpoint support (`ExperimentalBuild: true`) with the pinned
`fenix-runsc` runtime. Follow the [gVisor checkpoint documentation](https://gvisor.dev/docs/user_guide/checkpoint_restore/)
and qualify the workload on an isolated host. Neither Docker nor production is
restarted automatically; runtime isolation is never weakened as a fallback.
Enabling experimental mode alone does not prove compatibility with Chromium.

On 2026-09-28 Docker checkpoint support was enabled and real gVisor qualification
passed two consecutive Chromium hibernations. An unsent form and JavaScript-only
state survived both, including a broker restart between suspension and restoration.
The browser stopped during each pause, its compute reservation was released, only
encrypted checkpoint files remained at rest, another owner was refused, and no
submission was repeated. Save/restore round trips took 29.78 and 31.83 seconds in
this synthetic workload; these are not guarantees for arbitrary websites.

The deployed Rust controller also automatically suspended a synthetic browser:
its execution cgroup dropped from 302,628,864 bytes (about 289 MiB) to zero resident
accounted bytes and zero tasks. Its snapshot was 153,781,569 bytes (about 147 MiB).
This measurement excludes the controller, broker and OS disk cache.

Production restoration preserved the form text and JavaScript state without
submitting the form. A queued execution waited while the browser held capacity and
was admitted after the browser closed. The temporary browser was removed and the
original idle configuration restored. Authenticated public API checks and all
five systemd services passed after the final controller restart. See the
[validation record](../research/browser-hibernation-validation-2026-09-28.json)
for measurements and test coverage.

Unit tests cover queue fairness/cancellation and failure recovery. The local
Chromium fixture checks safe behavior without checkpoint support, observation before
actions, shared-task closure and persistent profile storage. Repeat qualification
after changing the runtime, image or host.

For qualification, fill an unsent form and set JavaScript-only state on an isolated
host; suspend, confirm the sandbox stops, restart the broker, resume as the same
owner, and verify the form, JS state and absence of duplicated submissions. Also
check site-session expiration, insufficient disk, interrupted restore and access
by another owner. Do not infer arbitrary multi-user capacity from one browser.

```sh
python3 -m unittest discover -s tests/isolation -p 'test_*.py' -q
cargo test -p fenix-browser -p fenix-execution --lib
cargo test -p fenix-browser --lib real_chromium_navigation_responsive_and_persistent_login -- --ignored --test-threads=1
sudo env FENIX_GVISOR_TESTS=1 FENIX_GVISOR_MANIFEST=/etc/fenix/gvisor/manifest.json FENIX_TEST_IMAGE=APPROVED_IMAGE python3 -m unittest discover -s tests/isolation -p test_browser_checkpoint_acceptance.py -v
```

[Español](../es/browser-lifecycle.md)
