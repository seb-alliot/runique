🌍 **Languages**: [English](https://github.com/seb-alliot/runique/blob/main/CHANGELOG.md) | [Français](https://github.com/seb-alliot/runique/blob/main/CHANGELOG.fr.md)

# Journal des modifications

Toutes les modifications notables de ce projet sont documentées dans ce fichier.

---

## [2.1.13] - 2026-06-01

### Correctif — `runique` (templates admin, CSS admin)

* **Double-encodage des valeurs chaîne dans les vues liste et détail admin :** `{{ value | escape }}` était utilisé dans `list_partial.html` et `detail.html` alors que l'autoescaping Tera était déjà actif. Le filtre `escape` convertit `/` en `&#x2F;`, puis l'autoescaping réencode `&` en `&amp;`, ce qui faisait afficher le texte littéral `&#x2F;` au lieu de `/`. Même problème dans les inputs cachés de filtre (`value="{{ val | escape }}"`), ce qui aurait cassé les comparaisons de filtre pour les valeurs contenant `/`. Correctif : les trois occurrences remplacées par `{{ value }}` / `{{ val }}` — l'autoescaping seul suffit pour la protection XSS.

* **CSS de la sidebar filtres admin avec sélecteurs non concordants :** la section CSS du panneau filtres utilisait des sélecteurs en tirets (`.admin-filter-sidebar`, `.filter-group`, `.admin-list-layout`, etc.) alors que les templates avaient déjà été refactorisés en BEM (`.admin-filter__sidebar`, `.admin-filter__group`, `.admin-list__layout`, etc.). Le panneau filtres n'avait aucun style effectif. Correctif : la section entière est réécrite avec les sélecteurs BEM ; le mobile utilise désormais un pattern offcanvas (`position: fixed; right: -300px` → `.mobile-open { right: 0 }`) avec un overlay de fond.

### Correctif — `runique` (templates)

* **Templates internes non autoéchappés — vecteur XSS sur les champs de formulaire admin :** l'autoescaping Tera s'active pour les clés logiques se terminant par `.html` ou `.xml`. Les templates internes du framework étaient enregistrés sans le suffixe `.html`, leurs variables étaient donc rendues brutes. En particulier, `{{ form_fields.html }}` (contenant le HTML de formulaire généré par Runique) n'était pas autoéchappé et aurait été interprété comme une variable manquante. Correctif : toutes les clés de templates internes incluent désormais le suffixe `.html` ; le préprocesseur de templates (`process_content`) réécrit `{{ form_fields.html }}` en `{{ form_fields.html | safe }}` via `ADMIN_FORM_HTML_REGEX` — seule variable exemptée de l'autoescaping car toujours du HTML généré par Runique, jamais une saisie utilisateur. Les clés `{% extends %}` dans les exemples de documentation ont été mises à jour en conséquence (`"admin/admin_template.html"`, `"admin_base.html"`).

* **`resolve_og_image` : hash de version CSS appliqué aux URLs media, et double slash potentiel :** la fonction ajoutait `?v=<hash>` à l'URL de l'image OG sans condition quand un token de build CSS était présent. Ce hash est calculé à partir des assets statiques (CSS/JS) au moment du build et n'a aucun lien avec le contenu du fichier media — l'appliquer à une image uploadée est sémantiquement faux et casse le cache-busting côté scrapers quand l'image change sans redéploiement. Par ailleurs, si une entrée `allowed_hosts` contenait un slash final, le préfixe de host (`https://host/`) concaténé à un `og_image` commençant par `/` produisait un double slash (`https://host//media/...`). Correctif : le `?v=` est supprimé des URLs og:image ; `trim_end_matches('/')` est appliqué au host et `trim_start_matches('/')` au chemin de l'image avant concaténation ; les URLs absolues (`http://` / `https://`) sont retournées directement sans modification.

---

## [2.1.12] - 2026-05-30

### Correctif — `runique` (sessions)

* **Fallback DB des sessions cassé après `cycle_id()` (régression critique introduite en 2.1.9) :** le correctif de fixation de session avait ajouté `session.cycle_id()` à chaque élévation de privilège. tower-sessions appelle `create()` (et non `save()`) au commit de la réponse pour une session recyclée. `create()` dans `CleaningMemoryStore` ne persistait pas en DB, ce qui empêchait toute écriture des données de session authentifiée dans le store DB après un login — cassant entièrement le fallback de redémarrage à chaud. Correctif : `create()` persiste désormais dans `RuniqueSessionStore` lorsque `SESSION_USER_ID_KEY` est présent, identique à la logique déjà présente dans `save()`.

* **Entrées DB orphelines après `cycle_id()` (régression de nettoyage) :** tower-sessions appelle `delete(old_id)` après `create(new_id)` lors d'un `cycle_id()`. `CleaningMemoryStore::delete()` ne supprimait l'entrée qu'en mémoire, laissant l'ancien identifiant de session comme orphelin en DB. Correctif : `delete()` appelle désormais aussi `db.delete()` lorsque le fallback DB est configuré. L'opération est idempotente — `logout()` supprime déjà l'entrée via `RuniqueSessionStore::delete()` avant d'appeler `session.delete()`.

* **`exclusive_login` n'invalidait que les sessions mémoire, pas les sessions DB :** `CleaningMemoryStore::save()` évictait les sessions mémoire du même utilisateur quand `exclusive_login = true`, mais n'appelait jamais `RuniqueSessionStore::invalidate_other_sessions()`. Après un redémarrage serveur, les sessions évictées étaient restaurées depuis la DB, rendant la garantie de connexion exclusive ineffective. Correctif : l'invalidation DB est désormais collectée sous le verrou et exécutée après sa libération, symétrique au nettoyage mémoire. Utilisation de `Pk` directement (`serde_json::from_value::<Pk>`) plutôt que `as_i64` — correct avec le feature flag `big-pk`.

### Sécurité — `runique` (formulaires, admin, templates)

* **Token CSRF calculé mais jamais appliqué sur les formulaires publics (critique) :** le pipeline Prisme calcule `csrf_valid` pour chaque requête mutante mais le retourne comme simple booléen sans rejeter. La couche formulaire (`Request::form()` / `is_valid()`) ne le consommait jamais, et le validateur du champ caché CSRF est inopérant depuis le retrait de `set_expected_value` (les tokens masqués diffèrent à chaque requête). Seul le panel admin revérifiait le booléen manuellement ; tout handler POST public bâti sur le pattern documenté `form.is_valid()` acceptait donc une soumission forgée cross-site — vérifié de bout en bout sur l'endpoint d'inscription en production (compte créé sans cookie de session ni token). Correctif : `Request::form()` positionne désormais `force_invalid` quand la requête est mutante et que `prisme.csrf_valid` est faux, de sorte que `is_valid()` échoue en mode fermé — réutilisation du mécanisme honeypot existant, sans réintroduire `set_expected_value`.

* **Injection SQL sur MySQL/MariaDB via interpolation de valeur en SQL brut (élevée) :** les filtres de liste admin, la recherche (`search_cond!`), le `group_set` bulk et la requête d'options m2m construisaient des conditions avec `Expr::cust(format!("... = '{}'", val))`, échappées uniquement par doublement des apostrophes (`'` → `''`). Suffisant sur PostgreSQL/SQLite (`standard_conforming_strings`) mais contournable sur MySQL/MariaDB, où un backslash échappe l'apostrophe suivante (`\'` suivi de `''` sort de la chaîne littérale). Un utilisateur staff authentifié avec accès lecture pouvait exécuter du SQL arbitraire. Correctif : toutes les valeurs attaquant-contrôlées sont désormais des paramètres liés via `Expr::cust_with_values(..., [val])`, déléguant l'échappement à la couche backend-aware de sea-query. Les identifiants de colonne restent inline mais demeurent whitelistés (`FILTER_COLS` / `SORT_COLS`) ou fixés par le schéma.

* **Injection SQL dans les ressources admin builtin (élevée) :** le `list_fn` écrit à la main des ressources builtin `users`, `groupes` et `droits` interpolait le nom de colonne de filtre (`?filter_<col>=`) directement dans `CAST({col} AS TEXT)` **sans** whitelist — une injection d'identifiant exploitable sur **tous** les backends, pas seulement MySQL — en plus de l'échappement de valeur non sûr. Correctif : les noms de colonne sont validés contre un charset `[A-Za-z0-9_]` avant usage, et les valeurs sont liées via `cust_with_values`.

* **XSS stocké via le filtre `| markdown` (élevée) :** le préprocesseur de templates réécrit chaque `{{ x | markdown }}` en `{{ x | markdown | safe }}`, et le filtre émettait la sortie de `pulldown-cmark` sans sanitisation. Le HTML brut inline (`<script>`, `onerror=`) et les URL de lien/image `javascript:` passaient sans échappement, rendant tout Markdown rédigé par un utilisateur vecteur de XSS stocké. Correctif : le filtre passe désormais sa sortie dans un nouveau `sanitize_markdown()` (ammonia) — schemes http/https/mailto uniquement, pas d'attribut `style`, HTML brut supprimé, `rel="noopener noreferrer"` sur les liens. Le whitelist partagé `ALLOWED_TAGS` / `ALLOWED_ATTRS` a été élargi (h1–h6, tables, `del`/`s`/`sub`/`sup`, `hr`, `img`, `code[class]`) pour couvrir la sortie Markdown sans activer aucun élément porteur de script.

* **Contournement du filtre open-redirect via backslash (moyenne) :** `is_safe_redirect` traitait `/\evil.com` comme un chemin relatif sûr (`starts_with('/')` mais pas `"//"`). Les navigateurs normalisent `\` en `/`, le transformant en `//evil.com` protocol-relative. Correctif : les backslashes sont normalisés en slashes avant la détermination de même origine.

* **IP spoofing via `X-Forwarded-For` en mode TLS autonome (moyenne) :** le serveur TLS intégré (`axum_server::bind_rustls`, pour ACME / HTTPS autonome) servait le routeur via `into_make_service()` sans connect-info. Sans `ConnectInfo<SocketAddr>`, `trusted_proxies` voyait `conn_ip = None`, retombait sur loopback (un CIDR de confiance), et faisait donc confiance au header `X-Forwarded-For` contrôlé par le client — permettant à n'importe qui de forger son IP (contournement du rate-limit, logs d'audit falsifiés). Corrigé sur trois niveaux : (1) le chemin TLS utilise désormais `into_make_service_with_connect_info::<SocketAddr>()`, exposant la vraie IP du pair ; (2) `extract_client_ip` retourne loopback sans jamais lire `X-Forwarded-For` quand l'IP du pair est inconnue, donc l'absence de connect-info ne peut plus activer le spoofing ; (3) les adresses IPv4-mappées en IPv6 (`::ffff:a.b.c.d`, vues sur socket dual-stack) sont canonicalisées en IPv4 avant la vérification des CIDR de confiance, pour qu'un reverse proxy privé soit correctement reconnu. Couvert par des tests unitaires dans `trusted_proxies.rs`.

