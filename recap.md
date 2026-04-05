# API publique — Runique

> Items `pub` accessibles aux utilisateurs du framework.
> Exclus : `lib.rs`, items `pub(crate)` (internes), tests.
>
> **Note CLI** : les fonctions `utils/cli/` restent `pub` car le binaire `src/bin/runique.rs`
> est une crate séparée de la lib — `pub(crate)` n'y est pas accessible.

---

## admin/

### config/config_admin.rs
| Item | Type | Description |
|------|------|-------------|
| `AdminConfig` | struct | Configuration du module admin |

### permissions/droit.rs
| Item | Type | Description |
|------|------|-------------|
| `Droit` | struct | Permission scopée (resource_key + access_type) |
| `pull_droits_db()` | async fn | Récupère les droits d'un utilisateur depuis la DB |

### permissions/groupe.rs
| Item | Type | Description |
|------|------|-------------|
| `Groupe` | struct | Groupe d'utilisateurs avec droits |
| `pull_groupes_db()` | async fn | Récupère les groupes d'un utilisateur depuis la DB |

### resource.rs
| Item | Type | Description |
|------|------|-------------|
| `AdminIdType` | enum | Type de clé primaire : I32, I64, Uuid |
| `ResourcePermissions` | struct | Permissions CRUD par opération (list/view/create/edit/delete) |
| `CrudOperation` | enum | List, View, Create, Edit, Delete |
| `ColumnFilter` | enum | All, Include(cols), Exclude(cols) |
| `DisplayConfig` | struct | Config d'affichage : icône, colonnes, pagination, filtres |
| `AdminResource` | struct | Métadonnées d'une ressource administrable |

---

## app/

### builder.rs
| Item | Type | Description |
|------|------|-------------|
| `RuniqueAppBuilder` | struct | Builder fluent pour construire l'application |

### error_build.rs
| Item | Type |
|------|------|
| `BuildError` | struct |
| `BuildErrorKind` | enum |
| `CheckReport` | struct |
| `CheckError` | struct |

### runique_app.rs
| Item | Type | Description |
|------|------|-------------|
| `RuniqueApp` | struct | Application Runique prête à lancer |

---

## config/

| Item | Type | Description |
|------|------|-------------|
| `RuniqueConfig` | struct | Configuration principale |
| `RuniqueRouter` | struct | Configuration du routeur |
| `SecurityConfig` | struct | Configuration de sécurité |
| `ServerConfig` | struct | Configuration du serveur |
| `StaticConfig` | struct | Configuration des assets statiques |
| `DatabaseConfig` | struct | Configuration de la base de données |
| `DatabaseEngine` | enum | PostgreSQL, MySQL, SQLite |
| `DatabaseConfigBuilder` | struct | Builder pour DatabaseConfig |

---

## context/

### context/template.rs
| Item | Type | Description |
|------|------|-------------|
| `AppError` | struct | Erreur applicative avec contexte de rendu |
| `Request` | struct | Requête enrichie (session, engine, contexte Tera) |

---

## errors/

| Item | Type | Description |
|------|------|-------------|
| `RuniqueResult<T>` | type | Result<T, RuniqueError> |
| `RuniqueError` | enum | Erreurs du framework |
| `ErrorContext` | struct | Contexte d'erreur détaillé |
| `ErrorType` | enum | Types d'erreurs |
| `TemplateInfo` | struct | Info debug template |
| `RequestInfo` | struct | Info debug requête |
| `StackFrame` | struct | Frame de stack trace |
| `EnvironmentInfo` | struct | Info environnement |

---

## flash/

| Item | Type | Description |
|------|------|-------------|
| `MessageLevel` | enum | Success, Error, Warning, Info |
| `Message` | struct | Message flash unitaire |
| `FlashMessage` | struct | Message flash complet avec niveau |

---

## forms/

### forms/base.rs
| Item | Type | Description |
|------|------|-------------|
| `FieldConfig` | struct | Configuration commune de tous les champs |
| `TextConfig` | struct | Config champs texte |
| `NumericConfig` | enum | Config champs numériques |
| `Range` | struct | Plage min/max |
| `CommonFieldConfig` | trait | Accès à la config commune |
| `FormField` | trait | Trait principal pour tous les champs |

