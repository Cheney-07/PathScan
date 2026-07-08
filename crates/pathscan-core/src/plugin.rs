use crate::types::{RequestContext, ResponseContext, ScanResult};

pub trait Plugin: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str;

    fn on_request(&self, req: RequestContext) -> RequestContext {
        req
    }

    fn on_response(&self, resp: ResponseContext) -> ResponseContext {
        resp
    }

    fn on_result(&self, _result: &mut ScanResult) {}
}

pub struct PluginChain {
    plugins: Vec<Box<dyn Plugin>>,
}

impl PluginChain {
    pub fn new() -> Self {
        Self { plugins: vec![] }
    }

    pub fn register(&mut self, plugin: Box<dyn Plugin>) {
        self.plugins.push(plugin);
    }

    pub fn run_on_request(&self, req: RequestContext) -> RequestContext {
        self.plugins.iter().fold(req, |r, p| p.on_request(r))
    }

    pub fn run_on_response(&self, resp: ResponseContext) -> ResponseContext {
        self.plugins.iter().fold(resp, |r, p| p.on_response(r))
    }

    pub fn run_on_result(&self, result: &mut ScanResult) {
        for p in &self.plugins {
            p.on_result(result);
        }
    }
}
