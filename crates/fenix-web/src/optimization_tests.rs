use super::*;
use crate::tests::{empty_config, fixture, html, robots};
use wiremock::{
    Mock, ResponseTemplate,
    matchers::{method, path},
};

fn reference(topic: &str, version: &str) -> String {
    format!(
        "<title>{topic}</title><main><h1>{topic}</h1>{version}<p>{}</p></main>",
        format!("{topic} has a detailed reference with examples and explanations. ").repeat(12)
    )
}

fn request(query: &str) -> SearchRequest {
    SearchRequest {
        query: query.into(),
        domains: vec!["example.com".into()],
        discover: true,
        ..Default::default()
    }
}

#[tokio::test]
async fn warm_search_reads_the_current_reference_without_recrawling_the_root() {
    let (_dir, web, server) = fixture(empty_config()).await;
    robots(&server, "User-agent: *\nAllow: /\n").await;
    html(
        &server,
        "/",
        "<nav><a href='/quantum'>Quantum sensors</a></nav>",
    )
    .await;
    html(
        &server,
        "/quantum",
        &reference("Quantum sensors", "old_method"),
    )
    .await;
    let cancel = CancellationToken::new();
    web.search_with(request("quantum sensors"), &cancel)
        .await
        .unwrap();
    server.reset().await;
    html(
        &server,
        "/quantum",
        &reference("Quantum sensors", "new_method"),
    )
    .await;
    let result = web
        .search_with(request("quantum sensors"), &cancel)
        .await
        .unwrap();
    assert_eq!(result.discovery.stop_reason, "indexed_sources");
    assert!(
        result.results[0]
            .page
            .as_ref()
            .unwrap()
            .passage
            .contains("new_method")
    );
    assert_eq!(result.results[0].read_status, "network");
    let reads = server.received_requests().await.unwrap();
    assert_eq!(reads.len(), 1, "only the selected reference should be read");
    assert_eq!(reads[0].url.path(), "/quantum");
    assert_eq!(web.network.pooled_origins(), 1);
}

#[tokio::test]
async fn changed_indexed_evidence_triggers_fresh_discovery_of_a_replacement() {
    let (_dir, web, server) = fixture(empty_config()).await;
    robots(&server, "User-agent: *\nAllow: /\n").await;
    html(
        &server,
        "/",
        "<nav><a href='/old'>Quantum sensors</a></nav>",
    )
    .await;
    html(
        &server,
        "/old",
        &reference("Quantum sensors", "Old reference"),
    )
    .await;
    let cancel = CancellationToken::new();
    web.search_with(request("quantum sensors"), &cancel)
        .await
        .unwrap();
    server.reset().await;
    html(
        &server,
        "/old",
        "<title>Gardening</title><main>Roses and tulips.</main>",
    )
    .await;
    html(
        &server,
        "/",
        "<nav><a href='/new'>Quantum sensors</a></nav>",
    )
    .await;
    html(
        &server,
        "/new",
        &reference("Quantum sensors", "Replacement reference"),
    )
    .await;
    let result = web
        .search_with(request("quantum sensors"), &cancel)
        .await
        .unwrap();
    assert_ne!(result.discovery.stop_reason, "indexed_sources");
    assert!(result.results.iter().any(|hit| hit.url.ends_with("/new")));
    assert!(result.results.iter().all(|hit| !hit.url.ends_with("/old")));
    assert_eq!(
        server
            .received_requests()
            .await
            .unwrap()
            .iter()
            .filter(|r| r.url.path() == "/old")
            .count(),
        1
    );
}

#[tokio::test]
async fn related_new_query_uses_cached_navigation_but_reads_fresh_evidence() {
    let (_dir, web, server) = fixture(empty_config()).await;
    robots(&server, "User-agent: *\nAllow: /\n").await;
    html(&server, "/", "<main>Reference directory</main><nav><a href='/alpha'>Alpha sensors</a><a href='/beta'>Beta lasers</a></nav>").await;
    html(
        &server,
        "/alpha",
        &reference("Alpha sensors", "First topic"),
    )
    .await;
    let cancel = CancellationToken::new();
    web.search_with(request("alpha sensors"), &cancel)
        .await
        .unwrap();
    server.reset().await;
    html(
        &server,
        "/beta",
        &reference("Beta lasers", "Current second topic"),
    )
    .await;
    let result = web
        .search_with(request("beta lasers"), &cancel)
        .await
        .unwrap();
    assert!(
        result
            .discovery
            .sources
            .iter()
            .any(|s| s.url == "https://example.com/" && s.source == Some("cache"))
    );
    assert!(
        result
            .results
            .iter()
            .any(|h| h.url.ends_with("/beta") && h.read_status == "network")
    );
    assert!(
        server
            .received_requests()
            .await
            .unwrap()
            .iter()
            .all(|r| r.url.path() == "/beta")
    );
}

