use super::*;
use portable_pty::{CommandBuilder, MasterPty, PtySize};
use std::{
    io::{Read, Write},
    sync::{Arc, Mutex},
};
use std::{
    pin::Pin,
    task::{Context as TaskContext, Poll},
};
use tokio::io::{AsyncRead, AsyncWriteExt, ReadBuf};

struct TerminalReader {
    stream: tokio::io::DuplexStream,
    finished: tokio::sync::oneshot::Receiver<std::io::Result<()>>,
    ended: bool,
}
impl AsyncRead for TerminalReader {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut TaskContext<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        if self.ended || buf.remaining() == 0 {
            return Poll::Ready(Ok(()));
        }
        let before = buf.filled().len();
        match Pin::new(&mut self.stream).poll_read(cx, buf) {
            Poll::Ready(Ok(())) if buf.filled().len() == before => {
                use std::future::Future;
                match Pin::new(&mut self.finished).poll(cx) {
                    Poll::Ready(Ok(result)) => {
                        self.ended = true;
                        Poll::Ready(result)
                    }
                    Poll::Ready(Err(_)) => {
                        self.ended = true;
                        Poll::Ready(Err(std::io::Error::other(
                            "Terminal capture stopped unexpectedly",
                        )))
                    }
                    Poll::Pending => Poll::Pending,
                }
            }
            result => result,
        }
    }
}

pub struct Terminal {
    master: Arc<Mutex<Box<dyn MasterPty + Send>>>,
    writer: Arc<Mutex<Box<dyn Write + Send>>>,
}
impl Terminal {
    pub fn resize(&self, rows: u16, columns: u16) -> Result<()> {
        self.master
            .lock()
            .map_err(|_| anyhow::anyhow!("Terminal lock poisoned"))?
            .resize(PtySize {
                rows,
                cols: columns,
                pixel_width: 0,
                pixel_height: 0,
            })
    }
    pub async fn write(&self, text: Vec<u8>, eof: bool) -> std::io::Result<()> {
        #[cfg(windows)]
        let text = {
            let mut bytes = Vec::with_capacity(text.len());
            let mut previous = 0;
            for byte in text {
                if byte != b'\n' || previous != b'\r' {
                    bytes.push(if byte == b'\n' { b'\r' } else { byte });
                }
                previous = byte;
            }
            bytes
        };
        let writer = self.writer.clone();
        tokio::task::spawn_blocking(move || {
            let mut writer = writer
                .lock()
                .map_err(|_| std::io::Error::other("Terminal writer poisoned"))?;
            writer.write_all(&text)?;
            if eof {
                writer.write_all(if cfg!(windows) { b"\x1a\r\n" } else { b"\x04" })?;
            }
            writer.flush()
        })
        .await
        .map_err(std::io::Error::other)?
    }
}
pub(super) fn spawn(shell: &Shell, script: &str, cwd: &Path, home: &Path) -> Result<Spawned> {
    let pair = portable_pty::native_pty_system().openpty(PtySize {
        rows: 24,
        cols: 100,
        pixel_width: 0,
        pixel_height: 0,
    })?;
    let mut command = CommandBuilder::new(&shell.program);
    command.args(
        shell
            .args(script)
            .into_iter()
            .filter(|arg| *arg != "-NonInteractive"),
    );
    command.cwd(cwd);
    command.env_clear();
    for (key, value) in local_environment(home) {
        command.env(key, value);
    }
    let mut reader = pair.master.try_clone_reader()?;
    let writer = pair.master.take_writer()?;
    let mut child = pair.slave.spawn_command(command)?;
    let tree = match ProcessTree::terminal(child.as_ref()) {
        Ok(tree) => tree,
        Err(error) => {
            let _ = child.kill();
            let _ = child.wait();
            return Err(error);
        }
    };
    drop(pair.slave);
    let killer = child.clone_killer();
    let (send, done) = tokio::sync::oneshot::channel();
    tokio::task::spawn_blocking(move || {
        let _ = send.send(child.wait());
    });
    // Bounded backpressure; read errors reach the async reader rather than silently
    // truncating successful logs. Blocking PTY handles never occupy Tokio workers.
    let (async_reader, mut async_writer) = tokio::io::duplex(32 * 1024);
    let runtime = tokio::runtime::Handle::current();
    let (finish, finished) = tokio::sync::oneshot::channel();
    tokio::task::spawn_blocking(move || {
        let mut bytes = [0; 4096];
        let result = loop {
            match reader.read(&mut bytes) {
                Ok(0) => break Ok(()),
                Ok(n) => {
                    if runtime
                        .block_on(async_writer.write_all(&bytes[..n]))
                        .is_err()
                    {
                        break Ok(());
                    }
                }
                Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
                #[cfg(unix)]
                Err(error) if error.raw_os_error() == Some(nix::libc::EIO) => break Ok(()),
                Err(error) => break Err(error),
            }
        };
        let _ = finish.send(result);
    });
    Ok(Spawned {
        child: RunningChild::Terminal {
            done,
            killer,
            code: None,
        },
        input: Input::Terminal(Terminal {
            master: Arc::new(Mutex::new(pair.master)),
            writer: Arc::new(Mutex::new(writer)),
        }),
        readers: vec![Box::new(TerminalReader {
            stream: async_reader,
            finished,
            ended: false,
        })],
        tree,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::AsyncReadExt;
    #[tokio::test]
    async fn capture_errors_are_reported_and_eof_can_be_read_again() {
        let (stream, writer) = tokio::io::duplex(16);
        let (sender, finished) = tokio::sync::oneshot::channel();
        sender
            .send(Err(std::io::Error::other("capture failed")))
            .unwrap();
        drop(writer);
        let mut reader = TerminalReader {
            stream,
            finished,
            ended: false,
        };
        let mut byte = [0];
        assert_eq!(
            reader.read(&mut byte).await.unwrap_err().to_string(),
            "capture failed"
        );
        assert_eq!(reader.read(&mut byte).await.unwrap(), 0);
    }
}
