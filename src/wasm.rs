#![cfg(feature = "wasm")]

use wasm_bindgen::prelude::*;
use std::sync::Mutex;
use once_cell::sync::Lazy;

use crate::player::Player;
use crate::session::Session;
use crate::puzzle::Puzzle;
use crate::hints::{Hint, HintSystem};
use crate::persistence;

// A global lock for the current game state
static GAME_STATE: Lazy<Mutex<Option<GameState>>> = Lazy::new(|| Mutex::new(None));

struct GameState {
    session: Session,
    puzzle: Puzzle,
    hint_system: HintSystem,
}

#[allow(non_snake_case)]
#[wasm_bindgen]
pub fn startSession(_puzzle_id: &str, puzzle_json: &str) -> Result<(), JsValue> {
    // Attempt to load player or create a new one
    let player_id = "wasm-player"; // Hardcoded for simplicity in demo
    let player = persistence::load_player(player_id)
        .unwrap_or(None)
        .unwrap_or_else(|| Player::new(player_id));
        
    let mut session = Session::new(player);
    session.start();

    let puzzle = Puzzle::from_json(puzzle_json)
        .map_err(|e| JsValue::from_str(&format!("Invalid puzzle JSON: {}", e)))?;
        
    let hint_system = HintSystem::new(vec![Hint { text: "Here is a helpful hint for the puzzle.".to_string() }]); // Default hint

    let mut state = GAME_STATE.lock().unwrap();
    *state = Some(GameState {
        session,
        puzzle,
        hint_system,
    });
    
    Ok(())
}

#[allow(non_snake_case)]
#[wasm_bindgen]
pub fn submitAnswer(answer: &str) -> Result<bool, JsValue> {
    let mut guard = GAME_STATE.lock().unwrap();
    let state = guard.as_mut().ok_or_else(|| JsValue::from_str("Session not started"))?;
    
    let effects = state.puzzle.evaluate(answer).to_vec();
    if state.puzzle.is_solved() {
        for effect in effects {
            match effect {
                crate::puzzle::Effect::AwardScore(points) => state.session.add_score(points),
                _ => {} // Other effects ignored for simple demo
            }
        }
        state.session.complete_puzzle();
        
        // Save progress
        let _ = persistence::save_player(state.session.player(), state.session.player().id.as_str());
        
        return Ok(true);
    }
    
    Ok(false)
}

#[allow(non_snake_case)]
#[wasm_bindgen]
pub fn revealHint() -> Result<String, JsValue> {
    let mut guard = GAME_STATE.lock().unwrap();
    let state = guard.as_mut().ok_or_else(|| JsValue::from_str("Session not started"))?;
    
    // In a real app we'd load hints for the puzzle. Here we just return a dummy hint
    match state.hint_system.reveal_next() {
        Some((hint, _penalty)) => {
            // Apply penalty in a real app, here we just return the text
            Ok(hint.text.clone())
        }
        None => Err(JsValue::from_str("No more hints available")),
    }
}

#[allow(non_snake_case)]
#[wasm_bindgen]
pub fn getScore() -> Result<u64, JsValue> {
    let guard = GAME_STATE.lock().unwrap();
    let state = guard.as_ref().ok_or_else(|| JsValue::from_str("Session not started"))?;
    Ok(state.session.current_score() + state.session.player().score)
}

#[allow(non_snake_case)]
#[wasm_bindgen]
pub fn endSession() -> Result<u64, JsValue> {
    let mut guard = GAME_STATE.lock().unwrap();
    let state = guard.take().ok_or_else(|| JsValue::from_str("Session not started"))?;
    
    let data = state.session.end();
    Ok(data.final_score)
}
