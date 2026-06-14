# Architecture du tracing Runique — plan de consolidation

But : rendre le tracing **solide, cohérent et exploitable** sans le réécrire. L'infra
existe et est bonne ; ce document fixe les règles à suivre et liste le chantier de
couverture restant.

---

## 1. État actuel (verdict)

Deux piliers déjà en place — **ne pas les remplacer** :

- **Runtime** : `runique_log!` + `tracing`, ~149 sites, config par catégories
  ([runique/src/utils/config/runique_log.rs](runique/src/utils/config/runique_log.rs)),
  helper `dev()`, pilotage par `RUST_LOG`/`EnvFilter`.
- **Compile-time** : la crate `derive_form` utilise 27 `syn::Error`/`new_spanned`/
  `compile_error!` pour 2 `unwrap` → les erreurs de macro pointent déjà la bonne span.

Le problème n'est **pas** un manque d'infra, et surtout **pas** un manque d'`eprintln!`.
C'est un problème de **couverture inégale** : certaines zones (admin CRUD, auth, validator)
sont exemplaires, d'autres (pipeline `forms/`) avalent les erreurs en silence.

### Interdits

- ❌ `eprintln!` / `println!` pour du log applicatif — court-circuite `RUST_LOG`, les
  niveaux par catégorie et `dev()`, et pollue la prod. Utiliser `runique_log!`.
- ❌ Jeter un `Result` faillible (`.ok()`, `unwrap_or_default()`, `unwrap_or(None)`,
  `let _ = …await`) **sans** émettre au préalable un event avec l'erreur.
- ❌ Logger un secret : mot de passe, hash, token de session/reset, valeur de champ
  `password`/`FileField` sensible. Voir §6.

---

## 1.bis Structure cible de `RuniqueLog` (à atteindre)

L'actuel `RuniqueLog` est un hybride : certains sous-systèmes sont décomposés en
sous-structs (`forms`, `admin`, `auth`, `mailer`, `builder`) mais tout le volet
middleware/sécurité est **aplati à la racine** (`csrf`, `host_validation`, `rate_limit`,
`acme`, `exclusive_login`, `filter_fn`, `roles`, `password_init`, `session`, `db`).
De plus, **`csp` n'a aucune catégorie** — le middleware CSP ne loggue rien.

Cible : **refonte propre** — un arbre qui calque 1:1 la carte des modules `runique`,
une sous-struct par module, rien à plat. L'arbre absorbe aussi les **21 events
`tracing::` directs** restants (db, app/builder, acme, admin/daemon, makemigrations) qui
échappent aujourd'hui à la config (niveau figé, non désactivables).

```
log (RuniqueLog)
├── subscriber_level
├── forms       → field · set_value · validate · finalize · render
├── middleware  → csrf · csp · cors · rate_limit · host_validation · open_redirect · anti_bot · https
├── session     → store · cleanup · exclusive_login
├── auth        → login · reset · password_init
├── admin       → crud · list · bulk · auth · filter_fn · roles · daemon
├── db          → connect · query
├── mailer      → send
├── migration   → plan · apply · rollback        (makemigrations — aujourd'hui tracing direct)
├── templates   → load · render                  (Tera)
├── errors      → http · render                  (error_handler_middleware — aujourd'hui info!/error! directs)
└── builder     → templates · registry · middleware · statics · routes
```

Règle : **aucun `tracing::` direct** ne subsiste pour un event applicatif — tout passe
par un nœud de l'arbre. Exception tolérée : erreur fatale de démarrage (toujours émise).

Migration des champs plats actuels :

| Actuel (plat) | Cible |
|---|---|
| `csrf` | `middleware.csrf` |
| `host_validation` | `middleware.host_validation` |
| `rate_limit` | `middleware.rate_limit` |
| `acme` | `middleware.https` |
| (absent) | `middleware.csp` *(nouveau)* |
| `session` | `session.store` |
| `exclusive_login` | `session.exclusive_login` |
| `password_init` | `auth.password_init` |
| `filter_fn` | `admin.filter_fn` |
| `roles` | `admin.roles` |
| `db` | `db.connect` |

**Breaking change** assumé : `.csrf(L)` → `.middleware(|m| m.csrf(L))`. `dev()` recurse
dans tout l'arbre. C'est la **première étape de la Phase 0** (§7), avant de brancher
`TraceResult`.

---

## 2. Principes directeurs

