use crate::plugin::Plugin;
use crate::types::ScanResult;

pub struct FingerprintPlugin;

impl Plugin for FingerprintPlugin {
    fn name(&self) -> &str { "fingerprint" }
    fn version(&self) -> &str { "0.1.0" }

    fn on_result(&self, result: &mut ScanResult) {
        if let Some(server) = result.headers.get("server") {
            let current = result.content_type.as_deref().unwrap_or("unknown");
            result.content_type = Some(format!("{} (server: {})", current, server));
        }
    }
}
