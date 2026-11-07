# Configuration Programmatique — Builder

`RuniqueApp::builder(config)` retourne un `RuniqueAppBuilder`. C'est le seul builder — il n'y a pas deux versions séparées.

## Exemple minimal

```rust
let app = RuniqueApp::builder(config)
    .routes(url::routes())
    .with_database(db)
    .statics()
    .build()
    .await?;

app.run().await?;
```

---

## Méthodes disponibles

### Base de données

```rust
// Option 1 : connexion directe (DatabaseConnection)
let db_config = DatabaseConfig::from_env()?.build();
let db = db_config.connect().await?;

let app = RuniqueApp::builder(config)
    .with_database(db)       // prend une DatabaseConnection
    .routes(router)
    .build()
    .await?;

// Option 2 : connexion déférée (DatabaseConfig)
let db_config = DatabaseConfig::from_env()?.build();

let app = RuniqueApp::builder(config)
    .with_database_config(db_config)  // connexion lors du .build()
    .routes(router)
    .build()
    .await?;
```

### Routes

```rust
use runique::{urlpatterns, view};

pub fn routes() -> Routes {
    urlpatterns! {
        "/" => view!{ views::index }, name = "index",
        "/about" => view!{ views::about }, name = "about",
    }
}

let app = RuniqueApp::builder(config)
    .routes(routes())
    .build()
    .await?;
```

### Gestion des erreurs

```rust
let app = RuniqueApp::builder(config)
    .middleware(|m| m.with_debug_errors(false))  // Retire le gestionnaire d'erreurs (monté par défaut)
    .routes(router)
    .build()
    .await?;
```

Le gestionnaire d'erreurs est monté par défaut ; c'est `DEBUG` qui décide de son contenu (page détaillée avec `DEBUG=true`, page générique sinon). `with_debug_errors(false)` le retire entièrement : les erreurs sortent alors sans page Runique.

### Middlewares

Toute la configuration des middlewares passe par `.middleware(|m| { ... })` où `m` est un `MiddlewareStaging` :

```rust
let app = RuniqueApp::builder(config)
    .routes(router)
    .middleware(|m| {
        m.with_csp(|c| c)              // CSP toujours actif — personnalisable ici
         .with_allowed_hosts(|h| h.host("mondomaine.fr"))  // Active la validation des hosts
         .with_cache(true)              // Garde le cache HTTP en dev (pas d'en-têtes no-cache)
         .with_debug_errors(true)       // Monte le gestionnaire d'erreurs (DEBUG règle le contenu)
    })
    .build()
    .await?;
```

La validation des hosts se déclare via `.with_allowed_hosts(|h| h.host("..."))` dans le builder : active avec `RUNIQUE_ENV=prod`, inactive avec `RUNIQUE_ENV=dev`, et `.enabled(...)` a le dernier mot. Sans cet appel, rien n'est validé.

> **`is_dev()`** — helper global disponible via `use runique::prelude::*`. Retourne `true` si `RUNIQUE_ENV=dev`. Lu une seule fois (`LazyLock`), après chargement du `.env`, disponible partout sans paramètre. **`is_debug()`** fait de même pour `DEBUG`, qui ne pilote que les pages d'erreur détaillées.

### Durée de session

```rust
use runique::tower_sessions::cookie::time::Duration;

let app = RuniqueApp::builder(config)
    .with_session_duration(Duration::hours(2))  // Par défaut : 24h
    .routes(router)
    .build()
    .await?;
```

Ou via `.middleware()` pour les options avancées :

```rust
.middleware(|m| {
    m.with_session_duration(Duration::hours(2))
     .with_anonymous_session_duration(Duration::minutes(5))
     .with_session_memory_limit(128 * 1024 * 1024, 256 * 1024 * 1024)
})
```

### URL publique — `with_public_url`

