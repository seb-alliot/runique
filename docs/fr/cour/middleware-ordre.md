# Middlewares — Pièges et Solutions
> Retours d'expérience réels de la construction de Runique : l'ordre des couches, le corps de requête qu'on ne lit qu'une fois, les formulaires, les flash messages, et ce que les frameworks de haut niveau ne disent pas.

## Objectifs

- Comprendre le modèle en oignon de Tower et pourquoi l'ordre des couches est critique
- Connaître les différentes façons d'empiler des couches avec axum, et leur sens
- Comprendre comment Runique impose l'ordre avec un système de slots
- Comprendre pourquoi le corps HTTP ne peut être lu qu'une seule fois, et comment Runique s'en sert
- Distinguer `layer` et `route_layer`, et savoir où placer un middleware propre à certaines routes
- Écrire un middleware avec `from_fn`
- Choisir entre un flash message et un message affiché tout de suite

## Table des matières

1. Le modèle en oignon
2. L'ordre des couches avec axum : les différentes écritures
3. La solution Runique : les slots
4. Session avant CSRF : le bug silencieux
5. Écrire un middleware
6. Le corps HTTP ne peut être lu qu'une seule fois
7. Formulaires : afficher en GET, traiter en POST
8. Middlewares globaux et middlewares de route
9. Fausse route et redirection vers la connexion admin
10. Flash messages : redirection ou rendu direct
11. Exercices
12. Aide-mémoire

---

## 1. Le modèle en oignon

Un middleware (une « couche », *layer* dans Tower) **enveloppe** le service suivant. La requête traverse les couches de l'extérieur vers l'intérieur, la réponse les retraverse dans l'autre sens.

```
requête  ──►  ┌ ErrorHandler ┌ Session ┌ CSRF ┌ handler ┐ CSRF ┐ Session ┐ ErrorHandler ┐  ──► réponse
              └──────────────┴─────────┴──────┴─────────┘
              extérieur                                            intérieur
```

Une couche peut :

- agir **avant** : lire les en-têtes, refuser la requête (403, 413, 429), ajouter des données ;
- agir **après** : modifier la réponse (en-têtes de sécurité, compression), attraper une erreur ;
- ou les deux (mesurer la durée, ouvrir puis enregistrer une session).

Conséquence : une couche ne peut utiliser que ce qu'ont préparé les couches **plus extérieures**. Le CSRF lit la session : la session doit être **à l'extérieur** du CSRF.

---

## 2. L'ordre des couches avec axum : les différentes écritures

### 2.1 `.layer()` successifs : le dernier est le plus extérieur

```rust
use axum::Router;

let app = Router::new()
    .route("/", axum::routing::get(handler))
    .layer(couche_csrf)          // ajoutée en 1er : la plus INTÉRIEURE
    .layer(couche_session)       // enveloppe la précédente
    .layer(couche_erreurs);      // ajoutée en dernier : la plus EXTÉRIEURE

// Requête : erreurs -> session -> csrf -> handler
```

Chaque `.layer()` enveloppe **tout ce qui a été construit avant**. On lit donc la pile de bas en haut. C'est le piège classique : en lisant de haut en bas, on croit que le CSRF passe en premier.

### 2.2 `ServiceBuilder` : de haut en bas

```rust
use tower::ServiceBuilder;

let app = Router::new()
    .route("/", axum::routing::get(handler))
    .layer(
        ServiceBuilder::new()
            .layer(couche_erreurs)   // la plus extérieure
            .layer(couche_session)
            .layer(couche_csrf),     // la plus intérieure
    );

// Requête : erreurs -> session -> csrf -> handler
```

Le même ordre d'exécution, écrit dans l'ordre de lecture. Les deux formes sont correctes ; mélanger les deux dans un même projet est la meilleure façon de se tromper.

| Écriture | Se lit | Première couche traversée |
|---|---|---|
| `.layer(a).layer(b).layer(c)` | de bas en haut | `c` |
| `ServiceBuilder::new().layer(a).layer(b).layer(c)` | de haut en bas | `a` |

### 2.3 Le bug réel

Ce piège a causé un bug pendant la construction de Runique : le CSRF se retrouvait **à l'extérieur** de la session. Il cherchait un jeton dans une session pas encore chargée, et refusait toutes les requêtes POST en 403, sans message clair.