* **`is_authenticated` désérialisait l'id utilisateur en `i32` (faible) :** avec le feature `big-pk` (`i64`), un id utilisateur supérieur à `i32::MAX` échouait à la désérialisation, et `is_authenticated` retournait `false` de façon incohérente avec `get_user_id` (qui utilise `Pk`). Correctif : lecture en `Pk`.

---

## [2.1.10] - 2026-05-30

### Correctif — `runique` (admin)

* **Édition et suppression bloquées pour toutes les ressources sans `own_field` (régression critique) :** la précédence des opérateurs dans la vérification d'appartenance produisait `(action == "edit" && !can_update && !can_update_own) || !check_owns_record(...)`. Comme `check_owns_record` retourne `false` quand `own_field` n'est pas déclaré, `!check_owns_record()` était toujours `true`, provoquant un refus de permission sur chaque requête d'édition et de suppression indépendamment des droits réels de l'utilisateur. Correctif : la condition est désormais `!can_update && !(can_update_own && check_owns_record(...))`, appliquée séparément dans `admin_get_id` et `admin_post_id`.

---

## [2.1.9] - 2026-05-28

### Sécurité — `runique` (admin, auth)

* **Injection SQL dans les filtres de liste admin (élevée) :** le nom de colonne extrait des paramètres URL (`?filter_<col>=val`) était interpolé directement dans `Expr::cust(format!("CAST({} AS TEXT) = '{}'", col, ...))` sans aucune validation. Un utilisateur staff authentifié avec des droits minimaux pouvait exécuter du SQL arbitraire sur la base de données. Correctif : le générateur émet désormais deux listes statiques (`SORT_COLS`, `FILTER_COLS`) construites à la génération de code depuis les colonnes déclarées dans `list_display` et `list_filter`. Tout nom de colonne absent de ces listes est silencieusement ignoré avant d'atteindre la requête.

