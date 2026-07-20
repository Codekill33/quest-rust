//! Errors module.
//!
//! Centralises all custom error types and handling logic into a dedicated
//! [`AppError`] enum that every module in the codebase uses instead of
//! ad-hoc error strings or module-specific error types.

use std::fmt;

/// Top-level error type covering all major error categories in the game.
///
/// Every module converts its internal errors into [`AppError`] so that
/// callers can handle failures uniformly.
#[derive(Debug)]
pub enum AppError {
    // ── I/O ──────────────────────────────────────────────────────────────────
    /// An underlying I/O error (e.g. file not found, permission denied).
    Io(std::io::Error),

    // ── Serialisation ────────────────────────────────────────────────────────
    /// A JSON serialisation or deserialisation error.
    Serde(serde_json::Error),

    // ── Config ───────────────────────────────────────────────────────────────
    /// The TOML configuration could not be parsed.
    ConfigParse(String),

    // ── Input ────────────────────────────────────────────────────────────────
    /// The input string was not recognised as a valid game action.
    InputInvalid(String),
    /// No input was provided.
    InputEmpty,

    // ── NFT / Achievements ───────────────────────────────────────────────────
    /// The achievement has already been minted for this player.
    NftAlreadyMinted {
        /// The player ID.
        player_id: String,
        /// The milestone type that was already minted.
        milestone_type: String,
    },

    // ── Player ───────────────────────────────────────────────────────────────
    /// A player with the given identifier could not be found.
    PlayerNotFound(String),

    // ── Inventory ────────────────────────────────────────────────────────────
    /// An item with the given ID does not exist in the inventory.
    InventoryItemNotFound(String),

    // ── Puzzle ───────────────────────────────────────────────────────────────
    /// A general puzzle-domain error.
    Puzzle(String),
    /// A puzzle definition failed content-hash integrity verification.
    PuzzleIntegrity(crate::puzzle::PuzzleIntegrityError),

    // ── Leaderboard ──────────────────────────────────────────────────────────
    /// A leaderboard-domain error.
    Leaderboard(String),
}

// ── Display ──────────────────────────────────────────────────────────────────

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Io(e) => write!(f, "I/O error: {e}"),
            AppError::Serde(e) => write!(f, "serialisation error: {e}"),
            AppError::ConfigParse(msg) => write!(f, "config parse error: {msg}"),
            AppError::InputInvalid(s) => write!(f, "invalid input: '{s}'"),
            AppError::InputEmpty => write!(f, "input cannot be empty"),
            AppError::NftAlreadyMinted {
                player_id,
                milestone_type,
            } => {
                write!(
                    f,
                    "achievement '{milestone_type}' already minted for player '{player_id}'"
                )
            }
            AppError::PlayerNotFound(id) => write!(f, "player '{id}' not found"),
            AppError::InventoryItemNotFound(id) => {
                write!(f, "item '{id}' not found in inventory")
            }
            AppError::Puzzle(msg) => write!(f, "puzzle error: {msg}"),
            AppError::PuzzleIntegrity(e) => write!(f, "puzzle integrity error: {e}"),
            AppError::Leaderboard(msg) => write!(f, "leaderboard error: {msg}"),
        }
    }
}

// ── std::error::Error ────────────────────────────────────────────────────────

impl std::error::Error for AppError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            AppError::Io(e) => Some(e),
            AppError::Serde(e) => Some(e),
            AppError::PuzzleIntegrity(e) => Some(e),
            _ => None,
        }
    }
}

// ── From implementations ─────────────────────────────────────────────────────

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        AppError::Io(e)
    }
}

impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self {
        AppError::Serde(e)
    }
}

impl From<crate::puzzle::PuzzleIntegrityError> for AppError {
    fn from(e: crate::puzzle::PuzzleIntegrityError) -> Self {
        AppError::PuzzleIntegrity(e)
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    // ── Display tests ────────────────────────────────────────────────────────

    #[test]
    fn display_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file missing");
        let err = AppError::Io(io_err);
        let msg = err.to_string();
        assert!(msg.contains("I/O error"));
        assert!(msg.contains("file missing"));
    }

    #[test]
    fn display_serde_error() {
        let serde_err = serde_json::from_str::<()>("invalid").unwrap_err();
        let err = AppError::Serde(serde_err);
        let msg = err.to_string();
        assert!(msg.contains("serialisation error"));
    }

