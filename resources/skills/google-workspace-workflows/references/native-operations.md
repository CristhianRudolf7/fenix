# Native Google operation guide

All examples below are the `arguments` object for `connector_call` with `action=invoke`, the selected connection ID and the named operation. Discover/describe before execution. No example contains credentials.

## Schemas and service roots

`workspace_schema`: `{"service":"tasks","method_id":"tasks.tasks.insert"}`. Omit `method_id` for a method page; follow `next_offset` with `offset`. Request a referenced type with `schema_name`. Discovery data is fetched without account tokens from fixed Google URLs and cached for six hours. Fetch only the schema needed for the task.

`request` takes `service`, `method` (default GET), `path`, optional `query` and JSON `body`. The result identifies `api_base`: request paths are relative to it, so omit `/v1`, `/v2`, `/v4`, `/gmail/v1` or `/tasks/v1` prefixes already in that base. Never paste a full URL into `path`. Query arrays become repeated parameters. Do not put query strings in the path.

Supported IDs: `drive`, `docs`, `sheets`, `slides`, `gmail`, `calendar`, `tasks`, `people` (Contacts), `forms`, `chat`, `meet`. API enablement, Workspace edition, administrator policy and resource access still apply. Schema availability does not imply those requirements are met.

## Read examples

| Purpose | Operation | Arguments |
| --- | --- | --- |
| Task lists | request | `{"service":"tasks","path":"/users/@me/lists","page_all":true}` |
| List tasks | request | `{"service":"tasks","path":"/lists/LIST_ID/tasks","query":{"showCompleted":false},"page_all":true}` |
| Contacts | request | `{"service":"people","path":"/people/me/connections","query":{"personFields":"names,emailAddresses"},"page_all":true}` |
| Form responses | request | `{"service":"forms","path":"/forms/FORM_ID/responses","page_all":true}` |
| Chat spaces | request | `{"service":"chat","path":"/spaces","page_all":true}` |
| Meet records | request | `{"service":"meet","path":"/conferenceRecords","page_all":true}` |
| Meet transcript list | request | `{"service":"meet","path":"/conferenceRecords/RECORD_ID/transcripts","page_all":true}` |
| Transcript entries | request | `{"service":"meet","path":"/conferenceRecords/RECORD_ID/transcripts/TRANSCRIPT_ID/entries","page_all":true}` |
| Gmail message | mail_read | `{"message_id":"MESSAGE_ID","format":"full"}` |
| Calendar range | calendar_list | `{"calendar_id":"primary","time_min":"2026-10-01T00:00:00Z","time_max":"2026-10-08T00:00:00Z","page_all":true}` |

Substitute actual IDs and the requested dates. Interpret calendar times using the account/user timezone, not the example's UTC dates. `mail_read` returns Gmail's structured payload; MIME parts may contain base64url text, attachments or nested parts. Do not summarize encoded bytes as decoded text.

Google `page_all` reads at most `page_limit` pages (default 5, maximum 10), with an aggregate 3 MiB output budget and 60-second deadline. It returns `pages` without flattening schemas. It follows only `nextPageToken`, never arbitrary URLs, and never repeats writes. Resume only when the stop reason permits it, using `query.pageToken` for `request` or `page_token` for list helpers. Do not loop indefinitely after repeated tokens or provider errors. When selecting fields, include `nextPageToken`.

## Writes, previews and receipts

`request` with `dry_run=true` validates request shape/path locally and returns `executed=false`. It does not call Google or attest that the body satisfies every schema rule. Keep revision guards, explicit field masks and destination IDs in actual mutations. `POST` read APIs such as Calendar free/busy are conservatively subject to write permission in this transport; do not circumvent that policy.

- `mail_compose` accepts explicit `to`, optional `cc`/`bcc`, `subject`, plain `text` and optional attachment paths. It creates a **remote Gmail draft** by default, which requires write permission. `send=true` sends once; use only within the user's authorized scope. `dry_run=true` makes no Gmail call. The sender comes from the connected Gmail profile. HTML/alias selection is not exposed by this helper.
- For a reply, pass `reply_to_message_id` and explicit recipients; omit `subject` to preserve the original. The helper loads original headers and preserves Message-ID, References and Gmail threadId. It never guesses reply-all recipients. Attachments must be existing workspace files (10 maximum, 5 MiB total); absolute paths, credential paths and symlinks are rejected.
- `drive_upload`: `{"source":"reports/report.pdf","folder_id":"FOLDER_ID","mime_type":"application/pdf"}` creates a new file (25 MiB maximum); `name` is optional. Re-read the returned ID and metadata. This helper does not overwrite an existing file or convert formats automatically. After an uncertain failure, search the destination before another upload.
- `drive_download` downloads ordinary files; `drive_export` exports native Google content. Both need a new relative `destination`, preserve existing files and return bytes/hash evidence.

For durable recurrence, use Fénix's existing workflow/schedule tools with explicit steps and source references. Do not start a CLI watcher or create a Google event subscription as a side effect of a one-time read.

## Español

Consulta esquemas por método, usa rutas relativas y sigue la paginación con límites. Los borradores se guardan en Gmail; los envíos exigen autorización. Las previsualizaciones no ejecutan solicitudes ni garantizan permisos. Las subidas crean archivos nuevos y las transcripciones de Meet deben existir previamente.
