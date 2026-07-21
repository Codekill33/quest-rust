
app-title = Quest Rust
error-serialize-player = Échec de la sérialisation du joueur : {$error}
verified-puzzle-count =
    { $count ->
        [one] 1 fichier de puzzle vérifié : {$ok} ok, {$fail} échoué.
       *[other] {$count} fichiers de puzzles vérifiés : {$ok} ok, {$fail} échoués.
    }
ok-status = OK
fail-status = ÉCHEC
error-generate-hashes-admin = --generate-hashes réécrit les fichiers de puzzle sur place et est réservé aux administrateurs. Relancez avec --admin pour confirmer : --generate-hashes {$dir} --admin
success-generate-hashes = content_hash régénéré pour {$count} fichier(s) de puzzle dans '{$dir}'.
error-generate-hashes = Échec de la génération des hashes dans '{$dir}' : {$error}
