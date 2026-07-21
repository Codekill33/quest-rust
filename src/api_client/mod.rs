pub mod achievement;
pub mod auth;
pub mod config;
pub mod error;
pub mod leaderboard;
pub mod offline_queue;
pub mod retry;
pub mod session;

use achievement::MintResponse;
use auth::{LoginRequest, LoginResponse};
use config::ApiClientConfig;
use error::ApiClientError;
use leaderboard::LeaderboardEntry;
use offline_queue::{is_offline_error, OfflineQueue, QueuedAction};
use retry::with_retry;
use session::Session;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct ApiClient {
    http: reqwest::Client,
    config: ApiClientConfig,
    token: Arc<RwLock<Option<String>>>,
    queue: OfflineQueue,
}

impl ApiClient {
    pub fn new(config: ApiClientConfig) -> Self {
        let http = reqwest::Client::builder()
            .timeout(config.timeout)
            .build()
            .expect("failed to build reqwest client");

        Self {
            http,
            config,
            token: Arc::new(RwLock::new(None)),
            queue: OfflineQueue::new(),
        }
    }

    pub async fn token(&self) -> Option<String> {
        self.token.read().await.clone()
    }

    pub async fn authenticate(&self, username: &str, password: &str) -> Result<(), ApiClientError> {
        let url = format!("{}/auth/login", self.config.base_url);
        let body = LoginRequest { username, password };

        let resp = with_retry(self.config.max_retries, self.config.base_delay, || async {
            self.http
                .post(&url)
                .json(&body)
                .send()
                .await?
                .error_for_status()
                .map_err(ApiClientError::from)?
                .json::<LoginResponse>()
                .await
                .map_err(ApiClientError::from)
        })
        .await?;

        *self.token.write().await = Some(resp.access_token);
        Ok(())
    }

    pub async fn submit_session(&self, session: &Session) -> Result<(), ApiClientError> {
        let url = format!("{}/sessions", self.config.base_url);
        let token = self.token().await.ok_or(ApiClientError::Unauthorized)?;

        let result = with_retry(self.config.max_retries, self.config.base_delay, || async {
            self.http
                .post(&url)
                .bearer_auth(&token)
                .json(session)
                .send()
                .await?
                .error_for_status()
                .map(|_| ())
                .map_err(ApiClientError::from)
        })
        .await;

        if let Err(ApiClientError::Network(ref e)) = result {
            if is_offline_error(e) {
                let payload = serde_json::to_string(session)?;
                self.queue.push(QueuedAction::SubmitSession(payload)).await;
                return Err(ApiClientError::Offline);
            }
        }
        result
    }

    pub async fn fetch_leaderboard(&self) -> Result<Vec<LeaderboardEntry>, ApiClientError> {
        let url = format!("{}/leaderboard", self.config.base_url);

        with_retry(self.config.max_retries, self.config.base_delay, || async {
            self.http
                .get(&url)
                .send()
                .await?
                .error_for_status()
                .map_err(ApiClientError::from)?
                .json::<Vec<LeaderboardEntry>>()
                .await
                .map_err(ApiClientError::from)
        })
        .await
    }

    pub async fn mint_achievement(&self, achievement_id: &str) -> Result<MintResponse, ApiClientError> {
        let url = format!("{}/nft/mint", self.config.base_url);
        let token = self.token().await.ok_or(ApiClientError::Unauthorized)?;

        let result = with_retry(self.config.max_retries, self.config.base_delay, || async {
            self.http
                .post(&url)
                .bearer_auth(&token)
                .json(&serde_json::json!({ "achievement_id": achievement_id }))
                .send()
                .await?
                .error_for_status()
                .map_err(ApiClientError::from)?
                .json::<MintResponse>()
                .await
                .map_err(ApiClientError::from)
        })
        .await;

        if let Err(ApiClientError::Network(ref e)) = result {
            if is_offline_error(e) {
                self.queue
                    .push(QueuedAction::MintAchievement(achievement_id.to_string()))
                    .await;
                return Err(ApiClientError::Offline);
            }
        }
        result
    }

    pub async fn pending_actions_count(&self) -> usize {
        self.queue.len().await
    }

    /// Call this once connectivity is restored to flush queued actions.
    pub async fn sync_pending(&self) -> Vec<Result<(), ApiClientError>> {
        let actions = self.queue.drain().await;
        let mut results = Vec::new();

        for action in actions {
            let result = match action {
                QueuedAction::SubmitSession(json) => {
                    match serde_json::from_str::<Session>(&json) {
                        Ok(session) => self.submit_session(&session).await,
                        Err(e) => Err(ApiClientError::from(e)),
                    }
                }
                QueuedAction::MintAchievement(id) => {
                    self.mint_achievement(&id).await.map(|_| ())
                }
            };
            results.push(result);
        }
        results
    }
}

#[cfg(test)]
mod tests;