#[tokio::test]
async fn cached_discovery_text_is_never_reported_as_current_evidence() {
    let (_dir, web, server) = fixture(empty_config()).await;
    robots(&server, "User-agent: *\nAllow: /\n").await;
    html(
        &server,
        "/",
        "<main>Reference directory</main><nav><a href='/short'>Quantum sensors</a></nav>",
    )
    .await;
    html(
        &server,
        "/short",
        "<title>Quantum sensors</title><main>Quantum sensors old_method</main>",
    )
    .await;
    let cancel = CancellationToken::new();
    web.search_with(request("quantum sensors"), &cancel)
        .await
        .unwrap();
    server.reset().await;
    html(
        &server,
        "/short",
        "<title>Quantum sensors</title><main>Quantum sensors new_method</main>",
    )
    .await;
    let result = web
        .search_with(request("quantum sensors"), &cancel)
        .await
        .unwrap();
    assert!(
        result
            .discovery
            .sources
            .iter()
            .any(|source| source.url.ends_with("/short") && source.source == Some("cache"))
    );
    let hit = result
        .results
        .iter()
        .find(|hit| hit.url.ends_with("/short"))
        .unwrap();
    assert_eq!(hit.read_status, "network");
    assert!(hit.page.as_ref().unwrap().passage.contains("new_method"));
    assert_eq!(server.received_requests().await.unwrap().len(), 1);
}

#[tokio::test]
async fn failed_indexed_read_is_not_retried_or_mislabeled_as_fresh_in_the_same_call() {
    let (_dir, web, server) = fixture(empty_config()).await;
    robots(&server, "User-agent: *\nAllow: /\n").await;
    html(
        &server,
        "/",
        "<nav><a href='/reference'>Quantum sensors</a></nav>",
    )
    .await;
    html(
        &server,
        "/reference",
        &reference("Quantum sensors", "Old evidence"),
    )
    .await;
    let cancel = CancellationToken::new();
    web.search_with(request("quantum sensors"), &cancel)
        .await
        .unwrap();
    server.reset().await;
    html(
        &server,
        "/",
        "<nav><a href='/reference'>Quantum sensors</a></nav>",
    )
    .await;
    Mock::given(method("GET"))
        .and(path("/reference"))
        .respond_with(ResponseTemplate::new(503))
        .mount(&server)
        .await;
    let result = web
        .search_with(request("quantum sensors"), &cancel)
        .await
        .unwrap();
    assert_eq!(result.results.len(), 1);
    assert_eq!(result.results[0].read_status, "failed");
    assert!(result.results[0].page.is_none());
    assert_eq!(
        server
            .received_requests()
            .await
            .unwrap()
            .iter()
            .filter(|r| r.url.path() == "/reference")
            .count(),
        1
    );
}

#[tokio::test]
async fn publication_window_search_revisits_the_listing_for_new_articles() {
    let (_dir, web, server) = fixture(empty_config()).await;
    robots(&server, "User-agent: *\nAllow: /\n").await;
    html(
        &server,
        "/",
        "<nav><a href='/old'>Quantum sensors</a></nav>",
    )
    .await;
    html(
        &server,
        "/old",
        &reference("Quantum sensors", "Previously indexed"),
    )
    .await;
    let cancel = CancellationToken::new();
    web.search_with(request("quantum sensors"), &cancel)
        .await
        .unwrap();
    server.reset().await;
    html(
        &server,
        "/",
        "<nav><a href='/new'>Quantum sensors</a></nav>",
    )
    .await;
    html(
        &server,
        "/new",
        &format!(
            "<meta property='article:published_time' content='{}'>{}",
            Utc::now().to_rfc3339(),
            reference("Quantum sensors", "New article")
        ),
    )
    .await;
    let result = web
        .search_with(
            SearchRequest {
                days: Some(30),
                ..request("quantum sensors")
            },
            &cancel,
        )
        .await
        .unwrap();
    assert_ne!(result.discovery.stop_reason, "indexed_sources");
    assert!(result.results.iter().any(|h| h.url.ends_with("/new")));
    assert!(
        server
            .received_requests()
            .await
            .unwrap()
            .iter()
            .any(|r| r.url.path() == "/")
    );
}

