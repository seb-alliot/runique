🌍 **Languages**: [English](https://github.com/seb-alliot/runique/blob/main/CHANGELOG.md) | [Français](https://github.com/seb-alliot/runique/blob/main/CHANGELOG.fr.md)

# Journal des modifications

Toutes les modifications notables de ce projet sont documentées dans ce fichier.

---

## [2.1.18] - 2026-06-17

### Correctif / Fonctionnalité — `runique` (scripts de formulaire : accesseur `{% form.x.js %}` + fin des tags Tera littéraux)

* **Les `<script>` auto-injectés par un formulaire partaient en texte brut en rendu champ par champ :** quand un formulaire déclarant des `js_files` (via `add_js`) était rendu champ par champ (`{% form.x.field %}`), le filtre `form` fabriquait la balise script à la main avec des tags Tera **littéraux** — `<script {% csp %} src="{% static "…" %}" defer>` — et l'injectait après le « dernier champ » deviné par index. Deux problèmes. (1) Le préprocesseur de template (`process_content`) qui convertit `{% csp %}` / `{% static %}` ne tourne qu'au **chargement** des fichiers, jamais sur une chaîne produite au runtime par un filtre ; et la sortie du filtre `form` est `| safe`, donc Tera ne la re-parse pas : le `<script>` partait avec `src="{% static "…" %}"` en littéral (jamais chargé) et sans nonce CSP réel (bloqué par la CSP). (2) L'ancrage « après le dernier champ » était fragile : un template au layout custom qui n'écrit pas le champ au plus haut index n'obtenait jamais ses scripts, silencieusement. Le rendu full-form (`{% form.x %}`, utilisé par l'admin via `form_fields.html`) n'était pas touché : il passe par `renderer::render_js` qui rend correctement le template `js.html` (vrai `{{ csp_nonce }}`, vrai `{{ … | static }}`).

