# Tracing structuré

Runique expose un système de tracing structuré via `RuniqueLog`. Par défaut, un subscriber **console** est installé et les **domaines sont opt-in** : tant qu'un domaine n'est pas activé, ses événements ne sont pas émis. Quelques sites critiques émettent toujours (voir [Erreurs inconditionnelles](#erreurs-inconditionnelles-toujours-actives)).

## Ce que le `.env` allume tout seul

Sans rien écrire dans `.with_log(...)`, les canaux suivent `DEBUG` et `RUNIQUE_ENV` :

| `.env` | Canaux allumés | Niveau |
| --- | --- | --- |
| `DEBUG=true` (avec ou sans `RUNIQUE_ENV=dev`) | **tous** : les canaux utiles, plus les canaux bavards (`forms.*`, `admin.auth`, `admin.list`) | utiles en `INFO`, bavards en `DEBUG` |
| `RUNIQUE_ENV=dev`, sans `DEBUG` | les canaux utiles : connexions, reset, refus CSRF, limite de débit, hôtes, redirections, anti-bot, écritures et actions groupées de l'admin, base, mailer, erreurs HTTP, démarrage | `INFO` |
| production, sans `DEBUG` | aucun : seuls sortent les avertissements à plancher `WARN` (voir plus bas) | — |

Un canal réglé à la main garde son niveau. `.manual()` coupe ce mécanisme : seuls les canaux écrits dans `.with_log(...)` sont allumés.

```rust
.with_log(|l| l.manual().auth(|a| a.login(LogLevel::INFO)))
```

> **⚠️ `DEBUG=true` en production.** C'est permis, pour essayer une mise en service sur une adresse de test ou diagnostiquer en direct. Mais tant qu'il est actif :
>
> - tous les visiteurs voient les pages d'erreur détaillées ;
> - le canal `forms.set_value` écrit dans les journaux **la valeur de chaque champ saisi** : adresses e-mail, noms, messages. Seuls les mots de passe sont masqués, et le jeton CSRF est remplacé par un message ;
> - ces journaux deviennent des données personnelles : leur accès, leur durée de conservation et leur sauvegarde relèvent du RGPD ;
> - le filtre global passe à `debug`, ce qui laisse aussi passer les journaux de niveau `debug` des dépendances.
>
> À réserver à une adresse de test, ou à une fenêtre courte sur un site réel. Coupez-le dès que le problème est trouvé, et purgez les journaux écrits pendant ce temps.

> **Langue des journaux.** Les messages du CSRF, de l'admin, des sessions et du démarrage ajoutés en 4.1 suivent la langue de l'application (`set_lang`), comme les messages affichés aux visiteurs. Les noms de champs (`at`, `reason`, `resource`, `error`…) restent en anglais ; la valeur de `reason` et de `outcome` est traduite. Pour filtrer ou agréger des journaux par leur texte, gardez la langue par défaut (`en`).

---

## Activation rapide en développement

```rust
RuniqueApp::builder(config)
    .with_log(|l| l.dev())   // tout à DEBUG si RUNIQUE_ENV=dev
    // ...
```

`.dev()` est un no-op si `RUNIQUE_ENV` n'est pas `dev` — utilisable inconditionnellement.

---

## Configuration granulaire

```rust
.with_log(|l| l
    .forms(|f| f
        .validate(LogLevel::DEBUG)
        .finalize(LogLevel::DEBUG)
    )
    .admin(|a| a
        .crud(LogLevel::INFO)
        .auth(LogLevel::WARN)
    )
    .auth(|a| a
        .login(LogLevel::INFO)
        .reset(LogLevel::WARN)
    )
    .mailer(|m| m.send(LogLevel::INFO))
    .builder(|b| b
        .templates(LogLevel::INFO)
        .middleware(LogLevel::DEBUG)
        .routes(LogLevel::INFO)
        .statics(LogLevel::INFO)
    )
    .middleware(|m| m.rate_limit(LogLevel::WARN))
)
```

---

## Domaines disponibles

### `forms` — Pipeline formulaire

| Champ | Moment | Données loggées |
|-------|--------|-----------------|
| `field` | Enregistrement d'un champ | nom, type, required |
| `set_value` | Valeur assignée par `fill()` | nom, valeur (mot de passe masqué ; jeton CSRF remplacé par un message) |
| `validate` | Résultat de validation | champ, ok/error, nb global errors |
| `render` | Rendu HTML | champ, ok/error |
| `finalize` | Hash/move fichier | champ, ok/error |

### `admin` — Panel admin

| Champ | Moment | Données loggées |
|-------|--------|-----------------|
| `auth` | Vérification accès + CSRF fail | resource, action |
| `crud` | Écriture par un formulaire de l'admin (créer, modifier, supprimer) | resource, outcome (`saved`, `save refused`, `deleted`, `delete refused`) |
| `list` | Liste affichée ; comptages du tableau de bord en échec | resource, page, rows, total, filters, search |
| `bulk` | Action groupée faite ou refusée | resource, action, rows |

### `auth` — Authentification

| Champ | Moment | Données loggées |
|-------|--------|-----------------|
| `login` | Création de session | user_id, username, is_superuser, exclusive, db_persist |
| `reset` | Flux reset mot de passe | email, étape (token généré / email envoyé / invalide / ok / error) |
| `password_init` | `password_init()` appelé plusieurs fois | — |

### `mailer`

| Champ | Moment | Données loggées |
|-------|--------|-----------------|
| `send` | Envoi email | backend, to, subject, ok/error |

### `builder` — Démarrage (one-time)

| Champ | Moment | Données loggées |
|-------|--------|-----------------|
| `templates` | Chargement Tera | internal, user, total |
| `registry` | L'admin au démarrage : sa configuration (préfixe, tailles de page), chaque modèle enregistré, chaque route déclarée `admin`, les droits élagués | prefix, models, key, title, display, filters, actions, form_fields, pruned |
| `middleware` | Stack middleware | count + slot + name pour chaque entrée |
| `routes` | Registry URL | count |
| `statics` | Fichiers statiques | static_url, static_dir, media_url, media_dir |

### `templates` — Moteur de templates

| Champ | Moment | Données loggées |
|-------|--------|-----------------|
| `load` | Réservé : aucun événement émis pour l'instant (le chargement sort sous `builder.templates`) | — |
| `render` | Réservé : aucun événement émis pour l'instant (un échec de rendu passe par `errors`) | — |

### `migration` — Migrations

| Champ | Moment | Données loggées |
|-------|--------|-----------------|
| `plan` | Réservé : aucun événement émis pour l'instant (la CLI écrit sur la sortie standard) | — |
| `apply` | Réservé : aucun événement émis pour l'instant | — |
| `rollback` | Réservé : aucun événement émis pour l'instant (l'annulation d'un plan est écrite sur la sortie d'erreur) | — |

### `errors` — Pages d'erreur HTTP

| Champ | Moment | Données loggées |
|-------|--------|-----------------|
| `http` | Erreur HTTP gérée (404/validation/forbidden) | method, path, type / erreur |
| `render` | Échec de rendu d'un template d'erreur (404/429/500) | template, erreur — **plancher WARN** (toujours visible, voir plus bas) |

```rust
.with_log(|l| l.errors(|e| e.http(LogLevel::INFO).render(LogLevel::WARN)))
```

### `middleware` — Sécurité (`.middleware(|m| m....)`)

| Champ | Moment | Données loggées |
|-------|--------|-----------------|
| `csrf` | Le journal du CSRF. Chaque refus avec sa raison, où qu'il soit décidé : middleware (jeton invalide, absent, formulaire venu d'un autre site), formulaire (jeton absent, ou qui n'est plus celui de la session), connexion et déconnexion de l'admin, fichier envoyé avant le jeton. Chaque nouveau jeton et chaque rotation (connexion, déconnexion). Échec de lecture ou d'écriture en session | at, reason / erreur |
| `csp` | Réservé : aucun événement émis pour l'instant | — |
| `cors` | Réservé : aucun événement émis pour l'instant | — |
| `rate_limit` | Requête bloquée | ip, retry_after |
| `host_validation` | Hôte rejeté | host |
| `open_redirect` | Redirection refusée | location |
| `anti_bot` | Honeypot rempli (formulaire refusé) ; échec de lecture du nom du piège en session | form / erreur |
| `https` | Requête reçue en HTTP redirigée vers HTTPS ; arrêt du serveur du port 80 en mode ACME — **plancher WARN** pour cet arrêt | host / erreur |

### `session` — Store (`.session(|s| s....)`)

| Champ | Moment | Données loggées |
|-------|--------|-----------------|
| `store` | Opérations session store (watermarks, purge) | event |
| `cleanup` | Erreur de la purge périodique des sessions expirées (mémoire et base) | erreur |
| `exclusive_login` | Invalidation des autres sessions | — |

### `db` — Base de données (`.db(\|d\| d....)`)

| Champ | Moment | Données loggées |
|-------|--------|-----------------|
| `connect` | Connexion DB en cours / établie | — |
| `query` | Requêtes base de données | query, duration |

Ces trois domaines n'ont pas de méthode directe sur `RuniqueLog` — chacun passe par sa propre closure (`.middleware(|m| ...)`, `.session(|s| ...)`, `.db(|d| ...)`), au même titre que `forms`/`admin`/`auth` plus haut.

---

## Sorties de log

Par défaut Runique installe un subscriber console (`Stdout`, couleurs). On configure une ou plusieurs sorties **cumulables** via `.output()` :

```rust
use runique::prelude::{LogOutput, LogRotation};

.with_log(|l| l
    .output(LogOutput::stdout())                 // console couleurs
    .output(LogOutput::file("logs/app.json"))    // JSON (déduit de l'extension .json)
    .output(LogOutput::file("logs/app.log")      // texte brut
        .rotation(LogRotation::Daily)))
```

- Le **format est déduit de l'extension** : `.json` → JSON structuré (une ligne par événement), sinon texte brut.
- L'écriture fichier est **non bloquante** ; les logs sont vidés proprement à l'extinction.
- Rotation : `Daily` (défaut), `Hourly`, `Never`.
- `RUNIQUE_LOG_FILE=/chemin/app.json` ajoute une sortie fichier au runtime, sans recompiler.

## Sink personnalisé

Pour router les logs vers une destination arbitraire (base de données, collecteur HTTP, file de messages), implémente `LogSink` — aucun type `tracing` n'est exposé :

```rust
use runique::prelude::{LogOutput, LogRecord, LogSink};

struct MonSink;

impl LogSink for MonSink {
    fn log(&self, record: &LogRecord) {
        // record.level / target / message / file / line / fields
        // Ne bloque pas : pour de l'async, enfile dans ton propre channel.
    }
}

.with_log(|l| l.output(LogOutput::sink(MonSink)))
```

Le sink reçoit **tous** les événements du process (Runique **et** ton application) ; distingue-les par `record.target` (les événements Runique ont un target commençant par `runique`). Runique ne fournit volontairement **pas** de sink base de données (cela surchargerait la DB) — `LogSink` est la porte pour le brancher toi-même.

## Subscriber externe

Si ton application gère son propre subscriber `tracing` (stack de layers custom, OpenTelemetry…), déclare `.external()` : Runique **n'installe rien** et te laisse le créneau global unique, tout en **continuant d'émettre** ses événements vers la façade `tracing` (ton subscriber les reçoit).

Minimal :

```rust
.with_log(|l| l.external())
```

Complet :

```rust
use runique::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Tu poses TON subscriber, avant build()
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let db = DatabaseConfig::from_env()?.build().connect().await?;

    RuniqueApp::builder(RuniqueConfig::from_env())
        .with_database(db)
        .routes(url::routes())
        .with_log(|l| l.external())   // Runique n'installe pas son subscriber
        .build().await?
        .run().await
}
```

Pour **ignorer** les logs internes de Runique, filtre leur target dans ton `EnvFilter` :

```rust
tracing_subscriber::fmt()
    .with_env_filter("info,runique=off")   // garde tes logs, coupe ceux de Runique
    .init();
```

En mode `.external()`, les sorties `.output()` sont ignorées (c'est ton subscriber qui décide où vont les logs).

---

## Erreurs inconditionnelles (toujours actives)

Indépendamment de la config tracing, certains événements sont toujours émis :

- **Template invalide au démarrage** — `tracing::error!` (nom du template + ligne) avant l'arrêt.
- **Erreurs serveur critiques** (500 : base de données, IO, template, interne) — `tracing::error!`.
- **Sites sensibles à plancher `WARN`** — même domaine désactivé, ces échecs émettent au moins en `WARN`, car un échec silencieux y casserait une garantie : rotation de l'ID de session (anti-fixation), invalidation des autres sessions (login exclusif), persistance de la session au login, envoi de l'email de reset, échec de rendu d'un template d'erreur (`errors.render`), fermeture de la session d'un compte supprimé ou désactivé, sauvegarde d'une session en base, élagage des droits au démarrage (`builder.registry`) arrêt du serveur du port 80 en mode ACME (`middleware.https`), connexion de l'admin sans `LoginGuard` ni limiteur (hors développement), options de liste sans effet (`links`, `search_help`, `empty_value`) et feuilles de style de Runique introuvables (`STATIC_RUNIQUE_PATH`).

---

## Voir aussi

- [Variables d'environnement](/docs/fr/env) — `RUST_LOG`, `RUNIQUE_LOG_FILE`, `RUNIQUE_ENV`
- [Configuration programmatique](/docs/fr/configuration/builder) — `.with_log()`
