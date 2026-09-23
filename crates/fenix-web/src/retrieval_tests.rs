use super::*;
use crate::tests::{empty_config, fixture, html, page, robots};
use wiremock::{
    Mock, ResponseTemplate,
    matchers::{method, path},
};

#[test]
fn dotted_product_names_do_not_become_generic_navigation_terms() {
    assert_eq!(
        search::topic_terms("Next.js proxy middleware", "https://nextjs.org/docs"),
        vec!["proxy", "middleware"]
    );
    assert!(!search::relevant(
        "Next.js hosting",
        "https://nextjs.org/hosting",
        "Next.js hosting and JavaScript platforms.",
        "Next.js proxy middleware"
    ));
}

#[tokio::test]
async fn observed_specific_reference_outweighs_partial_configuration_names() {
    let (_dir, web, server) = fixture(empty_config()).await;
    robots(&server, "User-agent: *\nAllow: /\n").await;
    let landing = format!(
        "<title>Proxy middleware overview</title><main>{}<a href='/proxy'>Proxy</a><a href='/skipProxyUrlNormalize'>skipProxyUrlNormalize</a><a href='/proxyClientMaxBodySize'>proxyClientMaxBodySize</a></main>",
        "Proxy middleware overview. ".repeat(25)
    );
    html(
        &server,
        "/",
        "<a href='/overview'>Proxy middleware overview</a>",
    )
    .await;
    html(&server, "/overview", &landing).await;
    html(
        &server,
        "/skipProxyUrlNormalize",
        "<main>Configuration option.</main>",
    )
    .await;
    html(
        &server,
        "/proxyClientMaxBodySize",
        "<main>Configuration option.</main>",
    )
    .await;
    html(&server, "/proxy", "<title>Proxy middleware reference</title><main>Proxy middleware intercepts requests before routing.</main>").await;
    let result = web
        .search_with(
            SearchRequest {
                query: "Next.js proxy middleware".into(),
                domains: vec!["nextjs.org".into()],
                discover: true,
                ..Default::default()
            },
            &CancellationToken::new(),
        )
        .await
        .unwrap();
    assert!(
        result
            .discovery
            .sources
            .iter()
            .any(|source| source.url == "https://nextjs.org/proxy" && source.status == "read"),
        "a matching overview must not stop traversal before its specific reference"
    );
    let ranked = web
        .index
        .candidates(
            &SearchRequest {
                query: "Next.js proxy middleware".into(),
                ..Default::default()
            },
            vec!["https://nextjs.org".into()],
        )
        .await
        .unwrap();
    let pos = |suffix: &str| ranked.iter().position(|url| url.ends_with(suffix)).unwrap();
    assert!(pos("/proxy") < pos("/skipProxyUrlNormalize"));
}

#[tokio::test]
async fn partial_keywords_do_not_fill_results_with_unrelated_pages() {
    let (_dir, web, _) = fixture(empty_config()).await;
    web.index
        .save(page(
            "https://nasa.example/",
            "NASA",
            "Latest documentation on lunar exploration and cleanup",
        ))
        .await
        .unwrap();
    web.index
        .save(page(
            "https://react.example/effect",
            "React useEffect",
            "React useEffect cleanup releases connections",
        ))
        .await
        .unwrap();
    let result = web
        .search_with(
            SearchRequest {
                query: "Busca documentación actualizada React useEffect cleanup".into(),
                read_pages: 0,
                ..Default::default()
            },
            &CancellationToken::new(),
        )
        .await
        .unwrap();
    assert_eq!(result.results.len(), 1);
    assert_eq!(result.results[0].url, "https://react.example/effect");
    let missing = web
        .search(
            "unobtainium photosynthesis",
            5,
            0,
            &CancellationToken::new(),
        )
        .await
        .unwrap();
    assert!(missing.results.is_empty());
}

#[tokio::test]
async fn fresh_search_revalidates_and_removes_changed_evidence() {
    let (_dir, web, server) = fixture(empty_config()).await;
    robots(&server, "User-agent: *\nAllow: /\n").await;
    html(
        &server,
        "/docs",
        "<title>Widget API</title><main>Widget API uses old_method.</main>",
    )
    .await;
    let cancel = CancellationToken::new();
    web.read("https://example.com/docs", "Widget API", &cancel)
        .await
        .unwrap();
    server.reset().await;
    html(
        &server,
        "/docs",
        "<title>Widget API</title><main>Widget API uses new_method.</main>",
    )
    .await;
    let result = web.search("Widget API", 5, 3, &cancel).await.unwrap();
    assert!(
        result.results[0]
            .page
            .as_ref()
            .unwrap()
            .passage
            .contains("new_method")
    );
    assert!(!result.results[0].snippet.contains("old_method"));
    assert_eq!(server.received_requests().await.unwrap().len(), 1);
    server.reset().await;
    html(
        &server,
        "/docs",
        "<title>Garden</title><main>Growing roses.</main>",
    )
    .await;
    let result = web.search("Widget API", 5, 3, &cancel).await.unwrap();
    assert!(
        result.results.is_empty(),
        "a previously relevant URL must be checked again"
    );
}

