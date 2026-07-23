use smart_contract_game::{engine, l10n::L10n};

use std::time::Duration;

/// Default directory scanned for puzzle definition files by the
/// `--verify-puzzles` / `--generate-hashes` CLI commands.
const DEFAULT_PUZZLES_DIR: &str = "puzzles";

fn main() {
    use smart_contract_game::player::Player;

    let args: Vec<String> = std::env::args().collect();
    
    let lang = args.iter()
        .position(|a| a == "--lang")
        .and_then(|i| args.get(i + 1))
        .cloned()
        .unwrap_or_else(|| "en".to_string());
        
    let l10n = L10n::new(&lang);

    if args.iter().any(|a| a == "--verify-puzzles") {
        let dir = puzzles_dir_arg(&args);
        std::process::exit(run_verify_puzzles(&dir, &l10n));
    }

    if args.iter().any(|a| a == "--generate-hashes") {
        let dir = puzzles_dir_arg(&args);
        std::process::exit(run_generate_hashes(
            &dir,
            args.iter().any(|a| a == "--admin"),
            &l10n,
        ));
    }

    let mut engine = engine::Engine::new(Duration::from_millis(16));
    
    // Register the built-in core logic plugin with the engine
    use smart_contract_game::plugin::core_logic_plugin::CoreLogicPlugin;
    if let Err(e) = engine.register_plugin(Box::new(CoreLogicPlugin)) {
        eprintln!("Failed to register built-in plugins: {e}");
        std::process::exit(1);
    }

    if args.iter().any(|a| a == "--list-plugins") {
        std::process::exit(run_list_plugins(&engine));
    }

    // Initialize and run the core engine for a short duration to ensure clean startup/shutdown.
    engine.init();
    engine.run_for(Duration::from_millis(100));
    engine.shutdown();

    let mut player = Player::new("player-1");
    player.add_score(100);
    player.advance_puzzle();
    player.add_item("compass");

    match player.to_json() {
        Ok(json) => println!("Player state: {json}"),
        Err(e) => eprintln!("{}", l10n.get("error-serialize-player", Some(&{
            let mut args = fluent::FluentArgs::new();
            args.set("error", e.to_string());
            args
        }))),
    }
}

/// Reads the directory positional argument that follows a `--verify-puzzles`
/// or `--generate-hashes` flag, falling back to [`DEFAULT_PUZZLES_DIR`].
fn puzzles_dir_arg(args: &[String]) -> String {
    args.iter()
        .position(|a| a == "--verify-puzzles" || a == "--generate-hashes")
        .and_then(|i| args.get(i + 1))
        .filter(|a| !a.starts_with("--"))
        .cloned()
        .unwrap_or_else(|| DEFAULT_PUZZLES_DIR.to_string())
}

/// Runs the `--verify-puzzles` command: checks every puzzle file in `dir`
/// and reports integrity failures. Returns the process exit code.
fn run_verify_puzzles(dir: &str, l10n: &L10n) -> i32 {
    use smart_contract_game::loader::verify_puzzles_in_dir;

    let reports = match verify_puzzles_in_dir(dir) {
        Ok(reports) => reports,
        Err(e) => {
            eprintln!("Failed to scan puzzle directory '{dir}': {e}");
            return 1;
        }
    };

    let mut failures = 0;
    for report in &reports {
        match &report.result {
            Ok(()) => println!("{}   {}", l10n.get("ok-status", None), report.path.display()),
            Err(e) => {
                failures += 1;
                println!("{} {}: {e}", l10n.get("fail-status", None), report.path.display());
            }
        }
    }

    let mut args = fluent::FluentArgs::new();
    args.set("count", reports.len());
    args.set("ok", reports.len() - failures);
    args.set("fail", failures);

    println!("\n{}", l10n.get("verified-puzzle-count", Some(&args)));

    if failures > 0 { 1 } else { 0 }
}

/// Runs the `--generate-hashes` command: recomputes and writes the
/// `content_hash` for every puzzle file in `dir`. Admin-only — requires
/// `--admin` to also be passed, since this rewrites puzzle files in place.
/// Returns the process exit code.
fn run_generate_hashes(dir: &str, admin_confirmed: bool, l10n: &L10n) -> i32 {
    use smart_contract_game::loader::generate_hashes_in_dir;

    if !admin_confirmed {
        let mut args = fluent::FluentArgs::new();
        args.set("dir", dir);
        eprintln!("{}", l10n.get("error-generate-hashes-admin", Some(&args)));
        return 1;
    }

    match generate_hashes_in_dir(dir) {
        Ok(count) => {
            let mut args = fluent::FluentArgs::new();
            args.set("count", count);
            args.set("dir", dir);
            println!("{}", l10n.get("success-generate-hashes", Some(&args)));
            0
        }
        Err(e) => {
            let mut args = fluent::FluentArgs::new();
            args.set("dir", dir);
            args.set("error", e.to_string());
            eprintln!("{}", l10n.get("error-generate-hashes", Some(&args)));
            1
        }
    }
}

/// Runs the `--list-plugins` command: enumerates all registered plugins
/// and prints their metadata. Returns the process exit code.
fn run_list_plugins(engine: &smart_contract_game::engine::Engine) -> i32 {
    let plugins = engine.list_plugins();
    println!("Registered Plugins ({} total):", plugins.len());
    for (id, meta) in plugins {
        println!("\n• ID:      {id}");
        println!("  Name:    {} (v{})", meta.name, meta.version);
        println!("  Summary: {}", meta.description);
    }
    0
}
