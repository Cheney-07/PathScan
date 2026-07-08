use pathscan_core::engine::ScanEvent;
use pathscan_core::types::{ScanResult, ScanStatus};
use tokio::sync::mpsc::UnboundedReceiver;

pub struct App {
    pub results: Vec<ScanResult>,
    pub selected_index: usize,
    pub status: ScanStatus,
    pub completed: usize,
    pub total: usize,
    pub errors: Vec<String>,
    pub rx: UnboundedReceiver<ScanEvent>,
    pub focus: Focus,
    pub filter_text: String,
    pub scroll_offset: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    Log,
    Stats,
}

impl App {
    pub fn new(rx: UnboundedReceiver<ScanEvent>, total: usize) -> Self {
        Self {
            results: Vec::new(),
            selected_index: 0,
            status: ScanStatus::Running,
            completed: 0,
            total,
            errors: Vec::new(),
            rx,
            focus: Focus::Log,
            filter_text: String::new(),
            scroll_offset: 0,
        }
    }

    pub fn update(&mut self) {
        while let Ok(event) = self.rx.try_recv() {
            match event {
                ScanEvent::Progress { completed, total } => {
                    self.completed = completed;
                    self.total = total;
                }
                ScanEvent::Result(result) => {
                    self.results.push(result);
                }
                ScanEvent::Status(status) => {
                    self.status = status;
                }
                ScanEvent::Error(e) => {
                    self.errors.push(e);
                }
            }
        }
    }

    pub fn selected_result(&self) -> Option<&ScanResult> {
        if self.results.is_empty() {
            None
        } else {
            Some(&self.results[self.selected_index.min(self.results.len() - 1)])
        }
    }

    pub fn progress_pct(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            self.completed as f64 / self.total as f64
        }
    }

    pub fn status_counts(&self) -> (usize, usize, usize, usize) {
        let (mut s2xx, mut s3xx, mut s4xx, mut s5xx) = (0, 0, 0, 0);
        for r in &self.results {
            match r.status {
                200..=299 => s2xx += 1,
                300..=399 => s3xx += 1,
                400..=499 => s4xx += 1,
                _ => s5xx += 1,
            }
        }
        (s2xx, s3xx, s4xx, s5xx)
    }

    pub fn filtered_results(&self) -> Vec<&ScanResult> {
        if self.filter_text.is_empty() {
            self.results.iter().collect()
        } else {
            let q = self.filter_text.to_lowercase();
            self.results
                .iter()
                .filter(|r| r.url.to_lowercase().contains(&q))
                .collect()
        }
    }
}
