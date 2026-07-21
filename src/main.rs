pub mod engine;

use std::time::Duration;

/// Default directory scanned for puzzle definition files by the
/// `--verify-puzzles` / `--generate-hashes` CLI commands.
const DEFAULT_PUZZLES_DIR: &str = "puzzles";

fn main() {
    use smart_contract_game::player::Player;

    let args: Vec<String> = std::env::args().collect();

    if args.iter().any(|a| a == "--verify-puzzles") {
        let dir = puzzles_dir_arg(&args);
        std::process::exit(run_verify_puzzles(&dir));
    }

    if args.iter().any(|a| a == "--generate-hashes") {
        let dir = puzzles_dir_arg(&args);
        std::process::exit(run_generate_hashes(
            &dir,
            args.iter().any(|a| a == "--admin"),
        ));
    }

    // Initialize and run the core engine for a short duration to ensure clean startup/shutdown.
    let engine = engine::Engine::new(Duration::from_millis(16));
    engine.init();
    engine.run_for(Duration::from_millis(100));
    engine.shutdown();

    let mut player = Player::new("player-1");
    player.add_score(100);
    player.advance_puzzle();
    player.add_item("compass");

    match player.to_json() {
        Ok(json) => println!("Player state: {json}"),
        Err(e) => eprintln!("Failed to serialize player: {e}"),
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
fn run_verify_puzzles(dir: &str) -> i32 {
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
            Ok(()) => println!("OK   {}", report.path.display()),
            Err(e) => {
                failures += 1;
                println!("FAIL {}: {e}", report.path.display());
            }
        }
    }

    println!(
        "\nVerified {} puzzle file(s): {} ok, {} failed.",
        reports.len(),
        reports.len() - failures,
        failures
    );

    if failures > 0 { 1 } else { 0 }
}

/// Runs the `--generate-hashes` command: recomputes and writes the
/// `content_hash` for every puzzle file in `dir`. Admin-only — requires
/// `--admin` to also be passed, since this rewrites puzzle files in place.
/// Returns the process exit code.
fn run_generate_hashes(dir: &str, admin_confirmed: bool) -> i32 {
    use smart_contract_game::loader::generate_hashes_in_dir;

    if !admin_confirmed {
        eprintln!(
            "--generate-hashes rewrites puzzle files in place and is admin-only. \
             Re-run with --admin to confirm: --generate-hashes {dir} --admin"
        );
        return 1;
    }

    match generate_hashes_in_dir(dir) {
        Ok(count) => {
            println!("Regenerated content_hash for {count} puzzle file(s) in '{dir}'.");
            0
        }
        Err(e) => {
            eprintln!("Failed to generate hashes in '{dir}': {e}");
            1
        }
    }
}
