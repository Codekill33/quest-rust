//! Session module.
//!
//! Manages a single play session, tying together player, puzzle progress, timer, and score.

use crate::player::Player;
use crate::leaderboard::{Leaderboard, Entry};
use std::time::{Duration, Instant};

#[derive(Debug)]
pub struct Session {
    player: Player,
    active_puzzle: Option<usize>,
    timer: Timer,
    current_score: u64,
    /// Content hashes of puzzles completed so far this session, carried
    /// through to [`SessionData`] so they can be submitted to `logiquest_api`
    /// alongside the score for server-side authenticity verification.
    completed_puzzle_hashes: Vec<String>,
}

#[derive(Debug)]
struct Timer {
    start_time: Option<Instant>,
    elapsed: Duration,
    is_paused: bool,
}

impl Timer {
    fn new() -> Self {
        Self {
            start_time: None,
            elapsed: Duration::ZERO,
            is_paused: false,
        }
    }

    fn start(&mut self) {
        if !self.is_paused {
            self.start_time = Some(Instant::now());
        } else {
            self.start_time = Some(Instant::now() - self.elapsed);
            self.is_paused = false;
        }
    }

    fn pause(&mut self) {
        if let Some(start) = self.start_time {
            self.elapsed = start.elapsed();
            self.is_paused = true;
        }
    }

    fn stop(&mut self) -> Duration {
        if let Some(start) = self.start_time {
            if !self.is_paused {
                self.elapsed = start.elapsed();
            }
        }
        self.elapsed
    }
}

pub struct SessionData {
    pub player_id: String,
    pub final_score: u64,
    pub total_time: Duration,
    pub puzzles_completed: usize,
    /// Content hashes of every puzzle completed during the session, in
    /// completion order. Submitted to `logiquest_api` so the server can
    /// verify the score was derived from authentic, untampered puzzle
    /// content rather than a modified local puzzle file.
    pub puzzle_content_hashes: Vec<String>,
}

impl Session {
    pub fn new(player: Player) -> Self {
        Self {
            active_puzzle: Some(player.current_puzzle_index),
            player,
            timer: Timer::new(),
            current_score: 0,
            completed_puzzle_hashes: Vec::new(),
        }
    }

    pub fn start(&mut self) {
        self.timer.start();
    }

    pub fn pause(&mut self) {
        self.timer.pause();
    }

    pub fn end(mut self) -> SessionData {
        let total_time = self.timer.stop();
        let final_score = self.player.score + self.current_score;
        
        SessionData {
            player_id: self.player.id,
            final_score,
            total_time,
            puzzles_completed: self.player.current_puzzle_index,
            puzzle_content_hashes: self.completed_puzzle_hashes,
        }
    }

    pub fn add_score(&mut self, points: u64) {
        self.current_score = self.current_score.saturating_add(points);
    }

    pub fn complete_puzzle(&mut self) {
        self.player.add_score(self.current_score);
        self.player.advance_puzzle();
        self.active_puzzle = Some(self.player.current_puzzle_index);
        self.current_score = 0;
    }

    /// Like [`Session::complete_puzzle`], but also records the completed
    /// puzzle's content hash so it can be submitted to `logiquest_api` for
    /// server-side integrity verification. Call this instead of
    /// `complete_puzzle` when the puzzle was loaded (and hash-verified) via
    /// the [`crate::loader`] module.
    pub fn complete_puzzle_with_hash(&mut self, content_hash: impl Into<String>) {
        self.completed_puzzle_hashes.push(content_hash.into());
        self.complete_puzzle();
    }

    pub fn player(&self) -> &Player {
        &self.player
    }

    pub fn active_puzzle(&self) -> Option<usize> {
        self.active_puzzle
    }

    pub fn current_score(&self) -> u64 {
        self.current_score
    }
}

pub fn save_session_data(session_data: &SessionData, leaderboard: &mut Leaderboard) {
    let entry = Entry {
        player_id: session_data.player_id.clone(),
        score: session_data.final_score,
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
    };
    leaderboard.insert(entry);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_creation() {
        let player = Player::new("test");
        let session = Session::new(player);
        
        assert_eq!(session.player().id, "test");
        assert_eq!(session.current_score(), 0);
        assert_eq!(session.active_puzzle(), Some(0));
    }

    #[test]
    fn session_lifecycle() {
        let player = Player::new("test");
        let mut session = Session::new(player);
        
        session.start();
        session.add_score(100);
        session.pause();
        
        assert_eq!(session.current_score(), 100);
        
        let data = session.end();
        assert_eq!(data.player_id, "test");
        assert_eq!(data.final_score, 100);
        assert_eq!(data.puzzles_completed, 0);
    }

    #[test]
    fn puzzle_completion() {
        let player = Player::new("test");
        let mut session = Session::new(player);
        
        session.add_score(50);
        session.complete_puzzle();
        
        assert_eq!(session.player().score, 50);
        assert_eq!(session.player().current_puzzle_index, 1);
        assert_eq!(session.active_puzzle(), Some(1));
        assert_eq!(session.current_score(), 0);
    }

    #[test]
    fn session_data_handoff() {
        let player = Player::new("test");
        let mut session = Session::new(player);
        let mut leaderboard = Leaderboard::new(10);
        
        session.start();
        session.add_score(200);
        session.complete_puzzle();
        
        let data = session.end();
        save_session_data(&data, &mut leaderboard);
        
        assert_eq!(leaderboard.top().len(), 1);
        assert_eq!(leaderboard.top()[0].score, 200);
        assert_eq!(leaderboard.top()[0].player_id, "test");
    }

    #[test]
    fn session_data_carries_no_hashes_when_none_completed() {
        let player = Player::new("test");
        let session = Session::new(player);
        let data = session.end();
        assert!(data.puzzle_content_hashes.is_empty());
    }

    #[test]
    fn complete_puzzle_with_hash_records_hash_for_submission() {
        let player = Player::new("test");
        let mut session = Session::new(player);

        session.add_score(50);
        session.complete_puzzle_with_hash("hash-of-puzzle-1");
        session.add_score(30);
        session.complete_puzzle_with_hash("hash-of-puzzle-2");

        let data = session.end();
        assert_eq!(
            data.puzzle_content_hashes,
            vec!["hash-of-puzzle-1".to_string(), "hash-of-puzzle-2".to_string()]
        );
        assert_eq!(data.final_score, 80);
    }
}