* **Correctif :** suppression du second générateur de balises et de l'injection devinée. Le js d'un formulaire est désormais rendu **une seule fois** par `render_js` (single source = `js.html`), exposé en deux points cohérents : en **full-form** il est émis en dernière position du `html` (après les champs qu'il pilote) ; en **champ par champ** un nouvel accesseur explicite **`{% form.x.js %}`** (façon `{{ form.media }}` de Django) émet le même bloc — nonce CSP réel et URL statique résolue, quel que soit le nombre de fichiers `add_js`. Plus de tags Tera morts, plus d'échec silencieux : le dev place `{% form.x.js %}` là où il veut quand il rend les champs un par un. Honeypot et CSRF gardent leurs ancrages (premier / dernier champ) inchangés.

---

## [2.1.17] - 2026-06-15

### Correctif — `runique` (CLI `makemigrations` — defaults des colonnes enum)

* **`[default: …]` était toujours perdu sur les colonnes enum :** le correctif de 2.1.15 faisait émettre `.default(<valeur>)` à `render_column_def`, mais uniquement sur la branche non-enum — la branche `ColumnType::Enum` était rendue avec `{null}{uniq}` sans jamais ajouter `{default}`. Une colonne `choice [enum(X)]` avec un `[default: "Y"]` perdait donc son default, et une colonne enum `required` (NOT NULL) ajoutée à une table peuplée échouait à la migration avec `column "…" contains null values`. Correctif : la branche enum de `render_column_def` ajoute désormais `{default}` comme toute autre colonne, de sorte qu'un `ADD COLUMN <enum> NOT NULL DEFAULT '<variante>'` laisse Postgres backfiller les lignes existantes au lieu d'échouer. L'émission est indépendante du moteur (Postgres/MySQL/SQLite) ; le gating Postgres-only du `CREATE TYPE` est inchangé.

* **3 tests pipeline ajoutés** (`tests_pipeline.rs`, désormais 14) : le default enum atteint le `ParsedColumn` parsé, est émis en CREATE sur les trois moteurs, et est émis sur la passe `extend!{}` ADD COLUMN sur les trois moteurs (avec le `CREATE TYPE` toujours Postgres-only).

### Correctif — `runique` (affichage admin des champs rich-text + deux filtres template)

* **Les valeurs de champ rich-text étaient doublement échappées dans les vues admin detail/list :** un champ `richtext` est sanitizé à l'écriture par ammonia, qui normalise un `>` isolé dans le texte vers l'entité `&gt;` (HTML valide). Les templates detail et list rendaient la valeur stockée via `{{ value }}` (auto-échappé), donc le `&gt;` déjà encodé était ré-échappé en `&amp;gt;` et affiché littéralement `&gt;` à l'écran. Corrigé sans affaiblir la sécurité, par sanitization **côté sortie** : l'admin connaît désormais les colonnes rich (`RICH_CONTENT_FIELDS`, la même classification qu'à l'écriture, injectée comme `rich_fields`) et les rend via de nouveaux filtres au lieu de faire confiance au stockage.

* **Nouveau filtre `| sanitize` (HTML rich, sanitizé à la sortie) :** relance `sanitize_rich` (ammonia) sur la valeur **au moment du rendu**, et le préprocesseur de template force `| safe` sur chaque `| sanitize` (comme `| markdown`). Le `| safe` n'émet donc jamais que l'output ammonia sans vecteur XSS, re-nettoyé quel que soit le chemin par lequel la valeur a atteint la base — sanitize-on-output, pas trust-on-input. Utilisé par la vue detail admin pour rendre les champs rich en vrai HTML.

* **Nouveau filtre `| plaintext` (aperçu texte) :** projette une valeur en texte brut via `sanitize_strict` — strip de tous les tags et décodage des entités, donc un `&gt;` stocké redevient un vrai `>` que Tera échappe ensuite une seule fois. Aucun `| safe` forcé (l'output est du texte brut et reste auto-échappé). Utilisé par les cellules de liste admin, où rendre du HTML rich bloc casserait la mise en page tronquée sur une ligne.

### Correctif — `runique` (champs texte des formulaires : mutation destructive en saisie)

* **Les champs texte supprimaient silencieusement les `<`, `>`, `&` légitimes :** `TextField::set_value` exécutait `value.replace(['<', '>', '&'], "")` dès que l'entrée contenait `<` ou `>`, donc tout champ non-rich stockait une donnée mutilée — `one bug => fix` devenait `one bug = fix`, `R&D` devenait `RD`, `a < b` devenait `a  b`. Le remplacement court-circuitait aussi entièrement `sanitize_strict`. Correctif : les champs non-rich passent désormais par `sanitize_strict` seul, qui retire déjà toutes les balises (scripts compris) via ammonia et décode les entités, en laissant la ponctuation légitime intacte. La protection XSS est inchangée — elle n'a jamais reposé sur cette mutation d'entrée mais sur l'auto-échappement à la sortie (et le strip de balises de `sanitize_strict`). Deux tests ajoutés dans `sanitizer.rs` : `=>` / `&` légitimes préservés, `<script>` autour de ponctuation toujours retiré.

### Correctif — `runique` (backup DB des sessions : sessions authentifiées perdues au restart)

* **Le backup DB des sessions figeait son expiration et déconnectait au restart :** le `CleaningMemoryStore` est mémoire-first avec un fallback DB (`eihwaz_sessions`) qui permet aux sessions authentifiées de survivre à un redémarrage serveur. `save()` persistait via `update_session_data`, qui n'écrivait que `session_data` et **ne rafraîchissait jamais `expires_at`**. L'expiration de la ligne restait figée à ce que `create()` avait écrit en premier — et au login, c'était encore la fenêtre d'inactivité **anonyme** de 5 minutes, car le middleware d'upgrade de TTL ne promeut la session à la durée authentifiée qu'à partir de la requête *suivante*. Résultat : quelques minutes après le login, la ligne DB paraissait expirée (`find_by_cookie_id` filtre `expires_at > now`), donc tout restart déconnectait l'utilisateur, sans aucune erreur pour l'expliquer. Corrigé sur trois fronts : (1) un unique `persist_to_db(record)` est désormais le seul chemin d'écriture DB pour `create()` et `save()` — il écrit le snapshot **complet** (cookie_id, user_id, expiry, data) via `upsert_session`, rendant un backup partiel/périmé impossible par construction (l'`update_session_data` devenu inutile a été supprimé) ; (2) `login()` promeut le TTL à la durée authentifiée dès la requête de login, de sorte que la première ligne persistée porte déjà la longue expiration ; (3) la lecture DB de `load()` et l'écriture du backup **logguent** désormais leurs erreurs au lieu de les avaler (`if let Ok(...)` / `.ok()`), pour qu'un futur problème de schéma/connexion soit visible au lieu de tuer le fallback en silence. Trois tests de non-régression ajoutés dans `tests/middleware/test_session_db.rs`, dont un second `upsert_session` qui doit faire avancer `expires_at` au lieu de le laisser figé.

### Fonctionnalité — `runique` (tracing : sorties fichier/JSON, sinks custom, subscriber externe)

* **Le tracing par module peut désormais écrire dans des fichiers et des destinations custom, plus seulement la console.** `RuniqueLog` gagne un `.output(LogOutput)` répétable : `LogOutput::stdout()` (console couleurs), `LogOutput::file("logs/app.json")` (fichier roulant non bloquant — format déduit de l'extension, JSON pour `.json` sinon texte brut ; rotation `Daily`/`Hourly`/`Never`), et `LogOutput::sink(impl LogSink)` pour une destination fournie par le développeur (base de données, collecteur HTTP, file) qui reçoit un `LogRecord` propre à Runique sans qu'aucun type `tracing` ne fuite dans l'API publique. `RUNIQUE_LOG_FILE` ajoute une sortie fichier au runtime sans recompiler. Les writers fichier sont non bloquants ; leurs `WorkerGuard` vivent dans `RuniqueApp` et sont vidés à l'extinction. Aucun sink base de données intégré, volontairement — `LogSink` est la porte pour en brancher un.

* **`.with_log(|l| l.external())` laisse l'application posséder l'unique subscriber global.** Un processus ne peut installer qu'un seul subscriber `tracing` global ; quand l'app installe le sien (stack de layers custom, OpenTelemetry…), `external()` fait que Runique n'installe rien tout en continuant d'émettre ses événements vers la façade `tracing`, de sorte que le subscriber de l'app les reçoit (filtrer le target `runique` pour les écarter). Aucune feature Cargo, aucun changement cassant.

### Fonctionnalité / Sécurité — `runique` (reset de mot de passe : tokens persistés en DB, hashés, durcis IDOR)

* **Les tokens de reset sont persistés en base, survivent à un redémarrage et fonctionnent en multi-instance.** Le flux forgot/reset stockait ses tokens en mémoire processus (`LazyLock<Mutex<HashMap>>`), donc tout redéploiement entre l'envoi de l'email et le clic tuait le lien, et le flux cassait en montée en charge horizontale. Les tokens vivent désormais dans une nouvelle table framework `eihwaz_reset_tokens` (`EihwazResetTokensMigration`, auto-injectée dans le `Migrator` du projet par la CLI `makemigrations` comme la table des sessions, et exclue du scan des modèles utilisateur). Durcissement : la table stocke le **hash SHA-256** du token, jamais le token brut, donc une fuite en lecture de la DB ne peut être rejouée ; le single-use est atomique (la ligne est supprimée — seul le delete qui affecte une ligne gagne) ; demander un nouveau token supprime les précédents de l'utilisateur. La durée de vie est configurable via `PasswordResetConfig::token_ttl(Duration)` (défaut 1 heure). `encrypt_email`/`decrypt_email` (email jamais en clair dans l'URL) sont inchangés.

* **Durcissement IDOR de la mutation de reset.** La mise à jour du mot de passe porte désormais sur le `user_id` du token (secret, résolu côté serveur) au lieu du champ email venant de l'URL/du formulaire. Un nouveau `UserEntity::update_password_by_id` (impl par défaut, non cassant ; surchargé par l'utilisateur intégré pour une seule requête) effectue la mise à jour ; l'email de l'URL n'est plus qu'un cross-check UX contre l'utilisateur lié au token.

