#[derive(Debug, Clone)]
pub enum Action {
    TestLatency { group: String, proxies: Vec<String> },
    ApplySelection { group: String, proxy: String },
}
