use crate::error::{PathScanError, Result};
use crate::types::{RequestContext, ResponseContext, ScanConfig};
use std::collections::HashMap;
use std::time::Instant;

pub struct HttpClient {
    client: reqwest::Client,
    user_agent: String,
}

impl HttpClient {
    pub fn new(config: &ScanConfig) -> Result<Self> {
        let mut builder = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout_secs))
            .redirect(reqwest::redirect::Policy::none());

        if config.insecure {
            builder = builder.danger_accept_invalid_certs(true);
        }

        if let Some(ref proxy) = config.proxy {
            builder = builder.proxy(
                reqwest::Proxy::all(proxy)
                    .map_err(|e| PathScanError::Other(format!("invalid proxy: {e}")))?,
            );
        }

        let user_agent = config
            .user_agent
            .clone()
            .unwrap_or_else(|| "PathScan/0.1.0".to_string());

        Ok(Self {
            client: builder.build()?,
            user_agent,
        })
    }

    pub async fn execute(&self, ctx: RequestContext) -> Result<ResponseContext> {
        let start = Instant::now();

        let mut req = self
            .client
            .request(ctx.method.clone(), &ctx.url)
            .header("User-Agent", &self.user_agent);

        for (k, v) in &ctx.headers {
            req = req.header(k.as_str(), v.as_str());
        }

        let resp = req.send().await?;
        let _elapsed = start.elapsed();

        let status = resp.status().as_u16();
        let mut headers: HashMap<String, String> = HashMap::new();
        for (k, v) in resp.headers() {
            if let Ok(v) = v.to_str() {
                headers.insert(k.to_string(), v.to_string());
            }
        }

        let body = resp.text().await.unwrap_or_default();
        let content_length = body.len() as u64;
        let body_snippet = body.chars().take(200).collect();

        Ok(ResponseContext {
            url: ctx.url,
            status,
            headers,
            content_length,
            body_snippet,
        })
    }
}