1. **Un `Result` perdu = un log.** Avant tout `.ok()`/`unwrap_or_*` sur une opération
   faillible (DB, parse, IO, render), émettre `runique_log!(level, ?err, …)`.
2. **Capturer la donnée, pas seulement le message.** Sigils `tracing` :
   - `?value` → `Debug` (`{:?}`) — structures, erreurs, AST.
   - `%value` → `Display` — identifiants, chemins, clés.
   - `champ = expr` → champ structuré nommé.
3. **Le niveau vient de la config, pas du code.** Toujours passer par la catégorie :
   `if let Some(level) = get_log().<catégorie> { runique_log!(level, …) }`.
4. **Noms de champs canoniques** (cohérence cross-module, §5).
5. **Une catégorie par sous-système.** Si un nouveau sous-système loggue, lui ajouter
   un champ dans `RuniqueLog` (+ `dev()`), ne pas réutiliser une catégorie sans rapport.

---

## 2.bis Sémantique des niveaux (convention)

`tracing` fournit le **mécanisme** (les 5 niveaux + le filtrage). Il ne décide pas **quel
niveau pour quel type d'événement** — c'est une règle humaine, à fixer pour rester cohérent.

| Niveau | Sens : « un humain doit-il regarder ? » | Exemples Runique |
|---|---|---|
| `ERROR` | oui, action requise | DB down, panic, **5xx**, envoi mail échoué |
| `WARN` | dégradé mais géré, surveiller si récurrent | rate-limit atteint, CSRF rejeté, host refusé, retry, fallback |
| `INFO` | jalon de cycle de vie | requête servie (access log), login, migration appliquée, serveur démarré |
| `DEBUG` | diagnostic dev | coercion de champ, valeurs, flux du pipeline forms |
| `TRACE` | firehose | chaque `set_value`, chaque requête SQL |

**Règle clé : une erreur client (4xx — 404, 400 validation) n'est PAS `ERROR`.** C'est du
trafic normal (`INFO`/access log, ou rien). Seule une **faute serveur (5xx)** est `ERROR`.
Ne pas confondre le statut HTTP « error » et le niveau de log `ERROR`.

---

## 3. Deux mécanismes complémentaires

### 3.a — Mécanisme générique : trait `TraceResult` (longue traîne)

Pour la **majorité** des sites qui avalent un `Result`, on ne veut pas écrire un
`runique_log!` à la main. On utilise un trait d'extension sur `Result` qui capture
**automatiquement le `file:line` de l'appelant** via `#[track_caller]` +
`std::panic::Location::caller()`.

À créer dans `runique/src/utils/config/` (ex. `trace_ext.rs`) :

```rust
use tracing::Level;

pub trait TraceResult<T> {
    /// Loggue l'erreur (file:line de l'appelant + ctx) au niveau donné, puis `None`.
    #[track_caller]
    fn trace(self, level: Option<Level>, ctx: &'static str) -> Option<T>;
    /// Idem mais renvoie `T::default()` au lieu de `None`.
    #[track_caller]
    fn trace_or_default(self, level: Option<Level>, ctx: &'static str) -> T
    where
        T: Default;
}

impl<T, E: std::fmt::Debug> TraceResult<T> for Result<T, E> {
    #[track_caller]
    fn trace(self, level: Option<Level>, ctx: &'static str) -> Option<T> {
        if let Err(e) = &self
            && let Some(level) = level
        {
            let loc = std::panic::Location::caller();
            crate::runique_log!(level, error = ?e, at = %loc, "{}", ctx);
        }
        self.ok()
    }

    #[track_caller]
    fn trace_or_default(self, level: Option<Level>, ctx: &'static str) -> T
    where
        T: Default,
    {
        self.trace(level, ctx).unwrap_or_default()
    }
}
```

Substitution aux sites avaleurs :

```rust
// avant — silencieux
renderer.render_field(&hp).unwrap_or_default()
// après — loggue file:line + erreur si la catégorie est active, sinon coût nul
renderer.render_field(&hp).trace_or_default(get_log().forms_render(), "honeypot render")
```

Accessoires recommandés : de petits accesseurs sur `RuniqueLog` pour éviter
`get_log().forms.as_ref().and_then(|f| f.render)` répété — ex. `get_log().forms_render()`
renvoyant `Option<Level>`.

**Limites (à connaître) :**
- Le helper loggue `error` + `at` + `ctx`, **pas** les champs métier (`resource`, `id`,
  `field`). Pour les chemins chauds, préférer 3.b.