* **Fixation de session au login (moyenne) :** `login()` n'appelait pas `session.cycle_id()` lors de l'élévation de privilège (anonyme → authentifié). Un attaquant ayant planté un identifiant de session dans le navigateur de la victime avant le login pouvait le réutiliser après l'authentification. Correctif : `session.cycle_id().await` est désormais appelé à chaque élévation de privilège (nouvelle session ou changement d'utilisateur). Atténué en pratique par les attributs `SameSite=Strict` + `HttpOnly`, mais la mitigation standard était absente.

* **Granularité des droits d'écriture admin (moyenne) :** `check_write_access` retournait `true` si n'importe lequel parmi `can_create`, `can_update` ou `can_delete` était activé. Un utilisateur staff avec seulement `can_create` pouvait éditer et supprimer n'importe quel enregistrement. Correctif : trois gardes distincts (`check_can_create`, `check_can_update`, `check_can_delete`) sont désormais appliqués par opération et par méthode HTTP. Les actions bulk POST sont également contrôlées par type d'action (`delete` → `can_delete`, les autres → `can_update`).

* **IDOR — `can_update_own` / `can_delete_own` non appliqués (faible) :** les flags de permission "own" existaient dans le modèle de permissions et étaient injectés dans les templates, mais les closures CRUD `(db, id)` / `(db, id, data)` ne transportaient aucune identité utilisateur, rendant la vérification d'appartenance structurellement impossible. Les routes edit et delete autorisaient silencieusement n'importe quel enregistrement. Correctif : une nouvelle option DSL `own_field: "nom_champ"` déclare le champ JSON utilisé pour la comparaison d'appartenance. Quand un utilisateur a `can_update_own` (sans `can_update`), le handler récupère l'enregistrement via `get_fn` et compare `record[own_field]` avec `current_user.id`. Si `own_field` n'est pas déclaré, les permissions "own" sont bloquées par défaut (repli sûr).

### Ajouté — `runique` (formulaires, debug)

* **Sortie `eprintln!` debug pour tout le pipeline de traitement des formulaires :** quand `DEBUG=true` et que le champ `FormTracing` correspondant est configuré, chaque étape émet désormais à la fois un événement `tracing` structuré (filtré par le niveau du subscriber) et un `eprintln!` directement sur stderr (contourne le filtre du subscriber). Étapes couvertes : enregistrement des champs, `set_value` par champ (POST), normalisation des checkboxes, validation par champ, résultat de validation, finalisation par champ, rendu par champ.

