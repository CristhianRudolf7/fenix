# Native Google Workspace improvements

Fénix extends its own per-owner connector rather than installing Google Workspace CLI or sharing a terminal login. The implementation uses existing Rust dependencies and the existing encrypted account vault. No gws binary, new hosted service or paid integration subscription is introduced. Google account/API availability, quota and Workspace licensing still apply.

## Account setup and actual readiness

Integrations lets the owner select Drive, Docs, Sheets, Slides, Gmail, Calendar, Tasks, Contacts, Forms, Chat and Meet. A new account defaults to read-only Drive. OAuth presets are constructed from selected services; custom scopes remain an explicit advanced override. The native transport also enforces the selected services and read/write switch. Existing vault records without service selection keep the original six-service surface; editing a modern connection without a services field preserves its previous selection. Selecting a service does not automatically grant permissions or revoke historical Google grants.

Google OAuth retains PKCE, one-use expiring state and refresh support. Reconnecting clears any old-account refresh token before applying new tokens; actual scope strings returned by Google are exposed separately from requested configuration. Consent completion now produces `authorized`, not a successful service check. Legacy Google records labeled connected without per-service evidence are also displayed as authorized. If Google omits scope information, Fénix leaves it unknown instead of reporting requested scopes as verified.

Checks perform harmless reads independently for Drive, Gmail, Calendar, Tasks, Contacts, Chat and Meet, with a 60-second overall deadline. Docs, Sheets, Slides and Forms require a specific resource ID and remain `not_checked` at this stage. Mixed outcomes are visible per service in Integrations. A connection with no completed read probe remains unverified. A successful read probe establishes only that request, never general write access. `partially_checked` is not an error when document-specific verification remains outstanding.

## Agent operations

- `workspace_schema` fetches public Discovery documents from eleven fixed official endpoints without tokens, caches them for six hours and exposes paged method listings, individual methods, request/response schemas and separately retrievable referenced types. Documents are capped at 4 MiB; schema results at 256 KiB. Metadata cannot introduce new hosts or grant permissions. This is inspection, not automatic generation/execution of arbitrary Discovery methods.
- `request` retains fixed service-relative routes. New native roots cover Tasks, People/Contacts, Forms, Chat and Meet. Query arrays become repeated parameters. `dry_run=true` returns the intended URL/method/body without obtaining a token or making a Google request. It validates local transport shape, not every provider schema rule or resource permission. Conservatively, POST read methods still require write-enabled connections.
- Google GET/list pagination follows only `nextPageToken` and returns page envelopes, completeness, stop reason and continuation. Default five, maximum ten pages, 3 MiB aggregate payload and 60 seconds; repeated tokens and errors stop the run. Writes are never automatically paginated or retried. Custom field projections must retain the continuation token.
- `mail_read` supports metadata/full Gmail payloads without marking messages read. `mail_compose` builds MIME with explicit To/CC/BCC, connected-profile sender, plain text and up to ten confined workspace attachments totaling 5 MiB. Default behavior creates a remote draft; only `send=true` sends. Replies load and preserve original Subject, Message-ID, References and Gmail threadId. It does not infer reply-all recipients, choose aliases or expose HTML composition. Local preview performs no Gmail reads or writes.
- `drive_upload` creates a new binary file using multipart upload, with optional folder/name/MIME type and a 25 MiB source limit. Local preview never uploads. Existing download/export operations retain path confinement and byte/hash receipts. Uploads do not silently overwrite, retry or convert documents.
- `calendar_list` supports explicit calendar, start/end range, bounded limit and page continuation. It expands recurring instances and orders by start time; omitting the window does not imply today's agenda.

Use `connector_call action=list`, then `describe`, then `invoke` with a selected account. Token management remains in Integrations. Inspect write receipts and re-read changed resources before claiming success. Previewing a proposed send is not authorization to send it.

## Skills and preservation

One original skill, `google-workspace-workflows`, coordinates cross-service work and Tasks, Contacts and existing Meet records. Ten existing skills now explain native operations: Drive, Docs, Sheets, Slides, email, calendar, meeting notes, forms, team messaging and personal organization. Their recognized stock hashes support upgrades without overwriting customized instructions or changing disabled state. The catalog contains 187 skills.

Meet support retrieves existing conference records, transcripts and entries subject to actual access. It does not join meetings, record calls, generate missing transcripts or bypass Workspace restrictions. Administrative APIs, persistent event subscriptions and an always-running inbox watcher are not part of this change.

## Validation

See [the validation record](../research/google-workspace-validation-2026-09-21.json) for completed checks and limits. Local mock tests cover selected-service scopes, OAuth state/reconnect, fixed endpoints, anonymous/cached Discovery, offline previews, pagination budgets, partial diagnostics, MIME threading, attachment boundaries, upload shape and non-retried sends. UI tests verify service selection payloads, mobile layout and visible partial results. Skill checks cover routing, stock migration and customization preservation.

An explicitly run read-only live check validates the eleven public Discovery documents and all preset scope names. This is not authenticated Google account validation, message delivery or production activation. No user account is connected by building these changes.

## Sources and design decisions

- [Google Workspace CLI](https://github.com/googleworkspace/cli): inspiration for discoverable schemas, bounded pagination, previews and task-oriented helpers. Its README identifies it as not officially supported by Google. Fénix does not import its skills, auth store or runtime code.
- [Google Discovery format](https://developers.google.com/discovery/v1/using): public machine-readable method/schema metadata.
- [Gmail message creation](https://developers.google.com/workspace/gmail/api/guides/sending) and [thread handling](https://developers.google.com/workspace/gmail/api/guides/threads): MIME/base64url requests and reply headers.
- [Google OAuth web-server flow](https://developers.google.com/identity/protocols/oauth2/web-server): authorization codes and returned access/scopes.

The original implementation stays within Fénix's per-owner credentials, task workspace and action policy. Broader API reach comes from explicit service adapters and inspected schemas, not a global shell credential.
