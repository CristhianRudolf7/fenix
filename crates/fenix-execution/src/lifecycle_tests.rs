use super::*;
use tokio::net::UnixListener;

fn client(path: PathBuf) -> Executor {
    Executor::configured(
        Some(path),
        Some("00000000-0000-4000-8000-000000000001".into()),
        Some("a".repeat(64)),
        true,
    )
    .unwrap()
    .unwrap()
}

#[tokio::test]
async fn cancelling_a_waiting_client_releases_its_admission_ticket() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("broker.sock");
    let listener = UnixListener::bind(&path).unwrap();
    let (polled, ready) = tokio::sync::oneshot::channel();
    let server = tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.unwrap();
        let request = read_json(&mut stream).await.unwrap();
        assert_eq!(request["operation"], "create");
        write_json(
            &mut stream,
            &json!({"ok":true,"queued":true,"ticket":"waiting"}),
        )
        .await
        .unwrap();
        let (mut stream, _) = listener.accept().await.unwrap();
        let request = read_json(&mut stream).await.unwrap();
        assert_eq!(request["operation"], "admission_poll");
        write_json(
            &mut stream,
            &json!({"ok":true,"queued":true,"ticket":"waiting"}),
        )
        .await
        .unwrap();
        polled.send(()).unwrap();
        let (mut stream, _) = listener.accept().await.unwrap();
        let request = read_json(&mut stream).await.unwrap();
        assert_eq!(request["operation"], "admission_cancel");
        assert_eq!(request["ticket"], "waiting");
        write_json(&mut stream, &json!({"ok":true})).await.unwrap();
    });
    let executor = client(path);
    let waiting =
        tokio::spawn(async move { executor.admitted(json!({"operation":"create"})).await });
    tokio::time::timeout(Duration::from_secs(5), ready)
        .await
        .unwrap()
        .unwrap();
    waiting.abort();
    assert!(matches!(waiting.await, Err(error) if error.is_cancelled()));
    tokio::time::timeout(Duration::from_secs(5), server)
        .await
        .unwrap()
        .unwrap();
}

#[tokio::test]
async fn dropping_an_unattached_resume_releases_it_without_explicit_cancellation() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("broker.sock");
    let listener = UnixListener::bind(&path).unwrap();
    let server = tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.unwrap();
        assert_eq!(
            read_json(&mut stream).await.unwrap()["operation"],
            "browser_resume"
        );
        write_json(&mut stream, &json!({"ok":true,"id":"00000000-0000-4000-8000-000000000002","capability":"x".repeat(32),"timeout":300})).await.unwrap();
        let (mut stream, _) = listener.accept().await.unwrap();
        assert_eq!(
            read_json(&mut stream).await.unwrap()["operation"],
            "release"
        );
        write_json(&mut stream, &json!({"ok":true})).await.unwrap();
    });
    let lease = client(path)
        .resume_browser("00000000-0000-4000-8000-000000000002")
        .await
        .unwrap();
    drop(lease);
    tokio::time::timeout(Duration::from_secs(5), server)
        .await
        .unwrap()
        .unwrap();
}

#[tokio::test]
async fn lost_suspend_reply_checks_committed_state_before_releasing_the_lease() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("broker.sock");
    let listener = UnixListener::bind(&path).unwrap();
    let server = tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.unwrap();
        assert_eq!(
            read_json(&mut stream).await.unwrap()["operation"],
            "browser_suspend"
        );
        drop(stream);
        let (mut stream, _) = listener.accept().await.unwrap();
        assert_eq!(read_json(&mut stream).await.unwrap()["operation"], "status");
        write_json(&mut stream, &json!({"ok":true,"status":"suspended"}))
            .await
            .unwrap();
    });
    let lease = Lease {
        executor: client(path),
        id: "00000000-0000-4000-8000-000000000002".into(),
        capability: Zeroizing::new("x".repeat(32)),
        timeout: 300,
        suspended: AtomicBool::new(false),
    };
    assert!(lease.suspend_browser().await.is_err());
    assert!(lease.is_suspended());
    server.await.unwrap();
}