> Le bug ne se voit pas toujours tout de suite : si le CSRF est désactivé ou contourné en développement, il peut arriver en production sans avoir jamais été visible.

---

## 3. La solution Runique : les slots

Plutôt que de dépendre de l'ordre d'écriture, Runique attribue à chaque middleware un **numéro de slot fixe**. Au moment du build, les couches sont triées par slot et appliquées dans le bon ordre, quel que soit l'ordre dans lequel le développeur les configure.

```rust
// runique/src/app/staging/middleware_staging/applicator.rs (extrait)
const SLOT_EXTENSIONS: u16 = 0;        // Engine/Tera/Config injection (outermost)
const SLOT_TRUSTED_PROXIES: u16 = 2;   // Real client IP extraction — before everything
const SLOT_COMPRESSION: u16 = 5;
const SLOT_CORS: u16 = 8;              // Outside ErrorHandler — OPTIONS preflight never reaches CSRF
const SLOT_ERROR_HANDLER: u16 = 10;    // Catches errors of the WHOLE stack
const SLOT_HOST_VALIDATION: u16 = 15;  // After ErrorHandler, before Session/Auth/CSRF cost
const SLOT_HTTPS_REDIRECT: u16 = 17;   // After HostValidation: the redirect is built from a checked Host
const SLOT_CUSTOM_BASE: u16 = 20;      // Dev's custom middlewares start here
const SLOT_OPEN_REDIRECT: u16 = 25;
const SLOT_SECURITY_HEADERS: u16 = 30;
const SLOT_CACHE: u16 = 40;
const SLOT_SESSION: u16 = 50;          // Before CSRF (CSRF depends on it)
const SLOT_SESSION_UPGRADE: u16 = 55;
const SLOT_AUTH: u16 = 57;             // After Session — reads the signed-in account from the DB
const SLOT_CSRF: u16 = 60;             // After Session (reads/writes in session)
const SLOT_ANTI_BOT: u16 = 65;         // After CSRF — injects honeypot field name extension
```

Lecture : un slot **bas** est **extérieur**. La requête traverse dans l'ordre croissant :

```
Extensions(0) → TrustedProxies(2) → Compression(5) → CORS(8) → ErrorHandler(10)
→ HostValidation(15) → HttpsRedirect(17) → Custom(20+) → OpenRedirect(25)
→ CSP/Headers(30) → Cache(40) → Session(50) → SessionUpgrade(55) → Auth(57)
→ CSRF(60) → AntiBot(65) → handler
```

Chaque position a une raison, écrite dans le commentaire :

| Couche | Pourquoi à cet endroit |
|---|---|
| TrustedProxies (2) | la vraie adresse du client doit être connue avant tout ce qui la lit (limiteurs, journaux) |
| CORS (8) | la requête de pré-vérification `OPTIONS` du navigateur ne doit jamais atteindre le CSRF |
| ErrorHandler (10) | attrape les erreurs (et les paniques) de toute la pile intérieure |
| HostValidation (15) | refuser un `Host` inconnu **avant** de payer la session, l'authentification et le CSRF |
| HttpsRedirect (17) | la redirection est construite à partir d'un `Host` déjà vérifié : pas de redirection ouverte |
| Session (50) avant Auth (57) et CSRF (60) | les deux lisent la session |

```rust
// L'ordre d'écriture n'a aucune importance
RuniqueAppBuilder::new(config)
    .middleware(|m| {
        m.with_csp(|c| c.policy(SecurityPolicy::strict()))                 // slot 30
            .with_allowed_hosts(|h| h.host("monsite.fr"))                  // slot 15
            .with_session_memory_limit(5 * 1024 * 1024, 10 * 1024 * 1024)   // slot 50
    })
```

Technique : le tri par slot transforme une règle que le développeur devait **retenir** en une règle que le framework **applique**. C'est la même idée que les types de colonnes (une faute devient impossible) ou le typestate du builder (cours `traits-avances`).

---

## 4. Session avant CSRF : le bug silencieux

### 4.1 Pourquoi le CSRF dépend de la session

La protection CSRF repose sur un jeton secret lié à la session, que chaque formulaire doit renvoyer. Le middleware CSRF doit lire ce jeton pour valider une requête POST.

### 4.2 Le bug

```
Requête → CSRF (cherche le jeton dans la session) → Session (chargée trop tard) → handler
```