### Ajouté — `runique` (DSL admin)

* **`own_field` dans `admin!{}`:** nouvelle clé DSL optionnelle qui déclare le champ d'appartenance d'un enregistrement pour l'application de `can_update_own` / `can_delete_own`. Exemple : `own_field: "user_id"`.

### Sécurité — `runique` (formulaires)

* **Garde sur `save()` / `save_as()` contre la validation contournée (faible) :** un développeur pouvait appeler `form.save()` sans avoir appelé `is_valid()` au préalable, contournant entièrement la validation des champs, la vérification du token CSRF et les règles métier de `clean()`. Correctif : les deux méthodes retournent désormais `Err(DbErr::Custom(...))` immédiatement si `is_valid()` n'a pas été appelé ou a retourné `false`. La vérification s'effectue via la méthode interne `is_save_allowed()` (`!force_invalid && validated && !has_errors()`). Un helper `#[doc(hidden)]` `Forms::mark_validated()` est fourni pour les tests qui vérifient le comportement des hooks save en isolation.

---

## [2.1.8] - 2026-05-28

### Corrigé — `runique` (admin, bulk)

* **`bulk_create` violait la contrainte UNIQUE à la re-soumission :** le `create_fn` généré effectuait un INSERT simple par valeur. Resoumettre les mêmes jours causait une violation UNIQUE arrêtant la boucle. Le générateur émet désormais un upsert : pour chaque valeur, il vérifie si un enregistrement avec cette valeur existe déjà, puis met à jour si trouvé ou insère sinon.
* **La vue edit utilisait le formulaire multi-sélection quand `bulk_create` était déclaré :** quand `bulk_create` est déclaré sans `edit_form` explicite, le daemon génère désormais automatiquement un `edit_form_builder` utilisant `module::AdminForm` (formulaire standard mono-enregistrement).
* **Les champs uniques apparaissaient dans le formulaire d'édition en masse :** l'édition en masse d'une ressource avec des champs à contrainte UNIQUE pouvait produire une violation UNIQUE. Le générateur émet désormais `UNIQUE_FIELDS` par entité (depuis les contraintes `unique` de `derive_form!{}`). Ces champs sont automatiquement exclus du formulaire d'édition en masse.

### Ajouté — `runique` (middleware)

* **Middleware anti-bot honeypot :** `AntiBot::new("nom_champ")` injecte un champ piège caché dans tous les formulaires du scope protégé. Si le champ est rempli au POST, `form.is_valid()` retourne `false` immédiatement.
* **`RateLimiter` par méthode HTTP :** `rate_limit_get()`, `rate_limit_post()`, `rate_limit_put()`, `rate_limit_delete()` permettent de définir des limites indépendantes par méthode HTTP en plus du `rate_limit()` global.

### Ajouté — `runique` (formulaires)

* **`FormTracing` pour toutes les étapes du pipeline formulaire :** quand `RuniqueLog::forms` est configuré, chaque étape (enregistrement des champs, `set_value`, validation, finalisation, rendu) émet un événement `tracing` structuré au niveau configuré.
* **`cleaned_enum<T>()` sur `RuniqueForm` :** lit la valeur validée d'un champ et tente de la convertir en `ActiveEnum` SeaORM.
* **`add_value()` sur `RuniqueForm` :** force une valeur sur un champ nommé, contournant `fill()`. Utile pour les champs ignorés par le pipeline (ex. hash de mot de passe pré-calculé).

---

## [2.1.6] - 2026-05-23

### Ajouté — `derive_form` (extend)

* **Bloc `extend!{}` dans `derive_form!{}`:** un nouveau bloc `extend { Table { fields: { ... } } }` permet d'ajouter des colonnes personnalisées à des tables framework (ex. `eihwaz_users`) en utilisant le même DSL de champs que `derive_form!{}`. La macro génère la migration `ALTER TABLE`, injecte les colonnes dans l'entité SeaORM existante et produit un `AdminForm` utilisable dans `admin!{}`. Les colonnes de base de la table restent invisibles — seules les extensions déclarées sont exposées.

### Ajouté — `runique` (admin)

* **Tracing structuré dans les opérations CRUD de l'admin et tout le reste du framework :** `handle_create_post` et `handle_edit_post` émettent désormais des événements de log structurés contrôlés par `RuniqueLog::admin.crud`. Les événements couvrent le résultat de la validation du formulaire, la sauvegarde réussie et les erreurs de base de données (violations d'unicité distinguées des autres erreurs).

### Corrigé — `runique` (migrations)

* **`EihwazSessionsMigration::down()` échouait avec "no such table: eihwaz_sessions" :** `AdminTableMigration::down()` supprime déjà `eihwaz_sessions` (avec `.if_exists()`). Lors d'un `migrate reset`, les migrations DOWN s'exécutent en ordre inverse — `AdminTableMigration::down()` s'exécutait en premier, supprimant la table. `EihwazSessionsMigration::down()` tentait ensuite de la supprimer à nouveau sans `.if_exists()` et plantait. Corrigé en ajoutant `.if_exists()` à `EihwazSessionsMigration::down()`.

