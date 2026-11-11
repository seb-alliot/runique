# Anomalies ouvertes

Révélées par la mise en diagramme et le suivi des flux de données. Seules les anomalies
**encore présentes dans le code** sont listées ici, vérifiées le **2026-11-12**. Les entrées
corrigées ou devenues sans objet (admin 3.x, chantiers CSRF, uploads, HSTS, formulaires…) ont
été retirées : leur historique est dans git (`git log -p -- diagramme/anomalies.md`) et dans
le CHANGELOG.

Sévérité : 🔴 bloquant/sécurité · 🟠 sérieux · 🟡 mineur/dette.

## 🟠 Sérieux

| ID | Anomalie | Localisation | Piste |
|----|----------|--------------|-------|
| **S2** | `save` relâche le verrou **avant** l'écriture en base (« Release the lock before the async DB writes ») : deux `save` concurrents d'une même session peuvent atteindre la base dans l'ordre inverse, et la sauvegarde garde l'état d'avant | [cleaning_store.rs](../runique/src/middleware/session/cleaning_store.rs) (`save`, `drop(guard)` puis `update_db_backup`) | sérialiser l'écriture par session, ou versionner l'enregistrement |
| **IM1** | `init_migrate::write` écrit le crate `migration/` fichier par fichier : un échec au milieu laisse un crate incomplet, et `ensure` ne le répare jamais (il ne fait rien dès que le dossier existe) | [init_migrate.rs](../runique/src/migration/cli_migration/init_migrate.rs) (`write`, `ensure`) | **Prévu : complétion de makemigrations** (atomicité, nettoyage) |

## 🟡 Mineurs / dette

| ID | Anomalie | Localisation | Piste |
|----|----------|--------------|-------|
| **AM2** | Connexion : deux écritures dans `eihwaz_sessions` (`create`, puis la mise à jour de `save`) | [auth/session.rs](../runique/src/auth/session.rs) (`create`), [cleaning_store.rs](../runique/src/middleware/session/cleaning_store.rs) (`update_db_backup`) | **À concevoir avec S2** : écriture unique par `persist_to_db` avec le store par défaut ; l'appel explicite reste la seule écriture d'un store personnalisé (révocation, S6). Piste : un trait de persistance selon le store |
| **D3** | Index manquants pour les requêtes de purge : `eihwaz_sessions(user_id)` (Postgres n'indexe pas une FK tout seul) et `eihwaz_reset_tokens(expires_at)`. `eihwaz_history` a ses index `(resource_key, object_pk)` et `(user_id)` | [auth/migrations_table.rs](../runique/src/auth/migrations_table.rs) | **À ajouter** : nouvelle migration du framework (`m000000_000006…`), la migration des sessions étant déjà appliquée partout |
| **D4/S4** | `eihwaz_sessions` porte deux identifiants, `cookie_id` (unique) et `session_id`, sans explication dans la doc | [auth/migrations_table.rs](../runique/src/auth/migrations_table.rs) (`create_eihwaz_sessions_table`) | documenter le rôle de chacun |
| **E1** | `enable_debug_errors` / `with_debug_errors` : nom trompeur. Le middleware d'erreurs est toujours monté (c'est `config.debug` qui décide du contenu) ; le désactiver retire **toutes** les pages d'erreur, pas seulement le détail de débogage | [middleware/config.rs](../runique/src/middleware/config.rs), [middleware_staging/mod.rs](../runique/src/app/staging/middleware_staging/mod.rs) (`with_debug_errors`) | renommer (rupture, version majeure) ou documenter |
| **C7** | Formulaire d'un autre site vers un handler à extracteur Axum : durci (`Sec-Fetch-Site`), mais une requête **sans** cet en-tête reste à vérifier par le handler lui-même | [middleware/security/csrf.rs](../runique/src/middleware/security/csrf.rs) | documenté ; à garder en vue |
| **A7** | La porte annonce un 3ᵉ niveau, « les règles métier du développeur, qui ne peuvent que restreindre », mais seulement dans ses commentaires : aucune API publique ne permet encore d'en déclarer une (pas d'équivalent de `get_queryset` de Django pour un prédicat arbitraire, région, statut…) | [auth/gate/mod.rs](../runique/src/auth/gate/mod.rs), [auth/gate/rights.rs](../runique/src/auth/gate/rights.rs) | concevoir l'API (restrictive uniquement) ; backlog framework |
| **MG1** | `makemigrations` crée le crate `migration/` (`ensure`) **avant** les contrôles destructifs : une commande refusée sans `--force` laisse quand même ce crate | [cli_migration/mod.rs](../runique/src/migration/cli_migration/mod.rs) (`run`) | **Prévu : complétion de makemigrations** (atomicité, nettoyage) |
| **DUP1** | `to_snake_case` existe en trois versions qui ne traitent pas pareil les lettres hors ASCII (`to_ascii_lowercase` / `to_lowercase`) | [helpers.rs](../runique/src/migration/helpers.rs), [schema/mod.rs](../runique/src/migration/schema/mod.rs), [parse/builder/to_schema.rs](../runique/src/migration/parse/builder/to_schema.rs) (`pascal_to_snake`) | **Prévu : complétion de makemigrations** (atomicité, nettoyage) |
| **DC2** | `detect_col_type_builder` n'est appelé que par un test : code mort | [parse/col_types.rs](../runique/src/migration/parse/col_types.rs) | **Prévu : complétion de makemigrations** (atomicité, nettoyage) |

## Comportements documentés et limites connues

| ID | Comportement | Où c'est dit |
|----|--------------|--------------|
| **S1** | Les sessions anonymes (jeton CSRF, messages flash) ne vivent qu'en mémoire : perdues au redémarrage, absentes d'une autre instance ; un formulaire public ouvert avant échoue au contrôle CSRF, avec le message `csrf.invalid_or_missing` | `docs/*/session/14-sessions.md`, `docs/*/session/store/store.md` |
| **C8** | Upload : staging → écriture → publication → commit ; tout échec supprime le fichier publié (`rollback_uploads`), le staging restant est purgé (`sweep_stale_staging`). Reste un orphelin possible sur coupure en cours de traitement, ou si la suppression échoue (tracée en `error!`). L'ancien fichier remplacé par un nouvel upload reste sur le disque : sa suppression revient au développeur | [forms/form.rs](../runique/src/forms/form.rs) (`publish_uploads`, `rollback_uploads`), [forms/fields/media.rs](../runique/src/forms/fields/media.rs) |

## Faux positifs (ne pas re-signaler)

| ID | Hypothèse | Réalité vérifiée |
|----|-----------|------------------|
| **M1/AM1** | `makemigrations` ne gérerait pas les `ALTER COLUMN` | Faux : `diff_schemas` ([cli_migration/plan.rs:33](../runique/src/migration/cli_migration/plan.rs#L33)) calcule `modified_columns` ([diff.rs:90](../runique/src/migration/diff.rs#L90)). `ModelSchema::diff` (ajouts / suppressions) n'est pas utilisé par la CLI. |
| **AM2 (divergence)** | `session_id` divergent entre `create` et la sauvegarde | Faux : la mise à jour ne touche jamais `session_id`. Seule reste la double écriture (AM2 ci-dessus). |
| **TR1** | `ErrorContext` exposé en production | Faux : le rendu détaillé dépend de `config.debug` ([middleware/errors/error.rs](../runique/src/middleware/errors/error.rs)). |
| **D2** | `eihwaz_history.user_id` sans clé étrangère | Voulu et documenté dans le code (`create_eihwaz_history_table`) : l'historique survit à la suppression du compte. |