- Ne couvre que `Result`. Un `unwrap_or(None)` dont la source est déjà `Option` n'a pas
  d'erreur à logguer → corriger en amont (remonter un vrai `Result`).
- `ctx` est sensible aux secrets comme tout le reste (§6).

### 3.b — Mécanisme structuré : `runique_log!` à la main (chemins chauds)

Pour les rares chemins à fort diagnostic, garder le log structuré complet.
Référence : [admin/admin_main/handle_crud.rs:216](runique/src/admin/admin_main/handle_crud.rs#L216) :

```rust
let level = get_log().admin.as_ref().and_then(|a| a.crud);
if let Some(level) = level {
    runique_log!(
        level,
        resource = %entry.meta.key,
        error = %e,
        unique = is_unique_violation(&e),
        "create POST — DB error"
    );
}
```

Caractéristiques à imiter : champ d'identité (`resource`/`id`), erreur en `%`/`?`,
booléens de diagnostic, message court et **stable** (valeurs dans les champs, pas dans le texte).

### 3.c — Diagnostic renderer : sortie façon `cargo check` (erreurs)

Pour le rendu **d'une** erreur (pas le flux d'events), produire une sortie terminal type
compilateur Rust : message + `--> fichier:ligne` + extrait de source surligné + notes.
C'est le mécanisme le plus à fort impact pour « magie mais debuggable ».

Atout : **les données sont déjà là.** `ErrorContext`
([errors/error.rs](runique/src/errors/error.rs)) collecte déjà `template_info { name,
source }`, `stack_trace` (chaîne + `debug_repr`) et `debug_repr`. La page HTML debug consomme
déjà ce `ErrorContext` → un renderer terminal donne la **parité terminal/page** (une source,
deux rendus).

**Décision : pas de numéro de ligne template.** Tera n'expose pas de ligne fiable pour les
erreurs de rendu (le `Display` n'a aucune ligne, et il n'y a pas de span au rendu — voir
analyse Tera 1.20.1). On n'essaie **pas** de la deviner (heuristique de recherche de token =
fausses lignes, pire que rien). On affiche le **fragment fautif** + le contexte, et l'IDE
permet de localiser le code. Corollaire : `extract_tera_line` et le surlignage de ligne dans
`template-info.html` deviennent inutiles → à retirer (cleanup).

Sortie cible :

```
error[template]: variable `prix` introuvable
  in: templates/menu/detail.html
   = fragment : {{ prix }}
   = note: variables disponibles : menu, theme, user
   = at: src/views.rs:88   (côté Rust, via TraceResult — fiable)
```

Distinction clé : la ligne **template** est abandonnée (Tera ne la donne pas), mais le
`at: fichier:ligne` **côté Rust** vient de `#[track_caller]` (§3.a) et reste fiable — c'est
le pointeur exploitable.

Implémentation : **renderer maison, zéro dépendance** (recommandé). `render_diagnostic(&ErrorContext)
-> String` : message + `in:` (nom du template) + `= fragment` + `= note` + `= at` (si présent).
Couleurs ANSI gated sur TTY + `is_debug()`. ~80 lignes, aucun parsing de ligne, aucune dep.

Alternative `miette`/`ariadne` : écartée — elle vit pour les spans octets, or on a justement
renoncé au niveau ligne. Aucun intérêt ici.

Branchement : en debug, le nœud `errors` (§1.bis) émet ce diagnostic sur stderr au lieu du
`error!("{}", …)` plat actuel ; la page debug reste le même rendu en HTML. Prod inchangée.

### Règle de choix

- Site qui jette un `Result` sans champ métier utile → **3.a** (`.trace(...)`).
- Hot path (admin CRUD, login, finalize) avec identité/diagnostic → **3.b** (structuré).
- Rendu d'une `RuniqueError` (terminal + page debug) → **3.c** (diagnostic renderer).

---

## 4. Gaps identifiés (chantier)