Le CSRF ne trouve rien, considère chaque requête comme invalide, et renvoie systématiquement 403.

### 4.3 La correction

Les slots fixent `Session(50) → SessionUpgrade(55) → Auth(57) → CSRF(60)`. Le CSRF est **toujours actif** : il n'y a pas de méthode pour le désactiver globalement.

La seule exception prévue est explicite et limitée à des chemins précis, pour les points d'entrée appelés par un service tiers qui signe ses requêtes :

```rust
RuniqueAppBuilder::new(config)
    .middleware(|m| {
        m.csrf_exempt(vec!["/webhook/stripe", "/webhook/github"])
    })
```

Ces routes doivent alors vérifier elles-mêmes la signature du service (en temps constant) : l'exemption remplace une protection par une autre, elle ne la supprime pas.

---

## 5. Écrire un middleware

### 5.1 La même couche écrite de trois façons

Ajouter un en-tête `X-Temps-Reponse` à chaque réponse.

```rust
use axum::{extract::Request, middleware::Next, response::Response};
use std::time::Instant;

// 1. Une fonction async avec from_fn (le plus simple)
async fn temps_reponse(req: Request, next: Next) -> Response {
    let debut = Instant::now();
    let mut reponse = next.run(req).await;                 // avant : rien ; après : on modifie la réponse
    let ms = debut.elapsed().as_millis().to_string();
    if let Ok(v) = ms.parse() {
        reponse.headers_mut().insert("x-temps-reponse", v);
    }
    reponse
}

let app = axum::Router::new()
    .route("/", axum::routing::get(|| async { "ok" }))
    .layer(axum::middleware::from_fn(temps_reponse));

// 2. Avec un état partagé : from_fn_with_state
#[derive(Clone)]
struct Etat { prefixe: &'static str }

async fn avec_etat(
    axum::extract::State(etat): axum::extract::State<Etat>,
    req: Request,
    next: Next,
) -> Response {
    tracing::info!("{} {}", etat.prefixe, req.uri());
    next.run(req).await
}

// .layer(axum::middleware::from_fn_with_state(Etat { prefixe: "web" }, avec_etat))

// 3. Une couche de tower-http toute faite
// .layer(tower_http::timeout::TimeoutLayer::new(Duration::from_secs(10)))
```

| Forme | Pour |
|---|---|
| `from_fn` | logique propre à l'application, quelques lignes |
| `from_fn_with_state` | idem, avec une configuration ou un état (limiteur, base) |
| couches `tower-http` | besoins standard : compression, délais, CORS, en-têtes, traces |
| `impl Layer` + `impl Service` à la main | bibliothèque, contrôle fin des performances |

### 5.2 Refuser avant le handler

```rust
use axum::http::StatusCode;
use axum::response::IntoResponse;

async fn exiger_en_tete(req: Request, next: Next) -> Response {
    if req.headers().get("x-cle-api").is_none() {
        return StatusCode::UNAUTHORIZED.into_response();   // le handler n'est jamais appelé
    }
    next.run(req).await
}
```

Une comparaison de clé secrète dans un middleware se fait en **temps constant** (`subtle::ConstantTimeEq`), jamais avec `==`, pour ne pas révéler par la durée de la réponse combien de caractères sont justes.

### 5.3 Transmettre une donnée au handler

```rust
#[derive(Clone)]
struct IdRequete(String);

async fn poser_id(mut req: Request, next: Next) -> Response {
    let id = IdRequete(format!("{:x}", rand_simple()));
    req.extensions_mut().insert(id);              // déposé dans la requête
    next.run(req).await
}

async fn handler(axum::Extension(id): axum::Extension<IdRequete>) -> String {
    id.0                                          // récupéré par le handler
}

fn rand_simple() -> u64 { 42 }
```

Les **extensions** de la requête sont une table typée (une valeur par type) : c'est le relais entre middlewares et handlers. Runique s'en sert pour transmettre l'engine, la session, le jeton CSRF, l'utilisateur connecté.

### 5.4 Dans Runique : `add_custom`

```rust
RuniqueAppBuilder::new(config)
    .middleware(|m| {
        m.add_custom(|router| router.layer(axum::middleware::from_fn(temps_reponse)))
    })
```

Les middlewares ajoutés ainsi prennent place à partir du slot 20 : **après** les protections d'hôte et la gestion d'erreurs, **avant** la session et le CSRF.

---