---

## [2.1.5] - 2026-05-20

### Corrigé — `runique` (formulaires)

* **`parse_constraint_name` extrayait des segments du nom de table comme noms de champ pour les tables multi-mots :** pour une table `changelog_entry`, la contrainte de clé primaire `changelog_entry_pkey` était découpée en `["changelog", "entry", "pkey"]` et la partie centrale `"entry"` était retournée comme nom de champ, produisant une erreur "La valeur du champ 'entry' est déjà utilisée" à chaque INSERT. Les contraintes se terminant par `_pkey` ou `_fkey` retournent désormais `None` immédiatement, laissant les violations de clé primaire et étrangère tomber dans le message d'erreur générique.

### Corrigé — `runique` (admin)

* **Les filtres de la sidebar admin n'étaient pas cumulables :** cliquer une valeur de filtre sur une colonne supprimait silencieusement les filtres actifs des autres colonnes, car chaque lien de filtre n'incluait que son propre paramètre `filter_col=val`. Les liens dans `list_partial.html` itèrent maintenant sur `active_filters` et préservent tous les autres filtres actifs dans l'URL générée, aussi bien pour la sélection d'une valeur que pour le lien de réinitialisation par colonne (✕).

---

## [2.1.4] - 2026-05-20

### Corrigé — `runique` (daemon admin)

* **Le générateur admin émettait `i32`/`i64` hardcodés pour le parsing de PK :** l'approche `detect_big_pk` lisait le `Cargo.toml` du projet pour déterminer le type, mais échouait lors de `cargo clippy --all-features` sur le workspace (l'activation globale des features rendait `Pk = i64` même pour les projets sans `big-pk` dans leur propre `Cargo.toml`). Le générateur émet désormais `parse::<Pk>()` par défaut, qui se résout au bon type à la compilation via l'alias `Pk`. Les surcharges explicites `id_type: I32 | I64 | Uuid` conservent les types concrets.

---

## [2.1.3] - 2026-05-20

### Corrigé — `runique` (uploads de fichiers)

* **`parse_multipart` créait les répertoires d'upload pour toutes les requêtes multipart :** `create_dir_all` était appelé inconditionnellement en tête de `parse_multipart`, provoquant un crash en production sur tout POST de formulaire lorsque `MEDIA_ROOT` n'était pas défini — même pour les formulaires sans champ fichier. Les répertoires d'upload sont désormais créés de façon lazy, uniquement quand une partie fichier effective est détectée.
* **`resolve_media_root()` utilisait le chemin relatif `"media"` comme fallback :** ce chemin relatif rendait le répertoire effectif imprévisible selon le répertoire de travail du processus. La résolution suit maintenant une chaîne de priorité à trois niveaux : variable `MEDIA_ROOT` → `{BASE_DIR}/media` → `{cwd}/media`, ancrant le chemin à la racine du projet dans tous les environnements.

### Corrigé — `runique` (daemon admin)

* **Le générateur admin utilisait `i32` pour tous les PKs quelle que soit la feature `big-pk` :** le daemon émettait toujours `id.parse::<i32>()` dans les handlers générés. Quand un projet active la feature `big-pk` (ce qui fait que `pk: id => Pk` génère `i64`), le `admin.rs` généré ne compilait pas avec des erreurs de type. Le daemon lit désormais le `Cargo.toml` du projet au démarrage — si `big-pk` est présent dans les features, le type id par défaut est `i64` ; sinon `i32`. Un `id_type: I32 | I64 | Uuid` explicite dans `admin!{}` prend toujours la priorité.

### Corrigé — `runique` (makemigrations)

* **Aucune confirmation avant la génération de migrations destructrices :** `makemigrations` générait silencieusement les DROP COLUMN, changements de type, passages nullable→NOT NULL, suppressions de clés étrangères et clés étrangères CASCADE sans avertissement. Une fonction `collect_destructive_messages()` inspecte désormais toutes les modifications en attente et, si certaines sont destructrices, affiche un résumé et demande une confirmation (contournable avec `--force`).

---

## [2.1.2] - 2026-05-17

### Corrigé — `runique` (utilitaires migration)

* **`unique_together` génère `.unique_key()` — méthode introuvable sur `IndexCreateStatement` :** sea-query rc.27+ a renommé `IndexCreateStatement::unique_key()` en `unique()`. L'appel dans `generators.rs` est corrigé ; `.unique_key()` sur `ColumnDef` n'est pas affecté.
* **Syntaxe tuple `Variant = ("db_value", "Display")` ignorée dans les migrations :** `parser_builder.rs` ne gérait que `syn::Lit` directement après `=`. Quand la valeur était un tuple `(...)`, le parsing échouait et revenait au nom de variant Rust (ex. `'Entree'` au lieu de `'entree'`), causant des erreurs de désérialisation SeaORM. Corrigé avec un branchement `parenthesized!` qui extrait la première chaîne du tuple.

### Corrigé — `runique` (préfixe admin)