### Correctif — `runique` (tracing : les sites critiques ne sont plus muets par défaut)

* **Une poignée de sites avalant un `Result` qui casse une garantie loggent désormais en `WARN` même quand leur catégorie de tracing est désactivée.** Un nouveau `TraceResult::trace_or(level, default, ctx)` pose un plancher de niveau : rotation d'ID de session (anti-fixation), invalidation du login exclusif, persistance de la session au login, envoi de l'email de reset et échec de rendu d'un template d'erreur ne sont plus muets dans une config de production par défaut. Le nœud de tracing `errors` (`http`/`render`) est désormais câblé sur le middleware d'erreur (les erreurs HTTP gérées sont gatées ; les 500 critiques restent en `error!` direct). Des tests d'émission via `tracing-test` valident que le `file:line` de l'erreur avalée est capturé.

### Correctif — `runique` (pages d'erreur : responsive + i18n)

* **La page d'erreur debug est désormais responsive et les messages opérateur sont localisés.** La page de diagnostic (stack traces, source de template, table des headers) n'avait aucune media query et une largeur fixe à 80 %, débordant sur mobile/tablette pour un contenu qui varie beaucoup selon l'erreur : ajout de sécurité anti-débordement sur les valeurs longues, breakpoints (992/768/600/480) et table des headers en scroll horizontal sur petits écrans. Les pages de prod 404/429/500 gagnent `overflow-wrap` (un message long/custom ne déborde plus) et les trois templates sont maintenant cohérents et 100 % pilotés par l'i18n (les lignes de statut en anglais en dur ont été retirées). Les bannières de démarrage, les warnings du subscriber et le message « feature ACME absente » passent désormais par `t()`/`tf()` (clés ajoutées dans les 9 langues) ; les tags de contexte `tracing` internes restent en anglais fixe pour l'agrégation/grep des logs.

