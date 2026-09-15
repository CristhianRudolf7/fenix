//! Bounded-memory redacted process logs with byte cursors and head/tail previews.
use super::*;
use tokio::io::{AsyncSeekExt, AsyncWriteExt};

const LOG_LIMIT: u64 = 64 * 1024 * 1024;
pub(super) const PAGE_LIMIT: usize = 12_000;

pub(super) async fn capture(
    mut reader: impl AsyncRead + Unpin,
    process: Arc<Process>,
    redactor: Arc<Redactor>,
) {
    let mut buffer = [0u8; 4096];
    let mut pending = Vec::new();
    let mut dropping_token = false;
    loop {
        let count = match reader.read(&mut buffer).await {
            Ok(count) => count,
            Err(error) if error.raw_os_error() == Some(nix::libc::EIO) => 0, // PTY EOF
            Err(error) => {
                process.output_failure(&error.to_string()).await;
                0
            }
        };
        pending.extend_from_slice(&buffer[..count]);
        let valid = match std::str::from_utf8(&pending) {
            Ok(_) => pending.len(),
            Err(e) if e.error_len().is_none() && count != 0 => e.valid_up_to(),
            Err(_) => pending.len(),
        };
        let text = String::from_utf8_lossy(&pending[..valid]);
        let (consumed, mut output) = if dropping_token {
            if let Some((at, _)) = text.char_indices().find(|(_, c)| c.is_whitespace()) {
                dropping_token = false;
                let (used, result) = redactor.stream_prefix(&text[at..], count == 0);
                (at + used, result)
            } else {
                (valid, String::new())
            }
        } else {
            redactor.stream_prefix(&text, count == 0)
        };
        // Lossy decoding changes byte lengths; invalid data is consumed as one chunk.
        let consumed = if std::str::from_utf8(&pending[..valid]).is_err() {
            valid
        } else {
            consumed
        };
        pending.drain(..consumed.min(pending.len()));
        if pending.len() > 65_536 {
            pending.clear();
            dropping_token = true;
            output.push_str("[REDACTED oversized unfinished token]");
            process.state.lock().await.log_complete = false;
        }
        if !output.is_empty() {
            let mut state = process.state.lock().await;
            if state.bytes + output.len() as u64 > LOG_LIMIT {
                drop(state);
                process
                    .output_failure("Process log reached its 64 MiB limit; process stopped")
                    .await;
                break;
            }
            match state.log.write_all(output.as_bytes()).await {
                Ok(()) => {
                    state.bytes += output.len() as u64;
                }
                Err(error) => {
                    state.log_complete = false;
                    state.output_error = Some(error.to_string());
                    process.cancel.cancel();
                    break;
                }
            }
        }
        if count == 0 {
            break;
        }
    }
}

pub(super) async fn read_page(
    path: &Path,
    start: u64,
    end: u64,
    limit: usize,
    head_tail: bool,
) -> Result<(String, u64, u64)> {
    if start > end {
        bail!("Output cursor is beyond available output");
    }
    let mut file = tokio::fs::File::open(path).await?;
    let available = (end - start) as usize;
    let size = available.min(limit);
    let first = if head_tail && available > limit {
        size / 3
    } else {
        size
    };
    file.seek(std::io::SeekFrom::Start(start)).await?;
    let mut bytes = vec![0; first];
    file.read_exact(&mut bytes).await?;
    if head_tail && available > limit {
        let tail = size - first;
        file.seek(std::io::SeekFrom::Start(end - tail as u64))
            .await?;
        let mut last = vec![0; tail];
        file.read_exact(&mut last).await?;
        return Ok((
            format!(
                "{}\n[{} log bytes omitted; use command_output to recover]\n{}",
                String::from_utf8_lossy(&bytes),
                available - size,
                String::from_utf8_lossy(&last)
            ),
            end,
            (available - size) as u64,
        ));
    }
    // Do not split a valid UTF-8 character between consecutive cursor pages.
    if let Err(e) = std::str::from_utf8(&bytes)
        && e.error_len().is_none()
        && first < available
    {
        bytes.truncate(e.valid_up_to());
    }
    let next = start + bytes.len() as u64;
    Ok((String::from_utf8_lossy(&bytes).into_owned(), next, 0))
}
