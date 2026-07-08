use crate::client::HttpClient;
use crate::error::Result;
use crate::types::{Baseline, RequestContext, ScanConfig};
use sha2::{Digest, Sha256};

const PROBE_SUFFIXES: &[&str] = &[
    "8a3f2b1c",
    "d7e4f9a0",
    "2c6b8f1e",
    "5f1a3d9b",
    "e8c2b47f",
];

pub struct BaselineCalibrator;

impl BaselineCalibrator {
    pub async fn calibrate(client: &HttpClient, config: &ScanConfig) -> Result<Baseline> {
        let base = config.target_url.trim_end_matches('/');
        let mut samples = Vec::new();

        for suffix in PROBE_SUFFIXES {
            let url = format!("{}/nonexistent_{}", base, suffix);
            let ctx = RequestContext {
                url,
                method: reqwest::Method::GET,
                headers: config.custom_headers.clone(),
                follow_redirects: false,
            };
            let resp = client.execute(ctx).await?;
            samples.push((resp.status, resp.content_length));
        }

        let typical = samples
            .iter()
            .max_by_key(|(s, l)| {
                samples.iter().filter(|(st, le)| st == s && le == l).count()
            })
            .cloned()
            .unwrap_or((404, 0));

        let mut hasher = Sha256::new();
        hasher.update(typical.1.to_le_bytes());
        let hash = format!("{:x}", hasher.finalize());

        Ok(Baseline {
            typical_status: typical.0,
            typical_length: typical.1,
            typical_hash: hash,
            samples,
        })
    }
}
