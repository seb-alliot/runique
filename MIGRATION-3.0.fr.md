🌍 **Langues** : [English](https://github.com/seb-alliot/runique/blob/main/MIGRATION-3.0.md) | [Français](https://github.com/seb-alliot/runique/blob/main/MIGRATION-3.0.fr.md)

# Passer de Runique 2.x à 3.0

Ce guide liste chaque rupture de la 3.0 et ce qu'il faut faire, dans l'ordre où vous les rencontrerez pendant la mise à jour. Le [journal des modifications](CHANGELOG.fr.md) donne la liste complète, y compris les correctifs de sécurité et les ajouts qui ne demandent rien.

---

## Étape par étape

1. **`Cargo.toml`** : choisir un seul moteur de base de données (voir [Features Cargo](#features-cargo)).
2. **Modèles** : ajouter `nullable` aux colonnes facultatives et remplacer `fk(...)` par `belongs_to` (voir [DSL des modèles](#dsl-des-modèles)).
3. **Base** : ajouter la colonne `activated_at` à `eihwaz_users` (voir [Comptes utilisateurs](#comptes-utilisateurs)).
4. **Builder** : ajouter `.with_public_url(...)` (voir [Builder](#builder)).
5. **Code** : remplacer `&DatabaseConnection` par `&ADb`, puis corriger ce que le compilateur signale, à l'aide des sections ci-dessous.
6. **Admin** : régénérer `src/admins/` avec `runique start`.
7. **Migrations** : lancer `runique makemigrations` ; supprimer `migration/src/applied/` s'il existe.

---

## Features Cargo

**`default` n'active plus tous les moteurs, et les moteurs s'excluent mutuellement.** `default` vaut maintenant `["orm"]` ; activez exactement un moteur parmi `sqlite`, `postgres` ou `mysql` (`mariadb` est un alias de `mysql`). Deux à la fois est une erreur de compilation, sauf via `all-databases`, réservé aux outils multi-moteurs.

```toml
# 2.x : compilait SQLite, Postgres et MySQL quel que soit le moteur utilisé
runique = "2.2"

# 3.0
runique = { version = "3.0.0", features = ["postgres"] }
```

**`argon2` 0.6 et `scrypt` 0.12** : ne concerne que le code qui utilise ces crates directement. `scrypt` 0.12 n'a aucune feature par défaut : déclarez `features = ["phc", "getrandom"]`.

---

## Builder

**`.with_public_url(...)` est obligatoire en production** dès que la réinitialisation du mot de passe ou l'admin est activée : tous les liens envoyés par email sont construits dessus, jamais sur l'en-tête `Host` de la requête. Sans elle, l'application refuse de démarrer hors mode debug. Elle remplace `PasswordResetConfig::base_url()` et `AdminConfig::reset_password_url()`.

```rust
RuniqueApp::builder(config)
    .with_public_url("https://monsite.fr")
```

Pour construire vous-même un lien absolu, utilisez `request.public_url()`.

**Les réglages uniques ne s'appellent plus deux fois.** `with_public_url`, `routes`, `with_log`, `with_password_reset`, `with_database` / `with_database_config`, `with_session_duration` et `with_mailer` / `with_mailer_from_env` : un second appel remplaçait le premier (ou était ignoré en silence) ; il ne compile plus. Une fonction qui renvoie un builder à moitié construit nomme son état avec `app::builder::state::{No, Yes}`.

**Doublons supprimés** :

| 2.x | 3.0 |
|---|---|
| `.no_statics()` | `.static_files(\|s\| s.enabled(false))` |
| `StaticStaging::enable()` / `disable()` | `.enabled(true)` / `.enabled(false)` |
| `.with_error_handler(b)` | `.middleware(\|m\| m.with_debug_errors(b))` |
| `SessionConfig`, `SessionBackend`, `ASessionStore` | `.with_session_duration(...)` sur le builder, `.middleware(\|m\| m.with_session_store(...))` |
| `PasswordConfig::oauth(p)` | `PasswordConfig::Delegated(p)` |
| `MiddlewareConfig::with_host_validation` | `.middleware(\|m\| m.with_allowed_hosts(...))` |
| `RuniqueEngine::attach_middlewares` | le builder (elle n'était jamais appelée) |

**Clés `.env` qui ne sont plus lues** :

| Clé | Remplacement |
|---|---|
| `RUNIQUE_ENABLE_CACHE` | suit `DEBUG` ; à surcharger avec `.middleware(\|m\| m.with_cache(bool))` |
| `ALLOWED_HOSTS` | `.middleware(\|m\| m.with_allowed_hosts(\|h\| h.enabled(true).host("monsite.fr")))` |
| `RATE_LIMITING` | `.rate_limit(...)` sur les routes, `with_rate_limiter(...)` sur l'admin |
| `RUNIQUE_USER_TABLE` | aucun : `eihwaz_users` est la seule table utilisateurs |

Aucune ne faisait ce que son nom annonçait : `ALLOWED_HOSTS=monsite.fr` laissait la validation d'hôte désactivée.

**`DEBUG` et les autres drapeaux du `.env` sont lus quelle que soit la casse** : `True`, `YES`, `On` valent maintenant vrai. Vérifiez qu'aucun `.env` de production ne contient une telle valeur par erreur. L'enum `RuniqueEnv` est supprimée ; utilisez `is_debug()`.

**Les niveaux de log s'appellent `LogLevel` dans le prelude** : `use runique::prelude::*` exporte `tracing::Level` sous le nom `LogLevel`, qui n'entre plus en conflit avec un enum de modèle nommé `Level`. Remplacer `Level::INFO` par `LogLevel::INFO` dans `.with_log(...)`, ou importer `tracing::Level` soi-même.

---

## Connexion à la base : `ADb`

Toute signature publique qui prenait `&DatabaseConnection` ou `Arc<DatabaseConnection>` prend `&ADb` : `engine.db`, `BuiltinUserEntity`, `RuniqueSessionStore::new`, `order_by_random`, `search!`, formulaires, admin, auth. `ADb` implémente directement `ConnectionTrait` et `TransactionTrait` : `.insert(db)`, `.one(db)` et `db.begin()` fonctionnent tels quels, sans `.as_ref()`.

```rust
// 2.x
pub async fn get_article(db: &DatabaseConnection, id: i32) -> Option<blog::Model>

// 3.0
pub async fn get_article(db: &ADb, id: i32) -> Option<blog::Model>
```

`.with_database(DatabaseConnection)` ne change pas ; `ADb::from_connection(conn)` en construit un à la main.

---

## Comptes utilisateurs

**Un seul modèle utilisateur : `eihwaz_users`**, étendu avec `extend!{}`. Supprimés : les traits `UserEntity` et `AdminAuth`, `DefaultAdminAuth<E>`, `AdminLoginResult`, `RuniqueAdminAuth`, `.auth()` sur le builder de l'admin, `PasswordResetAdapter` / `PasswordResetHandler`, et le paramètre de type de `with_password_reset`.

```rust
// 2.x
.with_password_reset::<UserEntity>(|pr| pr.forgot_route("/forgot"))
.with_admin(|a| a.auth(RuniqueAdminAuth::new()).routes(admins::routes("/admin")))

// 3.0
.with_password_reset(|pr| pr.forgot_route("/forgot"))
.with_admin(|a| a.routes(admins::routes("/admin")))
```

Les recherches passent par `BuiltinUserEntity::find_by_id` / `find_by_username` / `find_by_email` ; la connexion à l'admin par `auth::authenticate_admin()`.

**Nouvelle colonne `activated_at` et contrainte `CHECK`.** Une réinitialisation du mot de passe ne réactive plus un compte désactivé par l'équipe : le compte sépare maintenant son état (`is_active`) de sa première activation (`activated_at`), et la base le garantit. Mettez à jour une table existante sans la supprimer (ce qui supprimerait en cascade les sessions, les groupes et les colonnes `extend!{}`). Sous Postgres :

```sql
BEGIN;
ALTER TABLE eihwaz_users ADD COLUMN activated_at timestamp without time zone;
UPDATE eihwaz_users SET activated_at = COALESCE(created_at, now()) WHERE is_active;
ALTER TABLE eihwaz_users ADD CONSTRAINT eihwaz_users_active_needs_activation
    CHECK (NOT is_active OR activated_at IS NOT NULL);
COMMIT;
```

Un compte inactif devient « en attente » : son propriétaire l'active avec le lien reçu par email.

---

## Connexion et sessions

**`login()` prend l'utilisateur, et plus la base.**

```rust
// 2.x
login(&session, &db, user.id, &user.username, user.is_staff, user.is_superuser, db_store, exclusive)

// 3.0
login(&session, &user, db_store, exclusive)
```

**`login()` vérifie le compte et renvoie `LoginError`.** Un compte qui ne peut pas se connecter (`can_sign_in()` : inactif, ou jamais activé) n'obtient pas de session, quel que soit le chemin qui l'a chargé. Traitez le refus :

```rust
match login(&session, &user, None, false).await {
    Ok(()) => { /* connecté */ }
    Err(LoginError::CannotSignIn) => { /* inactif, ou pas encore activé */ }
    Err(LoginError::Session(e)) => { /* le store de session a échoué */ }
}
```

**`auth_login()` est supprimée.** Elle relisait un compte qu'on avait déjà, et renvoyait `Ok(())` sans connecter personne quand le compte ne le pouvait pas. Passez le compte à `login()` ; le store de session par défaut sauvegarde déjà les sessions connectées en base.

```rust
// 2.x
auth_login(&session, &db, user.id).await?;

// 3.0 — après authenticate_user, une inscription, une activation…
login(&session, &user, None, false).await?;
```

**`activate_pending()` devient `activate_account()`** et renvoie le compte activé (`Option<Model>`) au lieu d'un `bool`, prêt pour `login()`. `None` : déjà activé, ou désactivé depuis par le staff (la réactivation reste la sienne).

```rust
if let Some(user) = BuiltinUserEntity::activate_account(&db, id).await? {
    login(&session, &user, None, false).await?;
}
```

**La session ne contient plus que l'id de l'utilisateur.** Le nom et les drapeaux `is_staff` / `is_superuser` n'y sont plus copiés : le compte est relu en base à chaque requête, comme `request.user` chez Django, donc un renommage, une rétrogradation ou une désactivation s'applique dès la requête suivante.

| 2.x | 3.0 |
|---|---|
| `get_username(&session)` | `request.user` → `user.username` |
| `get_user_id(&session)` | `request.user` → `user.id` (`get_user_id` est désormais interne) |
| `is_admin_authenticated(&session)` | `request.user` → `user.can_access_admin()` ; dans un middleware, l'extension `CurrentUser` |
| `SESSION_USER_USERNAME_KEY`, `SESSION_USER_IS_SUPERUSER_KEY` | supprimées |
| `session::SESSION_USER_IS_STAFF_KEY`, `session::IS_ACTIVE` (noms de champs de formulaire, pas des clés de session) | `admin_context::user::IS_STAFF`, `admin_context::user::IS_ACTIVE` |
| `session::SESSION_USER_DROITS_KEY` (clé de ressource admin) | `admin_context::permission::DROITS` |
| `admin_context::<template>::REQUIRED` | supprimées (jamais vérifiées) |
| `.with_log(\|l\| l.auth(\|a\| a.permissions(...)))` | supprimé (il traçait le cache des permissions, disparu) |

`is_authenticated(&session)` ne change pas.

**`logout()` vide toute la session**, messages flash compris, comme Django. Ajoutez un message flash destiné à l'après-déconnexion *après* l'appel.

**Les droits sont lus en base, plus mis en cache.** `cache_permissions`, `get_permissions`, `evict_permissions`, `clear_cache`, `restore_permissions`, `CachedPermissions`, `refresh_cache_for_user` et `load_user_middleware` sont supprimés, sans remplaçant. Hors de l'admin, `CurrentUser.groupes` est vide sauf si le handler appelle `req.load_user_rights().await`.

**`GuardRules`** : `GuardContext` est supprimé (rien ne le remplissait, donc toute règle refusait). Les rôles sont des noms de groupes, avec une seule méthode :

```rust
// 2.x
GuardRules::login_and_role("editeur")

// 3.0
GuardRules::roles(["editeur"])   // aussi : GuardRules::staff(), GuardRules::superuser()
```

`role`, `login_and_role`, `login_and_roles` et `with_role` sont supprimés.

---

## Handlers et formulaires

**`Request::is_get` / `is_post` / `is_put` / `is_delete` sont supprimées.** Utilisez `ValidationForm` pour les formulaires, ou comparez la méthode :

```rust
match ValidationForm::try_new(form, &request).await {
    Ok(valid) => { /* enregistrer */ }
    Err(form) => { /* réafficher avec les erreurs */ }
}

// ou
if request.method == Method::POST { ... }
```

**`request.query::<T>()` renvoie `AppResult<T>`** : une query string qui ne correspond pas à `T` donne une 400, affichée avec `400.html` (surchargeable). `T` n'a plus besoin de `Default`.

```rust
// 2.x
let filtres: Filtres = request.query();

// 3.0
let filtres: Filtres = request.query()?;
```

**Une implémentation maison de `FormField`** : `validate` et `finalize` sont des `async fn`. Ajoutez `#[async_trait::async_trait]` au-dessus du bloc `impl`.

**`cleaned_enum::<T>()` s'appuie sur `FromStr`.** Les enums générées par `model!{}` l'ont ; un `ActiveEnum` écrit à la main doit l'implémenter.

**`Prisme::for_test` et `Forms::mark_validated`** ne sont compilés qu'avec la feature `test-utils`.

**Messages flash** : la classe CSS est en minuscules. Renommez `.message-Success` / `Error` / `Info` / `Warning` en `.message-success`, etc.

**Autres API supprimées** : `Request::render_with` (appelez `insert`, puis `render`), `ErrorContext::with_request` (`with_request_helper`), `ErrorContext::with_details`, `RuniqueUser::roles`, `RuniqueUser::password_hash` (lisez le champ `password` du modèle), `RuniqueSessionStore::find_by_user`, `update_password` par email, `RuniqueQueryBuilder::all_from_engine`, `sanitize_with_fallback`, les alias `Bdd`, `OADb`, `OSecurityCsp`, `OSecurityHosts`, `TResult`, `DbResult`, et les constantes `NONCE_KEY`, `SESSION_USER_ROLES_KEY`, `REGISTERED_ROLES`.

---

## Admin

Après la mise à jour, régénérez `src/admins/` avec `runique start` : le code généré suit toutes les ruptures ci-dessous.

**`extra_routes` nomme l'opération**, qui décide du droit vérifié (lecture, création, modification, suppression) :

```rust
("/commandes/{id}/detail", "commandes", CrudOperation::Edit, get(detail_commande))
```

**`AdminResource::new` prend 4 arguments** : `ResourcePermissions`, `with_permissions`, le champ `permissions`, le paramètre `roles` et le registre des rôles (`register_roles`, `get_roles`) sont supprimés. Les droits viennent des groupes.

**Un `count_fn` écrit à la main** reçoit les filtres de colonnes : `(ADb, search, column_filters, scope)`, à appliquer comme dans `list_fn`.

---

## DSL des modèles

Le DSL `model!{}` / `extend!{}` est désormais lu par `runique_dsl`, partagé par la macro et `makemigrations`. Grammaire complète : [README de `runique_dsl`](runique/runique_dsl/README.md).

**Les colonnes sont NOT NULL par défaut.** `nullable` autorise NULL ; `required` ne rend obligatoire que le champ du formulaire. Sans `nullable` sur une colonne facultative existante, `makemigrations` s'arrête sur `nullable -> not_null`.

```rust
// 2.x : facultatif parce que non `required`
telephone: text [max_length: 20],

// 3.0
telephone: text [max_length: 20, nullable],
```

**`[step: x]` est supprimé** : il était accepté puis ignoré sur `int`, `float`, `decimal` et `percent`. Retirez-le ; un curseur écrit à la main garde `NumericField::range(...).step(x)`.

**`fk(...)` est supprimé** : déclarez les clés étrangères dans `relations` avec `belongs_to`.

```rust
// 2.x
auteur_id: int [fk(eihwaz_users.id, cascade)],

// 3.0
auteur_id: int,
// ...
relations: {
    belongs_to: eihwaz_users via auteur_id [cascade],
},
```

**`auto_now` / `auto_now_update`** : le champ Rust est `T`, plus `Option<T>`. Ils sont remplis par l'entité (`before_save`), de la même façon sur tous les moteurs : les migrations ne créent plus de trigger Postgres ni de `ON UPDATE`.

**Le nom d'une colonne ne décide plus de rien** : `created_at` / `updated_at` sans `auto_now` perdent leur `DEFAULT CURRENT_TIMESTAMP`, et une colonne `cache_key` n'est plus écartée des migrations.

**Enums `i32` / `i64`** : chaque variante doit avoir sa propre valeur entière.

**`checkbox [enum(X)]` est une liste**, stockée dans sa propre table `{table}_{champ}`.

**Aussi** : la CLI refuse un modèle illisible, un type v1 (`String`, `i32`…), deux `model!{}` dans un même fichier ou une cible `belongs_to` introuvable ; `customize` panique s'il assouplit `min_length`, `max_length`, `min` ou `max` ; `RelationDef`, `RelationKind` et `ModelSchema::relation()` sont supprimés.

Mettez à jour `derive_form` en même temps que `runique` : les conversions générées renvoient un `Result`.

---

## Migrations

**`runique migration down` et `runique migration status` sont supprimées** : elles ne mettaient jamais à jour `seaql_migrations`. Passez par SeaORM, qui exécute le vrai `down()` :

```bash
sea-orm-cli migrate down -n 1
sea-orm-cli migrate status
```

`runique migration up` reste. Le dossier `migration/src/applied/` n'est plus généré : supprimez-le. Gardez `snapshots/`, sur lequel `makemigrations` calcule les différences.
