use pathscan_core::types::{ScanReport, ScanResult};

pub struct TextFormatter {
    pub color: bool,
}

impl TextFormatter {
    fn status_color(&self, status: u16, text: &str) -> String {
        if !self.color {
            return text.to_string();
        }
        match status {
            200..=299 => format!("\x1b[32m{}\x1b[0m", text),
            300..=399 => format!("\x1b[33m{}\x1b[0m", text),
            400..=499 => format!("\x1b[34m{}\x1b[0m", text),
            _ => format!("\x1b[31m{}\x1b[0m", text),
        }
    }

    pub fn format_result(&self, result: &ScanResult) -> String {
        let status = format!("[{:>3}]", result.status);
        let colored = self.status_color(result.status, &status);
        let redirect = result
            .redirect_location
            .as_ref()
            .map(|l| format!(" -> {}", l))
            .unwrap_or_default();
        format!("{} {}{}", colored, result.url, redirect)
    }

    pub fn format_report(&self, report: &ScanReport) -> String {
        let mut out = String::new();
        out.push_str(&format!("Target:   {}\n", report.target));
        out.push_str(&format!("Duration: {:.2}s\n", report.duration_secs));
        out.push_str(&format!("Requests: {}\n", report.total_requests));
        out.push_str(&format!("Found:    {}\n", report.results.len()));
        out.push_str(&format!("Errors:   {}\n\n", report.errors));
        for r in &report.results {
            out.push_str(&self.format_result(r));
            out.push('\n');
        }
        out
    }
}

pub struct JsonFormatter;

impl JsonFormatter {
    pub fn format_report(&self, report: &ScanReport) -> String {
        serde_json::to_string_pretty(report).unwrap_or_default()
    }
}

pub struct HtmlFormatter;

impl HtmlFormatter {
    pub fn format_report(&self, report: &ScanReport) -> String {
        let mut html = String::from(r##"<!DOCTYPE html>
<html lang="en"><head><meta charset="UTF-8"><title>PathScan Report</title>
<style>
body { font-family: -apple-system, sans-serif; max-width: 960px; margin: 0 auto; padding: 2rem; background: #1a1a2e; color: #eee; }
table { width: 100%; border-collapse: collapse; margin-top: 1rem; }
th, td { padding: 0.5rem 1rem; text-align: left; border-bottom: 1px solid #333; }
th { background: #16213e; }
.status-2xx { color: #4caf50; } .status-3xx { color: #ff9800; }
.status-4xx { color: #2196f3; } .status-5xx { color: #f44336; }
.summary { display: flex; gap: 2rem; margin: 1rem 0; }
.summary div { background: #16213e; padding: 1rem 2rem; border-radius: 8px; text-align: center; }
.summary .value { font-size: 2rem; font-weight: bold; }
</style></head><body>
<h1>PathScan Report</h1>
<div class="summary">
<div><div class="value">"##);
        html.push_str(&report.total_requests.to_string());
        html.push_str("</div>Requests</div><div><div class=\"value\">");
        html.push_str(&report.results.len().to_string());
        html.push_str("</div>Found</div><div><div class=\"value\">");
        html.push_str(&report.errors.to_string());
        html.push_str("</div>Errors</div><div><div class=\"value\">");
        html.push_str(&format!("{:.1}s", report.duration_secs));
        html.push_str("</div>Duration</div></div><table><thead><tr><th>Status</th><th>URL</th><th>Length</th><th>Redirect</th></tr></thead><tbody>");

        for r in &report.results {
            let cls = match r.status {
                200..=299 => "status-2xx",
                300..=399 => "status-3xx",
                400..=499 => "status-4xx",
                _ => "status-5xx",
            };
            html.push_str(&format!(
                "<tr><td class=\"{}\">{}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
                cls,
                r.status,
                r.url,
                r.content_length,
                r.redirect_location.as_deref().unwrap_or("-")
            ));
        }
        html.push_str("</tbody></table></body></html>");
        html
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::time::Duration;

    fn sample_result(url: &str, status: u16) -> ScanResult {
        ScanResult {
            url: url.into(),
            path: "test".into(),
            status,
            content_length: 1000,
            content_type: Some("text/html".into()),
            response_time: Duration::from_millis(50),
            redirect_location: None,
            is_directory: false,
            content_hash: "abc".into(),
            headers: HashMap::new(),
        }
    }

    #[test]
    fn test_text_formatter_color() {
        let fmt = TextFormatter { color: true };
        let out = fmt.format_result(&sample_result("http://x.com/admin", 200));
        assert!(out.contains("\x1b[32m"));
    }

    #[test]
    fn test_text_formatter_no_color() {
        let fmt = TextFormatter { color: false };
        let out = fmt.format_result(&sample_result("http://x.com/admin", 200));
        assert!(!out.contains("\x1b"));
    }

    #[test]
    fn test_json_formatter() {
        let report = ScanReport {
            target: "http://x.com".into(),
            duration_secs: 1.5,
            total_requests: 10,
            results: vec![sample_result("http://x.com/admin", 200)],
            errors: 1,
            baseline: None,
        };
        let json = JsonFormatter.format_report(&report);
        assert!(json.contains("\"target\""));
    }

    #[test]
    fn test_html_formatter() {
        let report = ScanReport {
            target: "http://x.com".into(),
            duration_secs: 1.0,
            total_requests: 5,
            results: vec![sample_result("http://x.com/admin", 200)],
            errors: 0,
            baseline: None,
        };
        let html = HtmlFormatter.format_report(&report);
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("200"));
    }
}
