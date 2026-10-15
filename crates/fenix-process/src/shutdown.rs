//! Portable shutdown source. Mobile hosts request cancellation explicitly; no desktop signal assumptions.
use anyhow::Result;
use tokio_util::sync::CancellationToken;

pub struct Shutdown {
    requested: CancellationToken,
    #[cfg(all(unix, not(any(target_os = "android", target_os = "ios"))))]
    terminate: tokio::signal::unix::Signal,
    #[cfg(all(unix, not(any(target_os = "android", target_os = "ios"))))]
    interrupt: tokio::signal::unix::Signal,
    #[cfg(windows)]
    interrupt: tokio::signal::windows::CtrlC,
    #[cfg(windows)]
    close: tokio::signal::windows::CtrlClose,
    #[cfg(windows)]
    terminate: tokio::signal::windows::CtrlShutdown,
    #[cfg(windows)]
    pause: tokio::signal::windows::CtrlBreak,
}
impl Shutdown {
    pub fn listen() -> Result<Self> {
        Ok(Self {
            requested: CancellationToken::new(),
            #[cfg(all(unix, not(any(target_os = "android", target_os = "ios"))))]
            terminate: tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?,
            #[cfg(all(unix, not(any(target_os = "android", target_os = "ios"))))]
            interrupt: tokio::signal::unix::signal(tokio::signal::unix::SignalKind::interrupt())?,
            #[cfg(windows)]
            interrupt: tokio::signal::windows::ctrl_c()?,
            #[cfg(windows)]
            close: tokio::signal::windows::ctrl_close()?,
            #[cfg(windows)]
            terminate: tokio::signal::windows::ctrl_shutdown()?,
            #[cfg(windows)]
            pause: tokio::signal::windows::ctrl_break()?,
        })
    }
    /// A Swift/Kotlin host, service controller, or embedding application can stop gracefully.
    pub fn token(&self) -> CancellationToken {
        self.requested.clone()
    }
    pub async fn wait(&mut self) {
        #[cfg(all(unix, not(any(target_os = "android", target_os = "ios"))))]
        tokio::select! { _=self.requested.cancelled()=>{}, _=self.terminate.recv()=>{}, _=self.interrupt.recv()=>{} }
        #[cfg(windows)]
        tokio::select! { _=self.requested.cancelled()=>{}, _=self.terminate.recv()=>{}, _=self.interrupt.recv()=>{}, _=self.close.recv()=>{}, _=self.pause.recv()=>{} }
        #[cfg(any(target_os = "android", target_os = "ios"))]
        self.requested.cancelled().await;
        self.requested.cancel();
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn an_embedding_host_can_request_shutdown_and_repeated_waits_do_not_hang() {
        let mut shutdown = Shutdown::listen().unwrap();
        let token = shutdown.token();
        token.cancel();
        tokio::time::timeout(std::time::Duration::from_secs(1), shutdown.wait())
            .await
            .unwrap();
        tokio::time::timeout(std::time::Duration::from_secs(1), shutdown.wait())
            .await
            .unwrap();
    }
}
