use crate::baseline::BaselineCalibrator;
use crate::client::HttpClient;
use crate::error::Result;
use crate::filter::FilterPipeline;
use crate::plugin::PluginChain;
use crate::types::{
    Baseline, RequestContext, ScanConfig, ScanReport, ScanResult, ScanStatus,
};
use crate::wordlist::Wordlist;
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::{mpsc, Semaphore};

pub enum ScanEvent {
    Progress { completed: usize, total: usize },
    Result(ScanResult),
    Status(ScanStatus),
    Error(String),
}

pub struct ScanEngine;

impl ScanEngine {
    pub async fn run(
        config: ScanConfig,
        event_tx: Option<mpsc::UnboundedSender<ScanEvent>>,
    ) -> Result<ScanReport> {
        let start = Instant::now();
        let mut wordlist = Wordlist::load(
            config.wordlist.clone(),
            Some(std::path::PathBuf::from("wordlists")),
        )?;
        wordlist.deduplicate();

        let client = Arc::new(HttpClient::new(&config)?);
        let baseline: Option<Baseline> = if config.no_baseline {
            None
        } else {
            match BaselineCalibrator::calibrate(&client, &config).await {
                Ok(b) => Some(b),
                Err(_) => None,
            }
        };

        let filter = FilterPipeline::new(&config, baseline.as_ref());
        let mut plugin_chain = PluginChain::new();
        plugin_chain.register(Box::new(
            crate::plugins::recursion::RecursionPlugin,
        ));
        plugin_chain.register(Box::new(
            crate::plugins::fingerprint::FingerprintPlugin,
        ));

        let status_whitelist = Arc::new(config.status_whitelist.clone());
        let status_blacklist = Arc::new(config.status_blacklist.clone());
        let entries = wordlist.entries().to_vec();
        let total = entries.len();
        let semaphore = Arc::new(Semaphore::new(config.concurrency));
        let mut results: Vec<ScanResult> = Vec::new();
        let mut errors = 0usize;
        let mut seen_hashes: HashSet<String> = HashSet::new();
        let base_url = config.target_url.trim_end_matches('/').to_string();
        let mut handles = Vec::new();

        for (i, entry) in entries.into_iter().enumerate() {
            let permit = semaphore.clone().acquire_owned().await.map_err(|_| {
                crate::error::PathScanError::Cancelled
            })?;

            let client = Arc::clone(&client);
            let client_url = base_url.clone();
            let headers = config.custom_headers.clone();
            let tx = event_tx.clone();
            let wl = Arc::clone(&status_whitelist);
            let bl = Arc::clone(&status_blacklist);

            handles.push(tokio::spawn(async move {
                let _permit = permit;
                let path = entry.trim_start_matches('/');
                let url = format!("{}/{}", client_url, path);

                let ctx = RequestContext {
                    url: url.clone(),
                    method: reqwest::Method::GET,
                    headers,
                    follow_redirects: false,
                };

                match client.execute(ctx).await {
                    Ok(resp) => {
                        let status = resp.status;
                        let passed = (wl.is_empty() || wl.contains(&status)) && !bl.contains(&status);

                        let mut hasher = Sha256::new();
                        hasher.update(url.as_bytes());
                        hasher.update(status.to_le_bytes());
                        hasher.update(resp.content_length.to_le_bytes());
                        let hash = format!("{:x}", hasher.finalize());

                        let result = ScanResult {
                            url: url.clone(),
                            path: path.to_string(),
                            status,
                            content_length: resp.content_length,
                            content_type: resp.headers.get("content-type").cloned(),
                            response_time: std::time::Duration::from_millis(0),
                            redirect_location: resp.headers.get("location").cloned(),
                            is_directory: false,
                            content_hash: hash,
                            headers: resp.headers.clone(),
                        };

                        if passed {
                            if let Some(ref tx) = tx {
                                let _ = tx.send(ScanEvent::Result(result.clone()));
                            }
                        }

                        if let Some(ref tx) = tx {
                            let _ = tx.send(ScanEvent::Progress { completed: i + 1, total });
                        }
                        (Some(result), None)
                    }
                    Err(e) => {
                        if let Some(ref tx) = tx {
                            let _ = tx.send(ScanEvent::Progress { completed: i + 1, total });
                            let _ = tx.send(ScanEvent::Error(e.to_string()));
                        }
                        (None, Some(e))
                    }
                }
            }));
        }

        for handle in handles {
            match handle.await {
                Ok((Some(mut result), _)) => {
                    plugin_chain.run_on_result(&mut result);
                    if let Some(r) = filter.run(result) {
                        if seen_hashes.insert(r.content_hash.clone()) {
                            results.push(r);
                        }
                    }
                }
                Ok((None, _)) => errors += 1,
                Err(_) => errors += 1,
            }
        }

        if let Some(ref tx) = event_tx {
            let _ = tx.send(ScanEvent::Status(ScanStatus::Completed));
        }

        Ok(ScanReport {
            target: config.target_url,
            duration_secs: start.elapsed().as_secs_f64(),
            total_requests: total,
            results,
            errors,
            baseline,
        })
    }
}
