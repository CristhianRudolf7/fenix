use regex::Regex;
use serde_json::Value;
use std::sync::RwLock;

pub struct Redactor {
    secrets: RwLock<Vec<String>>,
    pattern: Regex,
}
impl Default for Redactor {
    fn default() -> Self {
        Self {
            secrets: RwLock::new(vec![]),
            pattern: Regex::new(r"(?:sk-[A-Za-z0-9_-]{12,}|[0-9]{8,12}:[A-Za-z0-9_-]{30,})")
                .expect("static secret regex"),
        }
    }
}
impl Redactor {
    /// Return a stable redacted prefix and its consumed byte count. Retain only
    /// unfinished known secrets/key patterns, so ordinary terminal prompts render.
    pub(crate) fn stream_prefix(&self, text: &str, finished: bool) -> (usize, String) {
        if finished {
            return (text.len(), self.text(text));
        }
        let mut end = text.len();
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
        (end, self.text(&text[..end]))
    }
    pub fn add(&self, value: &str) {
        if !value.is_empty() {
            self.secrets
                .write()
                .expect("secret lock")
                .push(value.into());
        }
    }
    pub fn text(&self, text: &str) -> String {
        let mut out = text.to_owned();
        for secret in self.secrets.read().expect("secret lock").iter() {
            out = out.replace(secret, "[REDACTED]");
        }
        self.pattern.replace_all(&out, "[REDACTED]").into_owned()
    }
    pub fn value(&self, value: &mut Value) {
        match value {
            Value::String(s) => *s = self.text(s),
            Value::Array(items) => items.iter_mut().for_each(|v| self.value(v)),
            Value::Object(items) => items.values_mut().for_each(|v| self.value(v)),
            _ => {}
        }
    }

    /// Withhold the last token and any unfinished known secret before exposing
    /// a preview. Redacting each individual delta could leak a split API key.
    pub fn preview(&self, text: &str) -> String {
        let mut end = text
            .char_indices()
            .rev()
            .find(|(_, c)| c.is_whitespace())
            .map(|(i, c)| i + c.len_utf8())
            .unwrap_or(0);
        for secret in self.secrets.read().expect("secret lock").iter() {
            for (start, _) in text.match_indices(secret.as_str()) {
                if start < end && end < start + secret.len() {
                    end = start;
                }
            }
            for (length, _) in secret.char_indices().skip(1) {
                if text.ends_with(&secret[..length]) {
                    end = end.min(text.len() - length);
                }
            }
        }
        self.text(&text[..end])
    }
}

pub fn truncate(text: &str, max: usize) -> String {
    if text.len() <= max {
        return text.into();
    }
    let mut end = max;
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    format!(
        "{}\n[truncated; use a narrower query or inspect the saved tool event]",
        &text[..end]
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn streaming_never_reveals_split_known_secrets_or_key_patterns() {
        let redactor = Redactor::default();
        redactor.add("private token with spaces");
        let text = "Answer private token with spaces then sk-abcdefghijklmnop end";
        for (end, _) in text.char_indices() {
            let preview = redactor.preview(&text[..end]);
            assert!(!preview.contains("private"));
            assert!(!preview.contains("sk-"));
        }
        assert_eq!(redactor.preview("Fénix está listo "), "Fénix está listo ");
    }
}