* **Le middleware admin redirige vers `/` si non authentifié :** redirige maintenant vers `{prefix}/login` en utilisant le préfixe configuré depuis `AdminState`. Les routes sans correspondance passent sans déclencher la redirection.
* **`admin_prefix` absent de tous les contextes de templates admin :** `inject_admin_prefix` n'était pas appelé dans `inject_context` (point d'entrée partagé des handlers), causant `Variable admin_prefix not found` dans les templates. Désormais injecté centralement pour que toutes les vues admin y aient accès.
* **Struct `AdminRoutes` ajoutée :** `admins::routes(prefix)` retourne maintenant `AdminRoutes { router, prefix }` au lieu d'un `axum::Router` nu, permettant à la couche staging de propager automatiquement le préfixe vers `AdminConfig` sans appel séparé à `.prefix()`.
* **`list_filter` dans `configure {}` pour les ressources builtin :** les filtres de sidebar déclarés via `configure { users: { list_filter: [...] } }` étaient ignorés silencieusement — le générateur ne les transmettait pas à `DisplayConfig`. Le générateur inclut désormais la chaîne `list_filter` dans l'appel `configure`, cohérent avec les déclarations au niveau ressource.

### Corrigé — `derive_form` 2.0.3

* **Champs Time/Date/Datetime non sauvegardés dans `partial_update` :** un bras `return None` en tête du match dans `generate_partial_update` écartait silencieusement tous les champs temporels avant d'atteindre les bras chrono corrects ajoutés en 2.0.2 — ces bras étaient du code mort inatteignable. Le bras bloquant est supprimé ; `NaiveTime`, `NaiveDate`, `NaiveDateTime` et `DateTime<Utc>` sont désormais correctement persistés via `admin_partial_update`.
* **Champs `auto_now`/`auto_now_update` absents du `Column` enum et du struct `Model` :** le filtre dans `generate_sea_model` excluait ces champs à la fois de `ActiveModel` et de `Column`, rendant `Entity::Column::CreatedAt` inaccessible pour le tri ou le filtrage. Le filtre est supprimé ; les champs `auto_now` apparaissent désormais dans `Model` et `Column` en `Option<T>` et restent exclus uniquement de `ActiveModel` pour éviter les écrasements manuels.

### Ajouté — `runique` 2.1.2

* **Support CORS :** nouveau `with_cors(|c| c.origin("https://app.example.com").allow_credentials(true))` sur `MiddlewareStaging`. `CorsConfig` accepte `.origin()`, `.any_origin()`, `.allow_credentials()`, `.max_age()`. L'association origine wildcard + `allow_credentials(true)` est rejetée au démarrage avec un `BuildError`.
* **Proxies de confiance :** nouveau middleware `with_trusted_proxies(|t| t.private_networks().proxy("203.0.113.5"))`. Valide les chaînes `X-Forwarded-For` et injecte `ClientIp` dans les extensions des handlers. Par défaut : réseaux RFC 1918 + loopback — couvre nginx sur la même machine et les réseaux Docker sans configuration. `.none()` supprime toute confiance pour les déploiements exposés directement.
* **En-tête `Permissions-Policy` :** nouveau middleware `with_permissions_policy(|p| ...)`. Envoie l'en-tête `Permissions-Policy` ; tous les capteurs, APIs matérielles et paiement sont interdits par défaut. Les directives individuelles peuvent être surchargées via le builder.
* **Protection open redirect :** middleware automatique sur toutes les réponses 3xx. Les en-têtes `Location` pointant vers des origines externes sont bloqués sauf si la destination fait partie des hôtes autorisés configurés. Bloque les redirections involontaires introduites par la logique des handlers.
* **`RuniqueAppBuilder::with_custom_db` :** attache n'importe quelle valeur `Any + Send + Sync + 'static` comme extension Axum, rendant les connexions secondaires (pools Redis, bases alternatives) accessibles dans les handlers via `Extension<T>`.
* **`EihwazSessionsMigration` incluse dans `AdminTableMigration` :** `create_eihwaz_sessions_table()` est maintenant appelée dans `AdminTableMigration::up()` (entre `eihwaz_users_groupes` et `eihwaz_history`). Le `DROP` correspondant est ajouté dans `down()`. Les nouveaux projets n'ont plus besoin d'ajouter cette migration manuellement.
* **`makemigrations` injecte `EihwazSessionsMigration` :** `ensure_admin_migration_positioned()` insère maintenant `Box::new(migrations_table::EihwazSessionsMigration)` entre `EihwazUsersMigration` et `AdminTableMigration` dans le `lib.rs` généré. Le filtre de doublons et `FRAMEWORK_TABLE_PATTERNS` sont mis à jour en conséquence.
* **Login admin — `admin_prefix` injecté dans tous les chemins d'erreur :** `inject_admin_prefix` était absent des quatre chemins d'erreur de `admin_login_post` (CSRF invalide, compte bloqué, erreur session, mauvais identifiants), provoquant une erreur 500 `Variable admin_prefix not found` sur les échecs de connexion. Corrigé dans les quatre chemins.
* **JS bulk admin — checkboxes rebindées après swap HTMX :** `admin-bulk.js` écoute maintenant `htmx:afterSwap` sur `#list-content` et rattache tous les listeners de checkboxes (`#bulk-check-all` et `.bulk-check`). Auparavant, la navigation par pagination et filtres via HTMX recréait les éléments DOM sans listeners, cassant la checkbox "tout sélectionner".
* **Bulk edit admin :** nouveaux handlers `GET /{resource}/bulk_edit` et `POST /{resource}/bulk_edit`. Quand des IDs sont sélectionnés dans la vue liste et que l'action bulk-edit est déclenchée, un formulaire est rendu avec les champs communs éditables. À la soumission, chaque enregistrement est mis à jour indépendamment ; les violations de contrainte unique sont ignorées avec un avertissement plutôt que d'interrompre le lot.
* **Support M2M dans le DSL admin :** `m2m: [["field", "Libellé", "junction_table", "self_fk", "target_fk", "entity::path"]]` dans `admin!{}` génère une closure `M2mLoaderFn`. Dans les formulaires create/edit, tous les choix disponibles sont chargés depuis la table cible et les IDs existants sont pré-sélectionnés depuis la table de jonction. Les valeurs soumises (préfixées `m2m_field__`) sont comparées à l'état courant ; seuls les inserts et suppressions nécessaires sont appliqués.
* **`AdminConfig::extra_routes()` :** `.with_admin(|a| a.extra_routes(vec![("/path", get(handler))]))` attache des routes personnalisées dans le préfixe admin sans nécessiter un `merge()` séparé sur le router.
* **Helpers query/path sur `Request` :** quatre nouvelles méthodes sur `runique::context::Request` :
  * `get_path(key) -> Option<&str>` — paramètre de chemin brut.
  * `get_path_as::<T>(key) -> Option<T>` — paramètre de chemin typé (parsé via `FromStr`).
  * `get_query(key) -> Option<&str>` — paramètre de query string brut (remplace `from_url`).
  * `query::<T>() -> T` — désérialise toute la query string en struct via `serde_qs` ; `raw_query` est désormais stocké sur `Request` à l'extraction.
