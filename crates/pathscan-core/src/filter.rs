use crate::types::{Baseline, ScanConfig, ScanResult};
use regex::Regex;

pub struct FilterPipeline {
    status_whitelist: Vec<u16>,
    status_blacklist: Vec<u16>,
    enable_length_filter: bool,
    baseline_length: Option<u64>,
    content_regex: Option<Regex>,
}

impl FilterPipeline {
    pub fn new(config: &ScanConfig, baseline: Option<&Baseline>) -> Self {
        let baseline_length = if config.enable_length_filter {
            baseline.map(|b| b.typical_length)
        } else {
            None
        };
        let content_regex = config
            .content_regex
            .as_ref()
            .and_then(|p| Regex::new(p).ok());

        Self {
            status_whitelist: config.status_whitelist.clone(),
            status_blacklist: config.status_blacklist.clone(),
            enable_length_filter: config.enable_length_filter,
            baseline_length,
            content_regex,
        }
    }

    pub fn run(&self, result: ScanResult) -> Option<ScanResult> {
        if self.status_blacklist.contains(&result.status) {
            return None;
        }

        if !self.status_whitelist.is_empty()
            && !self.status_whitelist.contains(&result.status)
        {
            return None;
        }

        if self.enable_length_filter {
            if let Some(bl) = self.baseline_length {
                if result.content_length == bl {
                    return None;
                }
            }
        }

        if let Some(ref regex) = self.content_regex {
            if !regex.is_match(&result.url) {
                return None;
            }
        }

        Some(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::time::Duration;

    fn make_result(status: u16, len: u64) -> ScanResult {
        ScanResult {
            url: "http://example.com/test".into(),
            path: "test".into(),
            status,
            content_length: len,
            content_type: None,
            response_time: Duration::from_millis(100),
            redirect_location: None,
            is_directory: false,
            content_hash: "abc".into(),
            headers: HashMap::new(),
        }
    }

    #[test]
    fn test_status_filter_passes() {
        let config = ScanConfig::new("http://example.com".into());
        let pipeline = FilterPipeline::new(&config, None);
        assert!(pipeline.run(make_result(200, 1000)).is_some());
    }

    #[test]
    fn test_status_filter_blocks() {
        let mut config = ScanConfig::new("http://example.com".into());
        config.status_whitelist = vec![200, 301];
        let pipeline = FilterPipeline::new(&config, None);
        assert!(pipeline.run(make_result(404, 1000)).is_none());
    }

    #[test]
    fn test_length_filter_blocks_baseline_match() {
        let config = ScanConfig::new("http://example.com".into());
        let baseline = Baseline {
            typical_status: 404,
            typical_length: 1234,
            typical_hash: "hash".into(),
            samples: vec![(404, 1234)],
        };
        let pipeline = FilterPipeline::new(&config, Some(&baseline));
        assert!(pipeline.run(make_result(200, 1234)).is_none());
    }

    #[test]
    fn test_length_filter_disabled() {
        let mut config = ScanConfig::new("http://example.com".into());
        config.enable_length_filter = false;
        let baseline = Baseline {
            typical_status: 404,
            typical_length: 1234,
            typical_hash: "hash".into(),
            samples: vec![(404, 1234)],
        };
        let pipeline = FilterPipeline::new(&config, Some(&baseline));
        assert!(pipeline.run(make_result(200, 1234)).is_some());
    }
}