---

## [2.1.16] - 2026-06-15

### Maintenance — `runique` (dépendances, toolchain)

* **SeaORM passé à `2.0.0-rc.40` et MSRV relevée à Rust 1.94 :** mise à jour de `sea-orm` / `sea-orm-migration` de `rc.38` vers le `=2.0.0-rc.40` épinglé. La nouvelle release candidate relève sa version minimale de Rust supportée, le `rust-version` du workspace passe donc à `1.94` en conséquence.

---

## [2.1.15] - 2026-06-13

### Fonctionnalité — `runique` (routing, templates)

* **URLs nommées pour les routes intégrées (`forgot_password`, `reset_password`, `admin`) :** les routes enregistrées par le framework (reset de mot de passe et panel admin) étaient montées directement via `Router::route()` d'Axum sans être enregistrées dans le registre de noms d'URL. Elles ne pouvaient pas être référencées via `{% link %}` dans les templates. Correctif : `build.rs` appelle désormais `register_name_url` après le montage des routes de reset (`"forgot_password"` → `forgot_route` configuré, `"reset_password"` → `reset_route` configuré avec les placeholders `{token}/{encrypted_email}`) et après le montage du panel admin (`"admin"` → le préfixe configuré). L'enregistrement prend en compte les routes personnalisées définies par le développeur via `PasswordResetConfig::forgot_route()` / `.reset_route()` / `AdminConfig::prefix()`.

### Sécurité — `runique` (auth)

* **Énumération d'utilisateurs par timing attack sur le login (moyenne) :** `authenticate_user` et `DefaultAdminAuth::authenticate` retournaient `None` immédiatement via `?` quand le nom d'utilisateur n'était pas trouvé en base, court-circuitant entièrement la vérification du hash de mot de passe. Un attaquant mesurant les temps de réponse pouvait distinguer "l'utilisateur n'existe pas" (rapide — pas de travail de hash) de "mauvais mot de passe" (lent — vérification Argon2 complète), permettant une énumération silencieuse des noms d'utilisateur. Correctif : les deux fonctions appellent désormais `verify()` avant tout court-circuit. Quand l'utilisateur n'est pas trouvé, le mot de passe est vérifié contre un hash dummy Argon2 pré-calculé (`DUMMY_HASH`, initialisé une fois au premier appel via `LazyLock`) — consommant le même temps CPU que l'utilisateur existe ou non. Le `?` sur la recherche utilisateur est différé après le retour de `verify()`, de sorte que le résultat est toujours ignoré une fois le travail sensible au timing effectué.

### Sécurité — `runique` (admin)

