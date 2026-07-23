pub mod cli;
pub mod engine;
pub mod logic;
pub mod session;
pub mod engine;
pub mod l10n;

#[cfg(not(feature = "wasm"))]
pub mod api_client;

pub mod config;
pub mod difficulty;
pub mod errors;
pub mod events;
pub mod hints;
pub mod input;
pub mod inventory;
pub mod leaderboard;
pub mod loader;
pub mod player;
pub mod puzzle;
pub mod nft;
pub mod plugin;
pub mod score;
pub mod timer;
pub mod time;
pub mod persistence;

#[cfg(feature = "wasm")]
pub mod wasm;
