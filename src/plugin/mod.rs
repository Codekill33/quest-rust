//! Plugin module.
//!
//! Defines a plugin architecture that allows custom puzzle logic to be loaded
//! at runtime via dynamic traits. Plugins are Rust structs implementing the
//! [`PuzzlePlugin`] trait, registered at startup via [`PluginRegistry`].
//!
//! The built-in AND/OR/NOT chain evaluator is itself a plugin — see
//! [`core_logic_plugin::CoreLogicPlugin`].

pub mod core_logic_plugin;

use crate::errors::AppError;
use std::collections::HashMap;

// ── EvalResult ───────────────────────────────────────────────────────────────

/// The outcome of a plugin evaluation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EvalResult {
    /// The input satisfied the puzzle condition(s).
    Satisfied,
    /// The input did not satisfy the puzzle condition(s).
    Unsatisfied {
        /// Human-readable explanation of why the input was insufficient.
        reason: String,
    },
    /// The plugin encountered an error during evaluation.
    Error {
        /// Description of the error.
        message: String,
    },
}

// ── PluginMeta ───────────────────────────────────────────────────────────────

/// Metadata describing a plugin, returned by [`PuzzlePlugin::describe`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginMeta {
    /// Human-readable name of the plugin.
    pub name: String,
    /// Semantic version string (e.g. `"1.0.0"`).
    pub version: String,
    /// Short description of what kind of puzzles this plugin supports.
    pub description: String,
}

// ── PuzzlePlugin trait ───────────────────────────────────────────────────────

/// Trait that all puzzle logic plugins must implement.
///
/// Each plugin provides a unique [`PuzzlePlugin::id`], an [`PuzzlePlugin::evaluate`]
/// method that determines whether player input satisfies puzzle conditions, and
/// a [`PuzzlePlugin::describe`] method for introspection.
pub trait PuzzlePlugin: Send + Sync {
    /// Returns the stable, unique identifier for this plugin (e.g. `"core_logic"`).
    fn id(&self) -> &str;

    /// Evaluates the given `input` against the puzzle `context`.
    ///
    /// The `context` is a map of named facts to their boolean truth values,
    /// identical to the context used by the built-in logic module.
    fn evaluate(&self, input: &str, context: &HashMap<&str, bool>) -> EvalResult;

    /// Returns metadata describing this plugin.
    fn describe(&self) -> PluginMeta;
}

// ── PluginRegistry ───────────────────────────────────────────────────────────

/// A registry of [`PuzzlePlugin`] instances, keyed by plugin id.
///
/// Plugins are registered at startup and then looked up by the engine when
/// evaluating puzzles that reference a `plugin_id`.
pub struct PluginRegistry {
    plugins: HashMap<String, Box<dyn PuzzlePlugin>>,
}