## 6. Le corps HTTP ne peut être lu qu'une seule fois

### 6.1 Pourquoi

Le corps d'une requête est un **flux** : les octets arrivent du réseau et sont consommés à la lecture. Il n'y a pas de retour en arrière.

```rust
async fn handler(req: axum::extract::Request) {
    let corps = axum::body::to_bytes(req.into_body(), 2 * 1024 * 1024).await;
    // req a été consommée par into_body : il n'existe plus de corps à lire
}
```

Le système de types le montre : `into_body(self)` **prend** la requête. Impossible de la relire, c'est une erreur de compilation.

### 6.2 Le problème avec les middlewares

Si un middleware lit le corps (pour journaliser, valider...), le handler reçoit un corps vide, sauf à le reconstruire.

### 6.3 Les différentes solutions

| Solution | Principe | Coût |
|---|---|---|
| tout mettre en mémoire dès l'arrivée (Django, Rails, Express) | le corps est disponible partout | tout le corps en RAM, même un fichier de 500 Mo |
| le middleware lit puis **remet** le corps (`Request::from_parts(parts, Body::from(octets))`) | le handler relit les octets | une copie, et il faut limiter la taille |
| **une seule lecture**, au bon endroit, résultat partagé | le corps est analysé une fois et le résultat est transmis | aucun |

### 6.4 La solution Runique

Le corps n'est lu **qu'une seule fois**, par l'extracteur `Request`. C'est pour cela que `Request` est toujours le **dernier paramètre** d'un handler : axum exige que l'extracteur qui consomme le corps soit le dernier.

Les données analysées sont gardées dans la requête (`request.prisme`), et `request.form("nom")` construit le formulaire à partir d'elles, sans relire le flux. Les middlewares ne touchent jamais au corps ; ce dont ils ont besoin passe par les extensions (section 5.3).

```rust
pub async fn connexion(request: Request) -> AppResult<Response> {
    let form: LoginForm = request.form("login");   // rempli depuis les données déjà analysées
    // ...
}
```

Les limites de taille sont vérifiées **avant** et **pendant** cette lecture unique (en-tête `Content-Length`, puis limite pendant la lecture : 2 Mo pour un formulaire, `RUNIQUE_MAX_UPLOAD_MB` par fichier, `RUNIQUE_MAX_BODY_MB` pour la requête entière) : un corps trop gros est refusé en 413 sans avoir été chargé en mémoire.

### 6.5 Les champs mot de passe

À l'affichage (GET), le remplissage ignore les champs `password` : un mot de passe n'est jamais réaffiché dans la page. À la soumission (POST, PUT, PATCH), `request.form("nom")` les remplit comme les autres champs.

`add_value(nom, valeur)` pose une valeur depuis le code (pré-remplissage, colonne que le visiteur ne choisit pas) :

```rust
let mut form: LoginForm = request.form("login");
form.add_value("username", &valeur);
```

---

## 7. Formulaires : afficher en GET, traiter en POST

### 7.1 Le piège

Il est tentant de n'écrire que le handler POST. Mais le formulaire HTML doit d'abord avoir été **affiché** : c'est à ce moment que le jeton CSRF masqué et le nom du formulaire sont insérés dans la page.

```rust
// Seul un POST : la page qui contient le formulaire n'a jamais été rendue par Runique,
// le navigateur n'a donc ni jeton CSRF, ni nom de formulaire à renvoyer.
```

### 7.2 La forme correcte

```rust
// urlpatterns! { "/login" => [get: views::login_page, post: views::login_post], }

pub async fn login_page(mut request: Request) -> AppResult<Response> {
    let form: LoginForm = request.form("login");      // vide : première visite
    context_update!(request => { "form" => &form });
    request.render("auth/login.html")
}

pub async fn login_post(mut request: Request) -> AppResult<Response> {
    let form: LoginForm = request.form("login");      // rempli : c'est sa propre soumission
    // validation, puis redirection ou nouvel affichage avec les erreurs
    context_update!(request => { "form" => &form });
    request.render("auth/login.html")
}
```

### 7.3 Ce que fait `request.form("nom")`

- il construit le formulaire avec le jeton CSRF masqué de la requête ;
- il le remplit **seulement si la requête est sa propre soumission** : le nom revient dans un champ caché. Un autre formulaire de la même page, ou un premier affichage, reste vide : on ne réaffiche jamais ce qui a été tapé dans un autre formulaire, ni son erreur CSRF ;
- il plafonne la taille des fichiers acceptés à la limite du serveur.

