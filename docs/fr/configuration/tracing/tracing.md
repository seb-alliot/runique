# Tracing structuré

Runique expose un système de tracing structuré via `RuniqueLog`. Par défaut, un subscriber **console** est installé et les **domaines sont opt-in** : tant qu'un domaine n'est pas activé, ses événements ne sont pas émis. Quelques sites critiques émettent toujours (voir [Erreurs inconditionnelles](#erreurs-inconditionnelles-toujours-actives)).

## Activation rapide en développement

```rust
RuniqueApp::builder(config)
    .with_log(|l| l.dev())   // tout à DEBUG si DEBUG=true
    // ...
```

`.dev()` est un no-op si `DEBUG` n'est pas `true` — utilisable inconditionnellement.

---

## Configuration granulaire

```rust
.with_log(|l| l
    .forms(|f| f
        .validate(Level::DEBUG)
        .finalize(Level::DEBUG)
    )
    .admin(|a| a
        .crud(Level::INFO)
        .auth(Level::WARN)
    )
    .auth(|a| a
        .login(Level::INFO)
        .reset(Level::WARN)
    )
    .mailer(|m| m.send(Level::INFO))
    .builder(|b| b
        .templates(Level::INFO)
        .middleware(Level::DEBUG)
        .routes(Level::INFO)
        .statics(Level::INFO)
    )
    .rate_limit(Level::WARN)
)
```

---

## Domaines disponibles

### `forms` — Pipeline formulaire

| Champ | Moment | Données loggées |
|-------|--------|-----------------|
| `field` | Enregistrement d'un champ | nom, type, required |
| `set_value` | Valeur assignée par `fill()` | nom, valeur (password masqué) |
| `validate` | Résultat de validation | champ, ok/error, nb global errors |
| `render` | Rendu HTML | champ, ok/error |
| `finalize` | Hash/move fichier | champ, ok/error |

### `admin` — Panel admin

| Champ | Moment | Données loggées |
|-------|--------|-----------------|
| `auth` | Vérification accès + CSRF fail | resource, action |
| `crud` | Dispatch + résultat create/edit/delete | resource, action, ok/error |
| `list` | Dispatch + résultat liste | resource, rows, total, page |
| `bulk` | Actions de masse | resource, action |

### `auth` — Authentification

| Champ | Moment | Données loggées |
|-------|--------|-----------------|
| `login` | Création de session | user_id, username, is_superuser, exclusive, db_persist |
| `reset` | Flux reset mot de passe | email, étape (token généré / email envoyé / invalide / ok / error) |

### `mailer`

| Champ | Moment | Données loggées |
|-------|--------|-----------------|
| `send` | Envoi email | backend, to, subject, ok/error |

### `builder` — Démarrage (one-time)

| Champ | Moment | Données loggées |
|-------|--------|-----------------|
| `templates` | Chargement Tera | internal, user, total |
| `registry` | Ressources admin | count |
| `middleware` | Stack middleware | count + slot + name pour chaque entrée |
| `routes` | Registry URL | count |
| `statics` | Fichiers statiques | static_url, static_dir, media_url, media_dir |

### `errors` — Pages d'erreur HTTP

| Champ | Moment | Données loggées |
|-------|--------|-----------------|
| `http` | Erreur HTTP gérée (404/validation/forbidden) | method, path, type / erreur |
| `render` | Échec de rendu d'un template d'erreur (404/429/500) | template, erreur — **plancher WARN** (toujours visible, voir plus bas) |

```rust
.with_log(|l| l.errors(|e| e.http(Level::INFO).render(Level::WARN)))
```

### Champs plats sur `RuniqueLog`

| Champ | Moment | Données loggées |
|-------|--------|-----------------|
| `rate_limit` | Requête bloquée | ip, retry_after |
| `csrf` | Token CSRF détecté dans une URL GET | path |
| `session` | Opérations session store | event |
| `db` | Requêtes base de données | query, duration |
| `host_validation` | Hôte rejeté | host |

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

    RuniqueApp::builder(RuniqueConfig::from_env())
        .with_database().await
        .routes(url::urlpatterns())
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
- **Sites sensibles à plancher `WARN`** — même domaine désactivé, ces échecs émettent au moins en `WARN`, car un échec silencieux y casserait une garantie : rotation de l'ID de session (anti-fixation), invalidation des autres sessions (login exclusif), persistance de la session au login, envoi de l'email de reset, et échec de rendu d'un template d'erreur (`errors.render`).

---

## Voir aussi

- [Variables d'environnement](/docs/fr/configuration/variables) — `RUST_LOG`, `DEBUG`
- [Configuration programmatique](/docs/fr/configuration/builder) — `.with_log()`
