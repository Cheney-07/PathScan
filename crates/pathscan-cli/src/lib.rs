mod args;
mod output;

use args::CliArgs;
use clap::Parser;
use output::{HtmlFormatter, JsonFormatter, TextFormatter};
use pathscan_core::engine::{ScanEngine, ScanEvent};
use pathscan_core::types::ScanConfig;
use std::fs;
use std::io::{self, Write};
use tokio::sync::mpsc;

pub async fn run() {
    let args = CliArgs::parse();
    let output_prefix = args.output.clone().unwrap_or_else(|| "pathscan_report".to_string());
    let use_json = args.json;
    let use_html = args.html;
    let no_color = args.no_color;
    let config: ScanConfig = args.into();

    let (tx, mut rx) = mpsc::unbounded_channel();

    let config_clone = config.clone();
    let handle = tokio::spawn(async move {
        ScanEngine::run(config_clone, Some(tx)).await
    });

    while let Some(event) = rx.recv().await {
        match event {
            ScanEvent::Result(ref result) => {
                let fmt = TextFormatter { color: !no_color };
                println!("{}", fmt.format_result(result));
            }
            ScanEvent::Progress { completed, total } => {
                eprint!("\rProgress: {}/{}", completed, total);
                let _ = io::stderr().flush();
            }
            ScanEvent::Error(_) => {}
            ScanEvent::Status(_) => {}
        }
    }

    match handle.await {
        Ok(Ok(report)) => {
            eprintln!("");

            let text_fmt = TextFormatter { color: !no_color };
            eprintln!("{}", text_fmt.format_report(&report));

            if use_json {
                let json_fmt = JsonFormatter;
                let path = format!("{}.json", output_prefix);
                fs::write(&path, json_fmt.format_report(&report))
                    .unwrap_or_else(|e| eprintln!("Failed to write JSON: {}", e));
                eprintln!("JSON report: {}", path);
            }
            if use_html {
                let html_fmt = HtmlFormatter;
                let path = format!("{}.html", output_prefix);
                fs::write(&path, html_fmt.format_report(&report))
                    .unwrap_or_else(|e| eprintln!("Failed to write HTML: {}", e));
                eprintln!("HTML report: {}", path);
            }
        }
        Ok(Err(e)) => {
            eprintln!("Scan failed: {}", e);
            std::process::exit(1);
        }
        Err(_) => {
            eprintln!("Scan task panicked");
            std::process::exit(1);
        }
    }
}
