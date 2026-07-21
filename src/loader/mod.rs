//! Loader module.
//!
//! Loads puzzle definitions from JSON files on disk and enforces content
//! integrity: every puzzle file must carry a `content_hash` that matches its
//! canonical content before it is handed to the engine. This closes off
//! cheating via hand-edited local puzzle files.

use crate::errors::AppError;
use crate::puzzle::Puzzle;
use std::fs;
use std::path::{Path, PathBuf};

/// Loads a single puzzle definition from `path` and verifies its content
/// hash before returning it.
///
/// Returns an [`AppError::PuzzleIntegrity`] if the hash is missing or does
/// not match the file's actual content, and [`AppError::Io`] /
/// [`AppError::Serde`] for filesystem or parse failures.
pub fn load_puzzle_file(path: impl AsRef<Path>) -> Result<Puzzle, AppError> {
    let raw = fs::read_to_string(path.as_ref())?;
    let puzzle: Puzzle = serde_json::from_str(&raw)?;
    puzzle.verify_content_hash()?;
    Ok(puzzle)
}

/// Loads every `*.json` puzzle file in `dir`, verifying each one's content
/// hash. Fails fast on the first integrity or parse failure — use
/// [`verify_puzzles_in_dir`] instead when a full report of every failure is
/// needed (e.g. for the `--verify-puzzles` CLI command).
pub fn load_puzzles_from_dir(dir: impl AsRef<Path>) -> Result<Vec<Puzzle>, AppError> {
    puzzle_file_paths(dir.as_ref())?
        .into_iter()
        .map(load_puzzle_file)
        .collect()
}

/// The outcome of verifying a single puzzle file's integrity.
#[derive(Debug)]
pub struct VerifyReport {
    /// Path of the puzzle file that was checked.
    pub path: PathBuf,
    /// `Ok` if the file's content hash matched; `Err` otherwise.
    pub result: Result<(), AppError>,
}

/// Verifies the content hash of every `*.json` puzzle file in `dir`,
/// collecting a report for each file rather than stopping at the first
/// failure. Backs the `--verify-puzzles` CLI command.
pub fn verify_puzzles_in_dir(dir: impl AsRef<Path>) -> Result<Vec<VerifyReport>, AppError> {
    let paths = puzzle_file_paths(dir.as_ref())?;
    Ok(paths
        .into_iter()
        .map(|path| {
            let result = load_puzzle_file(&path).map(|_| ());
            VerifyReport { path, result }
        })
        .collect())
}

/// Recomputes and writes the `content_hash` for every `*.json` puzzle file
/// in `dir`, overwriting each file in place. Intended for admin/authoring
/// use only after a legitimate content change — callers must gate access to
/// this behind an explicit admin flag (see the `--generate-hashes` CLI
/// command, which requires `--admin`).
///
/// Returns the number of files updated.
pub fn generate_hashes_in_dir(dir: impl AsRef<Path>) -> Result<usize, AppError> {
    let paths = puzzle_file_paths(dir.as_ref())?;
    for path in &paths {
        let raw = fs::read_to_string(path)?;
        let mut puzzle: Puzzle = serde_json::from_str(&raw)?;
        puzzle.generate_content_hash();
        let updated = serde_json::to_string_pretty(&puzzle)?;
        fs::write(path, updated)?;
    }
    Ok(paths.len())
}