Audit `runique/src` (patterns d'erreur avalée). Densité par fichier :

| Fichier | Pattern dominant | Priorité |
|---|---|---|
| [forms/form.rs](runique/src/forms/form.rs) | `unwrap_or_default()` ×3, `.ok()` ×5 (render, parse contraintes) | **P1** |
| [forms/field.rs](runique/src/forms/field.rs) | `parse().ok()` ×12 (coercion silencieuse) | **P1** |
| [admin/daemon/generator.rs](runique/src/admin/daemon/generator.rs) | `let _ = …await` ×9, `.ok()` ×6 | P2 |
| [admin/router/admin_router.rs](runique/src/admin/router/admin_router.rs) | `unwrap_or_default()` ×17, `unwrap_or(None)` ×2 | P2 |
| [middleware/session/cleaning_store.rs](runique/src/middleware/session/cleaning_store.rs) | `.ok()` ×9 | P2 |
| [macros/context/flash.rs](runique/src/macros/context/flash.rs) | `let _ = …await` ×12 | P3 |
| [migration/utils/parser_builder.rs](runique/src/migration/utils/parser_builder.rs) | `.ok()` ×12 | P3 |

**P1 = pipeline `forms/`** : c'est la partie la plus « magique » et celle où l'échec
silencieux frustre le plus. Les catégories existent **déjà** dans `FormTracing`
(`field`, `set_value`, `validate`, `render`, `finalize`) mais ne sont pas émises à ces
sites. Exemples précis :

- [forms/form.rs:122](runique/src/forms/form.rs#L122) — `render_field(&hp).unwrap_or_default()`
  → si le rendu échoue, champ vide rendu, aucune trace. Ajouter `forms.render`.
- [forms/field.rs:59-71](runique/src/forms/field.rs#L59-L71) — `parse().ok()` → une
  coercion ratée (ex. `int` invalide) devient `None` sans trace. Ajouter `forms.field`/
  `forms.set_value` avec `field = %name`, `?raw`.

---

## 5. Conventions de champs (canoniques)

Mêmes noms partout pour pouvoir filtrer/corréler :

| Champ | Sens | Sigil |
|---|---|---|
| `resource` | clé de ressource admin | `%` |
| `id` | identifiant d'entité | `%` |
| `field` | nom de champ de formulaire | `%` |
| `raw` | valeur brute reçue (non sensible) | `?` |
| `error` | erreur (DB, parse, IO) | `%` ou `?` |
| `path` | chemin URL / fichier | `%` |
| `count` | cardinalité (routes, templates…) | brut |
| `user_id` | identifiant utilisateur | `%` |

Message : court, stable, sous la forme `"<contexte> — <événement>"`
(ex. `"edit POST — DB error"`). Pas de valeur interpolée dans le texte.

---

## 6. Sécurité — ne jamais logguer

Le pipeline `forms.finalize` hash les mots de passe et déplace des fichiers ; enrichir
ces logs avec `?value` risque la fuite. Règles :

- Jamais : `password`, hash, token de session/reset, secret, en-tête `Authorization`/`Cookie`.
- Champs `password` (skippés par `Forms::fill()`) → ne jamais émettre `raw`/`value`.
- `FileField` → logguer le nom/taille, pas le contenu.
- En cas de doute sur la sensibilité d'un champ : logguer la **présence** (`has_value = true`)
  plutôt que la valeur.

---

## 7. Plan d'exécution (ordonné)

- **Phase 0 — Refonte propre + outillage.** Réécriture top-down de l'arbre calqué
  modules (§1.bis), pas une migration de champs.
  - **0.a** Créer le dossier `utils/config/runique_log/` : `mod.rs` (racine `RuniqueLog`,
    statics `LOG_CONFIG`, `init_subscriber`, `log_init`/`get_log`, macro `runique_log!`)
    + un fichier par nœud (`forms.rs`, `middleware.rs`, `session.rs`, `auth.rs`,
    `admin.rs`, `db.rs`, `mailer.rs`, `migration.rs`, `templates.rs`, `builder.rs`).
  - **0.b** Définir tout l'arbre §1.bis : 5 sous-structs existantes reprises/étendues
    (`admin` +filter_fn/roles/daemon, `auth` +password_init) et 5 créées
    (`middleware` dont `csp`, `session`, `db`, `migration`, `templates`). Racine = juste
    `subscriber_level` + les `Option<…Tracing>`. `dev()` recurse partout.
  - **0.c** Recâbler **tous** les sites de lecture vers l'arbre : les ~20 `get_log().<plat>`
    (table §1.bis) **et** les 21 `tracing::` directs (db, app/builder, acme, admin/daemon,
    makemigrations) qui passent désormais par un nœud — plus aucun event applicatif hors arbre.
  - **0.d** Créer le trait `TraceResult` (§3.a) dans `utils/config/trace_ext.rs`,
    l'exporter au prélude, + accesseurs de niveau (`forms_render()`, `middleware_csrf()`…).
  - **0.e** Remplacer le `try_init().ok()` par un **warning** si un subscriber est déjà posé
    (§7.ter, conflit silencieux).
  - Aucun site avaleur substitué à cette phase — uniquement structure + outil.
- **Phase 1 — Pipeline forms (P1).** Substituer `.ok()`/`unwrap_or_default()` par
  `.trace(...)`/`.trace_or_default(...)` aux sites
  [forms/form.rs](runique/src/forms/form.rs) + [forms/field.rs](runique/src/forms/field.rs)
  (coercion/render/finalize), en respectant §6.
- **Phase 2 — Swallow-sites admin/session (P2).** Même substitution dans `admin/daemon`,
  `admin/router`, `cleaning_store`. Garder le structuré (§3.b) sur les hot paths déjà bons.
- **Phase 3 — Reste (P3).** `flash`, `migration/utils`, CLI.
- **Phase 4 — Compile-time.** Vérifier que les 2 `unwrap` restants de `derive_form`
  ([model/utils/foreignkey.rs:45](runique/derive_form/src/model/utils/foreignkey.rs#L45),
  [model/utils/relation_enum.rs:33](runique/derive_form/src/model/utils/relation_enum.rs#L33))
  deviennent des `syn::Error` spannées.
- **Phase 4.bis — Sorties (§7.ter).** Enum `LogOutput` + `.output()` répétable, format
  pretty/JSON, fichier via `tracing-appender` (non-bloquant + `WorkerGuard` remonté à
  `RuniqueApp`), override `RUNIQUE_LOG_FILE`. Le `db` reste hors scope (→ history).
- **Phase 4.ter — Observabilité (§7.quater).** `request_id` (span racine middleware),
  **access log** (event à la fermeture du span), hook de panic → diagnostic 3.c, newtype
  `Secret<T>`, conventions `#[instrument]`/champs paresseux.
- **Phase 5 — Docs.** Mettre à jour `docs/fr/configuration/tracing` + `docs/en` avec les
  conventions §2/§2.bis (niveaux)/§5/§6, l'API `with_log` (arbre + `LogOutput`, §7.ter) et un
  exemple `with_log(|l| l.dev())`.

---

## 7.bis Error templates — pendant visible du tracing

L'error page debug est le miroir UI du tracing : ce que le dev voit quand la magie casse.
L'existant est déjà riche
([middleware/errors/error.rs](runique/src/middleware/errors/error.rs),
partials `templates/errors/corps-error/` : request-info, stack-trace, template-info avec
source surlignée, zone-info) et sépare proprement debug vs prod, avec filtrage des headers
sensibles (`authorization`/`cookie`/`token`). Chantier ciblé, pas une réécriture :

- **Router le logging du middleware via l'arbre.** `error_handler_middleware` émet
  aujourd'hui des `info!`/`error!` directs → passer par le nœud `errors` (`http`/`render`),
  comme tout le reste (cohérence + désactivable).
- **Corréler page et log.** En debug, afficher le `at = file:line` capturé par
  `TraceResult` (§3.a) et la catégorie de log qui a déclenché, pour faire le pont entre la
  page et la sortie `tracing`.
- **Chaîne d'erreur.** Rendre `?source` (erreur + causes) dans le partial stack-trace
  quand disponible, plutôt que le seul message de surface.
- **Aligner la taxonomie.** Les libellés `zone-info` doivent reprendre les noms des modules
  de l'arbre (§1.bis) — même vocabulaire partout.
- **Prod inchangée.** 404/500 minimalistes, jamais de détail interne ni de secret (déjà le cas).

À traiter en **Phase 2.bis** (après le recâblage du middleware en Phase 0/2), car cela
dépend du nœud `errors` et du trait `TraceResult`.

---

## 7.ter Sorties : format × destination (API `with_log`)

Deux axes **indépendants** : le **format** (comment c'est écrit) et la **destination**
(où ça va). Le subscriber compose les deux via des *layers* `tracing` (chaque sortie = une
layer empilée, avec son propre format).

- **Format** : `pretty` (coloré, dev) ou `json` (une ligne JSON/event, pour un agrégateur
  type Loki/Datadog/ELK qui parse les champs structurés en colonnes).
- **Destination** : `stdout`, `fichier`, ou **les deux** (combinaison fréquente :
  console pretty + fichier JSON).

### Timestamp ≠ rotation (à ne pas confondre)

Deux notions de temps distinctes :

- **Timestamp par ligne** — le *quand* de chaque event, **toujours présent**. On configure son
  **format**, pas une période : fuseau (**UTC** recommandé serveur, ou local) + précision
  (s/ms). Via `fmt().with_timer(...)`. Réglage `.timestamp(Timestamp::Utc)`.
- **Rotation de fichier** — un *nouveau fichier* par période, dont le **nom porte la
  date/heure** automatiquement (suffixe ajouté par `tracing-appender`) :
  - `Daily`  → `runique.log.2026-06-13`
  - `Hourly` → `runique.log.2026-06-13-14`
  - `Never`  → `runique.log` (fichier unique)

  C'est le champ `rotation` de `LogOutput::File`, indépendant du timestamp par ligne (le nom
  du fichier dit la période couverte ; chaque ligne garde son horodatage précis). **Limite** :
  `tracing-appender` ne fait que `MINUTELY | HOURLY | DAILY | NEVER` — **pas de hebdomadaire
  natif** (semaine = writer custom ou `logrotate` OS).

### API cible

Le **niveau** reste l'arbre par module (§1.bis), pas un scalaire global. La **sortie** est un
enum **à données** (la variante `File` porte ses réglages) et `.output()` est **répétable**
pour combiner plusieurs sorties.

```rust
.with_log(|l| l
    .level("info")                          // fallback global ; RUST_LOG prime
    .middleware(|m| m.csrf(Level::DEBUG))   // affinage fin = l'arbre
    .timestamp(Timestamp::Utc)              // format du timestamp par ligne (toujours présent)
    .output(LogOutput::Stdout)              // sortie 1 : console
    .output(LogOutput::File {               // sortie 2 : fichier
        path: "logs/runique.log".into(),
        rotation: Rotation::Daily,          // Hourly | Daily | Never (PAS Weekly nativement)
        format: Format::Json,
    })
)
```

```rust
pub enum LogOutput {
    Stdout,
    File { path: PathBuf, rotation: Rotation, format: Format },
}
```

### Fichier : 3 contraintes techniques

- **Crate `tracing-appender`** pour l'écriture fichier + la rotation (un fichier par
  jour/heure, ou unique).
- **Non-bloquant obligatoire** : `tracing_appender::non_blocking(...)` (l'écriture disque
  est lente ; sans ça elle bloque le thread de requête).
- **`WorkerGuard` à garder vivant** : `non_blocking` renvoie un guard ; si on le drop, le
  thread d'écriture s'arrête et **les logs bufferisés sont perdus**. Donc `init_subscriber`
  doit **remonter le guard** ([runique_log.rs:276](runique/src/utils/config/runique_log.rs#L276)
  ne renvoie rien aujourd'hui) jusqu'à `RuniqueApp` qui le stocke jusqu'au shutdown.
- **Pas d'ANSI dans un fichier** : `.with_ansi(false)` (ou JSON) — sinon des `\x1b[..` illisibles.

### Override env

Comme `RUST_LOG`, prévoir `RUNIQUE_LOG_FILE` (chemin) qui prime sur la config code.

### Possession du subscriber (conflit silencieux)

Un subscriber global `tracing` ne se pose **qu'une fois par process**, et la lib **interdit
de le remplacer**. Donc si le dev a **déjà** son propre subscriber (il utilise `tracing` pour
son code), l'`init` de Runique est sans effet. Aujourd'hui `try_init().ok()`
([runique_log.rs:291](runique/src/utils/config/runique_log.rs#L291)) **avale ce cas en
silence** → la config par catégorie est inactive sans aucune erreur. Fix minimal (1 ligne) :
regarder le `Result` de `try_init()` et émettre **un warning** (« un subscriber tracing était
déjà installé, la config log Runique est inactive ») au lieu de `.ok()`. Le dev garde son
subscriber, mais sait pourquoi. *(Évolution possible, non v1 : exposer une `Layer` que le dev
branche sur son propre subscriber.)*

### Le cas `db` — écarté du tracing

Logguer chaque event en base est un piège : perf (une écriture DB par log sur le chemin
requête), **récursion** (la couche `db.query` loggue → requête → log…), fragilité (DB down =
logs perdus au pire moment). Le bon foyer d'une trace **métier** persistante existe déjà :
l'audit/history admin ([admin/history.rs](runique/src/admin/history.rs)). Un vrai sink DB,
si un jour nécessaire, devra être **async + batché** hors chemin requête — chantier séparé,
pas la v1.

---

## 7.quater Observabilité transverse

Quatre briques qui traversent tous les modules, par valeur décroissante.

### request_id — span de corrélation *(le plus impactant, absent)*

Un span est une **période avec contexte** : tout event émis dedans **hérite** de ses champs.
Un middleware en tête génère un `request_id` (uuid), ouvre un span racine
`info_span!("request", request_id = %id)` → **chaque** event de la requête le porte sans le
repasser. `grep request_id=<x>` = toute la vie d'une requête isolée. L'error middleware crée
déjà un span ([error.rs:58](runique/src/middleware/errors/error.rs#L58)) — y attacher le champ.

### Hook de panic → diagnostic *(robustesse, absent)*

Un `panic!` (unwrap sur None, index hors bornes) tue le thread avec un dump brut.
`std::panic::set_hook` (ou la layer tower `CatchPanic`) intercepte → loggue via l'arbre +
renvoie un 500 propre (rendu 3.c) au lieu de crasher. La magie casse proprement.

### `Secret<T>` — redaction par construction *(sécurité, absent)*

Newtype dont le `Debug` imprime `***` :
```rust
pub struct Secret<T>(pub T);
impl<T> std::fmt::Debug for Secret<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result { write!(f, "***") }
}
```
`password: Secret<String>` → même un `?password` par erreur affiche `***`. La fuite (§6)
devient **impossible**, pas seulement « interdite ». Réflexe juste pour un framework sécurité.

### Hygiène : champs paresseux + `#[instrument]` *(à peine utilisé)*

- **Jamais de `format!` dans un argument de log** : il s'exécute même niveau désactivé.
  Passer la valeur brute (`?x`), `tracing` formate à la demande.
- **`#[instrument(skip_all, fields(...))]`** sur les handlers clés : span auto avec args
  choisis (`skip_all` évite de logguer les gros/sensibles par défaut).

### Access log — l'événement « requête servie »

Une ligne par requête HTTP qui la résume : `method · path · status · latency · request_id`.
C'est le log d'exploitation n°1 (« quelles requêtes échouent / traînent ? »). Émis à la
**fermeture du span `request_id`** → quasi gratuit une fois le span en place. Niveau `INFO`
(succès) ; un 5xx déclenche en plus un `ERROR` côté handler/middleware (§2.bis).

```
INFO request method=GET path=/menu/3 status=200 latency=12ms request_id=a3f9
```

### Propagation de trace (OTel) — note de seam, rien à coder

Plus tard, pour [La Pieuvre](projet_futur.md) (reverse proxy + handoff multi-services) : il
faudra propager le contexte de trace entre services (header W3C `traceparent`) et brancher un
exporter OTLP (crate `tracing-opentelemetry`, une **Layer**, pas un trait). À faire **maintenant** :
rien — juste garder le design des spans (`request_id`) propre pour ne pas se peindre dans un coin.

---

## 8. Definition of done (anti-régression)

Une zone est « faite » quand :

1. Aucun `Result` faillible n'est jeté sans un `runique_log!` préalable.
2. Les events portent les champs canoniques (§5), erreur capturée comprise.
3. Le niveau passe par la catégorie de config (pas de `Level::X` en dur).
4. Aucun secret loggué (§6).
5. `with_log(|l| l.dev())` en `DEBUG` fait apparaître le flux complet de la zone.
6. Message stable (grep-able), valeurs dans les champs.

---

## 9. Checklist PR (à coller en revue)

- [ ] Pas de `eprintln!`/`println!` applicatif ajouté
- [ ] Chaque erreur avalée loggée avant d'être jetée
- [ ] Champs canoniques + `?`/`%` corrects
- [ ] Niveau via `get_log().<catégorie>`
- [ ] Niveau sémantiquement correct (§2.bis ; 4xx ≠ ERROR, seul 5xx l'est)
- [ ] Aucun secret dans les champs (secrets en `Secret<T>`)
- [ ] Pas de `format!` dans un argument de log (champs paresseux)
- [ ] Sortie fichier : `with_ansi(false)` + `WorkerGuard` conservé
- [ ] Doc tracing à jour si nouvelle catégorie