L'adresse publique de l'application, celle que tape un visiteur : la base de tous les liens absolus qu'elle envoie — les liens de reset de mot de passe, côté public comme depuis l'admin — et du lien « voir le site » de l'admin s'il n'en définit pas un lui-même :

```rust
let app = RuniqueApp::builder(config)
    .with_public_url("https://monsite.fr")
    .with_password_reset(|pr| pr)
    .build()
    .await?;
```

En production (`RUNIQUE_ENV=prod`), l'application **refuse de démarrer** sans elle dès que le reset de mot de passe ou l'admin est activé : un lien construit à partir de l'en-tête `Host` de la requête irait là où le client le décide (`Host: evil.com` → le jeton de reset part chez l'attaquant). Avec `RUNIQUE_ENV=dev`, `.with_public_url()` est ignorée : l'URL publique est toujours `http://127.0.0.1:{PORT}`, le port venant de `PORT` (3000 par défaut). Liens envoyés par mail, `og:image` et favicon la suivent tous : naviguez sur cette adresse plutôt que sur `localhost`, où la CSP bloque le favicon. En production sans URL (ni reset ni admin), `og:image` et le favicon restent en chemin relatif.

Vos propres liens (activation de compte, mail pointant vers le site…) suivent la même règle via `request.public_url()` : l'URL publique, sinon `None`, jamais le `Host`. Ne les construisez jamais vous-même à partir de l'en-tête `Host`.

```rust
let Some(base) = request.public_url() else {
    // production sans .with_public_url() : pas de lien plutôt qu'un lien forgé
    return request.render("auth/inscrit.html");
};
let activate_url = format!("{base}/activer/{token}");
```

Elle ne s'appelle **qu'une fois** : un second `.with_public_url(...)` ne compile pas (« `with_public_url()` has already been called on this builder »), plutôt que de remplacer le premier sans rien dire. Écrivez-y l'adresse de production : le développement n'en tient pas compte.

### Logs framework

`RuniqueLog` centralise toute la configuration des logs : le niveau du subscriber tracing global **et** les catégories internes du framework.

Le subscriber est initialisé automatiquement par `build()` — **aucun appel à `init_logging()` n'est nécessaire dans `main.rs`**.

Tout passe par `.with_log(|l| ...)` — la closure reçoit un `RuniqueLog` vide et retourne la configuration finale.

```rust
use runique::prelude::*; // LogLevel = tracing::Level

// Contrôle fin par catégorie — chaque sous-module reçoit sa propre closure
RuniqueApp::builder(config)
    .with_log(|l| l
        .middleware(|m| m.csrf(LogLevel::WARN))
        .session(|s| s.store(LogLevel::WARN))
        .db(|d| d.connect(LogLevel::INFO))
    )
    .routes(router)
    .build()
    .await?;
```

#### `subscriber_level` — niveau du subscriber

Par défaut : `"debug"` si `RUNIQUE_ENV=dev`, sinon `"warn"`. La variable `RUST_LOG` a toujours la priorité.

```rust
RuniqueApp::builder(config)
    .with_log(|l| l.subscriber_level("info"))
    .routes(router)
    .build()
    .await?;
```

#### `.dev()` — tout activer en développement

Preset qui active toutes les catégories au niveau `DEBUG`. Sans effet si `RUNIQUE_ENV` n'est pas `dev` — peut être utilisé inconditionnellement.

```rust
// Dev uniquement (no-op si RUNIQUE_ENV != dev)
RuniqueApp::builder(config)
    .with_log(|l| l.dev())
    .routes(router)
    .build()
    .await?;

// Dev avec surcharge du niveau subscriber
RuniqueApp::builder(config)
    .with_log(|l| l.dev().subscriber_level("info").db(|d| d.connect(LogLevel::INFO)))
    .routes(router)
    .build()
    .await?;
```

#### Catégories disponibles

