use super::*;
use std::net::IpAddr;
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{header, method, path},
};

pub(super) async fn fixture(mut config: Config) -> (tempfile::TempDir, Arc<Web>, MockServer) {
    let dir = tempfile::tempdir().unwrap();
    config.request_interval_ms = 100;
    std::fs::write(
        dir.path().join("web-search.json"),
        serde_json::to_vec(&config).unwrap(),
    )
    .unwrap();
    let server = MockServer::start().await;
    let mut web = Web::open(dir.path()).await.unwrap();
    // The production network policy cannot be disabled via configuration.
    Arc::get_mut(&mut web).unwrap().network.test_address = Some(*server.address());
    (dir, web, server)
}

pub(super) fn empty_config() -> Config {
    Config {
        seeds: vec![],
        ..Config::default()
    }
}

#[tokio::test]
async fn destination_policy_checks_initial_urls_robots_and_redirects() {
    let (_dir, web, server) = fixture(Config {
        allowed_hosts: Some(vec!["example.com".into()]),
        ..empty_config()
    })
    .await;
    robots(&server, "User-agent: *\nAllow: /\n").await;
    Mock::given(method("GET"))
        .and(path("/start"))
        .respond_with(
            ResponseTemplate::new(302).insert_header("Location", "https://other.example/escape"),
        )
        .mount(&server)
        .await;
    let cancel = CancellationToken::new();
    assert!(
        web.read("https://example.com/start", "", &cancel)
            .await
            .is_err()
    );
    assert!(
        web.read("https://other.example/direct", "", &cancel)
            .await
            .is_err()
    );
    let requests = server.received_requests().await.unwrap();
    assert!(requests.iter().any(|r| r.url.path() == "/start"));
    assert!(
        !requests
            .iter()
            .any(|r| matches!(r.url.path(), "/escape" | "/direct"))
    );
    // Test transport rewrites the host to the local fixture address.
    // A forbidden origin must not trigger even its robots lookup.
    assert_eq!(
        requests
            .iter()
            .filter(|r| r.url.path() == "/robots.txt")
            .count(),
        1
    );
}

#[test]
fn destination_policy_is_exact_and_empty_means_deny_all() {
    for host in [
        "*.example.com",
        "Example.com",
        "example.com/path",
        "example.com:443",
        "example.com.",
        "user@example.com",
        "127.0.0.1",
    ] {
        assert!(
            Config {
                allowed_hosts: Some(vec![host.into()]),
                ..empty_config()
            }
            .validate()
            .is_err(),
            "{host}"
        );
    }
    let config = Config {
        allowed_hosts: Some(vec!["example.com".into()]),
        ..empty_config()
    };
    assert!(config.validate().is_ok());
    assert!(
        config
            .authorize_url(&Url::parse("https://sub.example.com/").unwrap())
            .is_err()
    );
    assert!(
        config
            .authorize_url(&Url::parse("https://example.com.evil.test/").unwrap())
            .is_err()
    );
    assert!(
        config
            .authorize_url(&Url::parse("https://example.com/").unwrap())
            .is_ok()
    );
    assert!(
        Config {
            allowed_hosts: Some(vec![]),
            ..empty_config()
        }
        .authorize_url(&Url::parse("https://example.com/").unwrap())
        .is_err()
    );
}

pub(super) async fn robots(server: &MockServer, body: &str) {
    Mock::given(method("GET"))
        .and(path("/robots.txt"))
        .respond_with(ResponseTemplate::new(200).set_body_string(body))
        .mount(server)
        .await;
}

pub(super) async fn html(server: &MockServer, route: &str, body: &str) {
    Mock::given(method("GET"))
        .and(path(route))
        .respond_with(ResponseTemplate::new(200).set_body_raw(body, "text/html; charset=utf-8"))
        .mount(server)
        .await;
}

pub(super) fn page(url: &str, title: &str, text: &str) -> Page {
    let mut document =
        extract::extract(&Url::parse(url).unwrap(), text, "text/plain", 24_000).unwrap();
    document.title = title.into();
    Page {
        document,
        fetched_at: Utc::now().timestamp(),
        etag: None,
        modified: None,
    }
}

#[test]
fn url_policy_rejects_local_reserved_and_credential_targets() {
    for value in [
        "file:///etc/passwd",
        "ftp://example.com/",
        "http://localhost/",
        "http://a.local/",
        "http://127.0.0.1/",
        "http://2130706433/",
        "http://0x7f000001/",
        "http://[::1]/",
        "http://[::ffff:127.0.0.1]/",
        "http://169.254.169.254/",
        "http://100.64.0.1/",
        "http://10.0.0.1/",
        "http://172.16.1.2/",
        "http://192.168.0.1/",
        "http://192.0.2.1/",
        "http://[2001:db8::1]/",
        "http://[2002:7f00:1::]/",
        "http://[64:ff9b::7f00:1]/",
        "http://example.com:8080/",
        "https://user:password@example.com/",
        "https://example.com/?access_token=secret",
        "https://example.com/?%61pi_key=secret",
        "https://host.internal/",
    ] {
        assert!(
            fetch::public_url(value).is_err(),
            "unexpectedly accepted {value}"
        );
    }
    assert_eq!(
        fetch::public_url("https://example.com/a?q=caf%C3%A9#part")
            .unwrap()
            .as_str(),
        "https://example.com/a?q=caf%C3%A9"
    );
    for value in ["8.8.8.8", "93.184.216.34", "2606:4700:4700::1111"] {
        assert!(fetch::public_ip(value.parse::<IpAddr>().unwrap()));
    }
}

#[test]
fn html_extraction_keeps_article_evidence_and_filters_hidden_content() {
    let document = extract::extract(&Url::parse("https://example.com/start").unwrap(),
        r#"<html lang="es"><head><title>Órbita &amp; ciencia</title></head><body><nav>Navigation</nav>
        <main><h1>Café científico</h1><p>Evidence &amp; sources.</p><script>executeBadCode()</script>
        <div hidden>secret hidden</div><span aria-hidden="true">invisible</span>
        <span style="display: none">also hidden</span><a href="/evidence#part">Read more</a>
        <a href="https://second.example/paper">Another source</a><a href="/spam" rel="nofollow">Spam</a>
        <a href="http://127.0.0.1/">Local</a></main><footer>Footer</footer></body></html>"#,
        "application/xhtml+xml", 1000).unwrap();
    assert_eq!(document.title, "Órbita & ciencia");
    assert_eq!(document.language.as_deref(), Some("es"));
    assert!(
        document
            .text
            .contains("Café científico\nEvidence & sources.")
    );
    for excluded in [
        "executeBadCode",
        "secret hidden",
        "invisible",
        "also hidden",
        "Navigation",
        "Footer",
    ] {
        assert!(!document.text.contains(excluded), "leaked {excluded}");
    }
    assert_eq!(
        document.links,
        [
            "https://example.com/evidence",
            "https://second.example/paper"
        ]
    );
    let limited = extract::extract(
        &Url::parse("https://example.com/").unwrap(),
        &"á".repeat(500),
        "text/plain",
        256,
    )
    .unwrap();
    assert_eq!(limited.text.chars().count(), 256);
    assert!(limited.truncated);
    assert_eq!(
        extract::passage("Texto con café y ciencia", "ciencia", 12)
            .chars()
            .count(),
        12
    );
    assert!(
        extract::extract(
            &Url::parse("https://example.com/").unwrap(),
            "%PDF",
            "application/pdf",
            256
        )
        .is_err()
    );
}

#[test]
fn extraction_honors_meta_policies_and_bounded_sitemaps() {
    let url = Url::parse("https://example.com/sitemap.xml").unwrap();
    let document = extract::extract(&url,
        r#"<meta name="ROBOTS" content="noindex, nofollow"><main>Hidden from index</main><a href="/a">Link</a>"#,
        "text/html", 256).unwrap();
    assert!(!document.indexable);
    assert!(document.links.is_empty());
    let sitemap = extract::extract(&url,
        r#"<?xml version="1.0"?><urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9"><url><loc>https://example.com/article?a=1&amp;b=2</loc></url><url><loc>https://other.example/spam</loc></url></urlset>"#,
        "application/xml", 256).unwrap();
    assert!(!sitemap.indexable);
    assert_eq!(sitemap.links, ["https://example.com/article?a=1&b=2"]);
    let complex = format!("{}text{}", "<div>".repeat(20_001), "</div>".repeat(20_001));
    assert!(extract::extract(&url, &complex, "text/html", 256).is_err());
}

