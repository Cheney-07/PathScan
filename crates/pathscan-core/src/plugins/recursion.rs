use crate::plugin::Plugin;
use crate::types::ScanResult;

pub struct RecursionPlugin;

impl Plugin for RecursionPlugin {
    fn name(&self) -> &str { "recursion" }
    fn version(&self) -> &str { "0.1.0" }

    fn on_result(&self, result: &mut ScanResult) {
        let redirects_to_dir = matches!(result.status, 301 | 302)
            && result.redirect_location.as_deref().map_or(false, |loc| loc.ends_with('/'));
        if result.status == 200 || redirects_to_dir {
            result.is_directory = true;
        }
    }
}
