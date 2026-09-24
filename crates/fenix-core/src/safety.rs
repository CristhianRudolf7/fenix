use base64::{
    Engine,
    engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD},
};
use regex::Regex;
use serde_json::Value;
use std::sync::RwLock;

pub struct Redactor {
    secrets: RwLock<Vec<String>>,
    pattern: Regex,
    labeled: Regex,
    private_key: Regex,
}
impl Default for Redactor {
    fn default() -> Self {
        Self {
            secrets: RwLock::new(vec![]),
            pattern: Regex::new(r"(^|[^A-Za-z0-9_-])(?:sk-[A-Za-z0-9_-]{12,}|[0-9]{8,12}:[A-Za-z0-9_-]{30,}|gh[pousr]_[A-Za-z0-9]{20,}|github_pat_[A-Za-z0-9_]{20,}|xox[baprs]-[A-Za-z0-9-]{10,}|(?:AKIA|ASIA)[A-Z0-9]{16}|AIza[A-Za-z0-9_-]{30,}|(?:sk|rk)_(?:live|test)_[A-Za-z0-9]{16,}|rp_[A-Za-z0-9]{20,}|vck_[A-Za-z0-9_-]{20,}|cfk_[A-Za-z0-9]{20,}|eyJ[A-Za-z0-9_-]{8,}\.[A-Za-z0-9_-]{8,}\.[A-Za-z0-9_-]{8,})")
                .expect("static secret regex"),
            labeled: Regex::new(r#"(?im)(\b(?:password|passwd|contraseña|api[_ -]?key|client[_ -]?secret|access[_ -]?token|refresh[_ -]?token|authorization|(?:set[_ -]?)?cookie|(?:aws[_ -]?)?secret[_ -]?access[_ -]?key)[ \t]*[\"']?[ \t]*[:=][ \t]*)(?:\"[^\"\r\n]*\"|'[^'\r\n]*'|`[^`\r\n]*`|(?:Bearer|Basic)\s+[^\s,;}]+|[^\s,;}]+)"#).expect("static label regex"),
            private_key: Regex::new(r"(?s)-----BEGIN (?:[A-Z0-9 ]*PRIVATE KEY)-----.*?(?:-----END [A-Z0-9 ]*PRIVATE KEY-----|\z)").expect("static private key regex"),
        }
    }
}
impl Redactor {
    /// Return a stable redacted prefix and its consumed byte count. Retain only
    /// unfinished known secrets/key patterns, so ordinary terminal prompts render.
    pub fn stream_prefix(&self, text: &str, finished: bool) -> (usize, String) {
        if finished {
            return (text.len(), self.text(text));
        }
        let mut end = text.len();
        let line_start = text.rfind('\n').map_or(0, |i| i + 1);
        static LABEL_START: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
            Regex::new(r"(?i)\b(?:password|passwd|contraseña|api(?:[_ -]?key)?|client(?:[_ -]?secret)?|access(?:[_ -]?token)?|refresh(?:[_ -]?token)?|authorization|(?:set[_ -]?)?cookie|(?:aws[_ -]?)?secret(?:[_ -]?access[_ -]?key)?)\b").expect("static label prefix")
        });
        if LABEL_START.is_match(&text[line_start..]) {
            end = end.min(line_start);
        }
        // Keep the whole PEM block until its closing boundary arrives.
        if let Some(start) = text.find("-----BEGIN") {
            static PEM_END: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
                Regex::new(r"-----END [A-Z0-9 ]*PRIVATE KEY-----\r?\n").expect("static PEM end")
            });
            if !PEM_END.is_match(&text[start..]) {
                end = end.min(start);
            }
        }
        for secret in self.secrets.read().expect("secret lock").iter() {
            for (length, _) in secret.char_indices().skip(1) {
                if text.ends_with(&secret[..length]) {
                    end = end.min(text.len() - length);
                }
            }
            for (start, _) in text.match_indices(secret.as_str()) {
                if start < end && end < start + secret.len() {
                    end = start;
                }
            }
        }
        static PARTIAL: std::sync::OnceLock<Regex> = std::sync::OnceLock::new();
        let pattern = PARTIAL.get_or_init(|| {
            Regex::new(r"(?:s(?:k(?:-[A-Za-z0-9_-]*)?)?|[0-9]{1,12}(?::[A-Za-z0-9_-]*)?)$")
                .expect("static regex")
        });
        if let Some(partial) = pattern.find(text) {
            end = end.min(partial.start());
        }
        // Retain a trailing token so new credential formats cannot leak when
        // their identifying prefix is split across process reads.
        let token_start = text
            .char_indices()
            .rev()
            .find(|(_, c)| !c.is_ascii_alphanumeric() && !"_-./+=:%\"'`".contains(*c))
            .map_or(0, |(i, c)| i + c.len_utf8());
        if token_start < text.len() {
            end = end.min(token_start);
        }
        // Token/label retention may move the boundary inside a complete
        // multiword credential. Never release such a partially masked prefix.
        let secrets = self.secrets.read().expect("secret lock");
        loop {
            let adjusted = secrets
                .iter()
                .flat_map(|secret| {
                    text.match_indices(secret.as_str())
                        .filter_map(move |(start, _)| {
                            (start < end && end < start + secret.len()).then_some(start)
                        })
                })
                .min()
                .unwrap_or(end);
            if adjusted == end {
                break;
            }
            end = adjusted;
        }
        drop(secrets);
        (end, self.text(&text[..end]))
    }
    pub fn add(&self, value: &str) {
        if !value.is_empty() {
            let mut secrets = self.secrets.write().expect("secret lock");
            let mut variants = vec![value.to_owned()];
            if value.len() >= 8 {
                variants.extend([
                    STANDARD.encode(value),
                    URL_SAFE_NO_PAD.encode(value),
                    value
                        .as_bytes()
                        .iter()
                        .map(|b| format!("{b:02x}"))
                        .collect(),
                ]);
                let encoded: String = value
                    .bytes()
                    .map(|b| {
                        if b.is_ascii_alphanumeric() || b"-_.~".contains(&b) {
                            (b as char).to_string()
                        } else {
                            format!("%{b:02X}")
                        }
                    })
                    .collect();
                variants.push(encoded.clone());
                variants.push(encoded.replace("%20", "+"));
            }
            for variant in variants {
                if !secrets.contains(&variant) {
                    secrets.push(variant);
                }
            }
            secrets.sort_by_key(|s| std::cmp::Reverse(s.len()));
        }
    }
    pub fn text(&self, text: &str) -> String {
        let mut out = text.to_owned();
        for secret in self.secrets.read().expect("secret lock").iter() {
            out = out.replace(secret, "[REDACTED]");
        }
        out = self
            .private_key
            .replace_all(&out, "[REDACTED_PRIVATE_KEY]")
            .into_owned();
        out = self
            .pattern
            .replace_all(&out, "${1}[REDACTED]")
            .into_owned();
        static COOKIES: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
            Regex::new(r"(?im)(\b(?:set-cookie|cookie)[ \t]*[:=][ \t]*)[^\r\n]+")
                .expect("static cookie header regex")
        });
        out = COOKIES.replace_all(&out, "${1}[REDACTED]").into_owned();
        // Preserve labels and diagnostic structure, never the credential value.
        self.labeled
            .replace_all(&out, "${1}[REDACTED]")
            .into_owned()
    }
    pub fn value(&self, value: &mut Value) {
        match value {
            Value::String(s) => *s = self.text(s),
            Value::Array(items) => items.iter_mut().for_each(|v| self.value(v)),
            Value::Object(items) => {
                let old = std::mem::take(items);
                for (key, mut value) in old {
                    if value.is_string() && sensitive_field(&key) {
                        value = Value::String("[REDACTED]".into());
                    } else {
                        self.value(&mut value);
                    }
                    let mut key = self.text(&key);
                    while items.contains_key(&key) {
                        key.push('_');
                    }
                    items.insert(key, value);
                }
            }
            _ => {}
        }
    }

    /// Withhold the last token and any unfinished known secret before exposing
    /// a preview. Redacting each individual delta could leak a split API key.
    pub fn preview(&self, text: &str) -> String {
        self.stream_prefix(text, false).1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn masks_known_encodings_labels_and_structured_fields() {
        let redactor = Redactor::default();
        let secret = "synthetic credential/with spaces";
        redactor.add(secret);
        for value in [
            secret.to_owned(),
            STANDARD.encode(secret),
            URL_SAFE_NO_PAD.encode(secret),
            secret.bytes().map(|b| format!("{b:02x}")).collect(),
            "synthetic%20credential%2Fwith%20spaces".into(),
        ] {
            assert_eq!(redactor.text(&value), "[REDACTED]");
        }
        assert_eq!(
            redactor.text("password='sample value'"),
            "password=[REDACTED]"
        );
        assert_eq!(
            redactor.text("Authorization: Bearer arbitrary-value"),
            "Authorization: [REDACTED]"
        );
        assert_eq!(
            redactor.text("Cookie: first=secret; second=another"),
            "Cookie: [REDACTED]"
        );
        assert_eq!(
            redactor.text("AWS_SECRET_ACCESS_KEY=synthetic-value"),
            "AWS_SECRET_ACCESS_KEY=[REDACTED]"
        );
        let mut value = serde_json::json!({"nested":{"refresh_token":"opaque","count":3},"cookie":"session=opaque"});
        redactor.value(&mut value);
        assert_eq!(value["nested"]["refresh_token"], "[REDACTED]");
        assert_eq!(value["nested"]["count"], 3);
        assert_eq!(value["cookie"], "[REDACTED]");
        assert_eq!(
            redactor.text("Implement password reset using a secure hash."),
            "Implement password reset using a secure hash."
        );
        let code = "if parts.password:\n    raise ValueError(\"Credentials are forbidden\")";
        assert_eq!(redactor.text(code), code);
    }

    #[test]
    fn streaming_at_every_character_boundary_matches_complete_redaction() {
        let redactor = Redactor::default();
        redactor.add("synthetic known secret with spaces");
        for original in [
            "Fénix ready password: 'secret value'\nNext\n",
            "api key = \"secret value\"\nNext\n",
            "Value \"ghp_abcdefghijklmnopqrstuvwxyz\" done\n",
            "Value rp_abcdefghijklmnopqrstuvwxyz done\n",
            "synthetic known secret with spaces\n",
            "synthetic known secret with spaces",
            "-----BEGIN PRIVATE KEY-----\nbase64-secrets\n-----END PRIVATE KEY-----\nNext\n",
        ] {
            let mut pending = String::new();
            let mut output = String::new();
            for ch in original.chars() {
                pending.push(ch);
                let (used, safe) = redactor.stream_prefix(&pending, false);
                pending.drain(..used);
                output.push_str(&safe);
            }
            output.push_str(&redactor.stream_prefix(&pending, true).1);
            assert_eq!(output, redactor.text(original), "{original}");
        }
    }

    #[test]
    fn previews_do_not_cut_inside_complete_multiword_credentials() {
        let redactor = Redactor::default();
        redactor.add("private token with spaces");
        let text = "Answer private token with spaces then sk-abcdefghijklmnop end";
        for (end, _) in text.char_indices() {
            let preview = redactor.preview(&text[..end]);
            assert!(!preview.contains("private"));
            assert!(!preview.contains("sk-"));
        }
    }

    #[test]
    fn credential_patterns_do_not_match_inside_opaque_ciphertext() {
        let redactor = Redactor::default();
        let ciphertext = "gAAAAABopaqueBase64sk-abcdefghijklmnopMoreCiphertext==";
        let mut message = crate::Message::user("Safe");
        message.provider_data = Some(crate::ProviderData {
            protocol: crate::Protocol::OpenAiResponses,
            data: serde_json::json!({"type":"reasoning","encrypted_content":ciphertext}),
        });
        let original = message.clone();
        redactor.message(&mut message).unwrap();
        assert_eq!(message, original);
        for input in [
            "sk-abcdefghijklmnop",
            "key: sk-abcdefghijklmnop",
            "(sk-abcdefghijklmnop)",
            "\"sk-abcdefghijklmnop\"",
        ] {
            assert!(!redactor.text(input).contains("sk-abcdefghijklmnop"));
        }
        // Actual known credentials are matched literally, even inside other data.
        redactor.add("sk-abcdefghijklmnop");
        assert!(redactor.message(&mut message).is_err());
        assert_eq!(message, original);
    }

    #[test]
    fn opaque_replay_with_secrets_is_rejected_without_mutation() {
        let redactor = Redactor::default();
        let mut message = crate::Message::user("Safe");
        message.provider_data = Some(crate::ProviderData {
            protocol: crate::Protocol::OpenAiResponses,
            data: serde_json::json!({"token":"sk-abcdefghijklmnop"}),
        });
        let original = message.clone();
        assert!(redactor.message(&mut message).is_err());
        assert_eq!(message, original);
    }
}

fn sensitive_field(key: &str) -> bool {
    matches!(
        key.to_ascii_lowercase().replace(['-', ' '], "_").as_str(),
        "password"
            | "passwd"
            | "api_key"
            | "apikey"
            | "client_secret"
            | "access_token"
            | "refresh_token"
            | "authorization"
            | "cookie"
            | "set_cookie"
            | "private_key"
            | "secret_access_key"
            | "aws_secret_access_key"
    )
}

impl Redactor {
    /// Redact a normalized message. Opaque replay is never silently changed:
    /// callers must reject replay containing secrets rather than corrupt signatures.
    pub fn message(&self, message: &mut crate::Message) -> Result<(), &'static str> {
        if let Some(data) = &message.provider_data {
            let mut checked = data.data.clone();
            self.value(&mut checked);
            if checked != data.data {
                return Err(
                    "Provider replay contains sensitive data; start from a safe checkpoint",
                );
            }
        }
        for block in &mut message.content {
            match block {
                crate::ContentBlock::Text { text } => *text = self.text(text),
                crate::ContentBlock::ToolCall { arguments, .. } => self.value(arguments),
                crate::ContentBlock::ToolResult { content, .. } => self.value(content),
            }
        }
        Ok(())
    }
}
