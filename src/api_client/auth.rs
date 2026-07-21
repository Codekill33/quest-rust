use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct LoginRequest<'a> {
    pub username: &'a str,
    pub password: &'a str,
}

#[derive(Debug, Deserialize)]
pub struct LoginResponse {
    pub access_token: String,
}