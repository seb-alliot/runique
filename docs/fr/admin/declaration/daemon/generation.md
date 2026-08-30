# Génération de code admin

## Comportement du daemon

`runique start` n'est **pas** un watcher en tâche de fond : c'est une génération **one-shot séquentielle**, suivie du lancement bloquant de l'application.

1. `src/admin.rs` est lu une fois
2. La macro `admin! { ... }` est parsée via `syn`, produisant des `ResourceDef`
3. Le contenu de `src/admins/` est réécrit sur place (les fichiers sont tronqués puis réécrits — le dossier n'est jamais supprimé au préalable)
4. `cargo fmt --all` est lancé
5. `cargo run --release` est lancé, bloquant, dans le même processus

Il n'y a ni surveillance continue du fichier, ni debounce : une ancienne implémentation basée sur un thread séparé a été retirée car elle provoquait une race condition. Pour régénérer après une modification, relancez `runique start`.

---

## Structure générée

```text
src/admins/
  ├── README.md       ← avertissement : ne pas éditer manuellement
  ├── mod.rs          ← expose `routes` et `admin_state`
  └── admin.rs        ← fichier principal : wrappers DynForm + admin_register()
```

### `admin.rs`

Contient pour chaque ressource déclarée dans `admin!` :

- Un wrapper `DynForm` autour du formulaire Runique concret
- Les closures CRUD : `list_fn`, `get_fn`, `create_fn`, `update_fn`, `delete_fn`, `count_fn`, `partial_update_fn` (toujours générée, utilisée pour le bulk edit/group actions)
- Si `list_filter` est déclaré : une closure `filter_fn` par champ, qui charge les valeurs distinctes depuis la base (jusqu'à 10 par défaut)
- La configuration d'affichage (colonnes visibles, taille de page filtre) transmise via `.display(…)` sur le `ResourceEntry`
- La fonction `admin_register()` qui construit le `HashMap<String, ResourceEntry>` chargé au boot

### `mod.rs`

Ré-exporte `routes` et `admin_state` depuis `admin`.

---

## Le compromis : écrasement automatique

Chaque exécution de `runique start` **réécrit** le contenu de `src/admins/` (fichiers tronqués puis régénérés, jamais supprimés au préalable).

Toute modification manuelle dans ce dossier sera **perdue** au prochain `runique start`.

## Quand basculer sur `cargo run`

Si des modifications manuelles du code généré sont nécessaires (logique métier spécifique, handler personnalisé), il faut **arrêter `runique start`** et passer à un workflow standard :

```bash
cargo run
```

Dans ce mode, `runique start` ne tourne plus, donc `src/admins/` n'est jamais réécrit. Les modifications persistent.

> Le `README.md` généré dans `src/admins/` rappelle ce comportement directement dans le dépôt.

## Autre section

| Section | Description |
| --- | --- |
| [CLI](/docs/fr/admin/declaration) | Fonctionnement de `runique start` |
| [Macro `admin!`](/docs/fr/admin/declaration-macro) | Déclaration des ressources administrables |

## Revenir au Sommaire

- [Sommaire Admin](/docs/fr/admin)