### forms/fields/
| Item | Type |
|------|------|
| `BooleanField` | struct |
| `ChoiceOption`, `ChoiceField`, `RadioField`, `CheckboxField` | struct |
| `DateField`, `TimeField`, `DateTimeField`, `DurationField` | struct |
| `IntoUploadPath` | trait |
| `FileFieldType` | enum |
| `AllowedExtensions` | struct |
| `UploadPathFn` | type |
| `FileUploadConfig` | struct |
| `FileField` | struct |
| `HiddenField` | struct |
| `NumericField` | struct |
| `ColorField`, `SlugField`, `UUIDField`, `JSONField`, `IPAddressField` | struct |
| `TextField` | struct |
| `SpecialFormat` | enum |

### forms/form.rs
| Item | Type | Description |
|------|------|-------------|
| `Forms` | struct | Gestionnaire de formulaires — accès aux champs, erreurs, valeurs |

### forms/generic.rs
| Item | Type |
|------|------|
| `GenericField` | struct |

### forms/extractor.rs
| Item | Type | Description |
|------|------|-------------|
| `Prisme<T>` | struct | Extracteur Axum pour formulaires validés (CSRF + validation) |

### forms/field.rs
| Item | Type | Description |
|------|------|-------------|
| `RuniqueForm` | trait | Trait principal — implémenter pour créer un formulaire |

### forms/model_form/mod.rs
| Item | Type | Description |
|------|------|-------------|
| `ModelForm` | trait | Formulaire lié à un modèle SeaORM |

---

## middleware/

### middleware/auth/auth_session.rs
| Item | Type | Description |
|------|------|-------------|
| `CurrentUser` | struct | Utilisateur authentifié injecté dans les extensions |
| `is_authenticated()` | async fn | Vérifie la présence d'un utilisateur en session |
| `is_admin_authenticated()` | async fn | Vérifie si l'utilisateur a accès à l'admin |
| `get_user_id()` | async fn | Retourne l'ID de l'utilisateur connecté |
| `get_username()` | async fn | Retourne le username de l'utilisateur connecté |
| `login()` | async fn | Connecte un utilisateur (charge droits, groupes, session) |
| `auth_login()` | async fn | Connecte depuis un user_id uniquement |
| `logout()` | async fn | Déconnecte — vide session et cache permissions |
| `protect_session()` | async fn | Protège une session anonyme contre le cleanup |
| `unprotect_session()` | async fn | Retire la protection d'une session anonyme |
| `has_permission()` | async fn | Vérifie si l'utilisateur a un droit donné |

**Méthodes de `CurrentUser`** :
| Méthode | Description |
|---------|-------------|
| `droits_effectifs()` | Droits directs + hérités des groupes, dédupliqués |
| `has_droit(nom)` | Vérifie un droit spécifique |
| `has_any_droit(noms)` | Vérifie au moins un droit parmi la liste |
| `can_access_admin()` | is_staff ou is_superuser |
| `can_admin(required)` | Vérifie les droits pour une opération admin |

### middleware/auth/login_guard.rs
| Item | Type | Description |
|------|------|-------------|
| `LoginGuard` | struct | Protection brute-force login (tentatives + lockout) |

### middleware/auth/user.rs
| Item | Type | Description |
|------|------|-------------|
| `BuiltinUserEntity` | struct | Implémentation du modèle utilisateur builtin |
| `RuniqueAdminAuth` | struct | Authentification admin intégrée |

### middleware/auth/user_trait.rs
| Item | Type | Description |
|------|------|-------------|
| `RuniqueUser` | trait | Trait pour les modèles utilisateur custom |

### middleware/auth/reset.rs
| Item | Type | Description |
|------|------|-------------|
| `ForgotPasswordForm` | struct | Formulaire mot de passe oublié |
| `PasswordResetForm` | struct | Formulaire réinitialisation |
| `PasswordResetAdapter` | trait | Adapter custom pour le reset |
| `PasswordResetConfig` | struct | Configuration du flux reset |
| `PasswordResetHandler` | struct | Handler complet reset |
| `PasswordResetStaging` | struct | Staging pour le reset |
| `handle_forgot_password()` | async fn | Handler GET/POST mot de passe oublié |
| `handle_password_reset()` | async fn | Handler GET/POST réinitialisation |