#[tokio::test]
async fn live_read_bypasses_ttl_but_explicit_cached_read_remains_available() {
    let (_dir, web, server) = fixture(empty_config()).await;
    robots(&server, "User-agent: *\nAllow: /\n").await;
    html(&server, "/doc", "<main>Version one</main>").await;
    let cancel = CancellationToken::new();
    web.read("https://example.com/doc", "", &cancel)
        .await
        .unwrap();
    server.reset().await;
    html(&server, "/doc", "<main>Version two</main>").await;
    assert!(
        web.read_with_fresh("https://example.com/doc", "", false, &cancel)
            .await
            .unwrap()
            .passage
            .contains("one")
    );
    let fresh = web
        .read_with_fresh("https://example.com/doc", "", true, &cancel)
        .await
        .unwrap();
    assert!(fresh.passage.contains("two"));
    assert_eq!(fresh.source, "network");
}

#[tokio::test]
async fn sitemap_can_discover_a_reference_beyond_the_old_hundred_link_limit() {
    let (_dir, web, server) = fixture(Config {
        cache_minutes: 0,
        ..empty_config()
    })
    .await;
    robots(
        &server,
        "User-agent: *\nAllow: /\nSitemap: https://example.com/sitemap.xml\n",
    )
    .await;
    html(&server, "/", "<main>Welcome</main>").await;
    let entries = (0..150)
        .map(|i| format!("<url><loc>https://example.com/unrelated-{i}</loc></url>"))
        .collect::<String>();
    Mock::given(method("GET")).and(path("/sitemap.xml")).respond_with(ResponseTemplate::new(200).set_body_raw(format!("<urlset>{entries}<url><loc>https://example.com/quantum-sensors</loc></url></urlset>"), "application/xml")).mount(&server).await;
    html(
        &server,
        "/quantum-sensors",
        "<title>Quantum sensors</title><main>Quantum sensors detect magnetic fields.</main>",
    )
    .await;
    let result = web
        .search_with(
            SearchRequest {
                query: "quantum sensors".into(),
                domains: vec!["example.com".into()],
                discover: true,
                ..Default::default()
            },
            &CancellationToken::new(),
        )
        .await
        .unwrap();
    assert_eq!(result.results.len(), 1);
    assert_eq!(result.results[0].url, "https://example.com/quantum-sensors");
    assert_eq!(
        server
            .received_requests()
            .await
            .unwrap()
            .iter()
            .filter(|r| r.url.path() == "/quantum-sensors")
            .count(),
        1,
        "a source checked during discovery should not be fetched twice"
    );
}

#[test]
fn code_fragments_and_dense_passages_preserve_useful_evidence() {
    let doc = extract::extract(&Url::parse("https://example.com/").unwrap(), "<main><h1>Example</h1><pre><code><span>def</span> sum(a, b):\n    return a + b\n</code></pre><p>Explanation.</p></main>", "text/html", 64_000).unwrap();
    assert!(doc.text.contains("def sum(a, b):\n    return a + b\n"));
    let body = format!(
        "react {}\nuseEffect cleanup releases the React connection",
        "irrelevant ".repeat(900)
    );
    let passage = extract::passage(&body, "React useEffect cleanup", 350);
    assert!(passage.contains("useEffect cleanup"));
}

#[test]
fn source_routing_preserves_explicit_domains_without_guessing_articles() {
    assert_eq!(
        sources::roots("React useEffect cleanup")[0],
        "https://react.dev/"
    );
    assert_eq!(
        sources::roots("Tokio timeout")[0],
        "https://docs.rs/tokio/latest/tokio/"
    );
    assert_eq!(
        sources::roots("example.com release notes")[0],
        "https://example.com/"
    );
    assert!(sources::roots("a subject outside the bootstrap catalog").is_empty());
    assert!(
        !sources::roots("Next.js proxy")
            .iter()
            .any(|url| url.contains("next.js/"))
    );
}

#[tokio::test]
async fn discovery_does_not_borrow_unrelated_links_from_other_queries() {
    let (_dir, web, server) = fixture(empty_config()).await;
    robots(&server, "User-agent: *\nAllow: /\n").await;
    html(&server, "/", "<main>Home <a href='/build'>Build</a></main>").await;
    html(
        &server,
        "/build",
        "<title>Multi stage builds</title><main>Multi stage builds produce smaller images.</main>",
    )
    .await;
    web.index
        .enqueue(
            vec!["https://unrelated.example/multi-stage-builds".into()],
            0,
        )
        .await
        .unwrap();
    web.index
        .label_links(vec![(
            "https://unrelated.example/multi-stage-builds".into(),
            "multi stage builds".into(),
        )])
        .await
        .unwrap();
    let result = web
        .search_with(
            SearchRequest {
                query: "multi stage builds".into(),
                source_urls: vec!["https://example.com/".into()],
                discover: true,
                ..Default::default()
            },
            &CancellationToken::new(),
        )
        .await
        .unwrap();
    assert!(
        result
            .discovery
            .sources
            .iter()
            .all(|s| !s.url.contains("unrelated.example"))
    );
}

