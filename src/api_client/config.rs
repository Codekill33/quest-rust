use std::time::Duration;

#[derive(Debug, Clone)]
pub struct ApiClientConfig {
    pub base_url: String,
    pub timeout: Duration,
    pub max_retries: u32,
    pub base_delay: Duration,
}

impl Default for ApiClientConfig {
    fn default() -> Self {
        Self {
            base_url: "http://localhost:3000".to_string(),
            timeout: Duration::from_secs(10),
            max_retries: 3,
            base_delay: Duration::from_millis(500),
        }
    }
}