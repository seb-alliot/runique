# 🛡️ Middleware & Sécurité

## Vue d'ensemble

Runique intègre des middlewares de sécurité configurables. Le **Builder Intelligent** les applique automatiquement dans l'ordre optimal grâce au système de slots.

---

## Stack Middleware (ordre d'exécution)

```
Requête entrante
    ↓
1. Extensions (slot 0)     → Injection Engine, Tera, Config
2. ErrorHandler (slot 10)  → Capture et rendu des erreurs
3. Custom (slot 20+)       → Vos middlewares personnalisés
4. CSP (slot 30)           → Content Security Policy & headers
5. Cache (slot 40)         → No-cache en développement
6. Session (slot 50)       → Gestion des sessions (MemoryStore par défaut)
7. CSRF (slot 60)          → Protection Cross-Site Request Forgery
8. Host (slot 70)          → Validation des hosts autorisés
    ↓
Handler (votre code)
    ↓
Réponse sortante (middlewares en sens inverse)
```

> 💡 Avec Axum, le dernier `.layer()` est le premier exécuté sur la requête. Le Builder Intelligent gère cet ordre automatiquement via les slots.

---

## Protection CSRF

### Fonctionnement

- Token généré **automatiquement** pour chaque session
- Pattern **Double Submit Cookie** (cookie + champ hidden)
- Vérifié sur les requêtes POST, PUT, PATCH, DELETE
- Ignoré sur les requêtes GET, HEAD, OPTIONS

### Dans les formulaires Runique

Quand vous utilisez `{% form.xxx %}`, le CSRF est **inclus automatiquement**. Pas besoin de l'ajouter manuellement.

### Dans les formulaires HTML manuels

```html
<form method="post" action="/submit">
    {% csrf %}
    <input type="text" name="data">
    <button type="submit">Envoyer</button>
</form>
```

### Pour les requêtes AJAX

```javascript
const csrfToken = document.querySelector('[name="csrf_token"]').value;

fetch('/api/endpoint', {
    method: 'POST',
    headers: {
        'Content-Type': 'application/json',
        'X-CSRF-Token': csrfToken
    },
    body: JSON.stringify(data)
});
```

---

## Content Security Policy (CSP)

### Fonctionnement

- **Nonce** généré automatiquement par requête
- Injecté dans le contexte Tera sous `csp_nonce`
- Headers CSP ajoutés à chaque réponse

### Usage dans les templates

```html
<!-- Scripts inline sécurisés -->
<script {% csp_nonce %}>
    console.log("Script avec nonce CSP");
</script>

<!-- Ou avec la variable directement -->
<script nonce="{{ csp_nonce }}">
    console.log("Alternative");
</script>
```

### Profils CSP

| Profil | Description |
|--------|-------------|
| `CspConfig::strict()` | Politique stricte (production) |
| `CspConfig::permissive()` | Politique permissive (développement) |
| `CspConfig::default()` | Profil par défaut |

---

## Validation des Hosts (Allowed Hosts)

### Fonctionnement

- Compare le header `Host` de la requête contre `ALLOWED_HOSTS`
- Bloque les requêtes avec un host non autorisé (HTTP 400)
- Protection contre les attaques Host Header Injection

### Configuration `.env`

```env
# Hosts autorisés (séparés par des virgules)
ALLOWED_HOSTS=localhost,127.0.0.1,example.com

# Patterns supportés :
# localhost       → match exact
# .example.com   → match example.com ET *.example.com
# *              → TOUS les hosts (⚠️ DANGEREUX en production !)
```

### Mode debug

En `DEBUG=true`, la validation des hosts est **désactivée par défaut** pour faciliter le développement.

---

## Cache-Control

### Mode développement (`DEBUG=true`)

Headers `no-cache` ajoutés pour forcer le rechargement :
```
Cache-Control: no-cache, no-store, must-revalidate
Pragma: no-cache
```

### Mode production (`DEBUG=false`)

Headers de cache activés pour les performances.

---

## Headers de sécurité

Runique injecte automatiquement des headers de sécurité standards :

