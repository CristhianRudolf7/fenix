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
