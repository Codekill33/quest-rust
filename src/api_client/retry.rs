use crate::api_client::error::ApiClientError;
use std::future::Future;
use std::time::Duration;

/// Retries an async operation up to `max_retries` times with exponential backoff.
/// Only retries on network-level errors (timeouts, connection failures).
pub async fn with_retry<F, Fut, T>(
    max_retries: u32,
    base_delay: Duration,
    mut op: F,
) -> Result<T, ApiClientError>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, ApiClientError>>,
{
    let mut attempt = 0;

    loop {
        match op().await {
            Ok(value) => return Ok(value),
            Err(err) if attempt < max_retries && is_retryable(&err) => {
                let delay = base_delay * 2u32.pow(attempt);
                tokio::time::sleep(delay).await;
                attempt += 1;
            }
            Err(err) => return Err(err),
        }
    }
}

fn is_retryable(err: &ApiClientError) -> bool {
    matches!(err, ApiClientError::Network(e) if e.is_timeout() || e.is_connect())
}