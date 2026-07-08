use crate::plugin::Plugin;
use crate::types::ScanResult;

pub struct ScreenshotPlugin {
    pub enabled: bool,
}

impl ScreenshotPlugin {
    pub fn new_disabled() -> Self { Self { enabled: false } }
}

impl Plugin for ScreenshotPlugin {
    fn name(&self) -> &str { "screenshot" }
    fn version(&self) -> &str { "0.1.0" }

    fn on_result(&self, result: &mut ScanResult) {
        if self.enabled && result.status == 200 {
            // Stub: requires headless browser integration
        }
    }
}