Une page peut donc contenir plusieurs formulaires (connexion et inscription, par exemple) sans qu'ils se mélangent, chacun avec son nom.

---

## 8. Middlewares globaux et middlewares de route

### 8.1 `layer` ou `route_layer`

```rust
let app = axum::Router::new()
    .route("/admin/{*chemin}", axum::routing::get(handler))
    .route_layer(axum::middleware::from_fn(garde_admin))   // seulement si une route correspond
    .layer(axum::middleware::from_fn(journal));            // toutes les requêtes, même les 404

async fn handler() {}
async fn garde_admin(req: Request, next: Next) -> Response { next.run(req).await }
async fn journal(req: Request, next: Next) -> Response { next.run(req).await }
```

| | `layer` | `route_layer` |
|---|---|---|
| S'applique à | toutes les requêtes du routeur, y compris celles sans route (404) | seulement les requêtes dont une route correspond |
| Pour | sécurité globale, journaux, compression | authentification d'une zone, limite d'un formulaire |

### 8.2 Les protections de Runique sont globales

Les middlewares configurés par le builder (session, CSRF, CSP, validation de l'hôte...) s'appliquent à **toutes** les routes. C'est voulu : une protection de sécurité qui comporte des exceptions est une faille potentielle, et une route oubliée ne doit pas se retrouver sans protection.

### 8.3 Un middleware pour certaines routes

Pour un besoin propre à une route, Runique fournit des extensions de routeur. Exemple réel, la limite de tentatives sur la suppression de compte de runique.io :

```rust
// demo-app/src/url.rs (extrait)
.rate_limit(
    "/profil/supprimer",
    "supprimer_compte",
    view!(supprimer_compte),
    3,                       // au plus 3 requêtes...
    300,                     // ...par fenêtre de 300 secondes
    vec![Method::POST],      // seules les soumissions comptent
)
```

`rate_limit` crée le limiteur, le pose en `route_layer` sur cette seule route, et lance son nettoyage périodique (cours `concurrence`, section 13). Pour une couche quelconque, `route_layer` sur un `Router` axum reste possible.

---

## 9. Fausse route et redirection vers la connexion admin

### 9.1 Le bug

Pendant le développement, une URL inexistante ne renvoyait pas 404 : elle redirigeait vers `/admin/login`.

```
GET /une-page-qui-nexiste-pas  →  302  →  /admin/login
```

### 9.2 La cause

La garde d'authentification de l'admin était posée avec `layer` sur tout le routeur : elle interceptait **toutes** les requêtes non authentifiées, y compris celles qui ne correspondaient à aucune route (section 8.1), avant que le routeur puisse répondre 404.

```
Requête → garde admin (redirige si non connecté) → routeur (jamais atteint)
```

### 9.3 La correction

La garde ne doit concerner que les routes de l'admin : elle est posée en `route_layer`, sur les seules routes de l'admin.

```rust
let admin = axum::Router::new()
    .route("/admin/{*chemin}", axum::routing::get(handler))
    .route_layer(axum::middleware::from_fn(garde_admin));
```

Dans l'admin actuel, cette garde est `declared_route_access` (`runique/src/admin/middleware/admin_middleware.rs`), posée sur les routes de l'admin seulement. Les routes publiques ne sont plus interceptées, et les 404 remontent normalement jusqu'au gestionnaire d'erreurs.

---

## 10. Flash messages : redirection ou rendu direct

### 10.1 Le fonctionnement prévu

Un flash message est **stocké en session** pour être affiché à la **requête suivante**, typiquement après une redirection.

```rust
success!(request.notices => "Profil mis à jour.");
return Ok(Redirect::to("/profil").into_response());
// stocké en session → affiché sur /profil, puis effacé
```

C'est le schéma « POST, puis redirection, puis GET » : recharger la page après la redirection ne renvoie pas le formulaire une seconde fois.

### 10.2 Le piège : un rendu direct

```rust
warning!(request.notices => "Formulaire invalide.");
return request.render("forms/login.html");   // pas de redirection
// le message est en session : il s'affichera à la PROCHAINE page, pas sur celle-ci
```

### 10.3 La solution : `flash_now!`

```rust
context_update!(request => {
    "messages" => flash_now!(warning => "Formulaire invalide."),
});
return request.render("forms/login.html");
```

`flash_now!` construit directement les messages (`Vec<FlashMessage>`) à placer dans le contexte du rendu en cours, sans passer par la session :

```rust
// runique/src/macros/context/flash.rs (extrait)
macro_rules! flash_now {
    ($msg_type:ident => $content:expr) => {{
        vec![$crate::flash::FlashMessage::$msg_type($content)]
    }};
    // + une forme qui accepte plusieurs messages
}
```

### 10.4 Choisir

| Après l'action | Écriture |
|---|---|
| redirection | `success!` / `info!` / `warning!` / `error!` sur `request.notices` |
| nouvel affichage de la même page (erreurs de formulaire) | `flash_now!` dans le contexte |

### 10.5 La piste écartée

Une détection automatique (stocker en session si la réponse est une redirection, injecter sinon) demanderait d'inspecter la réponse **après** le handler, puis de modifier le rendu déjà produit. Le choix explicite entre deux écritures reste plus simple et plus prévisible.

---

## 11. Exercices

### Exercice 1 — ordre d'exécution

Dans quel ordre la requête traverse-t-elle ces couches ?

```rust
Router::new()
    .route("/", get(h))
    .layer(a)
    .layer(b)
    .layer(ServiceBuilder::new().layer(c).layer(d));
```

Réponse : `c`, `d`, `b`, `a`, puis le handler. Le `ServiceBuilder` est ajouté en dernier, donc il est le plus extérieur, et à l'intérieur de lui, `c` est avant `d`.

### Exercice 2 — middleware de clé d'API

Écris un middleware qui refuse en 401 une requête sans en-tête `x-cle-api` égal à une clé connue, en comparant en temps constant.

```rust
// Solution :
use subtle::ConstantTimeEq;

async fn cle_api(req: Request, next: Next) -> Response {
    const CLE: &[u8] = b"cle-secrete-de-demo";
    let fournie = req.headers().get("x-cle-api").map(|v| v.as_bytes()).unwrap_or(b"");
    if fournie.ct_eq(CLE).unwrap_u8() == 0 {
        return axum::http::StatusCode::UNAUTHORIZED.into_response();
    }
    next.run(req).await
}
```

`ct_eq` compare tous les octets quelle que soit la position de la première différence : la durée ne révèle pas le début correct de la clé. (Une clé réelle se lit depuis la configuration, jamais en dur dans le code.)

### Exercice 3 — `layer` ou `route_layer`

Une garde qui redirige vers la page de connexion est posée avec `.layer()` sur le routeur public. Quel est le symptôme, et la correction ?

Réponse : toute URL inexistante redirige vers la connexion au lieu de renvoyer 404 (section 9). Correction : `.route_layer()`, sur les seules routes protégées.

## 12. Aide-mémoire

```text
.layer(a).layer(b)                     b est extérieur à a (lecture de bas en haut)
ServiceBuilder::new().layer(a).layer(b)   a est extérieur à b (lecture de haut en bas)
layer                                  toutes les requêtes, même sans route
route_layer                            seulement les routes qui correspondent
slot bas = extérieur                   Session(50) < Auth(57) < CSRF(60)
```

```rust
axum::middleware::from_fn(f)                async fn f(req: Request, next: Next) -> Response
axum::middleware::from_fn_with_state(s, f)
req.extensions_mut().insert(v)              Extension(v): Extension<T>
m.add_custom(|router| router.layer(..))     m.csrf_exempt(vec!["/webhook/x"])
.rate_limit(chemin, nom, view!(h), max, fenetre_s, vec![Method::POST])
request.form("nom")                         flash_now!(warning => "..")
```

### Points clés

- Une couche ne voit que ce que les couches extérieures ont préparé.
- `.layer()` se lit de bas en haut ; `ServiceBuilder` de haut en bas.
- Runique trie ses couches par slot : l'ordre d'écriture ne compte pas.
- Le CSRF est toujours actif ; une exemption ne vaut que pour des chemins signés par un tiers.
- Le corps est lu une fois, par `Request`, toujours dernier paramètre ; les middlewares passent par les extensions.
- `route_layer` pour ce qui ne concerne que certaines routes, sinon les 404 sont interceptées.
- Flash message pour une redirection, `flash_now!` pour un rendu direct.