#[test]
fn source_names_and_distant_keywords_do_not_substitute_for_the_topic() {
    assert!(!search::relevant(
        "Moon atmosphere",
        "https://science.nasa.gov/moon/",
        "NASA moon atmosphere",
        "NASA Mars atmosphere"
    ));
    assert!(!search::relevant(
        "Docker products",
        "https://docker.com/products",
        "Docker multi platform builds",
        "Docker multi stage builds"
    ));
    let body = format!("Mars {} atmosphere", "unrelated ".repeat(400));
    assert!(!search::relevant(
        "NASA",
        "https://nasa.gov/",
        &body,
        "NASA Mars atmosphere"
    ));
    assert!(search::relevant(
        "Mars atmosphere",
        "https://science.nasa.gov/mars/",
        "The atmosphere of Mars is thin.",
        "NASA Mars atmosphere"
    ));
}

#[test]
fn highlighted_code_line_wrappers_do_not_merge_statements() {
    let html = "<main><pre><code><div class='cm-line'><span>def</span> add(a, b):</div><div class='cm-line'>    <span>return</span> a + b</div></code></pre></main>";
    let doc = extract::extract(
        &Url::parse("https://example.com/").unwrap(),
        html,
        "text/html",
        6_000,
    )
    .unwrap();
    assert!(doc.text.contains("def add(a, b):\n    return a + b"));
}

#[tokio::test]
async fn library_navigation_is_followed_without_matching_source_name_only() {
    let (_dir, web, server) = fixture(empty_config()).await;
    robots(&server, "User-agent: *\nAllow: /\n").await;
    html(
        &server,
        "/",
        "<main><a href='/library/'>Library Reference</a></main>",
    )
    .await;
    html(
        &server,
        "/library/",
        "<main><a href='/library/asyncio'>asyncio TaskGroup</a></main>",
    )
    .await;
    html(
        &server,
        "/library/asyncio",
        "<title>asyncio TaskGroup</title><main>Python asyncio TaskGroup manages tasks.</main>",
    )
    .await;
    let result = web
        .search_with(
            SearchRequest {
                query: "Python asyncio TaskGroup".into(),
                domains: vec!["docs.python.example".into()],
                discover: true,
                ..Default::default()
            },
            &CancellationToken::new(),
        )
        .await
        .unwrap();
    assert_eq!(
        result.results[0].url,
        "https://docs.python.example/library/asyncio"
    );
}

#[tokio::test]
async fn filler_only_queries_return_no_evidence() {
    let (_dir, web, _) = fixture(empty_config()).await;
    let result = web
        .search("the", 5, 0, &CancellationToken::new())
        .await
        .unwrap();
    assert!(result.results.is_empty());
}

#[tokio::test]
async fn a_full_background_frontier_does_not_hide_observed_query_links() {
    let (_dir, web, server) = fixture(Config {
        max_frontier: 2,
        max_urls_per_origin: 2,
        ..empty_config()
    })
    .await;
    robots(&server, "User-agent: *\nAllow: /\n").await;
    web.index
        .enqueue(
            vec![
                "https://example.com/unrelated-a".into(),
                "https://example.com/unrelated-b".into(),
            ],
            0,
        )
        .await
        .unwrap();
    html(
        &server,
        "/",
        "<main><a href='/quantum-sensors'>Quantum sensors</a></main>",
    )
    .await;
    html(
        &server,
        "/quantum-sensors",
        "<title>Quantum sensors</title><main>Quantum sensors measure fields.</main>",
    )
    .await;
    let result = web
        .search_with(
            SearchRequest {
                query: "quantum sensors".into(),
                domains: vec!["example.com".into()],
                discover: true,
                ..Default::default()
            },
            &CancellationToken::new(),
        )
        .await
        .unwrap();
    assert!(!result.results.is_empty());
    assert_eq!(result.results[0].url, "https://example.com/quantum-sensors");
    assert_eq!(
        result.coverage.known_urls, 2,
        "foreground discovery must retain configured storage caps"
    );
}

#[tokio::test]
async fn fragment_labels_survive_deduplication_and_later_generic_links() {
    let (_dir, web, server) = fixture(empty_config()).await;
    robots(&server, "User-agent: *\nAllow: /\n").await;
    html(&server, "/", "<main><a href='/reference'>Coroutines and tasks</a><a href='/reference#TaskGroup'>TaskGroup</a></main>").await;
    web.read("https://example.com/", "", &CancellationToken::new())
        .await
        .unwrap();
    web.index
        .label_links(vec![(
            "https://example.com/reference".into(),
            "tasks".into(),
        )])
        .await
        .unwrap();
    let candidates = web
        .index
        .candidates(
            &SearchRequest {
                query: "TaskGroup".into(),
                ..Default::default()
            },
            vec!["https://example.com".into()],
        )
        .await
        .unwrap();
    assert!(candidates.contains(&"https://example.com/reference".into()));
}
