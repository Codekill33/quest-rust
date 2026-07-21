use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct MintResponse {
    pub transaction_id: String,
    pub status: String,
}