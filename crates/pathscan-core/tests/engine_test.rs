use pathscan_core::client::HttpClient;
use pathscan_core::types::{RequestContext, ScanConfig, WordlistConfig};
use pathscan_core::engine::ScanEngine;
use wiremock::{Mock, MockServer, ResponseTemplate};
use wiremock::matchers::{method, path};
use std::collections::HashMap;

#[tokio::test]
async fn test_http_client_basic() {
    let server = MockServer::start().await;

    Mock::given(method("GET")).and(path("/admin"))
        .respond_with(ResponseTemplate::new(200).set_body_string("ok"))
        .mount(&server).await;

    let uri = server.uri();
    let config = ScanConfig::new(uri.clone());
    let client = HttpClient::new(&config).unwrap();

    let ctx = RequestContext {
        url: format!("{}/admin", uri),
        method: reqwest::Method::GET,
        headers: HashMap::new(),
        follow_redirects: false,
    };

    let resp = client.execute(ctx).await.unwrap();
    assert_eq!(resp.status, 200);
    assert_eq!(resp.content_length, 2);
}

#[tokio::test]
async fn test_scan_with_wiremock() {
    let server = MockServer::start().await;

    // Mount specific mocks first, fallback last (wiremock: first match wins)
    Mock::given(method("GET")).and(path("/a"))
        .respond_with(ResponseTemplate::new(200).set_body_string("a ok"))
        .mount(&server).await;
    Mock::given(method("GET")).and(path("/b"))
        .respond_with(ResponseTemplate::new(200).set_body_string("b ok"))
        .mount(&server).await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(404).set_body_string("not_found_page_with_enough_content_length_to_not_match_default"))
        .mount(&server).await;

    let uri = server.uri();
    let mut config = ScanConfig::new(uri.clone());
    config.no_baseline = true;
    config.concurrency = 1;
    config.status_whitelist = vec![200];

    let tmp = std::env::temp_dir().join("ps_int_test.txt");
    std::fs::write(&tmp, "a\nb\nc\n").unwrap();
    config.wordlist = WordlistConfig::External(tmp.to_string_lossy().to_string());

    let report = ScanEngine::run(config, None).await.unwrap();
    assert_eq!(report.total_requests, 3, "total requests");
    assert_eq!(report.results.len(), 2, "results count");
    assert!(report.results.iter().any(|r| r.path == "a" && r.status == 200));
    assert!(report.results.iter().any(|r| r.path == "b" && r.status == 200));
}