    #[test]
    fn display_config_parse() {
        let err = AppError::ConfigParse("bad toml".into());
        assert_eq!(err.to_string(), "config parse error: bad toml");
    }

    #[test]
    fn display_input_invalid() {
        let err = AppError::InputInvalid("xyz".into());
        assert_eq!(err.to_string(), "invalid input: 'xyz'");
    }

    #[test]
    fn display_input_empty() {
        let err = AppError::InputEmpty;
        assert_eq!(err.to_string(), "input cannot be empty");
    }

    #[test]
    fn display_nft_already_minted() {
        let err = AppError::NftAlreadyMinted {
            player_id: "p1".into(),
            milestone_type: "level10".into(),
        };
        assert_eq!(
            err.to_string(),
            "achievement 'level10' already minted for player 'p1'"
        );
    }

    #[test]
    fn display_player_not_found() {
        let err = AppError::PlayerNotFound("hero".into());
        assert_eq!(err.to_string(), "player 'hero' not found");
    }

    #[test]
    fn display_inventory_not_found() {
        let err = AppError::InventoryItemNotFound("sword".into());
        assert_eq!(err.to_string(), "item 'sword' not found in inventory");
    }

    #[test]
    fn display_puzzle_error() {
        let err = AppError::Puzzle("invalid move".into());
        assert_eq!(err.to_string(), "puzzle error: invalid move");
    }

    #[test]
    fn display_leaderboard_error() {
        let err = AppError::Leaderboard("capacity reached".into());
        assert_eq!(err.to_string(), "leaderboard error: capacity reached");
    }

    #[test]
    fn display_puzzle_integrity_error() {
        let inner = crate::puzzle::PuzzleIntegrityError::MissingHash {
            puzzle_id: "p1".into(),
        };
        let err = AppError::PuzzleIntegrity(inner);
        let msg = err.to_string();
        assert!(msg.contains("puzzle integrity error"));
        assert!(msg.contains("p1"));
    }

    // ── Error trait tests ────────────────────────────────────────────────────

    #[test]
    fn io_error_has_source() {
        let inner = std::io::Error::new(std::io::ErrorKind::Other, "reason");
        let err = AppError::Io(inner);
        assert!(err.source().is_some());
    }

    #[test]
    fn serde_error_has_source() {
        let inner = serde_json::from_str::<()>("[").unwrap_err();
        let err = AppError::Serde(inner);
        assert!(err.source().is_some());
    }

    #[test]
    fn puzzle_integrity_error_has_source() {
        let inner = crate::puzzle::PuzzleIntegrityError::MissingHash {
            puzzle_id: "p1".into(),
        };
        let err = AppError::PuzzleIntegrity(inner);
        assert!(err.source().is_some());
    }

    #[test]
    fn non_wrapping_variants_have_no_source() {
        assert!(AppError::ConfigParse("x".into()).source().is_none());
        assert!(AppError::InputInvalid("x".into()).source().is_none());
        assert!(AppError::InputEmpty.source().is_none());
        assert!(AppError::PlayerNotFound("x".into()).source().is_none());
        assert!(AppError::InventoryItemNotFound("x".into()).source().is_none());
        assert!(AppError::Puzzle("x".into()).source().is_none());
        assert!(AppError::Leaderboard("x".into()).source().is_none());
        assert!(AppError::NftAlreadyMinted {
            player_id: "p".into(),
            milestone_type: "m".into(),
        }
        .source()
        .is_none());
    }

    // ── From impl tests ──────────────────────────────────────────────────────

    #[test]
    fn from_io_error() {
        let io = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "denied");
        let app: AppError = io.into();
        assert!(matches!(app, AppError::Io(_)));
    }

    #[test]
    fn from_serde_error() {
        let serde = serde_json::from_str::<()>("%%%").unwrap_err();
        let app: AppError = serde.into();
        assert!(matches!(app, AppError::Serde(_)));
    }

    #[test]
    fn from_puzzle_integrity_error() {
        let inner = crate::puzzle::PuzzleIntegrityError::MissingHash {
            puzzle_id: "p1".into(),
        };
        let app: AppError = inner.into();
        assert!(matches!(app, AppError::PuzzleIntegrity(_)));
    }
}
