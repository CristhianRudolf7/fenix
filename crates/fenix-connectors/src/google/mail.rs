use super::*;
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use lettre::{
    Message,
    message::{Attachment, Mailbox, MultiPart, SinglePart, header::ContentType},
};

pub(crate) fn mail_inputs() -> Value {
    json!({"to":"Required array of explicit recipient addresses","cc":"Optional address array","bcc":"Optional address array","subject":"Required for a new message; replies preserve the original subject","text":"Required plain-text body","attachments":"Optional array of existing workspace-relative file paths; at most 10 files and 5 MiB total","reply_to_message_id":"Optional Gmail message ID; preserves Message-ID/References/threadId, never infers recipients","send":"Default false creates a Gmail draft; true sends once within user authorization","dry_run":"Preview recipients and attachment sizes locally; performs no provider calls"})
}
fn header_value(message: &Value, name: &str) -> Result<Option<String>> {
    let values = message["payload"]["headers"]
        .as_array()
        .context("Gmail metadata has no headers")?;
    let matches: Vec<_> = values
        .iter()
        .filter(|v| {
            v["name"]
                .as_str()
                .is_some_and(|n| n.eq_ignore_ascii_case(name))
        })
        .collect();
    ensure!(matches.len() <= 1, "Ambiguous reply header");
    matches
        .first()
        .map(|v| {
            let text = v["value"].as_str().context("Invalid reply header")?;
            ensure!(
                text.len() <= 12000 && !text.contains(['\r', '\n', '\0']),
                "Unsafe reply header"
            );
            Ok(text.to_owned())
        })
        .transpose()
}
fn addresses(args: &Value, key: &str, required: bool) -> Result<Vec<Mailbox>> {
    let Some(value) = args.get(key) else {
        ensure!(!required, "Explicit recipient list required");
        return Ok(Vec::new());
    };
    let values = value
        .as_array()
        .context("Recipients must be arrays of email addresses")?;
    ensure!(
        values.len() <= 50 && (!required || !values.is_empty()),
        "Use 1–50 To recipients and at most 50 CC/BCC recipients"
    );
    values
        .iter()
        .map(|v| {
            let s = v.as_str().context("Recipient must be a string")?;
            ensure!(
                s.len() <= 320 && !s.contains(['\r', '\n', '\0']),
                "Invalid recipient"
            );
            Ok(s.parse()?)
        })
        .collect()
}
impl Connectors {
    pub(crate) async fn google_compose(
        &self,
        c: Connection,
        args: &Value,
        workspace: &Path,
    ) -> Result<Value> {
        ensure_selected(&c, "gmail")?;
        for key in ["send", "dry_run"] {
            if let Some(value) = args.get(key) {
                ensure!(value.is_boolean(), "{key} must be a boolean");
            }
        }
        let preview = args["dry_run"] == true;
        ensure!(
            c.allow_write || preview,
            "Enable write access before creating drafts or sending mail"
        );
        let to = addresses(args, "to", true)?;
        let cc = addresses(args, "cc", false)?;
        let bcc = addresses(args, "bcc", false)?;
        let text = args["text"].as_str().context("Plain-text body required")?;
        ensure!(text.len() <= 512 * 1024, "Message body exceeds 512 KiB");
        let reply = args
            .get("reply_to_message_id")
            .map(|value| {
                value
                    .as_str()
                    .context("reply_to_message_id must be a string")
            })
            .transpose()?;
        if let Some(id) = reply {
            ensure!(
                !id.is_empty()
                    && id.len() <= 128
                    && id
                        .bytes()
                        .all(|b| b.is_ascii_alphanumeric() || b"-_".contains(&b)),
                "Invalid reply message ID"
            );
        }
        let mut subject = args
            .get("subject")
            .map(|value| value.as_str().context("subject must be a string"))
            .transpose()?
            .unwrap_or("")
            .to_owned();
        ensure!(
            reply.is_some() || !subject.is_empty(),
            "Subject required for a new message"
        );
        ensure!(
            subject.len() <= 1000 && !subject.contains(['\r', '\n', '\0']),
            "Invalid subject"
        );
        let mut attachments = Vec::new();
        let mut sizes = Vec::new();
        let mut total = 0;
        if let Some(paths) = args.get("attachments") {
            let paths = paths.as_array().context("attachments must be an array")?;
            ensure!(paths.len() <= 10, "At most 10 attachments are supported");
            for value in paths {
                let relative = value.as_str().context("Attachment path must be a string")?;
                let path = crate::database::workspace_path(workspace, relative, false)?;
                let bytes = fenix_execution::fs::read(&path, 5 * 1024 * 1024)?;
                total += bytes.len();
                ensure!(total <= 5 * 1024 * 1024, "Attachments exceed 5 MiB total");
                let name = path
                    .file_name()
                    .and_then(|v| v.to_str())
                    .context("Invalid attachment filename")?;
                ensure!(
                    !name.contains(['\r', '\n', '\0']),
                    "Invalid attachment filename"
                );
                sizes.push(json!({"path":relative,"name":name,"bytes":bytes.len()}));
                attachments.push((name.to_owned(), bytes));
            }
        }
        if preview {
            return Ok(
                json!({"dry_run":true,"executed":false,"send":args["send"] == true,"to":args["to"],"cc":args["cc"],"bcc":args["bcc"],"subject":subject,"attachments":sizes,"reply_to_message_id":reply,"note":"Local preview; sender identity and original reply headers will be read from Gmail during execution. No Gmail request was made."}),
            );
        }
        let profile = self
            .http_call(
                c.clone(),
                "request",
                &json!({"service":"gmail","path":"/users/me/profile"}),
                workspace,
            )
            .await?;
        let from: Mailbox = profile["emailAddress"]
            .as_str()
            .context("Gmail profile has no email address")?
            .parse()?;
        let mut builder = Message::builder().from(from).keep_bcc();
        for address in to {
            builder = builder.to(address);
        }
        for address in cc {
            builder = builder.cc(address);
        }
        for address in bcc {
            builder = builder.bcc(address);
        }
        let mut thread = None;
        if let Some(id) = reply {
            let original = self
                .http_call(
                    c.clone(),
                    "mail_read",
                    &json!({"message_id":id,"format":"metadata"}),
                    workspace,
                )
                .await?;
            let original_subject = header_value(&original, "Subject")?.unwrap_or_default();
            ensure!(
                subject.is_empty() || subject == original_subject,
                "Reply subject must match the original; omit subject to preserve it"
            );
            subject = original_subject;
            let message_id = header_value(&original, "Message-ID")?
                .context("Original message has no Message-ID; cannot safely preserve the thread")?;
            let references = header_value(&original, "References")?.unwrap_or_default();
            builder = builder
                .in_reply_to(message_id.clone())
                .references(format!("{references} {message_id}").trim().to_owned());
            thread = Some(
                original["threadId"]
                    .as_str()
                    .context("Original message has no thread ID")?
                    .to_owned(),
            );
        }
        builder = builder.subject(subject);
        let mut parts = MultiPart::mixed().singlepart(SinglePart::plain(text.to_owned()));
        for (name, bytes) in attachments {
            parts = parts.singlepart(
                Attachment::new(name).body(bytes, ContentType::parse("application/octet-stream")?),
            );
        }
        let raw = URL_SAFE_NO_PAD.encode(builder.multipart(parts)?.formatted());
        let mut message = json!({"raw":raw});
        if let Some(thread) = thread {
            message["threadId"] = json!(thread);
        }
        let send = args["send"] == true;
        let path = if send {
            "/users/me/messages/send"
        } else {
            "/users/me/drafts"
        };
        let body = if send {
            message
        } else {
            json!({"message":message})
        };
        // One mutation, no automatic retries after an ambiguous provider response.
        let response = self
            .http_call(
                c,
                "request",
                &json!({"service":"gmail","method":"POST","path":path,"body":body}),
                workspace,
            )
            .await?;
        Ok(json!({"draft":!send,"sent":send,"receipt":response}))
    }
}
