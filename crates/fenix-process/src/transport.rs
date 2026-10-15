//! Local benchmark transport only, independent of execution supervision and account routing.
use anyhow::{Context, Result, ensure};
use std::{net::SocketAddr, path::Path, time::Duration};
use tokio::io::{AsyncRead, AsyncWrite};
pub trait Stream: AsyncRead + AsyncWrite + Send + Unpin {}
impl<T: AsyncRead + AsyncWrite + Send + Unpin> Stream for T {}
pub type LocalStream = Box<dyn Stream>;

pub async fn connect(endpoint: &str) -> Result<LocalStream> {
    if let Some(address) = endpoint.strip_prefix("tcp://") {
        let address: SocketAddr = address
            .parse()
            .context("Use a literal loopback address and port for local benchmark TCP")?;
        ensure!(
            address.ip().is_loopback() && address.port() != 0,
            "Benchmark transport accepts only loopback TCP"
        );
        let stream = tokio::time::timeout(
            Duration::from_secs(10),
            tokio::net::TcpStream::connect(address),
        )
        .await
        .context("Benchmark transport connection timed out")??;
        return Ok(Box::new(stream));
    }
    let path = Path::new(endpoint.strip_prefix("unix://").unwrap_or(endpoint));
    ensure!(
        path.is_absolute(),
        "Unix benchmark socket requires an absolute path"
    );
    #[cfg(unix)]
    {
        let stream = tokio::time::timeout(
            Duration::from_secs(10),
            tokio::net::UnixStream::connect(path),
        )
        .await
        .context("Benchmark transport connection timed out")??;
        Ok(Box::new(stream))
    }
    #[cfg(not(unix))]
    anyhow::bail!(
        "Unix benchmark sockets are unavailable on this platform; use tcp://127.0.0.1:PORT"
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    #[tokio::test]
    async fn loopback_transport_preserves_bidirectional_bytes() {
        let listener = tokio::net::TcpListener::bind((std::net::IpAddr::from([127, 0, 0, 1]), 0))
            .await
            .unwrap();
        let address = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut bytes = [0; 4];
            stream.read_exact(&mut bytes).await.unwrap();
            assert_eq!(&bytes, b"ping");
            stream.write_all(b"pong").await.unwrap();
        });
        let mut stream = connect(&format!("tcp://{address}")).await.unwrap();
        stream.write_all(b"ping").await.unwrap();
        let mut bytes = [0; 4];
        stream.read_exact(&mut bytes).await.unwrap();
        assert_eq!(&bytes, b"pong");
        server.await.unwrap();
    }
    #[tokio::test]
    async fn transport_rejects_remote_hosts_and_relative_paths() {
        for endpoint in [
            "tcp://192.0.2.1:1234",
            "tcp://example.com:1234",
            "tcp://127.0.0.1:0",
            "relative.sock",
        ] {
            assert!(connect(endpoint).await.is_err(), "{endpoint}");
        }
    }
}
