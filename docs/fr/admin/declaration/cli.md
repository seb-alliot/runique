# `runique start`

La commande `runique start` est le point d'entrée du workflow admin.
Elle enchaîne, **séquentiellement sur un seul thread** : génération du code admin, `cargo fmt`, puis lancement bloquant du serveur.

---

## Détection de l'admin dans `main.rs`

Au démarrage, `runique start` lit `src/main.rs` et recherche la présence de `.with_admin(` :

```rust
// src/main.rs
RuniqueApp::builder(config)
    .with_admin(|a| a.routes(admins::routes("/admin")))
    // ...
```

La détection se fait par simple recherche de chaîne dans le fichier source.
**Elle fonctionne même si la ligne est commentée** (`// .with_admin(...)`).

| Résultat de la détection | Comportement |
| --- | --- |
| `.with_admin(` trouvé | Génération + `cargo run` enchaînés |
| Absent | Message d'information, arrêt propre |

> Le chemin vers `main.rs` est configurable : `runique start --main src/main.rs`

---

## Ce qui se passe si `.with_admin(` est détecté

`runique start` exécute, **dans l'ordre, sur le même thread** :

1. **Génération** — lecture de `src/admin.rs`, parsing de `admin!{}`, réécriture de `src/admins/`
2. **`cargo fmt --all`**
3. **`cargo run --release`** — bloquant jusqu'à arrêt du programme

```text
runique start
  ├── generate_admin(src/admin.rs) → réécrit src/admins/
  ├── cargo fmt --all
  └── cargo run --release          → serveur HTTP (bloquant)
```

Un ancien design lançait la génération dans un thread séparé en parallèle de `cargo run` : ça créait une race condition (le build pouvait lire un `admin.rs` à moitié écrit, échec non reproductible). Le flux est désormais strictement séquentiel pour l'éliminer. Il n'y a **pas de surveillance continue** : pour régénérer après une modification de `src/admin.rs`, relancez `runique start`.

---

## Autre section

| Section | Description |
| --- | --- |
| [Génération de code admin](/docs/fr/admin/declaration-daemon) | Fichiers générés
| [Macro `admin!`](/docs/fr/admin/declaration-macro) | Déclaration des ressources administrables

## Voir aussi

| Section | Description |
| --- | --- |
| [Mise en place](/docs/fr/admin/setup) | Câbler l'admin dans un projet existant, créer un superuser |
| [Permissions](/docs/fr/admin/permission) | Rôles, `is_staff` / `is_superuser`, vérification runtime |
| [Templates](/docs/fr/admin/template) | Hiérarchie de templates, blocks, surcharge du visuel |
| [Évolutions](/docs/fr/admin/evolution) | Axes d'évolution et état bêta |

## Revenir au Sommaire

- [Sommaire](/docs/fr/admin) - Sommaire Admin