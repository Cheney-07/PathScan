use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    pub url: String,
    pub path: String,
    pub status: u16,
    pub content_length: u64,
    pub content_type: Option<String>,
    pub response_time: Duration,
    pub redirect_location: Option<String>,
    pub is_directory: bool,
    pub content_hash: String,
    pub headers: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct RequestContext {
    pub url: String,
    pub method: reqwest::Method,
    pub headers: HashMap<String, String>,
    pub follow_redirects: bool,
}

impl Default for RequestContext {
    fn default() -> Self {
        Self {
            url: String::new(),
            method: reqwest::Method::GET,
            headers: HashMap::new(),
            follow_redirects: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ResponseContext {
    pub url: String,
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub content_length: u64,
    pub body_snippet: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Baseline {
    pub typical_status: u16,
    pub typical_length: u64,
    pub typical_hash: String,
    pub samples: Vec<(u16, u64)>,
}

#[derive(Debug, Clone)]
pub struct ScanConfig {
    pub target_url: String,
    pub concurrency: usize,
    pub rate_limit: Option<u32>,
    pub timeout_secs: u64,
    pub custom_headers: HashMap<String, String>,
    pub cookie: Option<String>,
    pub user_agent: Option<String>,
    pub proxy: Option<String>,
    pub status_whitelist: Vec<u16>,
    pub status_blacklist: Vec<u16>,
    pub insecure: bool,
    pub enable_length_filter: bool,
    pub content_regex: Option<String>,
    pub recursive: bool,
    pub max_depth: usize,
    pub no_baseline: bool,
    pub wordlist: WordlistConfig,
}

#[derive(Debug, Clone)]
pub enum WordlistConfig {
    Builtin(BuiltinSize),
    External(String),
    Multiple(Vec<WordlistConfig>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BuiltinSize {
    Small,
    Medium,
    Large,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanReport {
    pub target: String,
    pub duration_secs: f64,
    pub total_requests: usize,
    pub results: Vec<ScanResult>,
    pub errors: usize,
    pub baseline: Option<Baseline>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScanStatus {
    Running,
    Paused,
    Completed,
    Cancelled,
}

impl ScanConfig {
    pub fn new(target_url: String) -> Self {
        Self {
            target_url,
            concurrency: 32,
            rate_limit: None,
            timeout_secs: 10,
            custom_headers: HashMap::new(),
            cookie: None,
            user_agent: None,
            proxy: None,
            status_whitelist: vec![200, 201, 204, 301, 302, 307, 401, 403, 405, 500],
            status_blacklist: vec![400, 404, 408, 410, 429, 502, 503, 504, 500],
            insecure: false,
            enable_length_filter: true,
            content_regex: None,
            recursive: false,
            max_depth: 1,
            no_baseline: false,
            wordlist: WordlistConfig::Builtin(BuiltinSize::Medium),
        }
    }
}