* **DSL `bulk_create: field` — création multi-enregistrements depuis un seul submit :** quand `bulk_create: field_name` est déclaré sur une ressource dans `admin!{}`, le `create_fn` généré découpe `data[field_name]` par virgule et insère un enregistrement par valeur. Conçu pour les `CheckboxField` multi-sélection (ex. : sélectionner plusieurs jours de la semaine pour créer un enregistrement `horaire` par jour).
* **Résolution FK dans `list_display` — 3ème élément optionnel `"table.colonne"` :** déclarer `["col", "Libellé", "table.colonne"]` dans `list_display` remplace l'id FK brut par un libellé lisible dans la vue liste. Une requête `SELECT CAST(id AS TEXT), colonne FROM table WHERE id IN (...)` s'exécute après le fetch principal et remplace chaque id en place. Compatible `i32`, `i64` et UUID. Les colonnes FK sont automatiquement exclues de la recherche plein-texte.
* **Select FK dans les formulaires create/edit admin :** quand une entrée de `list_display` possède un 3ème élément FK, le `form_builder` généré charge toutes les lignes de la table liée et injecte un `<select>` (via `Forms::field_choices`) pour ce champ, avec la valeur existante pré-sélectionnée en mode édition.
* **`Forms::field_choices` ajouté :** nouvelle méthode sur `Forms` qui remplace un champ par son nom par un `ChoiceField` peuplé depuis un `Vec<(String, String)>` de paires `(valeur, libellé)`. Préserve la valeur courante et le flag required.
* **Pagination de l'historique liée à `AdminConfig::page_size` :** les deux handlers history (`/admin/history` et historique par objet) utilisaient un `PER_PAGE = 50` codé en dur. Ils lisent maintenant `admin.config.page_size`, contrôlé via `.with_admin(|a| a.page_size(N))` dans le builder.
* **`GroupAction::val(field, label, value)` — action groupe à valeur fixe :** nouveau constructeur pour les champs de type enum. La syntaxe DSL à 3 éléments `["field", "Libellé", "value"]` génère `GroupAction::val` au lieu de `GroupAction::bool`, soumettant la valeur exacte (ex. `"valide"`) plutôt que `"true"`/`"false"`.
* **`with_group_actions` fusionne les actions sur le même champ :** plusieurs entrées `GroupAction` ciblant le même champ sont fusionnées en un seul `<select>` regroupant tous les choix. Auparavant, les selects `name="ga_*"` en doublon provoquaient l'écrasement de la valeur sélectionnée par la valeur vide, abandonnant silencieusement la mise à jour.
* **`RuniqueQueryBuilder::order_by_random()` :** trie les résultats par `RANDOM()` sans SQL brut.
* **`RuniqueQueryBuilder::order_by_expr(expr, order)` :** tri par une expression SeaORM `IntoSimpleExpr` arbitraire.
* **`RuniqueQueryBuilder::one()` :** retourne `Result<Option<E::Model>, DbErr>`. Retourne `Err` si plus d'une ligne correspond — analogue au `.get()` de Django. Charge au plus 2 lignes en interne pour détecter le cas ambigu sans scan complet.
* **`Request::headers` :** les en-têtes HTTP (`axum::http::HeaderMap`) sont désormais accessibles sur `Request` dans tous les handlers.
* **`PasswordResetConfig::email_template(path)` :** template Tera personnalisé optionnel pour les emails de réinitialisation de mot de passe ; utilise le template intégré si non défini.
* **Placeholders de traduction unifiés :** tous les fichiers de langue (`fr`, `en`, `de`, `es`, `it`, `ja`, `pt`, `ru`, `zh`) migrés de `{0}`/`{1}`/`{2}` vers `{}` pour correspondre à la convention `format!` de Rust utilisée à l'exécution.

