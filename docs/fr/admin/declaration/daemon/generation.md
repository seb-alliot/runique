# Génération de code et daemon

## Structure générée

```
src/admins/
  ├── README.md       ← avertissement : dossier auto-généré
  ├── mod.rs          ← point d'entrée du module admin
  ├── router.rs       ← routes CRUD (list, create, detail, edit, delete)
  └── handlers.rs     ← handlers SeaORM + formulaires (GET/POST, validation, rendu)
```

### `router.rs`

Enregistre les routes pour chaque ressource déclarée dans `admin!` :

- `GET  /admin/{key}/`            → liste
- `GET  /admin/{key}/create`      → formulaire de création
- `POST /admin/{key}/create`      → soumission création
- `GET  /admin/{key}/{id}`        → détail
- `GET  /admin/{key}/{id}/edit`   → formulaire d'édition
- `POST /admin/{key}/{id}/edit`   → soumission édition
- `POST /admin/{key}/{id}/delete` → suppression

### `handlers.rs`

Contient les handlers Axum correspondant à chaque route. Chaque handler :

- extrait l'utilisateur authentifié et vérifie les permissions
- exécute la requête SeaORM appropriée
- instancie le formulaire Runique
- rend le template Tera correspondant

### `mod.rs`

Déclare les sous-modules `router` et `handlers`, et expose la fonction `admin_config()` issue de la macro.

## Le compromis : écrasement automatique

`runique start` **supprime et régénère intégralement** `src/admins/` à chaque modification de `src/admin.rs`.

Toute modification manuelle dans ce dossier sera **perdue** lors de la prochaine régénération.

## Quand basculer sur `cargo run`

Si des modifications manuelles du code généré sont nécessaires (logique métier spécifique, handler personnalisé), il faut **arrêter `runique start`** et passer à un workflow standard :

```bash
cargo run
```

Dans ce mode, `src/admins/` n'est plus surveillé ni écrasé. Les modifications persistent.

> Le README.md généré dans `src/admins/` rappelle ce comportement directement dans le dépôt.