| Header | Valeur | Protection |
|--------|--------|------------|
| `X-Content-Type-Options` | `nosniff` | Empêche le MIME sniffing |
| `X-Frame-Options` | `DENY` | Empêche le clickjacking |
| `X-XSS-Protection` | `1; mode=block` | Protection XSS navigateur |
| `Referrer-Policy` | `strict-origin-when-cross-origin` | Limite les referrers |
| `Content-Security-Policy` | Dynamique (avec nonce) | CSP |

---

## Sessions

### Store par défaut

Runique utilise `MemoryStore` par défaut (données en mémoire, perdues au redémarrage).

### Configuration

```rust
// Durée de session personnalisée
let app = RuniqueApp::builder(config)
    .with_session_duration(time::Duration::hours(2))
    .build()
    .await?;
```

### Durées de session

| Durée | Usage |
|-------|-------|
| `Duration::minutes(30)` | Sessions courtes (sécurité) |
| `Duration::hours(2)` | Usage standard |
| `Duration::hours(24)` | Défaut Runique |
| `Duration::days(7)` | "Se souvenir de moi" |

### Store personnalisé (production)

```rust
use tower_sessions::MemoryStore;

let app = RuniqueApp::builder(config)
    .with_session_store(MemoryStore::default())
    .build()
    .await?;
```

### Accès à la session dans les handlers

```rust
pub async fn dashboard(request: Request) -> AppResult<Response> {
    // Lire une valeur de session
    let user_id: Option<i32> = request.session
        .get("user_id")
        .await
        .ok()
        .flatten();

    // Écrire une valeur
    let _ = request.session.insert("last_visit", "2026-02-06").await;

    // ...
}
```

---

## Configuration du Builder

### Builder classique

```rust
let app = RuniqueApp::builder(config)
    .routes(url::routes())
    .with_database(db)
    .with_error_handler(true)   // Capture des erreurs
    .with_csp(true)             // CSP & headers sécurité
    .with_allowed_hosts(true)   // Validation des hosts
    .with_cache(true)           // No-cache en dev
    .with_static_files()        // Service fichiers statiques
    .build()
    .await?;
```

### Builder Intelligent (nouveau)

```rust
use runique::app::RuniqueAppBuilder as IntelligentBuilder;

let app = IntelligentBuilder::new(config)
    .routes(url::routes())
    .with_database(db)
    .statics()                  // Active les fichiers statiques
    .build()
    .await?;
```

Le Builder Intelligent :
- Applique **automatiquement** les middlewares dans l'ordre correct (slots)
- Utilise le **profil debug** pour les valeurs par défaut (permissif en dev, strict en prod)
- Permet la **personnalisation** via `middleware(|m| { ... })`

### Personnaliser les middlewares

```rust
let app = IntelligentBuilder::new(config)
    .routes(url::routes())
    .with_database(db)
    .middleware(|m| {
        m.disable_csp();           // Désactiver CSP
        m.disable_host_validation(); // Désactiver la validation des hosts
    })
    .build()
    .await?;
```

---

## Variables d'environnement liées à la sécurité

| Variable | Défaut | Description |
|----------|--------|-------------|
| `SECRETE_KEY` | *(requis)* | Clé secrète pour le CSRF |
| `ALLOWED_HOSTS` | `*` | Hosts autorisés |
| `DEBUG` | `true` | Mode debug (affecte CSP, cache, hosts) |
| `RUNIQUE_ENABLE_CSP` | *(auto)* | Force l'activation/désactivation CSP |
| `RUNIQUE_ENABLE_HOST_VALIDATION` | *(auto)* | Force la validation des hosts |
| `RUNIQUE_ENABLE_CACHE` | *(auto)* | Force le contrôle cache |

> En mode debug, les middlewares de sécurité sont permissifs par défaut. Les variables `RUNIQUE_ENABLE_*` permettent de forcer un comportement spécifique indépendamment du mode.

---

## Prochaines étapes

← [**ORM & Database**](https://github.com/seb-alliot/runique/blob/main/docs/fr/07-orm.md) | [**Flash Messages**](https://github.com/seb-alliot/runique/blob/main/docs/fr/09-flash-messages.md) →