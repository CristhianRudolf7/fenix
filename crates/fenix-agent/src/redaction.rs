pub use fenix_core::safety::Redactor;

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
