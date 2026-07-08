use clap::Parser;
use pathscan_core::types::{BuiltinSize, ScanConfig, WordlistConfig};
use std::collections::HashMap;

#[derive(Parser, Debug)]
#[command(name = "pathscan", version, about = "Web directory scanner")]
pub struct CliArgs {
    #[arg(short = 'u', long)]
    pub url: String,

    #[arg(short = 'w', long)]
    pub wordlist: Option<String>,

    #[arg(long)]
    pub builtin: Option<String>,

    #[arg(long, default_value = "32")]
    pub concurrency: usize,

    #[arg(long)]
    pub rate_limit: Option<u32>,

    #[arg(long, default_value = "10")]
    pub timeout: u64,

    #[arg(short = 'H', long = "header")]
    pub headers: Vec<String>,

    #[arg(long)]
    pub cookie: Option<String>,

    #[arg(long = "user-agent")]
    pub user_agent: Option<String>,

    #[arg(long)]
    pub proxy: Option<String>,

    #[arg(long)]
    pub status: Option<String>,

    #[arg(long = "exclude-status")]
    pub exclude_status: Option<String>,

    #[arg(short = 'k', long)]
    pub insecure: bool,

    #[arg(long)]
    pub no_length_filter: bool,

    #[arg(long = "content-regex")]
    pub content_regex: Option<String>,

    #[arg(short = 'r', long)]
    pub recursive: bool,

    #[arg(long, default_value = "1")]
    pub depth: usize,

    #[arg(short = 'o', long)]
    pub output: Option<String>,

    #[arg(long)]
    pub json: bool,

    #[arg(long)]
    pub html: bool,

    #[arg(long)]
    pub no_color: bool,

    #[arg(long)]
    pub no_baseline: bool,

    #[arg(long)]
    pub tui: bool,
}

impl From<CliArgs> for ScanConfig {
    fn from(args: CliArgs) -> Self {
        let mut config = ScanConfig::new(args.url);

        config.concurrency = args.concurrency;
        config.rate_limit = args.rate_limit;
        config.timeout_secs = args.timeout;
        config.cookie = args.cookie;
        config.user_agent = args.user_agent;
        config.proxy = args.proxy;
        config.recursive = args.recursive;
        config.max_depth = args.depth;
        config.no_baseline = args.no_baseline;
        config.insecure = args.insecure;
        config.enable_length_filter = !args.no_length_filter;
        config.content_regex = args.content_regex;

        let mut headers = HashMap::new();
        for h in &args.headers {
            if let Some((k, v)) = h.split_once(':') {
                headers.insert(k.trim().to_string(), v.trim().to_string());
            }
        }
        config.custom_headers = headers;

        if let Some(ref status) = args.status {
            config.status_whitelist = status
                .split(',')
                .filter_map(|s| s.trim().parse().ok())
                .collect();
        }

        if let Some(ref exclude) = args.exclude_status {
            config.status_blacklist = exclude
                .split(',')
                .filter_map(|s| s.trim().parse().ok())
                .collect();
        }

        config.wordlist = if let Some(ref path) = args.wordlist {
            WordlistConfig::External(path.clone())
        } else if let Some(ref size) = args.builtin {
            match size.as_str() {
                "small" => WordlistConfig::Builtin(BuiltinSize::Small),
                "medium" => WordlistConfig::Builtin(BuiltinSize::Medium),
                "large" => WordlistConfig::Builtin(BuiltinSize::Large),
                _ => WordlistConfig::Builtin(BuiltinSize::Medium),
            }
        } else {
            WordlistConfig::Builtin(BuiltinSize::Medium)
        };

        config
    }
}
