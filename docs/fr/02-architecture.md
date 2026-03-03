# 🏗️ Architecture

## Vue d'ensemble

Runique est organisée en **modules fonctionnels** basés sur la responsabilité :

```
runique/src/
├── app/                    #  App Builder, Templates & Builder Intelligent
│   ├── builder.rs          #  RuniqueAppBuilder avec slots
│   ├── error_build.rs      #  Erreurs de build
│   ├── templates.rs        #  TemplateLoader (Tera)
│   └── staging/            #  Staging structs
│       ├── core_staging.rs
│   │   ├── middleware_staging.rs
│   │   └── static_staging.rs
│   └── error_build.rs      #  BuildError & CheckReport
├── config/                 #  Configuration & Settings
├── context/                #  Request Context & Tera tools
│   ├── request.rs          #  Struct Request (extracteur)
│   └── tera/               #  Filtres et fonctions Tera
├── db/                     #  ORM & Database
├── engine/                 #  RuniqueEngine
├── errors/                 #  Gestion des erreurs
├── flash/                  #  Messages flash
├── forms/                  #  Système de formulaires
│   └── prisme/             #  Pipeline sécurité (Sentinel, Aegis, CSRF Gate)
├── macros/                 #  Macros utilitaires
│   ├── context_macro/      #  context!, context_update!
│   ├── flash_message/      #  success!, error!, info!, warning!, flash_now!
│   └── router/             #  urlpatterns!, view!, impl_objects!
├── middleware/             #  Middleware (Sécurité)
│   └── security/           #  CSRF, CSP, Host, Cache, Error Handler
├── utils/                  #  Utilitaires
├── lib.rs
└── prelude.rs
```

---

## Concepts Clés

### 1. RuniqueEngine

**État principal** partagé de l'application :

```rust
pub struct RuniqueEngine {
    pub db: Arc<DatabaseConnection>,
    pub tera: Arc<Tera>,
    pub config: Arc<RuniqueConfig>,
}
```

Injecté comme Extension Axum, accessible dans chaque handler via `request.engine`.

### 2. Request — L'extracteur principal

`Request` est l'extracteur central de Runique. Il remplace l'ancien `TemplateContext` et contient tout le nécessaire :

```rust
pub struct Request {
    pub engine: AEngine,       // Arc<RuniqueEngine>
    pub session: Session,      // Session tower-sessions
    pub notices: Message,      // Flash messages
    pub csrf_token: CsrfToken, // Token CSRF
    pub context: Context,      // Contexte Tera
    pub method: Method,        // Méthode HTTP
}
```

**Usage dans un handler :**

```rust
pub async fn index(mut request: Request) -> AppResult<Response> {
    context_update!(request => {
        "title" => "Accueil",
    });
    request.render("index.html")
}
```

**Méthodes :**
- `request.render("template.html")` — Rendu avec le contexte courant
- `request.is_get()` / `request.is_post()` — Vérification de la méthode HTTP

### 3. Prisme<T> — Extracteur de formulaire

```rust
pub async fn handler(
    mut request: Request,
    Prisme(mut form): Prisme<RegisterForm>,
) -> AppResult<Response> {
    if request.is_post() && form.is_valid().await {
        let user = form.save(&request.engine.db).await?;
        success!(request.notices => "Utilisateur créé !");
        return Ok(Redirect::to("/").into_response());
    }

    context_update!(request => {
        "form" => &form,
    });
    request.render("form.html")
}
```

Pipeline automatique en 4 étapes :
1. **Sentinel** — Vérifie les règles d'accès (login, rôles) via `GuardRules`
2. **Aegis** — Lit le body une seule fois (multipart, urlencoded, json)
3. **CSRF Gate** — Vérifie le token CSRF dans les données parsées
4. **Construction** — Crée `T`, remplit les champs, prêt pour `is_valid()`

---

## Macros Rust

Runique fournit un ensemble de macros pour simplifier le développement :

### Macros de contexte

| Macro | Description | Exemple |
|-------|-------------|---------|
| `context!` | Créer un contexte Tera | `context!("title" => "Page")` |
| `context_update!` | Ajouter au contexte d'une Request | `context_update!(request => { "key" => value })` |

### Macros flash messages

| Macro | Description | Exemple |
|-------|-------------|---------|
| `success!` | Message de succès (session) | `success!(request.notices => "OK !")` |
| `error!` | Message d'erreur (session) | `error!(request.notices => "Erreur")` |
| `info!` | Message info (session) | `info!(request.notices => "Info")` |
| `warning!` | Avertissement (session) | `warning!(request.notices => "Attention")` |
| `flash_now!` | Message immédiat (sans session) | `flash_now!(error => "Erreurs")` |

### Macros de routage

