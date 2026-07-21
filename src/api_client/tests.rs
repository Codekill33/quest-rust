use super::*;
use serde_json::json;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn test_config(base_url: String) -> ApiClientConfig {
    ApiClientConfig {
        base_url,
        timeout: std::time::Duration::from_secs(5),
        max_retries: 3,
        base_delay: std::time::Duration::from_millis(10),
    }
}

#[tokio::test]
async fn authenticate_stores_token_on_success() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/auth/login"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "access_token": "abc123" })))
        .mount(&server)
        .await;

    let client = ApiClient::new(test_config(server.uri()));
    client.authenticate("user", "pass").await.unwrap();

    assert_eq!(client.token().await, Some("abc123".to_string()));
}

#[tokio::test]
async fn authenticate_fails_on_401() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/auth/login"))
        .respond_with(ResponseTemplate::new(401))
        .mount(&server)
        .await;

    let client = ApiClient::new(test_config(server.uri()));
    let result = client.authenticate("bad", "creds").await;

    assert!(result.is_err());
}

#[tokio::test]
async fn submit_session_success() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/sessions"))
        .respond_with(ResponseTemplate::new(201))
        .mount(&server)
        .await;

    let client = ApiClient::new(test_config(server.uri()));
    *client.token.write().await = Some("fake-jwt".to_string());

    let session = session::Session {
        player_id: "p1".into(),
        score: 100,
        duration_secs: 60,
        completed_at: chrono::Utc::now(),
    };

    assert!(client.submit_session(&session).await.is_ok());
}

#[tokio::test]
async fn fetch_leaderboard_deserializes_entries() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/leaderboard"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            { "player_id": "p1", "username": "alice", "score": 500, "rank": 1 }
        ])))
        .mount(&server)
        .await;

    let client = ApiClient::new(test_config(server.uri()));
    let leaderboard = client.fetch_leaderboard().await.unwrap();

    assert_eq!(leaderboard.len(), 1);
    assert_eq!(leaderboard[0].username, "alice");
}

#[tokio::test]
async fn retries_then_succeeds() {
    let server = MockServer::start().await;

    // First call fails, second succeeds
    Mock::given(method("GET"))
        .and(path("/leaderboard"))
        .respond_with(ResponseTemplate::new(500))
        .up_to_n_times(1)
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/leaderboard"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
        .mount(&server)
        .await;

    let client = ApiClient::new(test_config(server.uri()));
    let result = client.fetch_leaderboard().await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn offline_action_gets_queued() {
    // Point at a port nothing is listening on to force a connection error.
    let client = ApiClient::new(test_config("http://127.0.0.1:1".to_string()));
    *client.token.write().await = Some("fake-jwt".to_string());

    let session = session::Session {
        player_id: "p1".into(),
        score: 10,
        duration_secs: 5,
        completed_at: chrono::Utc::now(),
    };

    let result = client.submit_session(&session).await;
    assert!(matches!(result, Err(ApiClientError::Offline)));
    assert_eq!(client.pending_actions_count().await, 1);
}