* **Contrôle d'accès manquant sur l'action admin `reset-password` (moyenne) :** dans `admin_post_id` (`admin/admin_main/mod.rs`), les actions `edit` et `delete` étaient protégées par une vérification de permission (`can_update` / `can_delete`, avec repli `_own` + propriété de l'enregistrement), mais l'action `reset-password` n'avait aucune garde d'autorisation — et, contrairement à `admin_get_id`, ce handler ne vérifie pas non plus `can_access`. Tout utilisateur authentifié au panel admin (`is_staff` ou `is_superuser`) avec **zéro permission** sur la ressource pouvait faire `POST {prefix}/{resource}/{id}/reset-password` pour déclencher une réinitialisation de mot de passe sur n'importe quel enregistrement, comptes superuser inclus. Le CSRF et l'authentification étaient vérifiés, mais pas l'autorisation. Facteur aggravant : quand aucun mailer n'est configuré, le lien de reset (token + email chiffré) est renvoyé à l'admin appelant dans un flash message — un staff à faibles privilèges pouvait ainsi obtenir un lien de reset valide pour un compte plus privilégié. Correctif : `reset-password` exige désormais `can_update` (global, ou `can_update_own` sur un enregistrement possédé), aligné sur `edit`.

### Sécurité — `runique` (générateur admin — durcissement SQL)

* **Échappement SQL manuel dans la résolution des labels de clé étrangère :** le générateur admin (`admin/daemon/generator.rs`) construisait les requêtes de résolution de label FK (`list_fn` et `get_fn`) en concaténant une clause `IN (...)` via `Expr::cust(format!("CAST(id AS TEXT) IN ({})", ids_csv))`, où `ids_csv` était assemblé avec `format!("'{}'", s.replace('\'', "''"))`. N'échapper que les apostrophes est insuffisant sur MySQL/MariaDB, où le backslash est un caractère d'échappement par défaut — une valeur contenant un backslash pouvait sortir du littéral. En pratique les valeurs sont des ids FK stockés en base (entiers), donc l'exploitabilité était négligeable, mais c'était de l'échappement manuel là où le reste du générateur utilise déjà des identifiants en liste blanche et des valeurs liées. Correctif : les deux requêtes utilisent désormais des valeurs liées via l'API typée de sea-query — `Expr::cust("CAST(id AS TEXT)").is_in(fk_ids.clone())` dans `list_fn` et `.eq(fk_key.clone())` dans `get_fn`. Plus aucune donnée n'est interpolée dans la chaîne SQL ; le placeholder est correct selon le backend sur PostgreSQL/MySQL/SQLite.

### Correctif — `runique` (CLI `makemigrations` — refactor pour un usage fiable)

* **Les valeurs `[default: …]` de colonne étaient parsées mais jamais émises :** `ParsedColumn` ne portait qu'un flag `has_default_now` (pour `CURRENT_TIMESTAMP`) ; la valeur littérale de `[default: 0]` / `[default: true]` / `[default: "x"]` était consommée par le parser puis jetée, donc aucun `.default(...)` n'atteignait le SQL généré — seuls les timestamps auto avaient un default. Pire cas : un `bool [default: true]` rendu `NOT NULL` *sans* default, un `ADD COLUMN` qui échoue sur une table peuplée. Correctif : `ParsedColumn` gagne un `default_value: Option<String>` ; `parser_builder` et `parser_extend` capturent désormais le littéral, `render_column_def` émet `.default(<valeur>)` (CREATE, snapshot et ALTER). Le round-trip snapshot est préservé — `parser_seaorm` ne traite que `.default(Expr::current_timestamp())` comme default timestamp et relit les defaults littéraux dans `default_value`.

* **`bool` était not-null par défaut (incohérent avec les autres scalaires) :** `bool`/`boolean` manquaient dans la liste des types sémantiques v2, donc un champ booléen sans `required` était généré `NOT NULL` (comportement v1) alors que `int`, `text`, etc. étaient nullable. Combiné au default perdu ci-dessus, un `bool [default: true]` produisait un `ADD COLUMN NOT NULL` incompilable. Correctif : `bool`/`boolean` ajoutés aux types v2 — nullable sauf `required`, cohérent avec les autres scalaires.

* **Les clés étrangères CASCADE sur des tables neuves étaient signalées comme destructives :** le garde destructif signalait `ADD FOREIGN KEY … ON DELETE CASCADE (existing rows may be deleted)` pour chaque FK CASCADE, y compris sur des tables créées dans le même batch — qui n'ont aucune ligne à perdre. Une génération initiale / from-scratch de tout schéma comportant des FK CASCADE exigeait à tort `--force`. Correctif : le contrôle CASCADE ignore désormais les tables neuves (`is_new_table`), seul le risque existant sur une table peuplée étant réel.

* **Suite de tests pure ajoutée :** `migration/utils/tests_pipeline.rs` — 11 tests sans dépendance (ni DB ni Docker) couvrant le parsing & l'émission des defaults, la nullabilité des bool, l'invariant de stabilité du round-trip snapshot (« générer deux fois = aucun changement »), le garde destructif (CASCADE neuve vs existante, DROP COLUMN), les ajouts de valeurs d'enum, et le gating Postgres-only des `CREATE TYPE`/`DROP TYPE`.

* **Le rollback supprimait les snapshots préexistants au lieu de les restaurer :** en cas d'échec d'écriture en cours de batch, le rollback faisait `fs::remove_file` sur tous les fichiers déjà écrits, y compris `snapshots/{table}.rs`. Pour un ALTER sur une table existante, le snapshot existait déjà et était *écrasé* pendant le run ; le supprimer perdait donc le contenu précédent — le `makemigrations` suivant ne voyait plus de snapshot et régénérait un `CREATE TABLE` complet pour une table déjà migrée. Correctif : le contenu de chaque cible existante est sauvegardé en mémoire avant écriture (comme le backup `lib.rs` déjà présent) ; au rollback, chaque fichier est *restauré* s'il avait un backup, sinon supprimé.

* **Le garde destructif ne couvrait pas les blocs `extend!{}` :** `check_destructive` ne s'exécutait que sur les changements des modèles principaux ; la passe extend calculait ses propres diffs et générait directement les fichiers ALTER, sans contrôle destructif et en ignorant `--force`. Un DROP COLUMN / changement de type / nullable→required / FK supprimée ou CASCADE introduit via un bloc `extend!{}` était émis silencieusement. Correctif : les changements extend sont désormais inclus dans un garde destructif unique qui respecte le même flag `--force` que la passe principale.

* **Génération non atomique entre les passes (refactor plan → validation → commit) :** la commande écrivait en trois passes indépendantes (modèles principaux, puis `extend!{}`, puis positionnement de `AdminTableMigration`), chacune committant pour son compte. Un échec dans une passe ultérieure laissait les précédentes committées. Réécrit en un flux unique : les changements principaux et extend sont d'abord planifiés en mémoire (`Plan { files, dirs, lib_modules }`), validés par un seul garde destructif, puis committés atomiquement par un unique `commit_plan` — création des dossiers, backups des cibles, écriture des fichiers, enregistrement dans `lib.rs` et positionnement de la migration admin s'exécutent tous sous un rollback unique. Effet de bord : un projet ne contenant *que* des blocs `extend!{}` (sans modèle propre) génère désormais correctement ses migrations — auparavant un `return` anticipé sur l'absence de changement de modèle sautait entièrement la passe extend.

### Nouveauté — `runique` / `derive_form` (DSL `model!` / `extend!`)

* **Enums dans `extend!{}` :** `extend!{}` accepte désormais un bloc `enums: { … }` optionnel (entre `table:` et `fields:`), identique à `model!`. Une colonne `choice [enum(NomEnum)]` génère le vrai type Rust enum (`DeriveActiveEnum`), mappe le champ de l'entité dessus (au lieu de `String`), le parse dans `admin_from_form` / `admin_partial_update` via `FromStr`, et peuple le `ChoiceField` admin avec les variantes. `makemigrations` émet la colonne (PostgreSQL : `CREATE TYPE … AS ENUM` ; autres moteurs : `VARCHAR`/`ENUM` natif). La fonction partagée `generate_enums` a été factorisée en `generate_enum_defs(&[EnumDef])`, réutilisée par les deux macros. Dégradation propre : `choice [enum(X)]` sans bloc `enums:` déclaré conserve l'ancien comportement `String`.

* **Renommage de colonne via `[renamed_from: "ancien"]` :** renommer un champ produisait auparavant un `DROP COLUMN` + `ADD COLUMN` — perte de données silencieuse. Le nouvel indice explicite fait émettre à `makemigrations` un `ALTER TABLE … RENAME COLUMN ancien TO nouveau` (portable PostgreSQL, MySQL/MariaDB et SQLite), sans perte de données. C'est une directive de migration uniquement (aucun effet sur l'entité/formulaire générés), tolérée par les deux parseurs de champ DSL (`FormFieldDecl` pour v2/`extend!`, `FieldDef` pour v1). Garde-fou contre un indice périmé : si l'ancienne colonne existe toujours dans le snapshot, aucun rename n'est émis (repli en ajout). `ParsedColumn.renamed_from` est transient et n'est jamais écrit dans les snapshots.

