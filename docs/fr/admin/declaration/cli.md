# `runique start`

La commande `runique start` est le point d'entrée du workflow admin.
Elle orchestre deux opérations en parallèle : la **surveillance de `src/admin.rs`** et le **lancement du serveur**.

---

## Détection de l'admin dans `main.rs`

Au démarrage, `runique start` lit `src/main.rs` et recherche la présence de `.with_admin(` :

```rust
// src/main.rs
RuniqueApp::new()
    .with_admin(|a| a.routes(admins::routes("/admin")))
    // ...
```

La détection se fait par simple recherche de chaîne dans le fichier source.
**Elle fonctionne même si la ligne est commentée** (`// .with_admin(...)`).

| Résultat de la détection | Comportement |
| --- | --- |
| `.with_admin(` trouvé | Daemon + `cargo run` lancés |
| Absent | Message d'information, arrêt propre |

> Le chemin vers `main.rs` est configurable : `runique start --main src/main.rs`

---

## Ce qui se passe si `.with_admin(` est détecté

`runique start` lance **deux processus simultanément** :

1. **Le daemon admin** — thread séparé qui surveille `src/admin.rs` et régénère `src/admins/` à chaque modification
2. **`cargo run`** — lance le serveur applicatif (bloquant jusqu'à arrêt du programme)

```text
runique start
  ├── thread daemon → watch(src/admin.rs) [génération initiale immédiate]
  └── cargo run     → serveur HTTP (bloquant)
```

Le daemon effectue une **génération initiale au démarrage** — il n'est pas nécessaire de modifier `src/admin.rs` pour que le code soit produit.

---

## Autre section

| Section | Description |
| --- | --- |
| [Daemon & génération](daemon/generation.md) | Watcher, fichiers
| [Macro `admin!`](macro/macro.md) | Déclaration des ressources administrables

## Voir aussi

| Section | Description |
| --- | --- |
| [Mise en place](../setup/setup.md) | Câbler l'admin dans un projet existant, créer un superuser |
| [Permissions](../permission/permissions.md) | Rôles, `is_staff` / `is_superuser`, vérification runtime |
| [Templates](../template/templates.md) | Hiérarchie de templates, blocks, surcharge du visuel |
| [Évolutions](../evolution/futur_feature.md) | Axes d'évolution et état bêta |

## Revenir au Sommaire

- [Sommaire](../11-Admin.md) - Sommaire Admin