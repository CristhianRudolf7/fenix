//! Bounded SSE transport. Partial text never becomes an executable response.
use crate::{ProviderError, client::transport_error, protocol};
use fenix_core::{ChatResponse, Protocol};
use serde_json::Value;

#[derive(Default)]
struct Events {
    pending: Vec<u8>,
    data: Vec<u8>,
}

impl Events {
    fn push(&mut self, bytes: &[u8]) -> Result<Vec<Value>, ProviderError> {
        self.pending.extend_from_slice(bytes);
        let mut events = Vec::new();
        let mut start = 0;
        for end in 0..self.pending.len() {
            if self.pending[end] != b'\n' {
                continue;
            }
            let line = &self.pending[start..end];
            let line = line.strip_suffix(b"\r").unwrap_or(line);
            if line.is_empty() {
                if !self.data.is_empty() {
                    events.push(
                        serde_json::from_slice(&self.data)
                            .map_err(|_| ProviderError::MalformedResponse)?,
                    );
                    self.data.clear();
                }
            } else if let Some(data) = line.strip_prefix(b"data:") {
                if !self.data.is_empty() {
                    self.data.push(b'\n');
                }
                self.data
                    .extend_from_slice(data.strip_prefix(b" ").unwrap_or(data));
            }
            start = end + 1;
        }
        self.pending.drain(..start);
        Ok(events)
    }
}

pub(crate) async fn read_responses(
    mut response: reqwest::Response,
    limit: usize,
    preview: &mut (dyn FnMut(&str) + Send),
) -> Result<ChatResponse, ProviderError> {
    let mut parser = Events::default();
    let mut received = 0usize;
    let mut last_part = None;
    while let Some(chunk) = response.chunk().await.map_err(transport_error)? {
        if chunk.len() > limit.saturating_sub(received) {
            return Err(ProviderError::ResponseTooLarge { limit });
        }
        received += chunk.len();
        for event in parser.push(&chunk)? {
            match event["type"].as_str() {
                Some("response.output_text.delta" | "response.refusal.delta") => {
                    let part = (
                        event["output_index"].as_u64(),
                        event["content_index"].as_u64(),
                    );
                    if last_part.is_some_and(|last| last != part) {
                        preview("\n");
                    }
                    last_part = Some(part);
                    let text = event["delta"]
                        .as_str()
                        .ok_or(ProviderError::MalformedResponse)?;
                    preview(text);
                }
                Some("response.completed" | "response.incomplete") => {
                    return Ok(protocol::decode(
                        Protocol::OpenAiResponses,
                        event["response"].clone(),
                    )?);
                }
                Some("response.failed" | "error") => return Err(ProviderError::MalformedResponse),
                Some(_) => {} // Tool arguments and reasoning are never previews.
                None => return Err(ProviderError::MalformedResponse),
            }
        }
    }
    // A clean HTTP EOF is still an incomplete inference without a terminal event.
    Err(ProviderError::MalformedResponse)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sse_handles_split_utf8_crlf_comments_and_multiline_data() {
        let input = ": ping\r\nevent: response.output_text.delta\r\ndata: {\"type\":\"response.output_text.delta\",\r\ndata: \"delta\":\"Fénix\"}\r\n\r\n";
        let mut parser = Events::default();
        let mut values = vec![];
        for byte in input.as_bytes() {
            values.extend(parser.push(&[*byte]).unwrap());
        }
        assert_eq!(values.len(), 1);
        assert_eq!(values[0]["delta"], "Fénix");
        assert!(parser.pending.is_empty());
    }
}
