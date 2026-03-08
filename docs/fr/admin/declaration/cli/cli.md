## `runique start`

La commande `runique start` lance un daemon qui surveille `src/admin.rs` en continu via un watcher basé sur `notify`.

À chaque modification détectée :

1. `src/admin.rs` est relu
2. La macro `admin! { ... }` est parsée via `syn`, produisant des `ResourceDef`
3. Le dossier `src/admins/` est entièrement régénéré
4. Un retour est affiché (succès ou erreur de parsing)

Un mécanisme de **debounce** évite les régénérations multiples lors d'un même enregistrement de fichier.
