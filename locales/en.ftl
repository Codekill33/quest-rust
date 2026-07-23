
app-title = Quest Rust
error-serialize-player = Failed to serialize player: {$error}
verified-puzzle-count =
    { $count ->
        [one] Verified 1 puzzle file: {$ok} ok, {$fail} failed.
       *[other] Verified {$count} puzzle files: {$ok} ok, {$fail} failed.
    }
ok-status = OK
fail-status = FAIL
error-generate-hashes-admin = --generate-hashes rewrites puzzle files in place and is admin-only. Re-run with --admin to confirm: --generate-hashes {$dir} --admin
success-generate-hashes = Regenerated content_hash for {$count} puzzle file(s) in '{$dir}'.
error-generate-hashes = Failed to generate hashes in '{$dir}': {$error}