#[tokio::test]
async fn explicit_recency_without_a_date_filter_still_discovers_new_urls() {
    let (_dir, web, server) = fixture(empty_config()).await;
    robots(&server, "User-agent: *\nAllow: /\n").await;
    html(
        &server,
        "/",
        "<nav><a href='/old'>Quantum sensors</a></nav>",
    )
    .await;
    html(
        &server,
        "/old",
        &reference("Quantum sensors", "Previously indexed"),
    )
    .await;
    let cancel = CancellationToken::new();
    web.search_with(request("quantum sensors"), &cancel)
        .await
        .unwrap();
    server.reset().await;
    html(
        &server,
        "/",
        "<nav><a href='/new'>Quantum sensors</a></nav>",
    )
    .await;
    html(
        &server,
        "/new",
        &reference("Quantum sensors", "Newly published reference"),
    )
    .await;
    let result = web
        .search_with(request("latest quantum sensors"), &cancel)
        .await
        .unwrap();
    assert_ne!(result.discovery.stop_reason, "indexed_sources");
    assert!(result.results.iter().any(|hit| hit.url.ends_with("/new")));
    assert!(
        server
            .received_requests()
            .await
            .unwrap()
            .iter()
            .any(|r| r.url.path() == "/")
    );
}

#[tokio::test]
async fn background_priority_is_bounded_and_does_not_override_retry_backoff() {
    let (_dir, web, _) = fixture(empty_config()).await;
    web.index
        .enqueue(
            vec![
                "https://example.com/older".into(),
                "https://example.com/relevant".into(),
            ],
            0,
        )
        .await
        .unwrap();
    assert_eq!(
        web.index
            .prioritize(vec![
                "https://example.com/relevant".into(),
                "https://example.com/unobserved".into()
            ])
            .await
            .unwrap(),
        1
    );
    assert_eq!(
        web.index.next().await.unwrap().unwrap().0,
        "https://example.com/relevant"
    );
    web.index
        .finish("https://example.com/relevant", Some("Failed".into()))
        .await
        .unwrap();
    assert_eq!(
        web.index
            .prioritize(vec!["https://example.com/relevant".into()])
            .await
            .unwrap(),
        0
    );
    assert_eq!(
        web.index.next().await.unwrap().unwrap().0,
        "https://example.com/older"
    );
    assert!(web.index.next().await.unwrap().is_none());
}

#[tokio::test]
async fn a_foreground_read_satisfies_pending_crawl_work() {
    let (_dir, web, server) = fixture(empty_config()).await;
    robots(&server, "User-agent: *\nAllow: /\n").await;
    html(
        &server,
        "/reference",
        &reference("Quantum sensors", "Current version"),
    )
    .await;
    web.index
        .enqueue(vec!["https://example.com/reference".into()], 0)
        .await
        .unwrap();
    web.read_with_fresh(
        "https://example.com/reference",
        "quantum sensors",
        true,
        &CancellationToken::new(),
    )
    .await
    .unwrap();
    server.reset().await;
    assert!(!web.crawl_next().await.unwrap());
    assert!(server.received_requests().await.unwrap().is_empty());
}

#[tokio::test]
async fn many_observed_leads_cannot_grow_the_priority_queue_without_bound() {
    let (_dir, web, _) = fixture(empty_config()).await;
    let urls: Vec<_> = (0..320)
        .map(|i| format!("https://example.com/article-{i}"))
        .collect();
    web.index.enqueue(urls.clone(), 0).await.unwrap();
    for batch in urls.chunks(16) {
        web.index.prioritize(batch.to_vec()).await.unwrap();
    }
    assert_eq!(web.status().await.unwrap().coverage.priority_urls, 256);
    web.index.next().await.unwrap().unwrap();
    assert_eq!(web.status().await.unwrap().coverage.priority_urls, 255);
}

#[tokio::test]
async fn concurrent_origins_do_not_grow_the_http_client_pool_without_bound() {
    let (_dir, web, server) = fixture(empty_config()).await;
    robots(&server, "User-agent: *\nAllow: /\n").await;
    html(&server, "/reference", "<main>Public reference.</main>").await;
    let cancel = CancellationToken::new();
    let reads = futures_util::future::join_all((0..40).map(|i| {
        let web = &web;
        let cancel = &cancel;
        async move {
            web.read_with_fresh(
                &format!("https://origin-{i}.example.com/reference"),
                "",
                true,
                cancel,
            )
            .await
        }
    }))
    .await;
    assert!(reads.iter().all(Result::is_ok));
    assert_eq!(web.network.pooled_origins(), 32);
    assert_eq!(web.status().await.unwrap().active_downloads, 0);
}