### middleware/auth/admin_auth.rs
| Item | Type | Description |
|------|------|-------------|
| `AdminAuth` | trait | Interface d'authentification admin custom |
| `AdminLoginResult` | struct | Résultat d'authentification admin |

### middleware/auth/default_auth.rs
| Item | Type | Description |
|------|------|-------------|
| `DefaultAdminAuth` | struct | Authentification admin par défaut |
| `UserEntity` | struct | Entité utilisateur pour l'auth par défaut |

### middleware/security/allowed_hosts.rs
| Item | Type | Description |
|------|------|-------------|
| `HostPolicy` | struct | Politique des hôtes autorisés |

### middleware/security/csp.rs
| Item | Type | Description |
|------|------|-------------|
| `HTMX_STYLE_HASHES` | const | Hashes CSP pour les styles HTMX intégrés |
| `SecurityPolicy` | struct | Configuration Content Security Policy |

### middleware/security/csrf.rs
| Item | Type | Description |
|------|------|-------------|
| `CsrfToken` | struct | Token CSRF (pub(crate) en interne, exposé via prelude) |

### middleware/security/rate_limit.rs
| Item | Type | Description |
|------|------|-------------|
| `RateLimiter` | struct | Limiteur de débit par clé (fenêtre glissante) |

### middleware/session/session_db.rs
| Item | Type | Description |
|------|------|-------------|
| `RuniqueSessionStore` | struct | Store de sessions en base de données |

### middleware/session/session_memory.rs
| Item | Type | Description |
|------|------|-------------|
| `CleaningMemoryStore` | struct | Store sessions mémoire avec nettoyage automatique |

---

## macros/routeur/

### router_ext.rs
| Item | Type | Description |
|------|------|-------------|
| `RouterExt` | trait | Extension de Router Axum |
| `.login_required(path, name, handler, redirect_url)` | méthode | Protège une route — redirige si non authentifié |
| `.rate_limit(path, name, handler, max, retry_after)` | méthode | Protège une route avec rate limiting |
| `.rate_limit_many(max, retry_after, routes)` | méthode | Plusieurs routes, compteur partagé |

---

## utils/

### utils/aliases/
| Alias | Type réel |
|-------|-----------|
| `ATera` | `Arc<Tera>` |
| `ADb` | `Arc<DatabaseConnection>` |
| `Bdd` | `Option<DatabaseConnection>` |
| `ASecurityCsp` | `Arc<SecurityPolicy>` |
| `ASecurityHosts` | `Arc<HostPolicy>` |
| `AEngine` | `Arc<RuniqueEngine>` |
| `ARuniqueConfig` | `Arc<RuniqueConfig>` |
| `ASessionStore` | `Arc<dyn SessionStore>` |
| `OCurrentUser` | `Option<CurrentUser>` |
| `OCsrfToken` | `Option<CsrfToken>` |
| `OCspNonce` | `Option<CspNonce>` |
| `StrMap` | `HashMap<String, String>` |
| `StrVecMap` | `HashMap<String, Vec<String>>` |
| `JsonMap` | `HashMap<String, Value>` |
| `FieldsMap` | `IndexMap<String, Box<dyn FormField>>` |
| `Messages` | `Vec<FlashMessage>` |
| `ARlockmap` | `Arc<RwLock<HashMap<String, String>>>` |
| `AppResult<T>` | `Result<T, Box<AppError>>` |

### utils/cli/ — accessible via binaire uniquement (reste pub pour raison technique)
| Item | Fichier | Description |
|------|---------|-------------|
| `create_superuser()` | cli_admin.rs | Crée un superuser en interactif |
| `run()` | makemigration.rs | Lance la génération de migration |
| `up()` | migrate.rs | Applique les migrations |
| `down()` | migrate.rs | Annule les migrations |
| `status()` | migrate.rs | Affiche le statut des migrations |
| `create_new_project()` | new_project.rs | Scaffold d'un nouveau projet |
| `runique_start()` | start.rs | Lance le daemon admin |