#[tokio::test]
async fn crawl_discovers_new_domains_then_search_reads_actual_pages() {
    let (_dir, web, server) = fixture(Config {
        seeds: vec!["http://example.com/start".into()],
        cache_minutes: 0,
        ..empty_config()
    })
    .await;
    robots(
        &server,
        "User-agent: *\nAllow: /\nSitemap: http://example.com/sitemap.xml\n",
    )
    .await;
    html(&server, "/start", "<title>Science</title><main>Space research portal</main><a href='http://second.example/paper'>Paper</a>").await;
    Mock::given(method("GET"))
        .and(path("/sitemap.xml"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            "<urlset><url><loc>http://example.com/botany</loc></url></urlset>",
            "application/xml",
        ))
        .mount(&server)
        .await;
    html(
        &server,
        "/paper",
        "<title>Astronomy</title><main>Orbital evidence from telescope observations.</main>",
    )
    .await;
    html(
        &server,
        "/botany",
        "<title>Botany</title><main>Plants use sunlight in photosynthesis.</main>",
    )
    .await;
    let stop = CancellationToken::new();
    let worker = tokio::spawn(web.clone().run(stop.clone()));
    tokio::time::timeout(Duration::from_secs(5), async {
        while web.status().await.unwrap().coverage.indexed_pages < 3 {
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    })
    .await
    .expect("The background worker must discover and index the linked pages");
    stop.cancel();
    worker.await.unwrap();
    let status = web.status().await.unwrap();
    assert_eq!(status.coverage.indexed_pages, 3);
    assert_eq!(status.coverage.indexed_origins, 2);
    let result = web
        .search("orbital telescope", 5, 3, &CancellationToken::new())
        .await
        .unwrap();
    assert_eq!(result.results.len(), 1);
    let hit = &result.results[0];
    assert_eq!(hit.url, "http://second.example/paper");
    assert_eq!(hit.read_status, "network");
    assert!(
        hit.page
            .as_ref()
            .unwrap()
            .passage
            .contains("Orbital evidence")
    );
    assert_eq!(result.backend, "local_fts5");
    let requests = server.received_requests().await.unwrap();
    assert_eq!(
        requests.iter().filter(|r| r.url.path() == "/paper").count(),
        2
    );
    assert_eq!(
        requests
            .iter()
            .filter(|r| r.url.path() == "/robots.txt")
            .count(),
        2
    );
    assert!(
        requests
            .iter()
            .all(|r| r.headers.get("cookie").is_none() && r.headers.get("authorization").is_none())
    );
}

#[tokio::test]
async fn local_search_handles_accents_deduplication_updates_and_capacity() {
    let (dir, web, _server) = fixture(Config {
        max_pages: 2,
        max_frontier: 3,
        max_urls_per_origin: 2,
        ..empty_config()
    })
    .await;
    web.index
        .save(page(
            "https://one.example/a",
            "Café",
            "Información sobre café peruano.",
        ))
        .await
        .unwrap();
    web.index
        .save(page(
            "https://two.example/b",
            "Mirror",
            "Información sobre café peruano.",
        ))
        .await
        .unwrap();
    web.index
        .save(page(
            "https://three.example/c",
            "Extra",
            "must not be stored",
        ))
        .await
        .unwrap();
    assert_eq!(web.index.coverage().await.unwrap().indexed_pages, 2);
    assert_eq!(web.index.search("cafe", 5).await.unwrap().len(), 1);
    assert!(
        web.index
            .search("\" ) OR unknown : *", 5)
            .await
            .unwrap()
            .is_empty()
    );
    web.index
        .save(page(
            "https://one.example/a",
            "Revised",
            "New botanical evidence",
        ))
        .await
        .unwrap();
    assert_eq!(web.index.search("botanical", 5).await.unwrap().len(), 1);
    assert_eq!(web.index.search("cafe", 5).await.unwrap().len(), 1);
    web.index
        .enqueue(
            vec![
                "https://one.example/a".into(),
                "https://one.example/b".into(),
                "https://one.example/c".into(),
                "https://two.example/b".into(),
                "https://three.example/c".into(),
            ],
            0,
        )
        .await
        .unwrap();
    assert_eq!(web.status().await.unwrap().coverage.known_urls, 3);
    let first = web.index.next().await.unwrap().unwrap();
    let second = web.index.next().await.unwrap().unwrap();
    assert_ne!(first.0, second.0);
    assert!(web.index.next().await.unwrap().is_none());
    drop(web);
    let reopened = Web::open(dir.path()).await.unwrap();
    assert_eq!(
        reopened.index.search("botanical", 5).await.unwrap().len(),
        1
    );
    assert!(reopened.status().await.unwrap().coverage.page_limit_reached);
    reopened
        .index
        .remove("https://two.example/b")
        .await
        .unwrap();
    assert!(reopened.index.search("cafe", 5).await.unwrap().is_empty());
}

#[tokio::test]
async fn reading_uses_persistent_cache_and_conditional_revalidation() {
    let (dir, web, server) = fixture(empty_config()).await;
    robots(&server, "User-agent: *\nAllow: /\n").await;
    Mock::given(method("GET"))
        .and(path("/article"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("etag", "\"v1\"")
                .set_body_raw(
                    "<title>Evidence</title><main>A stable scientific observation</main>",
                    "text/html",
                ),
        )
        .mount(&server)
        .await;
    let token = CancellationToken::new();
    let first = web
        .read("http://example.com/article", "", &token)
        .await
        .unwrap();
    assert_eq!(first.source, "network");
    assert_eq!(
        web.read("http://example.com/article", "", &token)
            .await
            .unwrap()
            .source,
        "cache"
    );
    assert_eq!(server.received_requests().await.unwrap().len(), 2);
    drop(web);
    let mut web = Web::open(dir.path()).await.unwrap();
    Arc::get_mut(&mut web).unwrap().network.test_address = Some(*server.address());
    assert_eq!(
        web.read("http://example.com/article", "", &token)
            .await
            .unwrap()
            .source,
        "cache"
    );
    web.index
        .call(|db| {
            db.execute("UPDATE pages SET fetched_at=1", [])?;
            Ok(())
        })
        .await
        .unwrap();
    server.reset().await;
    Mock::given(method("GET"))
        .and(path("/article"))
        .and(header("if-none-match", "\"v1\""))
        .respond_with(ResponseTemplate::new(304))
        .expect(1)
        .mount(&server)
        .await;
    let checked = web
        .read("http://example.com/article", "", &token)
        .await
        .unwrap();
    assert_eq!(checked.source, "revalidated");
    assert_eq!(checked.content_hash, first.content_hash);
    assert_eq!(checked.passage, first.passage);
    assert_eq!(server.received_requests().await.unwrap().len(), 1);
}

#[tokio::test]
async fn robots_denial_and_temporary_failure_never_fetch_the_article() {
    let (_dir, web, server) = fixture(empty_config()).await;
    robots(
        &server,
        "User-agent: FenixBot\nDisallow: /private\nAllow: /private/public$\n",
    )
    .await;
    let token = CancellationToken::new();
    assert!(
        web.read("http://example.com/private", "", &token)
            .await
            .unwrap_err()
            .to_string()
            .contains("robots.txt")
    );
    html(&server, "/private/public", "<main>Public evidence</main>").await;
    assert!(
        web.read("http://example.com/private/public", "", &token)
            .await
            .is_ok()
    );
    assert!(
        !server
            .received_requests()
            .await
            .unwrap()
            .iter()
            .any(|r| r.url.path() == "/private")
    );
    server.reset().await;
    Mock::given(method("GET"))
        .and(path("/robots.txt"))
        .respond_with(ResponseTemplate::new(503))
        .expect(1)
        .mount(&server)
        .await;
    assert!(
        web.read("http://unavailable.example/article", "", &token)
            .await
            .is_err()
    );
    assert!(
        web.read("http://unavailable.example/article", "", &token)
            .await
            .is_err()
    );
    assert_eq!(server.received_requests().await.unwrap().len(), 1);
}

#[tokio::test]
async fn redirects_recheck_policy_and_never_contact_private_targets() {
    let (_dir, web, server) = fixture(empty_config()).await;
    robots(&server, "User-agent: *\nDisallow: /blocked\n").await;
    for (route, target) in [
        ("/local", "http://127.0.0.1/secret"),
        ("/redirect", "http://second.example/blocked"),
        ("/loop", "/loop"),
        ("/moved", "http://second.example/new-source"),
    ] {
        Mock::given(method("GET"))
            .and(path(route))
            .respond_with(ResponseTemplate::new(302).insert_header("location", target))
            .mount(&server)
            .await;
    }
    let token = CancellationToken::new();
    assert!(
        web.read("http://example.com/local", "", &token)
            .await
            .unwrap_err()
            .to_string()
            .contains("Private")
    );
    assert!(
        web.read("http://example.com/redirect", "", &token)
            .await
            .unwrap_err()
            .to_string()
            .contains("robots.txt")
    );
    assert!(
        web.read("http://example.com/loop", "", &token)
            .await
            .unwrap_err()
            .to_string()
            .contains("Too many redirects")
    );
    let mut old = page("http://example.com/moved", "Old URL", "Old source evidence");
    old.fetched_at = 1;
    old.etag = Some("\"original\"".into());
    web.index.save(old).await.unwrap();
    Mock::given(method("GET"))
        .and(path("/new-source"))
        .respond_with(ResponseTemplate::new(304))
        .mount(&server)
        .await;
    assert!(
        web.read("http://example.com/moved", "", &token)
            .await
            .unwrap_err()
            .to_string()
            .contains("without matching cached content")
    );
    let requests = server.received_requests().await.unwrap();
    assert!(
        requests
            .iter()
            .find(|r| r.url.path() == "/new-source")
            .unwrap()
            .headers
            .get("if-none-match")
            .is_none()
    );
    assert!(
        !server
            .received_requests()
            .await
            .unwrap()
            .iter()
            .any(|r| matches!(r.url.path(), "/secret" | "/blocked"))
    );
}

#[tokio::test]
async fn response_limits_noindex_nofollow_and_deleted_pages_are_respected() {
    let (_dir, web, server) = fixture(Config {
        max_body_bytes: 1024,
        cache_minutes: 0,
        ..empty_config()
    })
    .await;
    robots(&server, "User-agent: *\nAllow: /\n").await;
    html(&server, "/large", &"x".repeat(1025)).await;
    let token = CancellationToken::new();
    assert!(
        web.read("http://example.com/large", "", &token)
            .await
            .unwrap_err()
            .to_string()
            .contains("byte limit")
    );
    Mock::given(method("GET"))
        .and(path("/optout"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("x-robots-tag", "noindex, nofollow")
                .set_body_raw(
                    "<main>Not indexable</main><a href='/should-not-follow'>Link</a>",
                    "text/html",
                ),
        )
        .mount(&server)
        .await;
    web.index
        .save(page(
            "http://example.com/optout",
            "Old",
            "Withdrawn evidence",
        ))
        .await
        .unwrap();
    assert!(
        web.read("http://example.com/optout", "", &token)
            .await
            .is_err()
    );
    assert!(
        web.index
            .get("http://example.com/optout")
            .await
            .unwrap()
            .is_none()
    );
    assert_eq!(web.index.coverage().await.unwrap().known_urls, 1);
    web.index
        .save(page(
            "http://example.com/deleted",
            "Old",
            "Deleted evidence",
        ))
        .await
        .unwrap();
    assert!(
        web.read("http://example.com/deleted", "", &token)
            .await
            .unwrap_err()
            .to_string()
            .contains("404")
    );
    assert!(web.index.search("Deleted", 5).await.unwrap().is_empty());
}

#[tokio::test]
async fn daily_request_budget_survives_restart_and_keeps_cached_search_available() {
    let (dir, web, server) = fixture(Config {
        requests_per_day: 2,
        cache_minutes: 0,
        ..empty_config()
    })
    .await;
    robots(&server, "User-agent: *\nAllow: /\n").await;
    html(&server, "/article", "<main>Available evidence</main>").await;
    let token = CancellationToken::new();
    web.read("http://example.com/article", "", &token)
        .await
        .unwrap();
    assert!(
        web.read("http://example.com/another", "", &token)
            .await
            .unwrap_err()
            .to_string()
            .contains("Daily web request budget")
    );
    assert_eq!(server.received_requests().await.unwrap().len(), 2);
    let result = web.search("evidence", 5, 3, &token).await.unwrap();
    assert_eq!(result.results[0].read_status, "failed");
    assert!(result.results[0].page.is_none());
    assert!(result.warnings.iter().any(|s| s.contains("older evidence")));
    drop(web);
    let web = Web::open(dir.path()).await.unwrap();
    assert_eq!(web.status().await.unwrap().coverage.requests_today, 2);
    assert!(!web.crawl_next().await.unwrap());
    assert_eq!(
        web.search("evidence", 5, 0, &token)
            .await
            .unwrap()
            .results
            .len(),
        1
    );
}

#[tokio::test]
async fn cancellation_interrupts_fetch_and_stops_background_worker() {
    let (_dir, web, server) = fixture(empty_config()).await;
    robots(&server, "User-agent: *\nAllow: /\n").await;
    Mock::given(method("GET"))
        .and(path("/slow"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_delay(Duration::from_secs(5))
                .set_body_raw("<main>Slow</main>", "text/html"),
        )
        .mount(&server)
        .await;
    let token = CancellationToken::new();
    let reader = web.clone();
    let cancel = token.clone();
    let task =
        tokio::spawn(async move { reader.read("http://example.com/slow", "", &cancel).await });
    tokio::time::sleep(Duration::from_millis(150)).await;
    token.cancel();
    assert!(
        tokio::time::timeout(Duration::from_secs(1), task)
            .await
            .unwrap()
            .unwrap()
            .unwrap_err()
            .to_string()
            .contains("cancelled")
    );
    let stop = CancellationToken::new();
    let worker = tokio::spawn(web.clone().run(stop.clone()));
    tokio::task::yield_now().await;
    stop.cancel();
    tokio::time::timeout(Duration::from_secs(1), worker)
        .await
        .unwrap()
        .unwrap();
    assert!(!web.status().await.unwrap().crawler_running);
    assert_eq!(web.status().await.unwrap().coverage.indexed_pages, 0);
}

#[tokio::test]
async fn empty_search_is_honest_and_disabled_web_performs_no_requests() {
    let (_dir, web, server) = fixture(empty_config()).await;
    let token = CancellationToken::new();
    let result = web.search("anything", 5, 3, &token).await.unwrap();
    assert!(result.results.is_empty());
    assert!(
        result
            .warnings
            .iter()
            .any(|w| w.contains("No matching indexed pages"))
    );
    assert!(web.search("   ", 5, 0, &token).await.is_err());
    assert!(web.search("test", 0, 0, &token).await.is_err());
    assert!(web.search("test", 5, 4, &token).await.is_err());
    assert!(server.received_requests().await.unwrap().is_empty());
    let (_dir, disabled, server) = fixture(Config {
        enabled: false,
        ..empty_config()
    })
    .await;
    disabled.clone().run(token.clone()).await;
    assert!(disabled.search("test", 5, 0, &token).await.is_err());
    assert!(
        disabled
            .read("http://example.com/", "", &token)
            .await
            .is_err()
    );
    assert!(!disabled.status().await.unwrap().crawler_running);
    assert!(server.received_requests().await.unwrap().is_empty());
}

#[tokio::test]
async fn gzip_limits_apply_after_decompression() {
    let (_dir, web, server) = fixture(Config {
        max_body_bytes: 1024,
        ..empty_config()
    })
    .await;
    robots(&server, "User-agent: *\nAllow: /\n").await;
    let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
    encoder.write_all(&vec![b'x'; 8192]).unwrap();
    let compressed = encoder.finish().unwrap();
    assert!(compressed.len() < 1024);
    Mock::given(method("GET"))
        .and(path("/compressed"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-encoding", "gzip")
                .set_body_raw(compressed, "text/plain"),
        )
        .mount(&server)
        .await;
    let error = web
        .read(
            "http://example.com/compressed",
            "",
            &CancellationToken::new(),
        )
        .await
        .unwrap_err();
    assert!(error.to_string().contains("byte limit"));
    assert_eq!(web.status().await.unwrap().coverage.indexed_pages, 0);
}

#[tokio::test]
async fn revalidation_can_withdraw_previously_indexed_content() {
    let (_dir, web, server) = fixture(Config {
        cache_minutes: 0,
        ..empty_config()
    })
    .await;
    robots(&server, "User-agent: *\nAllow: /\n").await;
    let mut previous = page("http://example.com/article", "Old", "Withdrawn observation");
    previous.etag = Some("\"old\"".into());
    web.index.save(previous).await.unwrap();
    Mock::given(method("GET"))
        .and(path("/article"))
        .and(header("if-none-match", "\"old\""))
        .respond_with(ResponseTemplate::new(304).insert_header("x-robots-tag", "noindex"))
        .mount(&server)
        .await;
    assert!(
        web.read("http://example.com/article", "", &CancellationToken::new())
            .await
            .is_err()
    );
    assert!(web.index.search("observation", 5).await.unwrap().is_empty());
}

async fn wait_for_operations(web: &Web, active: usize, queued: usize, downloads: usize) {
    tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            let s = web.status().await.unwrap();
            if s.active_operations == active
                && s.queued_operations == queued
                && s.active_downloads == downloads
            {
                break;
            }
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    })
    .await
    .expect("Expected bounded operation/download state");
}

#[tokio::test]
async fn ten_searches_download_concurrently_and_eleventh_waits_for_capacity() {
    let (_dir, web, server) = fixture(Config {
        cache_minutes: 0,
        ..empty_config()
    })
    .await;
    let mut tasks = Vec::new();
    let mut cancels = Vec::new();
    for i in 0..10 {
        let origin = format!("http://topic{i}.example");
        let url = format!("{origin}/article{i}");
        let query = format!("topic{i}");
        web.index
            .save_robots(&origin, "User-agent: *\nAllow: /\n".into(), 3600)
            .await
            .unwrap();
        web.index.save(page(&url, &query, &query)).await.unwrap();
        Mock::given(method("GET"))
            .and(path(format!("/article{i}")))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_delay(Duration::from_secs(2))
                    .set_body_raw(format!("<main>{query}</main>"), "text/html"),
            )
            .expect(1)
            .mount(&server)
            .await;
        let reader = web.clone();
        let cancel = CancellationToken::new();
        cancels.push(cancel.clone());
        tasks.push(tokio::spawn(async move {
            reader.search(&query, 1, 1, &cancel).await
        }));
    }
    wait_for_operations(&web, 10, 0, 10).await;
    tokio::time::timeout(Duration::from_secs(1), async {
        while server.received_requests().await.unwrap().len() < 10 {
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    })
    .await
    .expect("All ten downloads reach the server before cancellation");
    let reader = web.clone();
    let queued = tokio::spawn(async move {
        reader
            .search("no-match", 1, 0, &CancellationToken::new())
            .await
    });
    wait_for_operations(&web, 10, 1, 10).await;
    assert!(!queued.is_finished());
    // A user cancellation releases both admission and network capacity promptly.
    cancels[0].cancel();
    let result = tokio::time::timeout(Duration::from_secs(1), queued)
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    assert_eq!(result.results.len(), 0);
    for (i, task) in tasks.into_iter().enumerate() {
        let result = task.await.unwrap();
        if i == 0 {
            assert!(result.is_err());
        } else {
            assert_eq!(result.unwrap().results[0].read_status, "network");
        }
    }
    wait_for_operations(&web, 0, 0, 0).await;
    assert_eq!(web.status().await.unwrap().max_concurrent_searches, 10);
}

#[tokio::test]
async fn ten_duplicate_reads_share_one_fetch_even_with_cache_disabled() {
    let (_dir, web, server) = fixture(Config {
        cache_minutes: 0,
        ..empty_config()
    })
    .await;
    robots(&server, "User-agent: *\nAllow: /\n").await;
    Mock::given(method("GET"))
        .and(path("/shared"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_delay(Duration::from_millis(200))
                .set_body_raw("<main>Shared evidence</main>", "text/html"),
        )
        .mount(&server)
        .await;
    let mut tasks = Vec::new();
    for _ in 0..10 {
        let reader = web.clone();
        tasks.push(tokio::spawn(async move {
            reader
                .read("http://example.com/shared", "", &CancellationToken::new())
                .await
                .unwrap()
        }));
    }
    let mut shared = 0;
    for task in tasks {
        shared += usize::from(task.await.unwrap().source == "shared");
    }
    assert_eq!(shared, 9);
    let requests = server.received_requests().await.unwrap();
    assert_eq!(
        requests
            .iter()
            .filter(|r| r.url.path() == "/robots.txt")
            .count(),
        1
    );
    assert_eq!(
        requests
            .iter()
            .filter(|r| r.url.path() == "/shared")
            .count(),
        1
    );
    // Completed flights are not a hidden cache that ignores cache_minutes=0.
    assert_eq!(
        web.read("http://example.com/shared", "", &CancellationToken::new())
            .await
            .unwrap()
            .source,
        "network"
    );
    assert_eq!(
        server
            .received_requests()
            .await
            .unwrap()
            .iter()
            .filter(|r| r.url.path() == "/shared")
            .count(),
        2
    );
}

#[tokio::test]
async fn cancelled_flight_leader_allows_a_waiter_to_finish_without_detached_work() {
    let (_dir, web, server) = fixture(Config {
        cache_minutes: 0,
        ..empty_config()
    })
    .await;
    robots(&server, "User-agent: *\nAllow: /\n").await;
    Mock::given(method("GET"))
        .and(path("/shared"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_delay(Duration::from_millis(350))
                .set_body_raw("<main>Surviving reader</main>", "text/html"),
        )
        .mount(&server)
        .await;
    let cancel = CancellationToken::new();
    let reader = web.clone();
    let token = cancel.clone();
    let leader =
        tokio::spawn(async move { reader.read("http://example.com/shared", "", &token).await });
    tokio::time::timeout(Duration::from_secs(3), async {
        while !server
            .received_requests()
            .await
            .unwrap()
            .iter()
            .any(|r| r.url.path() == "/shared")
        {
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    })
    .await
    .unwrap();
    let reader = web.clone();
    let follower = tokio::spawn(async move {
        reader
            .read("http://example.com/shared", "", &CancellationToken::new())
            .await
    });
    wait_for_operations(&web, 2, 0, 1).await;
    cancel.cancel();
    assert!(leader.await.unwrap().is_err());
    assert!(
        follower
            .await
            .unwrap()
            .unwrap()
            .passage
            .contains("Surviving reader")
    );
    wait_for_operations(&web, 0, 0, 0).await;
}

#[tokio::test]
async fn queued_cancellation_does_not_fetch_and_background_yields_to_foreground() {
    let (_dir, web, server) = fixture(Config {
        seeds: vec!["http://example.com/start".into()],
        ..empty_config()
    })
    .await;
    robots(&server, "User-agent: *\nAllow: /\n").await;
    html(&server, "/start", "<main>Background article</main>").await;
    let held = web.operations.acquire_many(10).await.unwrap();
    let cancel = CancellationToken::new();
    let token = cancel.clone();
    let reader = web.clone();
    let queued = tokio::spawn(async move { reader.search("article", 5, 3, &token).await });
    wait_for_operations(&web, 10, 1, 0).await;
    assert!(!web.crawl_next().await.unwrap());
    cancel.cancel();
    assert!(
        queued
            .await
            .unwrap()
            .unwrap_err()
            .to_string()
            .contains("cancelled")
    );
    assert!(server.received_requests().await.unwrap().is_empty());
    drop(held);
    assert!(web.crawl_next().await.unwrap());
    assert_eq!(web.status().await.unwrap().coverage.indexed_pages, 1);
}

#[tokio::test]
async fn search_reads_parallel_sources_with_a_global_download_cap() {
    let (_dir, web, server) = fixture(Config {
        max_concurrent_searches: 2,
        cache_minutes: 0,
        ..empty_config()
    })
    .await;
    for i in 0..3 {
        let origin = format!("http://source{i}.example");
        web.index
            .save_robots(&origin, "User-agent: *\nAllow: /\n".into(), 3600)
            .await
            .unwrap();
        web.index
            .save(page(
                &format!("{origin}/source{i}"),
                "Evidence",
                &format!("Concurrent evidence number {i}"),
            ))
            .await
            .unwrap();
        Mock::given(method("GET"))
            .and(path(format!("/source{i}")))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_delay(Duration::from_millis(300))
                    .set_body_raw(format!("<main>Concurrent evidence {i}</main>"), "text/html"),
            )
            .mount(&server)
            .await;
    }
    let reader = web.clone();
    let task = tokio::spawn(async move {
        reader
            .search("concurrent evidence", 5, 3, &CancellationToken::new())
            .await
            .unwrap()
    });
    wait_for_operations(&web, 1, 0, 2).await;
    while !task.is_finished() {
        assert!(web.network.active_downloads() <= 2);
        tokio::time::sleep(Duration::from_millis(5)).await;
    }
    let result = task.await.unwrap();
    assert_eq!(result.results.len(), 3);
    assert!(result.results.iter().all(|r| r.read_status == "network"));
    wait_for_operations(&web, 0, 0, 0).await;
}

#[tokio::test]
async fn a_busy_origin_does_not_block_other_origins() {
    let (_dir, web, server) = fixture(Config {
        max_concurrent_searches: 2,
        cache_minutes: 0,
        ..empty_config()
    })
    .await;
    for origin in ["http://busy.example", "http://fast.example"] {
        web.index
            .save_robots(origin, "User-agent: *\nAllow: /\n".into(), 3600)
            .await
            .unwrap();
    }
    for i in 0..3 {
        web.index
            .save(page(
                &format!("http://busy.example/slow{i}"),
                "slow",
                &format!("slow evidence {i}"),
            ))
            .await
            .unwrap();
        Mock::given(method("GET"))
            .and(path(format!("/slow{i}")))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_delay(Duration::from_secs(3))
                    .set_body_raw("<main>slow evidence</main>", "text/html"),
            )
            .mount(&server)
            .await;
    }
    html(&server, "/fast", "<main>Fast evidence</main>").await;
    let cancel = CancellationToken::new();
    let search_cancel = cancel.clone();
    let reader = web.clone();
    let busy = tokio::spawn(async move { reader.search("slow", 3, 3, &search_cancel).await });
    wait_for_operations(&web, 1, 0, 1).await;
    let fast = tokio::time::timeout(
        Duration::from_secs(1),
        web.read("http://fast.example/fast", "", &CancellationToken::new()),
    )
    .await
    .expect("Other origins retain capacity while three reads wait on one origin")
    .unwrap();
    assert_eq!(fast.source, "network");
    assert!(!busy.is_finished());
    cancel.cancel();
    assert!(busy.await.unwrap().is_err());
    wait_for_operations(&web, 0, 0, 0).await;
}

#[tokio::test]
async fn same_origin_requests_are_spaced_and_do_not_charge_budget_while_waiting() {
    let (_dir, web, server) = fixture(empty_config()).await;
    web.index
        .save_robots(
            "http://example.com",
            "User-agent: *\nAllow: /\n".into(),
            3600,
        )
        .await
        .unwrap();
    let starts = Arc::new(Mutex::new(Vec::new()));
    let observed = starts.clone();
    Mock::given(method("GET"))
        .respond_with(move |_: &wiremock::Request| {
            observed.lock().unwrap().push(std::time::Instant::now());
            ResponseTemplate::new(200).set_body_raw("<main>Evidence</main>", "text/html")
        })
        .mount(&server)
        .await;
    let token = CancellationToken::new();
    let (a, b) = tokio::join!(
        web.read("http://example.com/a", "", &token),
        web.read("http://example.com/b", "", &token)
    );
    a.unwrap();
    b.unwrap();
    {
        let starts = starts.lock().unwrap();
        assert_eq!(starts.len(), 2);
        assert!(starts[1].duration_since(starts[0]) >= Duration::from_millis(90));
    }
    assert_eq!(web.status().await.unwrap().coverage.requests_today, 2);
}

#[test]
fn older_configs_default_to_ten_and_cannot_exceed_the_limit() {
    let old: Config = serde_json::from_str("{\"enabled\":true}").unwrap();
    assert_eq!(old.max_concurrent_searches, 10);
    assert!(
        Config {
            max_concurrent_searches: 11,
            ..empty_config()
        }
        .validate()
        .is_err()
    );
    assert!(
        Config {
            max_concurrent_searches: 0,
            ..empty_config()
        }
        .validate()
        .is_err()
    );
}

#[test]
fn publication_dates_are_explicit_and_never_inferred_from_fetch_or_time_tags() {
    let url = Url::parse("https://example.com/article").unwrap();
    for (html, expected) in [
        (
            r#"<meta property="article:published_time" content="2025-03-02T14:00:00+02:00"><main>Evidence</main>"#,
            Some("2025-03-02T12:00:00+00:00"),
        ),
        (
            r#"<meta itemprop="datePublished" content="2025-03-02"><main>Evidence</main>"#,
            Some("2025-03-02T00:00:00+00:00"),
        ),
        (
            r#"<script type="application/ld+json">{"@type":"NewsArticle","datePublished":"2025-03-02"}</script><main>Evidence</main>"#,
            Some("2025-03-02T00:00:00+00:00"),
        ),
        (
            r#"<time datetime="2025-03-02">Updated recently</time><main>Evidence</main>"#,
            None,
        ),
        (
            r#"<meta property="article:modified_time" content="2025-03-02"><main>Evidence</main>"#,
            None,
        ),
        (
            r#"<meta property="article:published_time" content="2025-02-30"><main>Evidence</main>"#,
            None,
        ),
        (
            r#"<meta property="article:published_time" content="2999-01-01"><main>Evidence</main>"#,
            None,
        ),
        (
            r#"<script type="application/ld+json">{"comments":[{"datePublished":"2025-03-02"}]}</script>"#,
            None,
        ),
    ] {
        let document = extract::extract(&url, html, "text/html", 1024).unwrap();
        assert_eq!(
            document.published_at.map(timestamp).as_deref(),
            expected,
            "{html}"
        );
    }
}

#[test]
fn feeds_only_discover_public_originals_without_indexing_excerpts() {
    let url = Url::parse("https://feed.example/rss").unwrap();
    for body in [
        r#"<rss><channel><link>https://feed.example/</link><item><title>Not article evidence</title><description>Never index feed excerpts</description><pubDate>Fri, 18 Sep 2026 00:00:00 GMT</pubDate><link>https://source.example/post?a=1&amp;b=2</link></item><item><link>http://127.0.0.1/private</link></item></channel></rss>"#,
        r#"<feed xmlns="http://www.w3.org/2005/Atom"><link href="https://feed.example/"/><entry><title>Not article evidence</title><updated>2026-09-18T00:00:00Z</updated><link rel="self" href="https://feed.example/item.xml"/><link rel="alternate" href="https://source.example/post?a=1&amp;b=2"/></entry></feed>"#,
    ] {
        let document = extract::extract(&url, body, "application/atom+xml", 1024).unwrap();
        assert!(!document.indexable);
        assert!(document.text.is_empty());
        assert!(document.published_at.is_none());
        assert_eq!(document.links, ["https://source.example/post?a=1&b=2"]);
    }
}

#[tokio::test]
async fn search_filters_dates_domains_duplicates_and_diversity_without_network() {
    let (_dir, web, server) = fixture(empty_config()).await;
    let now = Utc::now().timestamp();
    for (url, text, publication) in [
        (
            "https://example.com/one",
            "Orbital systems primary evidence",
            Some(now - 3600),
        ),
        (
            "https://example.com/two",
            "Orbital systems secondary evidence",
            Some(now - 3600),
        ),
        (
            "https://news.example.com/three",
            "Orbital systems third evidence",
            Some(now - 3600),
        ),
        (
            "https://unrelated.example/copy",
            "Orbital systems primary evidence",
            Some(now - 3600),
        ),
        (
            "https://other.example/unique",
            "Orbital systems independent evidence",
            Some(now - 3600),
        ),
        (
            "https://badexample.com/not-subdomain",
            "Orbital systems misleading domain",
            Some(now - 3600),
        ),
        (
            "https://example.com.evil.example/not-subdomain",
            "Orbital systems misleading suffix",
            Some(now - 3600),
        ),
        (
            "https://example.com/old",
            "Orbital systems old evidence",
            Some(now - 60 * 86_400),
        ),
        (
            "https://example.com/undated",
            "Orbital systems unknown date evidence",
            None,
        ),
        (
            "https://example.com/future",
            "Orbital systems future evidence",
            Some(now + 86_400),
        ),
    ] {
        let mut page = page(url, "Orbital systems", text);
        page.document.published_at = publication;
        web.index.save(page).await.unwrap();
    }
    let cancel = CancellationToken::new();
    let request = SearchRequest {
        query: "What are the orbital systems".into(),
        domains: vec!["EXAMPLE.COM.".into()],
        days: Some(30),
        limit: 10,
        read_pages: 0,
        max_per_domain: 10,
        ..SearchRequest::default()
    };
    let response = web.search_with(request.clone(), &cancel).await.unwrap();
    assert_eq!(response.results.len(), 3);
    assert!(
        response
            .results
            .iter()
            .all(|hit| hit.published_at.is_some())
    );
    assert!(
        response
            .results
            .iter()
            .all(|hit| !hit.url.contains("evil") && !hit.url.contains("badexample"))
    );
    let diverse = web
        .search_with(
            SearchRequest {
                max_per_domain: 2,
                ..request.clone()
            },
            &cancel,
        )
        .await
        .unwrap();
    assert_eq!(
        diverse.results.len(),
        2,
        "Requested domain includes its subdomains for diversity"
    );
    let all = web
        .search_with(
            SearchRequest {
                domains: vec![],
                max_per_domain: 10,
                ..request
            },
            &cancel,
        )
        .await
        .unwrap();
    assert_eq!(
        all.results.len(),
        6,
        "Identical content is deduplicated across domains"
    );
    assert_eq!(all.coverage.requests_today, 0);
    assert!(server.received_requests().await.unwrap().is_empty());
}

#[tokio::test]
async fn explicit_source_reads_populate_own_index_and_report_partial_failures_and_feeds() {
    let (_dir, web, server) = fixture(empty_config()).await;
    robots(&server, "User-agent: *\nAllow: /\nDisallow: /private\n").await;
    html(
        &server,
        "/article",
        "<title>Original evidence</title><main>Orbital systems independent research.</main>",
    )
    .await;
    Mock::given(method("GET")).and(path("/feed")).respond_with(
        ResponseTemplate::new(200).set_body_raw("<rss><channel><item><link>http://example.com/follow-up</link><description>Orbital systems fabricated excerpt</description></item></channel></rss>", "application/rss+xml")
    ).mount(&server).await;
    let response = web
        .search_with(
            SearchRequest {
                query: "orbital systems".into(),
                read_pages: 0,
                source_urls: vec![
                    "http://example.com/article".into(),
                    "http://example.com/private".into(),
                    "http://example.com/feed".into(),
                ],
                ..SearchRequest::default()
            },
            &CancellationToken::new(),
        )
        .await
        .unwrap();
    assert_eq!(response.results.len(), 1);
    assert_eq!(response.results[0].read_status, "not_requested");
    assert_eq!(response.source_reads.len(), 3);
    assert_eq!(response.source_reads[0].status, "read");
    assert!(response.source_reads[0].indexed);
    assert_eq!(response.source_reads[1].status, "failed");
    assert!(
        response.source_reads[1]
            .error
            .as_ref()
            .unwrap()
            .contains("robots")
    );
    assert_eq!(response.source_reads[2].status, "discovered");
    assert!(!response.source_reads[2].indexed);
    assert_eq!(response.source_reads[2].discovered_urls, 1);
    assert!(
        web.index
            .get("http://example.com/feed")
            .await
            .unwrap()
            .is_none()
    );
    let requests = server.received_requests().await.unwrap();
    assert!(
        requests
            .iter()
            .all(|request| matches!(request.url.path(), "/robots.txt" | "/article" | "/feed"))
    );
    let follow_up: usize = web
        .index
        .call(|db| {
            Ok(db.query_row(
                "SELECT count(*) FROM frontier WHERE url='http://example.com/follow-up'",
                [],
                |row| row.get(0),
            )?)
        })
        .await
        .unwrap();
    assert_eq!(follow_up, 1);
}

#[tokio::test]
async fn own_search_validates_before_network_and_can_cancel_source_reads() {
    let (_dir, web, server) = fixture(empty_config()).await;
    for request in [
        SearchRequest {
            query: "orbital".into(),
            domains: vec!["example.com%' OR 1=1 --".into()],
            ..SearchRequest::default()
        },
        SearchRequest {
            query: "orbital".into(),
            domains: vec!["*.example.com".into()],
            ..SearchRequest::default()
        },
        SearchRequest {
            query: "orbital".into(),
            days: Some(0),
            ..SearchRequest::default()
        },
        SearchRequest {
            query: "orbital".into(),
            max_per_domain: 0,
            ..SearchRequest::default()
        },
        SearchRequest {
            query: "orbital".into(),
            source_urls: vec!["http://127.0.0.1/private".into()],
            ..SearchRequest::default()
        },
        SearchRequest {
            query: "???".into(),
            source_urls: vec!["http://example.com/article".into()],
            ..SearchRequest::default()
        },
    ] {
        assert!(
            web.search_with(request, &CancellationToken::new())
                .await
                .is_err()
        );
    }
    assert!(
        serde_json::from_str::<SearchRequest>(r#"{"query":"orbit","provider":"brave"}"#).is_err()
    );
    assert!(server.received_requests().await.unwrap().is_empty());
    robots(&server, "User-agent: *\nAllow: /\n").await;
    Mock::given(method("GET"))
        .and(path("/slow"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_delay(Duration::from_secs(10))
                .set_body_raw("<main>Orbital evidence</main>", "text/html"),
        )
        .mount(&server)
        .await;
    let cancel = CancellationToken::new();
    let request = web.search_with(
        SearchRequest {
            query: "orbital".into(),
            read_pages: 0,
            source_urls: vec!["http://example.com/slow".into()],
            ..SearchRequest::default()
        },
        &cancel,
    );
    tokio::pin!(request);
    tokio::select! {
        result = &mut request => panic!("Slow request finished unexpectedly: {result:?}"),
        _ = tokio::time::sleep(Duration::from_millis(200)) => cancel.cancel(),
    }
    let error = tokio::time::timeout(Duration::from_secs(1), request)
        .await
        .unwrap()
        .unwrap_err();
    assert!(error.to_string().contains("cancelled"));
    assert_eq!(web.status().await.unwrap().active_operations, 0);
    assert_eq!(web.status().await.unwrap().active_downloads, 0);
}

#[tokio::test]
async fn publication_migration_preserves_old_corpus_and_fts_without_guessing_dates() {
    let (dir, web, _server) = fixture(empty_config()).await;
    web.index
        .save(page(
            "https://example.com/legacy",
            "Legacy evidence",
            "Orbital legacy research",
        ))
        .await
        .unwrap();
    web.index.call(|db| {
        db.execute_batch("DROP INDEX pages_publication; ALTER TABLE pages DROP COLUMN published_at; ALTER TABLE frontier DROP COLUMN label; PRAGMA user_version=1;")?;
        Ok(())
    }).await.unwrap();
    drop(web);
    let reopened = Web::open(dir.path()).await.unwrap();
    let response = reopened
        .search("orbital", 5, 0, &CancellationToken::new())
        .await
        .unwrap();
    assert_eq!(response.results.len(), 1);
    assert_eq!(response.results[0].published_at, None);
    let recent = reopened
        .search_with(
            SearchRequest {
                query: "orbital".into(),
                read_pages: 0,
                days: Some(30),
                ..SearchRequest::default()
            },
            &CancellationToken::new(),
        )
        .await
        .unwrap();
    assert!(recent.results.is_empty());
    assert_eq!(recent.coverage.indexed_pages, 1);
    assert!(
        reopened
            .index
            .candidates(
                &SearchRequest {
                    query: "orbital".into(),
                    ..Default::default()
                },
                vec![]
            )
            .await
            .unwrap()
            .is_empty(),
        "The new frontier label column must migrate without losing indexed pages"
    );
}

#[tokio::test]
async fn search_candidate_bound_is_disclosed_and_publication_filters_survive_refresh() {
    let (_dir, web, server) = fixture(Config {
        cache_minutes: 0,
        ..empty_config()
    })
    .await;
    web.index.call(|db| {
        let tx = db.transaction()?;
        for i in 0..1002 {
            tx.execute("INSERT INTO pages(url,origin,title,text,hash,truncated,fetched_at) VALUES(?,'http://crowded.example','Orbit',?,?,0,0)", rusqlite::params![format!("http://crowded.example/{i:04}"), format!("Orbit source {i}"), format!("unique-{i}")])?;
        }
        tx.commit()?;
        Ok(())
    }).await.unwrap();
    let response = web
        .search("orbit", 10, 0, &CancellationToken::new())
        .await
        .unwrap();
    assert_eq!(response.results.len(), 2);
    assert!(
        response
            .warnings
            .iter()
            .any(|warning| warning.contains("1000 ranked candidates"))
    );
    robots(&server, "User-agent: *\nAllow: /\n").await;
    html(&server, "/revised", "<meta property='article:published_time' content='2020-01-01'><main>Telescopes revised evidence</main>").await;
    let mut indexed = page(
        "http://example.com/revised",
        "Telescopes",
        "Telescopes earlier evidence",
    );
    indexed.document.published_at = Some(Utc::now().timestamp() - 3600);
    web.index.save(indexed).await.unwrap();
    let revised = web
        .search_with(
            SearchRequest {
                query: "telescopes".into(),
                read_pages: 1,
                days: Some(30),
                ..SearchRequest::default()
            },
            &CancellationToken::new(),
        )
        .await
        .unwrap();
    assert!(revised.results.is_empty());
    assert!(
        revised
            .warnings
            .iter()
            .any(|warning| warning.contains("changed publication date"))
    );
}

#[tokio::test]
async fn crawler_preserves_daily_capacity_for_explicit_foreground_reads() {
    let (_dir, web, server) = fixture(Config {
        requests_per_day: 10,
        foreground_reserve_percent: 20,
        seeds: vec!["http://example.com/background".into()],
        ..empty_config()
    })
    .await;
    robots(&server, "User-agent: *\nAllow: /\n").await;
    html(
        &server,
        "/foreground",
        "<main>Requested public evidence</main>",
    )
    .await;
    web.index
        .call(|db| {
            db.execute(
                "INSERT INTO traffic(id,day,requests) VALUES(1,?,8)",
                [Utc::now().format("%Y-%m-%d").to_string()],
            )?;
            Ok(())
        })
        .await
        .unwrap();
    let status = web.status().await.unwrap();
    assert_eq!(status.crawler_request_budget, 8);
    assert_eq!(status.requests_per_day, 10);
    assert_eq!(status.coverage.requests_today, 8);
    assert!(!web.crawl_next().await.unwrap());
    assert!(server.received_requests().await.unwrap().is_empty());
    assert_eq!(
        web.status().await.unwrap().coverage.due_urls,
        1,
        "Paused crawling must not lease its pending work"
    );
    let page = web
        .read(
            "http://example.com/foreground",
            "evidence",
            &CancellationToken::new(),
        )
        .await
        .unwrap();
    assert!(page.passage.contains("Requested public evidence"));
    assert_eq!(
        web.status().await.unwrap().coverage.requests_today,
        10,
        "Foreground can use both reserved requests for robots and its source"
    );
    assert!(
        server
            .received_requests()
            .await
            .unwrap()
            .iter()
            .all(|request| matches!(request.url.path(), "/robots.txt" | "/foreground"))
    );
    assert!(!web.crawl_next().await.unwrap());
}

#[test]
fn foreground_reserve_defaults_for_old_configs_and_handles_small_budgets() {
    let default: Config = serde_json::from_str(r#"{"requests_per_day":2500,"seeds":[]}"#).unwrap();
    assert_eq!(default.foreground_reserve_percent, 20);
    assert_eq!(default.crawler_request_budget(), 2000);
    for (requests_per_day, reserve, expected) in [(1, 20, 1), (2, 20, 1), (10, 0, 10), (10, 90, 1)]
    {
        let config = Config {
            requests_per_day,
            foreground_reserve_percent: reserve,
            ..empty_config()
        };
        config.validate().unwrap();
        assert_eq!(config.crawler_request_budget(), expected);
    }
    assert!(
        Config {
            foreground_reserve_percent: 91,
            ..empty_config()
        }
        .validate()
        .is_err()
    );
}

#[tokio::test]
async fn live_discovery_follows_titles_from_an_empty_corpus_and_reuses_disk_links() {
    let (dir, mut web, server) = fixture(Config {
        seeds: vec!["https://example.com/".into()],
        ..empty_config()
    })
    .await;
    robots(&server, "User-agent: *\nAllow: /\n").await;
    html(&server, "/", "<title>Research listing</title><a href='/p/17'>Orbital navigation research</a><a href='/p/18' rel='nofollow'>Orbital blocked link</a><a href='http://127.0.0.1/private'>Orbital private</a>").await;
    let published = Utc::now().to_rfc3339();
    html(&server, "/p/17", &format!("<title>Orbital navigation</title><meta property='article:published_time' content='{published}'><main>Orbital navigation original evidence.</main>")).await;
    let request = SearchRequest {
        query: "orbital navigation".into(),
        discover: true,
        fresh: false,
        days: Some(30),
        ..Default::default()
    };
    let result = web
        .search_with(request.clone(), &CancellationToken::new())
        .await
        .unwrap();
    assert!(result.discovery.enabled);
    assert_eq!(result.discovery.followed_urls, 1);
    assert_eq!(result.results.len(), 1);
    assert_eq!(result.results[0].url, "https://example.com/p/17");
    assert!(
        result.results[0]
            .page
            .as_ref()
            .unwrap()
            .passage
            .contains("original evidence")
    );
    assert_eq!(result.coverage.requests_today, 3); // robots + listing + original
    drop(web);
    web = Web::open(dir.path()).await.unwrap();
    Arc::get_mut(&mut web).unwrap().network.test_address = Some(*server.address());
    let again = web
        .search_with(request, &CancellationToken::new())
        .await
        .unwrap();
    assert_eq!(
        again.discovery.followed_urls, 1,
        "Observed labels survive restart and cached entry pages"
    );
    assert_eq!(again.coverage.requests_today, 3);
    assert_eq!(server.received_requests().await.unwrap().len(), 3);
}

#[tokio::test]
async fn live_discovery_uses_feed_titles_only_as_leads_and_respects_domain_filter() {
    let (_dir, web, server) = fixture(empty_config()).await;
    robots(&server, "User-agent: *\nAllow: /\n").await;
    Mock::given(method("GET")).and(path("/feed")).respond_with(ResponseTemplate::new(200).set_body_raw(
        "<rss><channel><item><link>https://example.com/p/9</link><title>Quantum sensing</title><description>Fake feed evidence</description></item><item><title>Quantum sensing</title><link>https://other.example/p/10</link></item></channel></rss>", "application/rss+xml"
    )).mount(&server).await;
    html(
        &server,
        "/p/9",
        "<title>Quantum sensing</title><main>Independent original experiment.</main>",
    )
    .await;
    let result = web
        .search_with(
            SearchRequest {
                query: "quantum sensing".into(),
                discover: true,
                domains: vec!["example.com".into()],
                source_urls: vec!["https://example.com/feed".into()],
                ..Default::default()
            },
            &CancellationToken::new(),
        )
        .await
        .unwrap();
    assert_eq!(result.source_reads[0].status, "discovered");
    assert!(!result.source_reads[0].indexed);
    assert_eq!(result.discovery.followed_urls, 1);
    assert_eq!(result.results.len(), 1);
    assert!(
        !result.results[0]
            .page
            .as_ref()
            .unwrap()
            .passage
            .contains("Fake feed")
    );
    assert_eq!(server.received_requests().await.unwrap().len(), 3);
}

#[test]
fn hn_dates_belong_to_the_requested_permalink_and_titles_label_observed_threads() {
    let old = "2020-02-02T12:00:00";
    let recent = Utc::now().format("%Y-%m-%dT%H:%M:%S").to_string();
    let body = format!(
        "<table><tr class='athing' id='42'><td class='titleline'><a href='https://original.example/x'>Quantum systems</a></td></tr><tr><td><span class='age' title='{old} 123'><a href='item?id=42'>old</a></span><a href='item?id=42'>10 comments</a></td></tr><tr><td><span class='age' title='{recent} 999'><a href='item?id=43'>new comment</a></span></td></tr></table>"
    );
    let url = Url::parse("https://news.ycombinator.com/item?id=42").unwrap();
    let document = extract::extract(&url, &body, "text/html", 4096).unwrap();
    assert_eq!(
        document.published_at,
        Some(
            chrono::DateTime::parse_from_rfc3339(&format!("{old}Z"))
                .unwrap()
                .timestamp()
        )
    );
    assert!(
        document
            .link_labels
            .iter()
            .any(|(link, label)| link.ends_with("item?id=42") && label == "Quantum systems")
    );
    for value in [
        "https://news.ycombinator.com/",
        "https://news.ycombinator.com/item?id=44",
        "https://unrelated.example/item?id=42",
    ] {
        assert!(
            extract::extract(&Url::parse(value).unwrap(), &body, "text/html", 4096)
                .unwrap()
                .published_at
                .is_none(),
            "Must not assign another story or comment's date: {value}"
        );
    }
}

#[tokio::test]
async fn community_discovery_bootstraps_unseeded_domains_and_reports_denials() {
    let (_dir, web, server) = fixture(empty_config()).await;
    robots(&server, "User-agent: *\nDisallow: /\n").await;
    let result = web
        .search_with(
            SearchRequest {
                query: "Rust".into(),
                discover: true,
                domains: vec!["reddit.com".into()],
                ..Default::default()
            },
            &CancellationToken::new(),
        )
        .await
        .unwrap();
    assert_eq!(
        result.discovery.entrypoints,
        ["https://www.reddit.com/r/all/.rss"]
    );
    assert_eq!(result.discovery.sources[0].failure_kind, Some("blocked"));
    assert_eq!(result.discovery.sources[0].status, "failed");
    assert!(result.results.is_empty());
    assert_eq!(
        server.received_requests().await.unwrap().len(),
        1,
        "A denial must never request the article"
    );
}

#[tokio::test]
async fn discovery_bounds_entrypoints_targets_and_reports_unvisited_domains() {
    let (_dir, web, server) = fixture(empty_config()).await;
    robots(&server, "User-agent: *\nAllow: /\n").await;
    let links = (0..10)
        .map(|i| format!("<a href='/item-{i}'>Quantum experiment {i}</a>"))
        .collect::<String>();
    html(&server, "/", &links).await;
    let result = web
        .search_with(
            SearchRequest {
                query: "quantum".into(),
                discover: true,
                domains: (0..8).map(|i| format!("source{i}.example")).collect(),
                ..Default::default()
            },
            &CancellationToken::new(),
        )
        .await
        .unwrap();
    assert_eq!(result.discovery.entrypoints.len(), 4);
    assert_eq!(result.discovery.unvisited_domains.len(), 4);
    assert_eq!(result.discovery.followed_urls, 16);
    let mut counts = std::collections::HashMap::new();
    for source in &result.discovery.sources[4..] {
        *counts
            .entry(Url::parse(&source.url).unwrap().origin())
            .or_insert(0) += 1;
    }
    assert!(counts.values().all(|count| *count <= 8));
}

#[tokio::test]
async fn ten_simultaneous_live_searches_share_discovery_downloads() {
    let (_dir, web, server) = fixture(Config {
        seeds: vec!["https://example.com/".into()],
        ..empty_config()
    })
    .await;
    robots(&server, "User-agent: *\nAllow: /\n").await;
    html(&server, "/", "<a href='/article'>Orbital systems</a>").await;
    Mock::given(method("GET"))
        .and(path("/article"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_delay(Duration::from_millis(200))
                .set_body_raw(
                    "<title>Orbital systems</title><main>Verified original.</main>",
                    "text/html",
                ),
        )
        .mount(&server)
        .await;
    let results = futures_util::future::join_all((0..10).map(|_| async {
        web.search_with(
            SearchRequest {
                query: "orbital systems".into(),
                discover: true,
                ..Default::default()
            },
            &CancellationToken::new(),
        )
        .await
        .unwrap()
    }))
    .await;
    assert!(results.iter().all(|result| {
        result
            .results
            .iter()
            .any(|hit| hit.url.ends_with("/article"))
    }));
    assert_eq!(server.received_requests().await.unwrap().len(), 3);
    let status = web.status().await.unwrap();
    assert_eq!(status.active_operations, 0);
    assert_eq!(status.active_downloads, 0);
}

#[tokio::test]
async fn origin_pacing_longer_than_the_old_timeout_can_complete() {
    let (_dir, web, server) = fixture(empty_config()).await;
    web.index
        .save_robots(
            "https://example.com",
            "User-agent: *\nAllow: /\n".into(),
            3600,
        )
        .await
        .unwrap();
    web.index
        .call(|db| {
            db.execute(
                "UPDATE hosts SET next_request_ms=? WHERE origin='https://example.com'",
                [Utc::now().timestamp_millis() + 30_200],
            )?;
            Ok(())
        })
        .await
        .unwrap();
    html(
        &server,
        "/article",
        "<title>Paced source</title><main>Readable after origin waiting.</main>",
    )
    .await;
    let started = std::time::Instant::now();
    let result = web
        .read("https://example.com/article", "", &CancellationToken::new())
        .await
        .unwrap();
    assert!(started.elapsed() >= Duration::from_secs(30));
    assert!(result.passage.contains("Readable"));
    assert_eq!(web.status().await.unwrap().coverage.requests_today, 1);
}

#[tokio::test]
async fn cancelling_live_discovery_releases_operations_and_downloads() {
    let (_dir, web, server) = fixture(Config {
        seeds: vec!["https://example.com/".into()],
        ..empty_config()
    })
    .await;
    robots(&server, "User-agent: *\nAllow: /\n").await;
    Mock::given(method("GET"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(200).set_delay(Duration::from_secs(10)))
        .mount(&server)
        .await;
    let cancel = CancellationToken::new();
    let call = web.search_with(
        SearchRequest {
            query: "science".into(),
            discover: true,
            ..Default::default()
        },
        &cancel,
    );
    let cancellation = async {
        tokio::time::sleep(Duration::from_millis(250)).await;
        cancel.cancel();
    };
    let (result, _) = tokio::join!(call, cancellation);
    assert!(result.unwrap_err().to_string().contains("cancelled"));
    let status = web.status().await.unwrap();
    assert_eq!(status.active_operations, 0);
    assert_eq!(status.active_downloads, 0);
}

#[tokio::test]
async fn discovery_traverses_observed_noscript_contents_without_running_javascript() {
    let (_dir, web, server) = fixture(Config {
        seeds: vec!["https://example.com/".into()],
        ..empty_config()
    })
    .await;
    robots(&server, "User-agent: *\nAllow: /\n").await;
    html(&server, "/", "<nav aria-label='Table of contents'><noscript><iframe src='toc.html'></iframe></noscript></nav><script src='toc.js'></script>").await;
    html(
        &server,
        "/toc.html",
        "<a href='/opaque-42'>Ownership and borrowing</a>",
    )
    .await;
    html(
        &server,
        "/opaque-42",
        "<title>Ownership and borrowing</title><main>Original chapter evidence.</main>",
    )
    .await;
    let result = web
        .search_with(
            SearchRequest {
                query: "ownership borrowing".into(),
                discover: true,
                ..Default::default()
            },
            &CancellationToken::new(),
        )
        .await
        .unwrap();
    assert_eq!(result.discovery.followed_urls, 2);
    assert!(
        result
            .results
            .iter()
            .any(|hit| hit.url.ends_with("/opaque-42"))
    );
    let requests = server.received_requests().await.unwrap();
    assert_eq!(requests.len(), 4);
    assert!(
        !requests
            .iter()
            .any(|request| request.url.path().ends_with(".js"))
    );
}

#[tokio::test]
async fn short_acronyms_do_not_match_unrelated_url_substrings() {
    let (_dir, web, _server) = fixture(empty_config()).await;
    web.index
        .enqueue(
            vec![
                "https://example.com/from?site=john.example".into(),
                "https://example.com/item?id=42".into(),
            ],
            1,
        )
        .await
        .unwrap();
    web.index
        .label_links(vec![(
            "https://example.com/item?id=42".into(),
            "Show HN: a useful experiment".into(),
        )])
        .await
        .unwrap();
    let result = web
        .index
        .candidates(
            &SearchRequest {
                query: "HN".into(),
                ..Default::default()
            },
            vec![],
        )
        .await
        .unwrap();
    assert_eq!(result, ["https://example.com/item?id=42"]);
}

#[tokio::test]
async fn cold_hn_discovery_returns_dated_threads_and_excludes_recent_comments_on_old_stories() {
    let (_dir, web, server) = fixture(empty_config()).await;
    robots(&server, "User-agent: *\nAllow: /\n").await;
    html(&server, "/", "<table><tr class='athing' id='42'><td><span class='titleline'><a href='https://original.example/'>Quantum experiment</a></span></td></tr><tr><td><a href='item?id=42'>5 comments</a></td></tr></table>").await;
    let recent = Utc::now().format("%Y-%m-%dT%H:%M:%S").to_string();
    html(&server, "/item", &format!("<title>Quantum experiment | Hacker News</title><span class='age' title='{recent}'><a href='item?id=42'>today</a></span><p>Quantum experiment original discussion.</p>")).await;
    let request = SearchRequest {
        query: "quantum experiment".into(),
        domains: vec!["news.ycombinator.com".into()],
        days: Some(30),
        discover: true,
        ..Default::default()
    };
    let result = web
        .search_with(request.clone(), &CancellationToken::new())
        .await
        .unwrap();
    assert_eq!(result.results.len(), 1);
    assert_eq!(
        result.results[0].url,
        "https://news.ycombinator.com/item?id=42"
    );
    assert!(result.results[0].published_at.is_some());
    assert!(result.results[0].page.is_some());
    // A recent comment must not make this old submission satisfy days=30.
    let mut old = page(
        "https://news.ycombinator.com/item?id=43",
        "Quantum experiment",
        "Quantum experiment old story and recent comments",
    );
    old.document = extract::extract(&Url::parse(&old.document.url).unwrap(),
        &format!("<title>Quantum experiment</title><span class='age' title='2020-01-01T00:00:00'><a href='item?id=43'>old</a></span><span class='age' title='{recent}'><a href='item?id=44'>new comment</a></span>"), "text/html", 4096).unwrap();
    web.index.save(old).await.unwrap();
    let again = web
        .search_with(
            SearchRequest {
                discover: false,
                read_pages: 0,
                ..request
            },
            &CancellationToken::new(),
        )
        .await
        .unwrap();
    assert_eq!(again.results.len(), 1);
    assert_eq!(
        again.results[0].url,
        "https://news.ycombinator.com/item?id=42"
    );
}

// Characterization tests for the live-documentation audit. They expose current
// cache/extraction behavior; passing does not imply an always-live docs feature.
#[tokio::test]
async fn live_docs_audit_cached_read_misses_an_origin_update_until_revalidation() {
    let (_dir, web, server) = fixture(empty_config()).await;
    robots(&server, "User-agent: *\nAllow: /\n").await;
    html(
        &server,
        "/documentation",
        "<main>Release one: use legacyMethod.</main>",
    )
    .await;
    let cancel = CancellationToken::new();
    let first = web
        .read("https://example.com/documentation", "Release", &cancel)
        .await
        .unwrap();
    assert_eq!(first.source, "network");
    server.reset().await;
    html(
        &server,
        "/documentation",
        "<main>Release two: use currentMethod.</main>",
    )
    .await;
    let second = web
        .read("https://example.com/documentation", "Release", &cancel)
        .await
        .unwrap();
    assert_eq!(second.source, "cache");
    assert!(second.passage.contains("legacyMethod"));
    assert_eq!(server.received_requests().await.unwrap().len(), 0);
    web.index
        .call(|db| {
            db.execute("UPDATE pages SET fetched_at=1", [])?;
            Ok(())
        })
        .await
        .unwrap();
    let checked = web
        .read("https://example.com/documentation", "Release", &cancel)
        .await
        .unwrap();
    assert_eq!(checked.source, "network");
    assert!(checked.passage.contains("currentMethod"));
    eprintln!(
        "Controlled origin changed: repeated default read made 0 requests and returned old method; after expiration, new method was retrieved."
    );
}

#[tokio::test]
async fn live_docs_audit_code_indentation_and_unknown_publication_dates() {
    let (_dir, web, server) = fixture(empty_config()).await;
    robots(&server, "User-agent: *\nAllow: /\n").await;
    html(&server, "/docs", "<main><h1>Python function</h1><pre><code>def add(a, b):\n    return a + b\n</code></pre></main>").await;
    let cancel = CancellationToken::new();
    let read = web
        .read("https://example.com/docs", "function", &cancel)
        .await
        .unwrap();
    assert!(read.passage.contains("def add(a, b):\n    return a + b"));
    assert!(read.published_at.is_none());
    let ordinary = web
        .search_with(
            SearchRequest {
                query: "Python function".into(),
                read_pages: 0,
                ..Default::default()
            },
            &cancel,
        )
        .await
        .unwrap();
    assert_eq!(ordinary.results.len(), 1);
    let recent = web
        .search_with(
            SearchRequest {
                query: "Python function".into(),
                days: Some(30),
                read_pages: 0,
                ..Default::default()
            },
            &cancel,
        )
        .await
        .unwrap();
    assert!(recent.results.is_empty());
    eprintln!(
        "Controlled docs: HTML code indentation is preserved; the newly fetched page is excluded by days=30 because publication metadata is absent."
    );
}
