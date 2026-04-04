# Middlewares — Pièges et Solutions
> Retours d'expérience réels de la construction de Runique — ce que les frameworks haut niveau ne te disent pas

## Objectifs

- Comprendre pourquoi l'ordre de déclaration des middlewares est critique
- Identifier les pièges classiques rencontrés en construisant un framework web
- Comprendre pourquoi le body HTTP ne peut être lu qu'une seule fois
- Connaître les solutions adoptées dans Runique face à ces contraintes

---

## Table des matières

1. [L'ordre inversé d'Axum — le piège du builder](#1-lordre-inversé-daxum--le-piège-du-builder)
2. [Session avant CSRF — le bug silencieux](#2-session-avant-csrf--le-bug-silencieux)
3. [Le body HTTP ne peut être lu qu'une seule fois](#3-le-body-http-ne-peut-être-lu-quune-seule-fois)
4. [Les formulaires doivent être déclarés en GET](#4-les-formulaires-doivent-être-déclarés-en-get)
5. [Middlewares actifs sur toutes les routes](#5-middlewares-actifs-sur-toutes-les-routes)
6. [Fausse route et redirection admin/login](#6-fausse-route-et-redirection-adminlogin)
7. [Flash messages sur render vs redirect](#7-flash-messages-sur-render-vs-redirect)

---

## 1. L'ordre inversé d'Axum — le piège du builder

### Le comportement contre-intuitif

Quand tu empiles des middlewares avec Axum via `.layer()`, l'ordre d'exécution est **l'inverse** de l'ordre de déclaration. Le dernier `.layer()` appliqué est le premier exécuté sur la requête entrante.

```rust
// ❌ Ce qu'on écrit
Router::new()
    .layer(SessionLayer::new())     // déclaré en 1er
    .layer(CsrfLayer::new())        // déclaré en 2ème
    .layer(CompressionLayer::new()) // déclaré en 3ème
    .layer(ErrorHandlerLayer::new()) // déclaré en 4ème
```

```
// Ce qui s'exécute réellement sur la requête entrante :
ErrorHandler → Compression → CSRF → Session → Handler
```

La requête traverse les couches **de la dernière déclarée vers la première**. C'est le modèle en oignon de Tower — chaque `.layer()` enveloppe ce qui précède.

### Le bug en production

Ce comportement a causé un bug réel lors de la construction de Runique : le CSRF se retrouvait exécuté **avant** la session, cherchait un token en session inexistante, et rejetait toutes les requêtes POST avec une erreur 403 silencieuse.

> **Important :** Le bug ne se manifeste pas toujours immédiatement. Si le CSRF est désactivé en développement, il peut passer en production sans que le problème ait jamais été visible.

### La solution Runique — le système de slots

Au lieu de dépendre de l'ordre de déclaration, Runique attribue un **slot numéroté fixe** à chaque middleware. Au moment du build, tous les middlewares sont triés par slot et appliqués dans le bon ordre — automatiquement, indépendamment de l'ordre de déclaration du développeur.

```
Slots d'exécution (requête entrante) :
Extensions(0) → Compression(5) → ErrorHandler(10) → Custom(20+)
→ CSP/Headers(30) → Cache(40) → Session(50) → SessionUpgrade(55)
→ CSRF(60) → HostValidation(70) → Handler
```

```rust
// ✅ Avec Runique — l'ordre de déclaration n'a aucune importance
RuniqueApp::builder(config)
    .middleware(|m| {
        m.with_csp(|c| c.policy(SecurityPolicy::strict()))  // slot 30
         .with_allowed_hosts(|h| h.host("monsite.fr"))      // slot 70
         .with_session_memory_limit(5 * 1024 * 1024, 10 * 1024 * 1024) // slot 50
    })
```

Peu importe l'ordre dans le builder, Session sera toujours avant CSRF. Le framework impose la correction structurellement.

---

## 2. Session avant CSRF — le bug silencieux

### Pourquoi le CSRF dépend de la session

La protection CSRF fonctionne en générant un token secret stocké en session, que chaque formulaire doit renvoyer. Le middleware CSRF doit lire ce token en session pour valider la requête.

**Sans session initialisée, le CSRF ne peut pas fonctionner.**

### Le bug

Si CSRF s'exécute avant Session :

```
Requête → CSRF(cherche token en session) → Session(initialise) → Handler
```

Le CSRF tente de lire le token... la session n'existe pas encore. Il ne trouve rien, considère chaque requête invalide, et retourne systématiquement un 403.

Le développeur voit toutes ses requêtes POST rejetées sans message d'erreur clair. L'application est inutilisable, et la cause n'est pas évidente.

### La correction dans Runique

Le bug a été découvert et corrigé en imposant des slots fixes :

```
Session(50) → SessionUpgrade(55) → CSRF(60)
```

Le CSRF est désormais **non configurable** — il s'exécute toujours après la session, et le développeur ne peut pas changer ça même par erreur.

```rust
// Le CSRF est toujours activé, toujours au bon slot
// Aucune configuration requise, aucune erreur possible
RuniqueApp::builder(config)
    .middleware(|m| {
        m.with_session_memory_limit(5 * 1024 * 1024, 10 * 1024 * 1024)
        // CSRF garanti au slot 60, après Session(50) — automatique
    })
```

---

## 3. Le body HTTP ne peut être lu qu'une seule fois

### Pourquoi cette contrainte existe

Le body d'une requête HTTP est un **flux de données** (`Stream`), pas un tableau en mémoire. Les octets arrivent depuis le réseau et sont consommés au fil de la lecture. Il n'y a pas de curseur de retour — une fois lus, ils sont perdus.

```rust
// ❌ Impossible en Rust/Axum
async fn handler(req: Request) -> Response {
    let body1 = axum::body::to_bytes(req.into_body(), usize::MAX).await;
    // req est consommé — body1 contient les données
    // Il n'existe plus de body à lire
}
```

### Le problème avec les middlewares

Si un middleware lit le body (pour logger, valider, parser...), le handler en aval reçoit un body vide.

```
Requête → [Middleware lit le body] → Handler (body vide !)
```

C'est une contrainte du protocole HTTP et de la gestion mémoire Rust — pas un bug corrigeable.

### Solution haut niveau — le buffering

Les frameworks comme Django, Rails ou Express résolvent ça en chargeant **tout le body en mémoire** dès la réception de la requête.

```python
# Django — le body est toujours disponible
def ma_vue(request):
    data1 = request.body  # Disponible
    data2 = request.body  # Toujours disponible — Django a tout bufferisé
```

**Avantage :** simplicité totale.
**Inconvénient :** tout le body est en RAM, même pour un fichier de 500 Mo. Le streaming devient impossible.

### Solution Runique — le relais typé (Prisme)

Runique ne bufferise pas. Le body n'est lu **qu'une seule fois**, directement dans le handler via l'extracteur `Prisme`. Les middlewares en amont ne touchent jamais au body.

```rust
// ✅ Prisme consomme le body une seule fois, au bon endroit
pub async fn login_user(
    mut request: Request,
    Prisme(form): Prisme<LoginForm>, // lecture unique ici
) -> AppResult<Response> {
    // form contient les données parsées
    // Aucun middleware n'a touché au body avant
}
```

Les données dont les middlewares auraient besoin sont transmises via le système d'extensions de la requête — un relais typé en mémoire, pas une relecture du flux réseau.

### Conséquence sur les champs password

`Forms::fill()` ne peut pas remplir les champs `password` automatiquement — ils ne transitent pas par le système de relais (pour des raisons de sécurité). Ils s'utilisent via `add_value()` directement depuis les données de Prisme.

```rust
// ✅ Champs normaux
form.fill(&model);

// ✅ Champs password — directement depuis Prisme
form.add_value("password", &prisme_value);
```

---

## 4. Les formulaires doivent être déclarés en GET

### Le piège

En Runique, un formulaire HTML doit être **initialisé et rendu dans un handler GET** avant de pouvoir être soumis en POST. Il est tentant de ne déclarer que le handler POST et de construire le formulaire directement dedans.

```rust
// ❌ Tentant mais incorrect — pas de rendu initial
pub async fn login_user(
    mut request: Request,
    Prisme(form): Prisme<LoginForm>,
) -> AppResult<Response> {
    // Pas de GET handler → le formulaire n'a jamais été rendu
    // Les champs, erreurs et tokens CSRF n'existent pas côté client
}
```

### Pourquoi c'est nécessaire

Le GET sert à :
1. **Injecter le token CSRF** dans le formulaire HTML
2. **Rendre les champs vides** avec leur configuration (labels, types, validation)
3. **Afficher les erreurs** lors d'une soumission invalide (re-render du GET avec erreurs)

```rust
// ✅ Pattern correct — GET pour afficher, POST pour traiter
pub async fn login_page(mut request: Request) -> AppResult<Response> {
    let form = LoginForm::new();
    context_update!(request => { "form" => &form });
    request.render("auth/login.html")
}

pub async fn login_user(
    mut request: Request,
    Prisme(form): Prisme<LoginForm>,
) -> AppResult<Response> {
    // Traitement du POST
}
```

---

## 5. Middlewares actifs sur toutes les routes

### La contrainte

Dans Runique, les middlewares configurés via le builder s'appliquent à **toutes les routes** de l'application. Il n'y a pas de middleware conditionnel par groupe de routes dans le builder.

```rust
// Le rate limiter s'applique à TOUTES les routes
RuniqueApp::builder(config)
    .middleware(|m| {
        m.with_session_memory_limit(...)
    })
```

### Pourquoi ce choix

Les middlewares de sécurité (CSRF, Session, CSP, Host Validation) doivent s'appliquer universellement — une exception est une faille potentielle. Imposer cette contrainte structurellement empêche les oublis.

### La solution pour les cas particuliers

Pour un middleware sur une route spécifique, on utilise `route_layer` directement sur le Router, en dehors du builder :

```rust
let limiter = Arc::new(RateLimiter::new().max_requests(5).retry_after(60));

let upload_route = Router::new()
    .route("/upload", view!(upload_handler))
    .route_layer(middleware::from_fn_with_state(
        limiter,
        rate_limit_middleware,
    ));
```

Le builder gère la sécurité globale, `route_layer` gère les besoins spécifiques.

---

## 6. Fausse route et redirection admin/login

### Le bug

Lors du développement de Runique, une URL inexistante ne retournait pas une page 404 — elle redirectionnait systématiquement vers `/admin/login`.

```
GET /une-page-qui-nexiste-pas → 302 redirect → /admin/login
```

### La cause

Le middleware d'authentification admin interceptait **toutes les requêtes non authentifiées**, y compris les 404. Avant que le router puisse retourner une page d'erreur, le middleware admin court-circuitait la chaîne et redirigait.

L'ordre d'exécution était :

```
Requête → AuthAdmin (redirige si non auth) → Router (jamais atteint)
```

### La correction

Le middleware admin ne doit intercepter que les routes `/admin/*`, pas toutes les routes. La solution est de l'appliquer uniquement sur le groupe de routes admin via `route_layer`, et non comme middleware global.

```rust
// ✅ Middleware admin isolé sur son périmètre
let admin_router = Router::new()
    .route("/admin/*path", view!(admin_handler))
    .route_layer(middleware::from_fn(auth_admin_middleware));
```

Les routes publiques ne sont plus interceptées, les 404 remontent correctement vers l'ErrorHandler.

---

## 7. Flash messages sur render vs redirect

### Le comportement attendu

Les flash messages sont conçus pour **survivre à un redirect** : stockés en session à la fin d'un handler, affichés à la requête suivante après la redirection.

```rust
// Cas nominal — redirect après action
warning!(request.notices => "Mot de passe incorrect.");
return Ok(Redirect::to("/login").into_response());
// → message stocké en session → affiché sur /login
```

### Le piège — render sans redirect

Quand on fait un `render` direct (sans redirect), les flash messages sont déjà consommés ou ne s'affichent pas correctement — ils ont été conçus pour la prochaine requête, pas pour la réponse actuelle.

```rust
// ❌ Le message peut ne pas s'afficher
warning!(request.notices => "Formulaire invalide.");
return request.render("forms/login.html"); // render direct, pas de redirect
```

### La solution — flash_now

Runique introduit `flash_now` pour injecter un message directement dans le contexte du render courant, sans passer par la session.

```rust
// ✅ flash_now — affiché immédiatement dans le render
flash_now!(request => warning "Formulaire invalide.");
return request.render("forms/login.html");
```

### Le problème ouvert

La vraie solution serait une détection automatique : si la réponse est un redirect, stocker en session ; si c'est un render, injecter directement. Mais pour ça il faudrait inspecter la réponse *après* que le handler l'a produite — ce qui ramène au problème de lecture unique et de la chaîne middleware. `flash_now` reste un contournement explicite en attendant.

---
