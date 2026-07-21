//! Core logic plugin.
//!
//! Wraps the built-in [`crate::logic`] module's AND/OR/NOT chain evaluator as
//! a [`PuzzlePlugin`], so it participates in the plugin registry alongside any
//! custom plugins.

use crate::logic;
use crate::plugin::{EvalResult, PluginMeta, PuzzlePlugin};
use std::collections::HashMap;

/// The built-in puzzle plugin that evaluates boolean logic chains.
///
/// This plugin delegates to [`logic::evaluate`], treating the `input` string
/// as a [`logic::Condition::Fact`] lookup in the provided context. It returns
/// [`EvalResult::Satisfied`] when the fact is true, and
/// [`EvalResult::Unsatisfied`] otherwise.
pub struct CoreLogicPlugin;

impl CoreLogicPlugin {
    /// The stable plugin identifier for the built-in logic evaluator.
    pub const ID: &'static str = "core_logic";
}

impl PuzzlePlugin for CoreLogicPlugin {
    fn id(&self) -> &str {
        Self::ID
    }

    fn evaluate(&self, input: &str, context: &HashMap<&str, bool>) -> EvalResult {
        let condition = logic::Condition::Fact(input.to_string());
        if logic::evaluate(&condition, context) {
            EvalResult::Satisfied
        } else {
            EvalResult::Unsatisfied {
                reason: format!("fact '{input}' is not satisfied in the current context"),
            }
        }
    }

    fn describe(&self) -> PluginMeta {
        PluginMeta {
            name: "Core Logic".to_string(),
            version: "1.0.0".to_string(),
            description: "Built-in AND/OR/NOT boolean chain evaluator. Evaluates player \
                          actions as named facts against a boolean context map."
                .to_string(),
        }
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx(pairs: &[(&'static str, bool)]) -> HashMap<&'static str, bool> {
        pairs.iter().cloned().collect()
    }

    #[test]
    fn id_is_core_logic() {
        let plugin = CoreLogicPlugin;
        assert_eq!(plugin.id(), "core_logic");
        assert_eq!(plugin.id(), CoreLogicPlugin::ID);
    }

    #[test]
    fn evaluate_satisfied_when_fact_is_true() {
        let plugin = CoreLogicPlugin;
        let context = ctx(&[("lever_pulled", true)]);
        let result = plugin.evaluate("lever_pulled", &context);
        assert_eq!(result, EvalResult::Satisfied);
    }

    #[test]
    fn evaluate_unsatisfied_when_fact_is_false() {
        let plugin = CoreLogicPlugin;
        let context = ctx(&[("lever_pulled", false)]);
        let result = plugin.evaluate("lever_pulled", &context);
        assert!(matches!(result, EvalResult::Unsatisfied { .. }));
    }

    #[test]
    fn evaluate_unsatisfied_when_fact_missing() {
        let plugin = CoreLogicPlugin;
        let context = ctx(&[]);
        let result = plugin.evaluate("unknown_fact", &context);
        assert!(matches!(result, EvalResult::Unsatisfied { .. }));
    }

    #[test]
    fn describe_returns_correct_metadata() {
        let plugin = CoreLogicPlugin;
        let meta = plugin.describe();
        assert_eq!(meta.name, "Core Logic");
        assert_eq!(meta.version, "1.0.0");
        assert!(!meta.description.is_empty());
    }

    #[test]
    fn integrates_with_registry() {
        use crate::plugin::PluginRegistry;

        let mut registry = PluginRegistry::new();
        registry.register(Box::new(CoreLogicPlugin)).unwrap();

        let context = ctx(&[("door_open", true)]);
        let result = registry.evaluate("core_logic", "door_open", &context).unwrap();
        assert_eq!(result, EvalResult::Satisfied);

        let result = registry.evaluate("core_logic", "door_open", &ctx(&[])).unwrap();
        assert!(matches!(result, EvalResult::Unsatisfied { .. }));
    }
}