### Ajouté — `derive_form` 2.0.3

* **Macro `extend!{}` — extension des tables framework :** génère une fonction `schema()` que `makemigrations` utilise pour émettre des instructions `ALTER TABLE ADD COLUMN` sur la table framework nommée. Autorisé uniquement sur les tables intégrées (`eihwaz_users`, `eihwaz_groupes`, `eihwaz_droits`, `eihwaz_sessions`, `eihwaz_users_groupes`, `eihwaz_groupes_droits`). Les autres noms sont rejetés à la compilation.
* **Type de champ `phone` :** `phone: phone [required]` dans `model!{}` — stocké en VARCHAR, rendu en `<input type="tel">` dans les formulaires.

---

## [2.1.1] - 2026-05-02

### Corrigé — `derive_form` 2.0.2

* **`fk()` dans les blocs v2 ignoré silencieusement :** `FormFieldAttr::Fk(FkDef)` ajouté dans l'AST, le parser et la propagation vers `FieldOption::Fk`.
* **Attribut `skip` inconnu :** `FormFieldAttr::Skip` ajouté dans l'AST, le parser et le générateur (champ exclu du rendu formulaire).
* **Syntaxe `many_to_many(target).through(via)` cassée :** corrigée en `many_to_many(target, via)` dans `foreignkey.rs`.
* **`sea_query::ForeignKeyAction` introuvable :** re-exporté sous `runique::migration::ForeignKeyAction` ; chemins du générateur mis à jour.
* **Méthode `.references_column()` inexistante :** remplacée par `.to_column()` dans le builder FK.
* **Noms de modèles PascalCase dans les chemins de relations :** `to_snake_case()` utilisé partout à la place de `.to_lowercase()` dans `relation_enum.rs` et `foreignkey.rs` (ex. `super::menuimage` → `super::menu_image`).
* **`rust_decimal::Decimal` introuvable :** type mappé vers `::runique::sea_orm::prelude::Decimal` dans `sea_model.rs`.
* **`via_self` FK → mauvais variant de relation :** suffixe `_id` supprimé et PascalCase appliqué pour dériver le bon nom de variant dans l'impl `Related` de `ManyToMany`.
* **`Decimal` absent de `generate_partial_update` :** `FieldType::Decimal(_)` ajouté au bras numérique.
* **`Decimal` absent de `generate_from_str_map` :** `FieldType::Decimal(_)` ajouté au bras float/decimal.
* **`unique_together` / `indexes` jamais générés en SQL :** `parser_builder.rs` ignorait silencieusement le bloc `meta`. Désormais parsé et converti en entrées `ParsedIndex` (`{table}_{cols}_uniq` pour les contraintes uniques, `idx_{table}_{cols}` pour les index simples).

### Ajouté — `runique` 2.1.1-alpha.3

* **Enum `OrderDir`** ajoutée dans `migration::schema` (`Asc` / `Desc`).
* **Méthodes builder sur `ModelSchema` :** `order_by()`, `unique_together()`, `verbose_name()`, `verbose_name_plural()`.
* **`ForeignKeyAction` re-exporté** depuis `runique::migration`.
* **`RelationDef::as_name()`** méthode no-op ajoutée pour la compatibilité DSL.

---

## [2.1.0] - 2026-04-20

### Rupture

* **`Prisme<T>` supprimé — extraction via `req.form::<T>()` :**
  Les handlers n'acceptent plus `Prisme<MyForm>` comme paramètre. Utiliser `let form = req.form::<MyForm>()`.
  `Request` doit être le **dernier paramètre** de chaque handler (extracteur body-consuming).
  L'extracteur `AdminBody` est supprimé — les handlers admin POST lisent les données via `req.prisme.data`.

### Ajouté

* **`EihwazSessionsMigration` — table de sessions persistantes :**
  `migrations_table::EihwazSessionsMigration` crée la table `eihwaz_sessions`.
  À ajouter dans le vec du `Migrator` après `EihwazUsersMigration`.
  `eihwaz_sessions` est désormais dans `FRAMEWORK_TABLES` et exclue du scan `makemigrations`.

### Corrigé

* **`auth_login` — sessions persistées en base :**
  `auth_login()` passe maintenant un `RuniqueSessionStore` à `login()`, ce qui crée une ligne
  dans `eihwaz_sessions` à la connexion. Les sessions survivent aux redémarrages serveur via le fallback DB.

---
