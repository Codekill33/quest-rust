use crate::errors::AppError;
use crate::plugin::{EvalResult, PluginRegistry, PuzzlePlugin};
use crate::puzzle::Puzzle;
use crate::time::Instant;
use std::collections::HashMap;
use std::thread::sleep;
use std::time::Duration;


/// Core game engine that manages the main loop and lifecycle.
pub struct Engine {
    tick_rate: Duration,
    /// Registry storing all loaded puzzle plugins keyed by plugin id.
    plugin_registry: PluginRegistry,
}

impl Engine {
    /// Create a new engine with the given tick rate (frame duration).
    pub fn new(tick_rate: Duration) -> Self {
        Self {
            tick_rate,
            plugin_registry: PluginRegistry::new(),
        }
    }

    /// Registers a custom puzzle logic plugin with the engine.
    pub fn register_plugin(&mut self, plugin: Box<dyn PuzzlePlugin>) -> Result<(), AppError> {
        self.plugin_registry.register(plugin)
    }

    /// Returns a list of all registered plugins and their metadata.
    pub fn list_plugins(&self) -> Vec<(&str, crate::plugin::PluginMeta)> {
        self.plugin_registry.list()
    }

    /// Validates that a puzzle's requested plugin is registered before starting
    /// a session. Returns `PluginNotFound` if the plugin is missing.
    pub fn start_session_for_puzzle(&self, puzzle: &Puzzle) -> Result<(), AppError> {
        let plugin_id = puzzle.plugin_id.as_deref().unwrap_or("core_logic");
        if self.plugin_registry.get(plugin_id).is_none() {
            return Err(AppError::PluginNotFound(plugin_id.to_string()));
        }
        Ok(())
    }

    /// Routes a puzzle evaluation to the appropriate registered plugin.
    pub fn evaluate_puzzle(
        &self,
        puzzle: &Puzzle,
        input: &str,
        context: &HashMap<&str, bool>,
    ) -> Result<EvalResult, AppError> {
        let plugin_id = puzzle.plugin_id.as_deref().unwrap_or("core_logic");
        self.plugin_registry.evaluate(plugin_id, input, context)
    }

    /// Initialize engine resources.
    pub fn init(&self) {
        println!("Engine: init");
    }

    /// Run the game loop for a specified duration. This enforces a fixed tick rate.
    pub fn run_for(&self, run_duration: Duration) {
        println!("Engine: run_for {:?}", run_duration);
        let start = Instant::now();
        let mut ticks: u64 = 0;
        while start.elapsed() < run_duration {
            let frame_start = Instant::now();

            // Placeholder for per-tick logic.
            ticks += 1;

            // Enforce fixed tick rate by sleeping the remainder of the frame.
            let frame_time = frame_start.elapsed();
            if frame_time < self.tick_rate {
                sleep(self.tick_rate - frame_time);
            }
        }
        println!("Engine: run complete (ticks={})", ticks);
    }

    /// Cleanly shutdown and release resources.
    pub fn shutdown(&self) {
        println!("Engine: shutdown");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugin::core_logic_plugin::CoreLogicPlugin;
    use std::time::Duration;

    #[test]
    fn engine_runs_and_stops() {
        let engine = Engine::new(Duration::from_millis(16));
        engine.init();
        engine.run_for(Duration::from_millis(50));
        engine.shutdown();
    }

    #[test]
    fn engine_routes_evaluation_and_checks_session_start() {
        let mut engine = Engine::new(Duration::from_millis(16));
        engine.register_plugin(Box::new(CoreLogicPlugin)).unwrap();

        let mut puzzle = Puzzle::new("test", "test puzzle", vec![], vec![]);
        // Defaults to core_logic, which is registered
        assert!(engine.start_session_for_puzzle(&puzzle).is_ok());

        // Requires missing plugin
        puzzle.plugin_id = Some("missing_plugin".to_string());
        assert!(matches!(
            engine.start_session_for_puzzle(&puzzle),
            Err(AppError::PluginNotFound(_))
        ));

        // Evaluation routes correctly
        puzzle.plugin_id = Some("core_logic".to_string());
        let mut ctx = HashMap::new();
        ctx.insert("door_open", true);
        let result = engine.evaluate_puzzle(&puzzle, "door_open", &ctx).unwrap();
        assert_eq!(result, EvalResult::Satisfied);
    }
}