### Correctif — `runique` (CLI `makemigrations` — justesse cross-moteur)

* **Le renommage de valeur d'enum produisait du SQL cassé sur Postgres :** un changement positionnel de valeur d'enum générait un `UPDATE` de données *avant* `ALTER TYPE … ADD VALUE`, plus un avertissement « valeur supprimée » trompeur — non exécutable sur un enum natif PG (une valeur ne peut pas être écrite avant d'exister, et une valeur fraîchement ajoutée ne peut pas être utilisée dans la même transaction). Un rename est désormais modélisé comme **une seule** opération : sur Postgres il émet `ALTER TYPE … RENAME VALUE 'ancien' TO 'nouveau'` (atomique, met à jour les lignes existantes) ; sur les moteurs à base de VARCHAR il émet un simple `UPDATE`. La paire renommée est exclue des listes ajout/suppression. Le DDL d'ajout/suppression de valeur d'enum est aussi désormais gardé Postgres uniquement (auparavant `ALTER TYPE` était émis aussi sur MySQL/SQLite — SQL invalide).

* **Les clés étrangères cassaient `migrate` sur SQLite :** toutes les contraintes FK étaient regroupées dans une migration `create_relations` séparée appliquée via `ALTER TABLE … ADD CONSTRAINT`. PostgreSQL et MySQL/MariaDB l'acceptent, mais SQLite ne sait pas ajouter de FK à une table existante — `migrate` paniquait. Correctif : pour SQLite (`DbKind::Other`), les FK sont désormais déclarées **inline dans le `CREATE TABLE`** (`build_create_table_cols` gagne un flag `inline_fks`) et la migration `create_relations` est sautée. PostgreSQL/MariaDB inchangés ; le snapshot conserve les FK en instructions séparées (round-trip stable). Validé end-to-end (`fresh` + `reset` complet) sur les trois moteurs via un script smoke multi-moteur.

* **Suite de tests pipeline étendue (11 → 63) :** `migration/utils/tests_pipeline.rs` couvre désormais les enums dans `extend!`, le renommage de valeur d'enum par moteur, le renommage de colonne (indice, portabilité, indice périmé, `extend!`), les types de clé primaire (i32/i64/uuid), les timestamps par moteur, les actions FK, les enums backés `i32`, la matrice destructive complète, ALTER add/drop/modify colonne + FK + index, le mapping des types sémantiques, `Changes::is_empty`, le diff no-op, les colonnes ignorées (`readonly`), l'ordre topologique des FK, et le FK inline sur SQLite. Tous sans dépendance (ni DB ni Docker).

### Correctif — `runique` (formulaires, admin)

* **Scripts `add_js()` bloqués par la CSP `strict-dynamic` dans les formulaires admin :** `render_js()` construisait son propre contexte Tera isolé contenant uniquement `js_files`, sans accès au `csp_nonce` scopé à la requête. Avec `strict-dynamic`, tout tag `<script>` sans le nonce correspondant est bloqué par le navigateur. Correctif : `FormRenderer` gagne un champ `csp_nonce: Option<String>` et une méthode `set_nonce()` ; `Forms` expose `set_csp_nonce()` ; les handlers CRUD admin (`handle_crud.rs`) injectent le nonce depuis le contexte de requête dans le renderer du formulaire avant sérialisation, via un nouveau helper `inject_csp_nonce()` appelé à chaque point `context.insert(FORM_FIELDS, …)` (create GET/POST, edit GET/POST).

* **`TextField` double-encodait `&` lors de la sauvegarde d'un texte brut :** `sanitize_strict()` utilisait `ammonia::Builder::new().tags(HashSet::new()).clean(input)` pour supprimer toutes les balises HTML. C'est correct, mais ammonia encode toujours `&` → `&amp;` dans sa sortie, même quand aucune balise ne subsiste — effet de bord de son sérialiseur d'entités HTML. Le résultat brut était ensuite stocké en base. Lorsque Tera le rendait ensuite avec l'autoescaping actif, le `&` de `&amp;` était ré-encodé en `&amp;amp;`, ce qui faisait afficher le texte littéral `&amp;` au lieu de `&`. Correctif : après que ammonia a supprimé les balises, le résultat est décodé en texte brut via `html_escape::decode_html_entities()` — supprimant l'encodage d'entités parasite avant stockage. Le filtrage des protocoles dangereux (`javascript:`, `vbscript:`, `data:`, `file:`) s'applique sur la chaîne décodée, préservant les garanties de sécurité. Nouvelle dépendance : `html-escape = "0.2"`.

---

## [2.1.14] - 2026-06-06

### Correctif — `runique` (filtres & recherche admin — régression PostgreSQL)

* **Placeholder `?` dans `cust_with_values` cassait tous les filtres et la recherche admin sur PostgreSQL (régression introduite par le correctif injection SQL de 2.1.12) :** pour lier les valeurs proprement, 2.1.12 avait remplacé l'interpolation inline par `Expr::cust_with_values(format!("CAST(col AS TEXT) = ?"), [val])`. Le marqueur `?` n'est le placeholder de liaison que sur MySQL/SQLite — PostgreSQL utilise `$N`. Dans le rendu `CustomWithExpr` de sea-query, le placeholder est comparé au marqueur du backend (`$` sur Postgres) ; un `?` non reconnu est donc écrit littéralement et la valeur n'est jamais liée. La condition s'effondrait en `WHERE CAST(col AS TEXT) = ` immédiatement suivi de ` LIMIT`, produisant une erreur 500 `syntax error at or near "LIMIT"` sur toute liste filtrée ou recherche. Totalement silencieux sur MySQL/SQLite. Correctif : toutes les conditions SQL brutes sont reconstruites avec des expressions typées sea-query qui émettent automatiquement le bon placeholder selon le backend — égalité via `Expr::col(Alias::new(col)).cast_as(Alias::new("TEXT")).eq(val)`, recherche insensible à la casse via `Expr::expr(Func::lower(Expr::col(...).cast_as(Alias::new("TEXT")))).like(pattern)` avec le motif passé en minuscules côté Rust. Appliqué aux filtres de colonne du `list_fn` généré, à la macro `search_cond!` (branches `all_columns` et `or(...)`), à la requête d'upsert bulk et à la requête d'options m2m du générateur admin, et aux ressources builtin utilisateur / groupe / droit. La macro `search!` n'était pas concernée — elle construit déjà ses conditions via les méthodes typées `ColumnTrait`.

* **Valeurs de la sidebar filtres triées en Rust, décroissant :** la requête `DISTINCT` ne porte plus le `ORDER BY CAST(col AS TEXT) DESC` introduit en 2.1.13 (il était imbriqué avec le chemin SQL brut cassé). Les valeurs distinctes sont désormais triées dans la closure générée (`sort_by` — numérique quand les deux côtés se parsent en entiers, lexicographique sinon), décroissant — version / id le plus élevé et valeur la plus récente en premier dans chaque dropdown. Le tri de la liste principale est inchangé (toujours piloté par les en-têtes de colonne cliquables).

---

## [2.1.13] - 2026-06-01

### Correctif — `runique` (templates admin)

* **Liens de tri admin effaçaient les filtres actifs :** cliquer un en-tête de colonne pour trier réinitialisait tous les filtres `list_filter` actifs. Les liens de tri dans `list_partial.html` n'incluaient pas `{{ filter_qs }}`, contrairement aux liens de pagination. Correctif : `filter_qs` est désormais ajouté dans le `href` et le `hx-get` des en-têtes de colonne `id` et des colonnes dynamiques.

* **Valeurs de la sidebar filtres admin dans le mauvais ordre :** les closures `filter_fn` générées récupéraient les valeurs distinctes puis les retriaient en Rust avec `sort_by` (numérique puis lexicographique croissant), écrasant tout ordre DB. Correctif : le `sort_by` en mémoire est supprimé ; la requête `DISTINCT` utilise désormais `ORDER BY CAST(col AS TEXT) DESC` — les valeurs arrivent pré-triées depuis la base, les dates affichent les plus récentes en premier, les chaînes les plus élevées alphabétiquement en premier.

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