impl PluginRegistry {
    /// Creates a new empty registry.
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
        }
    }

    /// Registers a plugin. Returns an error if a plugin with the same id is
    /// already registered.
    pub fn register(&mut self, plugin: Box<dyn PuzzlePlugin>) -> Result<(), AppError> {
        let id = plugin.id().to_string();
        if self.plugins.contains_key(&id) {
            return Err(AppError::PluginAlreadyRegistered(id));
        }
        self.plugins.insert(id, plugin);
        Ok(())
    }

    /// Returns a reference to the plugin with the given `id`, or `None` if no
    /// such plugin is registered.
    pub fn get(&self, id: &str) -> Option<&dyn PuzzlePlugin> {
        self.plugins.get(id).map(|p| p.as_ref())
    }

    /// Returns a list of all registered plugins and their metadata, sorted by
    /// plugin id for deterministic output.
    pub fn list(&self) -> Vec<(&str, PluginMeta)> {
        let mut entries: Vec<(&str, PluginMeta)> = self
            .plugins
            .iter()
            .map(|(id, plugin)| (id.as_str(), plugin.describe()))
            .collect();
        entries.sort_by_key(|(id, _)| *id);
        entries
    }

    /// Routes an evaluation to the plugin identified by `plugin_id`.
    ///
    /// Returns [`AppError::PluginNotFound`] if no plugin with that id is
    /// registered.
    pub fn evaluate(
        &self,
        plugin_id: &str,
        input: &str,
        context: &HashMap<&str, bool>,
    ) -> Result<EvalResult, AppError> {
        match self.get(plugin_id) {
            Some(plugin) => Ok(plugin.evaluate(input, context)),
            None => Err(AppError::PluginNotFound(plugin_id.to_string())),
        }
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// A minimal test plugin that always returns `Satisfied` when the input
    /// matches a fixed password.
    struct EchoPlugin;

    impl PuzzlePlugin for EchoPlugin {
        fn id(&self) -> &str {
            "echo"
        }

        fn evaluate(&self, input: &str, _context: &HashMap<&str, bool>) -> EvalResult {
            if input == "open_sesame" {
                EvalResult::Satisfied
            } else {
                EvalResult::Unsatisfied {
                    reason: format!("expected 'open_sesame', got '{input}'"),
                }
            }
        }

        fn describe(&self) -> PluginMeta {
            PluginMeta {
                name: "Echo Plugin".to_string(),
                version: "0.1.0".to_string(),
                description: "A test plugin that checks for a magic password.".to_string(),
            }
        }
    }

    /// A second plugin used for multi-plugin tests.
    struct AlwaysSatisfiedPlugin;

    impl PuzzlePlugin for AlwaysSatisfiedPlugin {
        fn id(&self) -> &str {
            "always_ok"
        }

        fn evaluate(&self, _input: &str, _context: &HashMap<&str, bool>) -> EvalResult {
            EvalResult::Satisfied
        }

        fn describe(&self) -> PluginMeta {
            PluginMeta {
                name: "Always OK".to_string(),
                version: "1.0.0".to_string(),
                description: "Always returns Satisfied.".to_string(),
            }
        }
    }

    // ── Registration ─────────────────────────────────────────────────────────

    #[test]
    fn register_and_lookup_plugin() {
        let mut registry = PluginRegistry::new();
        registry.register(Box::new(EchoPlugin)).unwrap();

        let plugin = registry.get("echo");
        assert!(plugin.is_some());
        assert_eq!(plugin.unwrap().id(), "echo");
    }

    #[test]
    fn register_multiple_plugins() {
        let mut registry = PluginRegistry::new();
        registry.register(Box::new(EchoPlugin)).unwrap();
        registry.register(Box::new(AlwaysSatisfiedPlugin)).unwrap();

        assert!(registry.get("echo").is_some());
        assert!(registry.get("always_ok").is_some());
    }

    #[test]
    fn duplicate_registration_rejected() {
        let mut registry = PluginRegistry::new();
        registry.register(Box::new(EchoPlugin)).unwrap();

        let result = registry.register(Box::new(EchoPlugin));
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            AppError::PluginAlreadyRegistered(id) if id == "echo"
        ));
    }

    #[test]
    fn lookup_missing_plugin_returns_none() {
        let registry = PluginRegistry::new();
        assert!(registry.get("nonexistent").is_none());
    }

    // ── Evaluation routing ───────────────────────────────────────────────────

    #[test]
    fn evaluate_routes_to_correct_plugin() {
        let mut registry = PluginRegistry::new();
        registry.register(Box::new(EchoPlugin)).unwrap();
        registry.register(Box::new(AlwaysSatisfiedPlugin)).unwrap();

        let ctx = HashMap::new();

        // EchoPlugin rejects wrong input
        let result = registry.evaluate("echo", "wrong", &ctx).unwrap();
        assert!(matches!(result, EvalResult::Unsatisfied { .. }));

        // EchoPlugin accepts correct input
        let result = registry.evaluate("echo", "open_sesame", &ctx).unwrap();
        assert_eq!(result, EvalResult::Satisfied);

        // AlwaysSatisfiedPlugin accepts anything
        let result = registry.evaluate("always_ok", "anything", &ctx).unwrap();
        assert_eq!(result, EvalResult::Satisfied);
    }

    #[test]
    fn evaluate_unknown_plugin_returns_error() {
        let registry = PluginRegistry::new();
        let ctx = HashMap::new();

        let err = registry.evaluate("nonexistent", "input", &ctx).unwrap_err();
        assert!(matches!(err, AppError::PluginNotFound(id) if id == "nonexistent"));
    }

    // ── Listing ──────────────────────────────────────────────────────────────

    #[test]
    fn list_returns_all_plugins_sorted() {
        let mut registry = PluginRegistry::new();
        registry.register(Box::new(EchoPlugin)).unwrap();
        registry.register(Box::new(AlwaysSatisfiedPlugin)).unwrap();

        let listing = registry.list();
        assert_eq!(listing.len(), 2);
        // Sorted alphabetically: "always_ok" before "echo"
        assert_eq!(listing[0].0, "always_ok");
        assert_eq!(listing[1].0, "echo");
        assert_eq!(listing[0].1.name, "Always OK");
        assert_eq!(listing[1].1.name, "Echo Plugin");
    }

    #[test]
    fn list_empty_registry() {
        let registry = PluginRegistry::new();
        assert!(registry.list().is_empty());
    }

    // ── Metadata ─────────────────────────────────────────────────────────────

    #[test]
    fn plugin_describe_returns_correct_metadata() {
        let plugin = EchoPlugin;
        let meta = plugin.describe();
        assert_eq!(meta.name, "Echo Plugin");
        assert_eq!(meta.version, "0.1.0");
        assert!(!meta.description.is_empty());
    }

    // ── EvalResult variants ──────────────────────────────────────────────────

    #[test]
    fn eval_result_variants() {
        let satisfied = EvalResult::Satisfied;
        let unsatisfied = EvalResult::Unsatisfied {
            reason: "wrong answer".to_string(),
        };
        let error = EvalResult::Error {
            message: "boom".to_string(),
        };

        assert_eq!(satisfied, EvalResult::Satisfied);
        assert_ne!(satisfied, unsatisfied);
        assert_ne!(unsatisfied, error);
    }
}