`RuniqueLog` n'a que des méthodes racine par sous-module (`forms`, `middleware`, `session`, `auth`, `admin`, `db`, `mailer`, `migration`, `templates`, `errors`, `builder`), chacune prenant une closure vers son propre builder imbriqué — il n'y a pas de méthode plate `.csrf()`/`.session()` directement sur `RuniqueLog` :

| Catégorie (feuille imbriquée)     | Accès                              | Ce qui est journalisé                                      |
| ---------------------------------- | ----------------------------------- | ---------------------------------------------------------- |
| `csrf`            | `.middleware(\|m\| m.csrf(...))`         | Token CSRF détecté dans une URL GET (nettoyage silencieux) |
| `exclusive_login` | `.session(\|s\| s.exclusive_login(...))` | Sessions invalidées lors d'une connexion exclusive         |
| `password_init`   | `.auth(\|a\| a.password_init(...))`      | `password_init()` appelé plusieurs fois                    |
| `store`           | `.session(\|s\| s.store(...))`           | Watermarks mémoire, records volumineux, erreurs cleanup    |
| `connect`         | `.db(\|d\| d.connect(...))`              | Connexion DB en cours / connexion établie                  |

### Base de données secondaire — `with_custom_db`

Attache n'importe quel type `Any + Send + Sync + 'static` dans le `RuniqueEngine`.
Utile pour connecter MongoDB, Redis, ou toute autre source externe.
Peut être appelé plusieurs fois avec des types différents.

> **`with_custom_db` ne remplace pas la base de l'ORM.** La connexion SeaORM — entités, `search!`, `save`, l'admin et les migrations — reste celle de `with_database` / `with_database_config`. Les valeurs passées à `with_custom_db` ne sont accessibles **que manuellement** via `engine.custom_db::<T>()` ; l'ORM ne les utilise jamais. Pour changer le moteur de l'ORM, voir [Base de données](/docs/fr/installation/base-de-donnees).

```rust
let mongo = mongodb::Client::with_uri_str("mongodb://localhost:27017").await?;
let redis = redis::Client::open("redis://localhost")?;

let app = RuniqueApp::builder(config)
    .with_custom_db(mongo)
    .with_custom_db(redis)
    .routes(url::routes())
    .build()
    .await?;
```

Récupération dans un handler via `engine.extension::<T>()` — retourne `Option<Arc<T>>` :

```rust
async fn mon_handler(req: Request) -> Response {
    if let Some(mongo) = req.engine.extension::<mongodb::Client>() {
        let collection = mongo.database("mydb").collection::<Document>("users");
    }
    if let Some(redis) = req.engine.extension::<redis::Client>() {
        // ...
    }
}
```

`engine.custom_db::<T>()` est un alias conservé pour la compatibilité ascendante — les deux méthodes sont équivalentes.

#### Exemple complet — MongoDB (comme Campanile)

Un helper centralise l'accès au client et à la base, avec une erreur claire si le client n'a pas été enregistré :

```rust
use runique::prelude::*;

pub async fn get_mongo(request: &Request) -> Result<mongodb::Database, mongodb::error::Error> {
    let client = request
        .engine
        .custom_db::<mongodb::Client>()
        .ok_or_else(|| mongodb::error::Error::custom("client MongoDB non initialisé"))?;
    Ok(client.database("campanile"))
}
```

Le client est enregistré une fois dans `main.rs` :

```rust
let mongo = mongodb::Client::with_uri_str(
    std::env::var("MONGO_URI").unwrap_or_else(|_| "mongodb://localhost:27017".into()),
)
.await?;

let app = RuniqueApp::builder(config)
    .with_database(db)
    .with_custom_db(mongo)
    .routes(url::routes())
    .build()
    .await?;
```

Un handler écrit ensuite dans une collection — ici un compteur de vues en upsert :

```rust
use mongodb::bson::{DateTime as BsonDateTime, doc};