### utils/env.rs
| Item | Description |
|------|-------------|
| `RuniqueEnv` | enum — Development, Production |
| `is_debug()` | Retourne true si DEBUG=true |
| `css_token()` | Cache-buster 4 chiffres pour les assets |
| `load_env()` | Charge les variables d'environnement |

### utils/init_error/init.rs
| Item | Description |
|------|-------------|
| `init_logging()` | Initialise le logging (tracing) |

### utils/mailer/
| Item | Description |
|------|-------------|
| `MAILER_CONFIG` | Singleton de configuration mailer |
| `MailerConfig` | struct — config SMTP |
| `Email` | struct — email à envoyer |
| `mailer_init()` | Initialise le mailer depuis une config |
| `mailer_init_from_env()` | Initialise le mailer depuis les variables d'env |
| `mailer_configured()` | Vérifie si le mailer est configuré |

### utils/password/
| Item | Description |
|------|-------------|
| `PasswordHasher` | trait — interface de hachage |
| `BaseHash` | struct — implémentation de base |
| `PasswordConfig` | enum — External ou Manual |
| `External` | enum — Argon2, Bcrypt, Scrypt |
| `PasswordHandler` | trait — interface de manipulation |
| `PasswordService` | struct — service complet |
| `PASSWORD_CONFIG` | static — singleton |
| `password_init()` | Initialise le service password |
| `password_get()` | Retourne la config courante |
| `hash()` | Hache un mot de passe |
| `verify()` | Vérifie un mot de passe |

### utils/reset_token/
| Item | Description |
|------|-------------|
| `generate()` | Génère un token de réinitialisation |
| `consume()` | Consomme (invalide) un token |
| `encrypt_email()` | Chiffre un email dans un token |
| `decrypt_email()` | Déchiffre un email depuis un token |
| `peek()` | Vérifie un token sans le consommer |

### utils/trad/
| Item | Description |
|------|-------------|
| `Lang` | enum — langues supportées |
| `set_lang()` | Définit la langue du framework (FR, EN, etc.) |
| `current_lang()` | Retourne la langue courante |

### utils/pk.rs
| Item | Description |
|------|-------------|
| `Pk` | type alias — identifiant utilisateur |

### utils/resolve_ogimage/
| Item | Description |
|------|-------------|
| `resolve_og_image()` | Résout l'URL d'une image Open Graph |

---

## Passé en pub(crate) — internes au framework

| Catégorie | Items |
|-----------|-------|
| **utils/trad** | `t()`, `tf()` — traduction interne du framework |
| **admin/daemon** | `ResourceDef`, `ConfigureDef`, `ParsedAdmin`, `parse_admin_file()`, `generate()`, `watch()` |
| **admin handlers** | `AdminBody`, `PrototypeAdminState`, `admin_get/post/get_id/post_id()` |
| **admin interne** | `AdminRegistry`, `builtin_resources()`, `DynForm`, formulaires admin, `insert_admin_messages()` |
| **admin/middleware** | `admin_required()`, `check_permission()` |
| **resource_entry** | `ListFn`, `GetFn`, `DeleteFn`, `UpdateFn`, `CreateFn`, `CountFn`, `FilterFn`, `ResourceEntry`, `ListParams`, `SortDir` |
| **middleware fonctions** | `rate_limit_middleware()`, `csp_middleware()`, `csp_report_only_middleware()`, `security_headers_middleware()`, `https_redirect_middleware()`, `csrf_middleware()`, `allowed_hosts_middleware()`, `load_user_middleware()`, `login_required_middleware()` |
| **permissions cache** | `cache_permissions()`, `evict_permissions()`, `get_permissions()`, `clear_cache()` |
| **csrf utils** | `CsrfContext`, `generation_token()`, `generation_user_token()`, `mask_csrf_token()`, `unmask_csrf_token()` |
| **staging** | `AdminStaging`, `CoreStaging`, `CspConfig`, `HostConfig`, `MiddlewareStaging`, `StaticStaging`, `TemplateLoader` |
| **router admin** | `AdminState`, `build_admin_router()`, `AdminTemplate`, `PathAdminTemplate` |
| **roles** | `register_roles()`, `get_roles()` |
