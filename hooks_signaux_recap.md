# Hooks / Signaux SeaORM — récap de la discussion (2026-09-01)

Aucun code touché. Ce fichier récapitule où en est la réflexion pour reprendre la discussion.

## 1. État réel du code (vérifié, pas supposé)

- `runique/src/migration/hooks/mod.rs` : `HookType` (BeforeSave/AfterSave/BeforeDelete/AfterDelete), `Hook`, `HooksDef` (builder fluent, tri par slot). **Structure de données seule, jamais consommée.**
- `runique/src/migration/schema/mod.rs:28` : `ModelSchema.hooks: Option<HooksDef>` — champ présent, jamais lu par le générateur.
- Générateur (`derive_form/src/model/utils/active_model.rs:7` et `derive_form/src/extend_schema.rs:791`) : produit toujours `impl ::sea_orm::ActiveModelBehavior for ActiveModel {}` **vide**. Aucun branchement vers `HooksDef`.
- `runique/src/forms/field.rs` (trait `RuniqueForm`) : `on_save`, `before_save`, `after_save` existent déjà avec un `SaveContext` (Create/Update/Delete). Mais `save()` (ce que l'admin appelle réellement, `handle_crud.rs:253,600`) n'invoque que `on_save`. Seul `save_as()` invoque before/on/after — **et rien n'appelle `save_as()` nulle part dans le repo**. Code mort en pratique.
- `SaveContext::Delete` existe dans l'enum mais rien ne l'émet — pas de `delete()` sur `RuniqueForm`.
- Delete admin (`handle_crud.rs`, `handle_bulk.rs`, `builtin/groupe.rs:141`, `builtin/user.rs:232`) : passe par `delete_fn`, un closure généré `Fn(db, id) -> ...` qui fait `Entity::delete_by_id(id).exec(db)` — **aucune instance de formulaire en jeu**, spécifique à l'admin (pas déclenché par du code applicatif hors admin).
- Fait notable : dans le flux single-delete admin (`handle_crud.rs:726`), `get_fn` charge déjà le modèle avant delete (pour le 404) — la donnée est jetée, réutilisable gratuitement pour un `before_delete`. Le bulk delete (`handle_bulk.rs:408`) ne fait pas ce fetch.

## 2. Plan initial déjà conçu — [[project_hooks_signals]] (mémoire, jamais implémenté)

Architecture complète décidée en amont, résumé :

- `SignalBuilder<M: ActiveModelTrait>` : accumule un `Vec` de handlers par event (before_save, after_save, before_delete, after_delete), exécution séquentielle en pipeline (`run_before_save`, etc).
- Un `OnceLock<SignalBuilder<M>>` statique par modèle (ex: `hooks.rs::blog_signals()`), construit une seule fois.
- Macro `hooks!{}` — deux syntaxes (inline / builder avec fonctions nommées), les deux génèrent le même `OnceLock<SignalBuilder<M>>`.
- Opt-in par modèle via `derive_form!{ Blog { hooks: [before_save, after_delete], ... } }` — sans `hooks:`, comportement actuel (impl vide, zéro overhead).
- Avec opt-in, `derive_form!{}` génère un `impl ActiveModelBehavior` **délégant** vers `crate::hooks::<model>_signals()`.
- Règle framework : le framework ne doit jamais utiliser `ActiveModelBehavior` directement — `SignalBuilder` est le seul point d'entrée, framework et user peuvent tous deux hooker le même modèle (framework d'abord, user ensuite, ordre documenté).

Fichiers à toucher identifiés à l'époque : `derive_form/src/model/ast.rs`, `parser.rs`, `utils/active_model.rs`, `derive_form/src/lib.rs` (exposer `hooks!{}`), `demo-app/src/hooks.rs` (généré seulement si `hooks:` déclaré).

## 3. Piste explorée cette session — hooks côté `RuniqueForm` + `delete_fn`

Idée : brancher les hooks sur ce qui "sauve vraiment" côté site (le formulaire) et ce qui "supprime vraiment" (delete brut), plutôt que sur `ActiveModelBehavior`.

**Rejetée** : ne couvre pas les écritures DB qui ne passent ni par un formulaire ni par l'admin — tâche tokio en arrière-plan (ex: `appliquer_penalite` Campanile), script de données, vue custom appelant `model.save(db)`/`Entity::delete_by_id()` directement. Ni `RuniqueForm` ni `delete_fn` (généré uniquement par l'admin) ne sont un passage obligé de toute écriture DB — donc pas de "vrai signal".

Comparatif noté à ce moment :

| | Niveau `RuniqueForm`/`delete_fn` | Niveau `ActiveModelBehavior` |
|---|---|---|
| Déclenché par | formulaire (save) / admin (delete) seulement | tout save/delete SeaORM, peu importe le chemin |
| Bulk actions, scripts, tâches async, code applicatif hors admin | non couvert | couvert |
| Testabilité | form-level facile à driver (construire un form, save()) ; delete-level dur (pas d'instance form) | facile aussi : construire un `ActiveModel`, `.insert(db)`/`.delete(db)`, observer — pas besoin de simuler du HTTP |

Point Django vérifié : `pre_delete`/`post_delete` sont des signaux du **Model**, dispatchés par `queryset.delete()` — jamais du `ModelForm`. Confirme que delete ne doit pas être accroché au formulaire.

## 4. Où on en est — question ouverte

Retour vers le plan `SignalBuilder<M>`/`ActiveModelBehavior` de [[project_hooks_signals]] comme seule couche universelle. Mais Sébastien signale qu'**il manque une couche d'abstraction** — pas encore précisé laquelle. Pistes possibles à explorer à la reprise (non tranchées) :

- Manque une couche entre le `SignalBuilder<M>` bas niveau (par modèle, statique) et un point d'enregistrement plus haut niveau/central (registre global des signaux, découverte automatique, ou wiring au boot plutôt que par `OnceLock` dispersés) ?
- Manque une couche de contexte enrichi (qui a déclenché le save/delete — request/user courant — accessible depuis un hook `ActiveModelBehavior`, qui par nature n'a que `db`/`insert: bool` en paramètre, pas de `Request`) ? Ça rejoindrait le besoin "form-level" identifié en §3 (savoir qui a soumis) sans pouvoir le récupérer au niveau SeaORM pur.
- Autre chose — à clarifier avec Sébastien à la reprise.

Lien testabilité : rejoint [[project_phantom_data_test_harness_idee]] — le builder de test (transaction+rollback façon Django TestCase) doit pouvoir driver `ActiveModel::insert`/`.delete` directement pour exercer les hooks, sinon perte de la majorité de son utilité (constat initial de Sébastien qui a lancé cette discussion).
