# Quest Rust — Plugin System Guide

The plugin system allows developers to extend Quest Rust with entirely new puzzle types (e.g. physics simulations, cryptographic challenges, complex state machines) without having to modify the core game engine.

## How It Works

Instead of hardcoding how a puzzle determines whether the player's input is correct, puzzles can declare an optional `plugin_id`. During the game session, when a player attempts an action, the engine looks up the plugin registered under that ID and delegates the evaluation to it.

If a puzzle does not declare a `plugin_id`, it defaults to the built-in `core_logic` evaluator (the standard AND/OR/NOT condition chains).

## Implementing a Plugin

To create a new puzzle type, you implement the `PuzzlePlugin` trait found in `src/plugin/mod.rs`.

A plugin must define three methods:
1. `id(&self) -> &str` — The unique string ID for this plugin.
2. `describe(&self) -> PluginMeta` — A description of what the plugin does.
3. `evaluate(&self, input: &str, context: &HashMap<&str, bool>) -> EvalResult` — The actual evaluation logic.

### Evaluation Outcomes

The `evaluate` method returns an `EvalResult` which can be:
- `Satisfied`: The player's input successfully solved/progressed the puzzle.
- `Unsatisfied { reason }`: The player's input was incorrect (the `reason` string can be used to hint the player).
- `Error { message }`: The plugin encountered an internal error during evaluation.

## Example: A Crypto Challenge Plugin

Suppose you want to add a puzzle type where the player must input the correct SHA-256 hash of a specific string.

```rust
use smart_contract_game::plugin::{EvalResult, PluginMeta, PuzzlePlugin};
use std::collections::HashMap;

pub struct CryptoPlugin;

impl PuzzlePlugin for CryptoPlugin {
    fn id(&self) -> &str {
        "crypto_sha256"
    }

    fn describe(&self) -> PluginMeta {
        PluginMeta {
            name: "SHA-256 Crypto Challenge".to_string(),
            version: "1.0.0".to_string(),
            description: "Requires the player to submit a valid SHA-256 hash.".to_string(),
        }
    }

    fn evaluate(&self, input: &str, _context: &HashMap<&str, bool>) -> EvalResult {
        // In a real plugin, the expected hash might be passed via context or puzzle data.
        let expected_hash = "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824";
        
        if input.trim() == expected_hash {
            EvalResult::Satisfied
        } else {
            EvalResult::Unsatisfied {
                reason: "Invalid hash. Please try again.".to_string(),
            }
        }
    }
}
```

## Registering Your Plugin

Once your plugin struct is implemented, you must register it with the engine at startup.

In `src/main.rs`:

```rust
use smart_contract_game::plugin::{core_logic_plugin::CoreLogicPlugin, PluginRegistry};
// use my_custom_plugins::CryptoPlugin;

let mut plugin_registry = PluginRegistry::new();

// Register the built-in logic (always recommended)
plugin_registry.register(Box::new(CoreLogicPlugin)).unwrap();

// Register your custom plugin
// plugin_registry.register(Box::new(CryptoPlugin)).unwrap();

// The registry is then passed to the GameEngine (implementation depends on engine structure)
```

## Creating Puzzles for Your Plugin

When writing a puzzle JSON file, simply include the `plugin_id` matching your plugin's `id()`.

```json
{
  "id": "hash-puzzle-1",
  "plugin_id": "crypto_sha256",
  "description": "Calculate the SHA-256 hash of the word 'hello'.",
  "conditions": [],
  "effects": [
    { "AwardScore": 500 }
  ]
}
```

## Troubleshooting

- **Error: `plugin 'XYZ' not found`**
  Make sure you registered the plugin in `main.rs` and that the `plugin_id` in your puzzle JSON exactly matches the string returned by your plugin's `id()` method.
- **Error: `plugin 'XYZ' is already registered`**
  You accidentally called `registry.register()` twice for the same plugin ID. Plugin IDs must be globally unique.
- **CLI Verification**
  Run `cargo run -- --list-plugins` to view all plugins currently registered in the game, confirming that your new plugin was initialized correctly.