pub async fn track_view(request: &Request, plat_id: i32, titre: &str) -> Result<(), mongodb::error::Error> {
    let db = get_mongo(request).await?;
    let views = db.collection::<mongodb::bson::Document>("plat_views");
    views
        .update_one(
            doc! { "plat_id": plat_id },
            doc! {
                "$inc": { "vues": 1 },
                "$set": { "titre": titre, "derniere_vue": BsonDateTime::now() },
            },
        )
        .upsert(true)
        .await?;
    Ok(())
}
```

### Administration

`.with_admin(...)` n'existe que si `runique` est compilé avec la feature `admin`. Voir [Admin](/docs/fr/admin).

### Fichiers statiques

```rust
let app = RuniqueApp::builder(config)
    .statics()     // Active les fichiers statiques
    // ou
    .static_files(|s| s.enabled(false))  // Désactive explicitement
    .build()
    .await?;
```

## Réglages à appel unique

Certains réglages remplacent leur valeur à chaque appel : les déclarer deux fois perdrait le premier sans rien dire. Ils ne s'appellent donc **qu'une fois par builder**, et un second appel **ne compile pas** :

| Réglage | Remarque |
| --- | --- |
| `.with_public_url(…)` | |
| `.routes(…)` | construire un seul `Router` (fusionner ou imbriquer les autres dedans) |
| `.with_log(…)` | régler toutes les catégories dans le même appel |
| `.with_password_reset(…)` | |
| `.with_database(…)` / `.with_database_config(…)` | l'une **ou** l'autre, une fois |
| `.with_session_duration(…)` | |
| `.with_mailer(…)` / `.with_mailer_from_env()` | l'une **ou** l'autre, une fois |

L'erreur nomme le réglage, par exemple :

```
error[E0277]: `routes()` has already been called on this builder
```

Les réglages qui **composent** — `.core(…)`, `.middleware(…)`, `.static_files(…)`, `.with_admin(…)`, `.with_custom_db(…)`, `.statics()` — reprennent l'état existant : on peut les appeler plusieurs fois.

L'état est porté par le type du builder (`RuniqueAppBuilder<S>`), invisible tant qu'on enchaîne les appels. Une fonction qui renvoie un builder à mi-chemin doit l'écrire : `runique::app::builder::state::{No, Yes}`, un emplacement par réglage, dans l'ordre du tableau ci-dessus.

---

## Valeurs par défaut

| Configuration | Défaut | Notes |
| ------------ | ------ | ----- |
| **Session duration** | 24 heures | |
| **Session store** | `CleaningMemoryStore` | |
| **CSRF protection** | ✅ Toujours activé | Non désactivable |
| **Error handler** | ✅ Activé | |
| **CSP + headers de sécurité** | ✅ Toujours actifs | Inconditionnel, quel que soit le mode ; `.with_csp(...)` personnalise, n'active pas |
| **Host validation** | ❌ Désactivée | Aucune variable `.env` ne la contrôle — appeler `.with_allowed_hosts(...)` |
| **Cache control** | ✅ Activé | No-cache en développement ; `.with_cache(true)` le désactive |
| **Static files** | ✅ Activés | Servis sous `STATIC_URL` et `MEDIA_URL` ; `.static_files(\|s\| s.enabled(false))` pour désactiver (fichiers servis par Nginx, par exemple) |
| **URL publique** | — | Obligatoire en production si le reset ou l'admin est activé — `.with_public_url(...)` |
| **Hot reload admin** | Selon `RUNIQUE_ENV` | Automatique via `is_dev()` |
| **Logs framework** | ❌ Désactivés | Activer via `.with_log(\|l\| ...)` |

---

## Voir aussi

| Section | Description |
| --- | --- |
| [Variables d'environnement](/docs/fr/env) | Toutes les variables `.env` |
| [Accès dans le code](/docs/fr/configuration/code) | `RuniqueConfig`, validation |

## Retour au sommaire

- [Configuration](/docs/fr/configuration)
