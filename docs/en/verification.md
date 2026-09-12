# Initial application verification

[Español](../es/verification.md)

Verified on Linux during the initial implementation, 2026-09-12. No credentials
or personal runtime data are included in this document or the repository.

## Automated coverage

The final Rust run passed 75 unit/integration tests and two documentation examples;
all four Chromium scenarios passed. Rust formatting, Clippy with warnings denied,
frontend formatting/syntax, shell syntax and desktop-entry validation also passed.

The workspace suite covers the existing 17 provider adapters, default and exact
project credentials, five native/compatible protocol round trips, reasoning
metadata, error classification and HTTP limits. New application tests exercise:

- FIFO activation and separate context order, including a new request arriving
  between an earlier task's tool call and result.
- Mandatory approval, changed-argument rejection, denied effects, cancellation
  while waiting and clarification replies bypassing queued unrelated messages.
- Private configuration output, invalid credential setup, loop prevention and
  retention of original evidence after failure.
- Memory deduplication, accented lexical search, corrections, deletion and output
  recovery by pages.
- File path escapes, protected paths, non-overwriting creation, exact edits,
  commands outside a workspace, redacted output, process ownership and timeouts.
- Cron in Peru, reminder delivery, duplicate schedules, overlap protection,
  restart interruption and skipped offline executions.
- Manual skill preservation, versions and enabled states; compaction of complete
  tasks while retaining original history and tool pairs.
- Telegram pairing, owner isolation and text-only input; loopback host/origin and
  HTTP content-type checks.

The four Chromium scenarios use the actual Rust service and a local model fixture:
welcome/real-inference validation, chat with textual approval and FIFO, memory/
skills/schedule/activity panels, and persistent themes/mobile layout. They check
that first use is white even when the OS requests dark mode and that saved keys
are not returned to the browser. Screenshots were inspected in light desktop and
dark mobile layouts. Browser fixtures do not establish live Telegram connectivity.

## Real Luna requests

The authorized OpenAI key was located in a sibling project, read privately and
used for `gpt-5.6-luna` with `reasoning.effort: low`. A direct minimal Responses call
returned HTTP 200. The application's own setup then verified an inference, and
`scripts/smoke-live.py` completed these five scenarios against OpenAI:

| Scenario | Model calls | Executed tools | Result |
| --- | --- | --- | --- |
| Save explicit memory | 5 | `skill_load`, `memory_search`, `memory_manage` | Stored, deduplicated and read back with source metadata. |
| Retrieve memory | 2 | `memory_search` | Retrieved the expected fictitious preference. |
| Create and verify a file | 5 | `skill_load`, `files_list`, `file_write`, `file_read` | One textual approval; exact final file bytes verified outside the model. |
| Command outside the file workspace | 3 | `command_run`, `command_status` | One textual approval; expected output and exit code 0 verified. |
| Create a recurring reminder | 3 | `schedule_manage` | The persisted reminder, interval and next execution were checked. |

No tool errors occurred in those five runs. The report is generated at
`target/verification/live-smoke.json` and is intentionally ignored by Git. The
test's temporary database, key and files were removed after it finished. The
script remains opt-in; CI does not send paid model requests.

Live account access was checked **only for OpenAI Luna**. Other providers have
local protocol/authentication tests; their account permissions, credits and model
availability must be validated during setup with the owner's own credentials.
No Telegram bot token was supplied, so Telegram's remote connection and delivery
were not exercised. The Linux desktop installer is supplied with shell/static
checks; the implementation session did not install or enable a user service.

This is an initial functional application. It is not a benchmark score, a claim of
days-long autonomous reliability, an exhaustive security audit, or proof that
every model will make correct tool decisions.
