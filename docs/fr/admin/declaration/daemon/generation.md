# Daemon & génération de code

## Comportement du daemon

Le daemon surveille `src/admin.rs` en continu via `notify`.

À chaque modification détectée :

1. `src/admin.rs` est relu
2. La macro `admin! { ... }` est parsée via `syn`, produisant des `ResourceDef`
3. Le dossier `src/admins/` est supprimé puis entièrement régénéré
4. Un retour est affiché (succès ou erreur de parsing)

Un mécanisme de **debounce** (300 ms) évite les régénérations multiples lors d'un même enregistrement de fichier.

Une **génération initiale** est effectuée au démarrage du daemon, sans attendre de modification.

---

## Structure générée

```text
src/admins/
  ├── README.md       ← avertissement : ne pas éditer manuellement
  ├── mod.rs          ← expose `routes` et `admin_proto_state`
  └── admin_panel.rs  ← fichier principal : wrappers DynForm + admin_register()
```

### `admin_panel.rs`

Contient pour chaque ressource déclarée dans `admin!` :

- Un wrapper `DynForm` autour du formulaire Runique concret
- Les closures `list_fn`, `get_fn`, `create_fn`, `update_fn`, `delete_fn`, `count_fn`
- La fonction `admin_register()` qui construit le `HashMap<String, ResourceEntry>` chargé au boot

### `mod.rs`

Ré-exporte `routes` et `admin_proto_state` depuis `admin_panel`.

---

## Le compromis : écrasement automatique

`runique start` **supprime et régénère intégralement** `src/admins/` à chaque modification de `src/admin.rs`.

Toute modification manuelle dans ce dossier sera **perdue** lors de la prochaine régénération.

## Quand basculer sur `cargo run`

Si des modifications manuelles du code généré sont nécessaires (logique métier spécifique, handler personnalisé), il faut **arrêter `runique start`** et passer à un workflow standard :

```bash
cargo run
```

Dans ce mode, `src/admins/` n'est plus surveillé ni écrasé. Les modifications persistent.

> Le `README.md` généré dans `src/admins/` rappelle ce comportement directement dans le dépôt.

## Autre section

| Section | Description |
| --- | --- |
| [Cli](../cli.md) - Fonctionnement de runique start
| [Macro `admin!`](../macro/macro.md) — Déclaration des ressources administrables

## Revenir au Sommaire

- [Sommaire Admin](../../11-Admin.md)
