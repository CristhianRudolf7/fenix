//! Admission policy shared by foreground inference and compaction.
use crate::config::Settings;
pub(crate) struct ContextBudget {
    pub capacity: usize,
    pub input_limit: usize,
    pub recent: usize,
    pub output: usize,
}
impl ContextBudget {
    pub fn new(settings: &Settings) -> Self {
        let capacity = settings.coding.context_window_tokens;
        let output = settings.execution.max_output_tokens as usize;
        Self {
            capacity,
            output,
            input_limit: settings
                .coding
                .compact_at_tokens
                .min(capacity.saturating_sub(output + 1024)),
            recent: settings.coding.keep_recent_tokens,
        }
    }
    pub fn output_available(&self, input: usize) -> usize {
        self.output.min(self.capacity.saturating_sub(input + 1024))
    }
    pub fn tool_bytes(&self, calls: usize) -> usize {
        // Leave room for assistant arguments and result wrappers in the recent batch.
        (self.recent * 3 / 2 / calls.max(1)).clamp(256, 12_000)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn reserves_output_and_bounds_the_entire_batch() {
        let settings = Settings::initial("/tmp".into());
        let b = ContextBudget::new(&settings);
        assert!(b.input_limit + b.output + 1024 <= b.capacity);
        assert!(b.tool_bytes(3) * 3 < 40_096);
        assert_eq!(b.output_available(b.capacity), 0);
    }
}