/// Returns the sorted list of `*.json` files directly inside `dir`.
fn puzzle_file_paths(dir: &Path) -> Result<Vec<PathBuf>, AppError> {
    let mut paths: Vec<PathBuf> = fs::read_dir(dir)?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
        .collect();
    paths.sort();
    Ok(paths)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::puzzle::{Condition, Effect};
    use std::fs;
    use tempfile::TempDir;

    fn sample_puzzle() -> Puzzle {
        Puzzle::new(
            "loader-p1",
            "Open the vault",
            vec![Condition::new("enter_code", "Enter the code")],
            vec![Effect::AwardScore(75)],
        )
    }

    fn write_puzzle_file(dir: &Path, name: &str, puzzle: &Puzzle) -> PathBuf {
        let path = dir.join(name);
        fs::write(&path, serde_json::to_string_pretty(puzzle).unwrap()).unwrap();
        path
    }

    #[test]
    fn loads_puzzle_with_valid_hash() {
        let dir = TempDir::new().unwrap();
        let mut puzzle = sample_puzzle();
        puzzle.generate_content_hash();
        let path = write_puzzle_file(dir.path(), "p1.json", &puzzle);

        let loaded = load_puzzle_file(&path).expect("should load");
        assert_eq!(loaded.id, "loader-p1");
    }

    #[test]
    fn rejects_puzzle_with_missing_hash() {
        let dir = TempDir::new().unwrap();
        let puzzle = sample_puzzle(); // content_hash left as None
        let path = write_puzzle_file(dir.path(), "p1.json", &puzzle);

        let err = load_puzzle_file(&path).unwrap_err();
        assert!(matches!(err, AppError::PuzzleIntegrity(_)));
    }

    #[test]
    fn rejects_puzzle_with_tampered_content() {
        let dir = TempDir::new().unwrap();
        let mut puzzle = sample_puzzle();
        puzzle.generate_content_hash();
        let path = write_puzzle_file(dir.path(), "p1.json", &puzzle);

        // Simulate tampering: hand-edit the file after the hash was stamped.
        let raw = fs::read_to_string(&path).unwrap();
        let tampered = raw.replace("Open the vault", "Open the vault (free score!)");
        fs::write(&path, tampered).unwrap();

        let err = load_puzzle_file(&path).unwrap_err();
        assert!(matches!(err, AppError::PuzzleIntegrity(_)));
    }

    #[test]
    fn load_puzzles_from_dir_returns_all_valid_puzzles() {
        let dir = TempDir::new().unwrap();
        let mut p1 = sample_puzzle();
        p1.generate_content_hash();
        let mut p2 = Puzzle::new(
            "loader-p2",
            "Second puzzle",
            vec![],
            vec![Effect::AwardScore(5)],
        );
        p2.generate_content_hash();

        write_puzzle_file(dir.path(), "a.json", &p1);
        write_puzzle_file(dir.path(), "b.json", &p2);

        let puzzles = load_puzzles_from_dir(dir.path()).expect("should load all");
        assert_eq!(puzzles.len(), 2);
    }

    #[test]
    fn load_puzzles_from_dir_fails_fast_on_first_bad_file() {
        let dir = TempDir::new().unwrap();
        let mut good = sample_puzzle();
        good.generate_content_hash();
        let bad = Puzzle::new("loader-bad", "No hash", vec![], vec![]);

        write_puzzle_file(dir.path(), "a-good.json", &good);
        write_puzzle_file(dir.path(), "b-bad.json", &bad);

        let err = load_puzzles_from_dir(dir.path()).unwrap_err();
        assert!(matches!(err, AppError::PuzzleIntegrity(_)));
    }

    #[test]
    fn verify_puzzles_in_dir_reports_every_file() {
        let dir = TempDir::new().unwrap();
        let mut good = sample_puzzle();
        good.generate_content_hash();
        let bad = Puzzle::new("loader-bad", "No hash", vec![], vec![]);

        write_puzzle_file(dir.path(), "a-good.json", &good);
        write_puzzle_file(dir.path(), "b-bad.json", &bad);

        let reports = verify_puzzles_in_dir(dir.path()).expect("should enumerate both files");
        assert_eq!(reports.len(), 2);
        assert!(reports[0].result.is_ok());
        assert!(reports[1].result.is_err());
    }

    #[test]
    fn generate_hashes_in_dir_stamps_every_file() {
        let dir = TempDir::new().unwrap();
        let puzzle = sample_puzzle(); // no hash yet
        let path = write_puzzle_file(dir.path(), "p1.json", &puzzle);

        let updated = generate_hashes_in_dir(dir.path()).expect("should generate");
        assert_eq!(updated, 1);

        // The file should now load and verify successfully.
        let loaded = load_puzzle_file(&path).expect("should verify after generation");
        assert!(loaded.content_hash.is_some());
    }

    #[test]
    fn generate_hashes_then_verify_all_succeed() {
        let dir = TempDir::new().unwrap();
        let p1 = sample_puzzle();
        let p2 = Puzzle::new(
            "loader-p2",
            "Second puzzle",
            vec![],
            vec![Effect::AwardScore(5)],
        );
        write_puzzle_file(dir.path(), "a.json", &p1);
        write_puzzle_file(dir.path(), "b.json", &p2);

        generate_hashes_in_dir(dir.path()).expect("should generate");

        let reports = verify_puzzles_in_dir(dir.path()).expect("should enumerate");
        assert!(reports.iter().all(|r| r.result.is_ok()));
    }
}