| Macro | Description | Exemple |
|-------|-------------|---------|
| `urlpatterns!` | Définir des routes avec noms | `urlpatterns!("/" => view!{...}, name = "index")` |
| `view!` | Handler pour toutes méthodes HTTP | `view!{ handler }` |
| `impl_objects!` | Manager Django-like pour SeaORM | `impl_objects!(Entity)` |

### Macros d'erreur

| Macro | Description |
|-------|-------------|
| `impl_from_error!` | Génère `From<Error>` pour `AppError` |

---

## Tags et filtres Tera

### Tags Django-like (syntaxe sucrée)

| Tag | Transformé en | Description |
|-----|---------------|-------------|
| `{% static "..." %}` | `{{ "..." \| static }}` | URL d'un fichier statique |
| `{% media "..." %}` | `{{ "..." \| media }}` | URL d'un fichier média |
| `{% csrf %}` | `{% include "csrf/..." %}` | Champ CSRF caché |
| `{% messages %}` | `{% include "message/..." %}` | Affichage messages flash |
| `{% csp_nonce %}` | `{% include "csp/..." %}` | Attribut nonce CSP |
| `{% link "name" %}` | `{{ link(link='name') }}` | URL d'une route nommée |
| `{% form.xxx %}` | `{{ xxx \| form \| safe }}` | Rendu formulaire complet |
| `{% form.xxx.field %}` | `{{ xxx \| form(field='field') \| safe }}` | Rendu d'un champ |

### Filtres Tera

| Filtre | Description |
|--------|-------------|
| `static` | Préfixe URL statique de l'app |
| `media` | Préfixe URL média de l'app |
| `runique_static` | Assets statiques internes au framework |
| `runique_media` | Médias internes au framework |
| `form` | Rendu de formulaire complet ou par champ |
| `csrf_field` | Génère un input hidden CSRF |

### Fonctions Tera

| Fonction | Description |
|----------|-------------|
| `csrf()` | Génère un champ CSRF depuis le contexte |
| `nonce()` | Retourne le nonce CSP |
| `link(link='...')` | Résolution d'URL nommée |

---

## Stack Middleware

Runique applique les middlewares dans un **ordre optimal** via le système de slots :

```
Requête entrante
    ↓
1. Extensions (slot 0)     → Injection Tera, Config, Engine
2. ErrorHandler (slot 10)  → Capture et rendu des erreurs
3. Custom (slot 20+)       → Middlewares personnalisés
4. CSP (slot 30)           → Content Security Policy & headers
5. Cache (slot 40)         → No-cache en développement
6. Session (slot 50)       → Gestion des sessions
7. CSRF (slot 60)          → Protection CSRF
8. Host (slot 70)          → Validation Allowed Hosts
    ↓
Handler (votre code)
    ↓
Réponse sortante (middlewares en sens inverse)
```

> 💡 **Important** : Avec Axum, le dernier `.layer()` appliqué est le premier exécuté. Le Builder Intelligent gère cet ordre automatiquement.

---

## Injection de dépendances

Via les **Extensions Axum**, injectées automatiquement par le middleware Extensions :

```rust
// Enregistré automatiquement par le builder :
// Extension(engine)  → Arc<RuniqueEngine>
// Extension(tera)    → Arc<Tera>
// Extension(config)  → Arc<RuniqueConfig>

// Accessible dans les handlers via Request :
pub async fn handler(request: Request) -> AppResult<Response> {
    let db = request.engine.db.clone();
    let config = &request.engine.config;
    // ...
}
```

---

## Lifecycle d'une requête

```
1. Requête HTTP arrive
2. Middlewares traversés (order des slots)
3. Extensions injectées (Engine, Tera, Config)
4. Session chargée, CSRF vérifié
5. Handler appelé avec extracteurs (Request, Prisme<T>)
6. Handler retourne AppResult<Response>
7. Middlewares traversés en sens inverse
8. Réponse HTTP envoyée
```

---

## Bonnes Pratiques

1. **Cloner les Arc :**
   ```rust
   let db = request.engine.db.clone();
   ```

2. **Formulaires = copies par requête :**
   ```rust
   Prisme(mut form): Prisme<MyForm>
   // Chaque requête = formulaire isolé, zéro concurrence
   ```

3. **context_update! pour le contexte :**
   ```rust
   context_update!(request => {
       "title" => "Ma page",
       "data" => &my_data,
   });
   ```

4. **Flash messages pour les redirections :**
   ```rust
   success!(request.notices => "Action réussie !");
   return Ok(Redirect::to("/").into_response());
   ```

5. **flash_now! pour les rendus directs :**
   ```rust
   context_update!(request => {
       "messages" => flash_now!(error => "Erreur de validation"),
   });
   ```

---

## Prochaines étapes

← [Installation](https://github.com/seb-alliot/runique/blob/main/docs/fr/01-installation.md) | [**Configuration**](https://github.com/seb-alliot/runique/blob/main/docs/fr/03-configuration.md) →
