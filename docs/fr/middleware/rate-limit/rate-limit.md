# Rate Limiting

Runique propose deux approches de rate limiting : **déclarative** au niveau des routes, ou **fine** dans le handler.

---

## Approche déclarative — au niveau des routes

Directement dans `url.rs`, via le trait `RouterExt` :

```rust
use runique::prelude::*;

pub fn routes() -> Routes {
    urlpatterns! {
        "/" => view!{ index }, name = "index",
        // ...
    }
    // Route unique
    .rate_limit("/upload-image", "upload_image", view!(upload_image_submit), 5, 60, vec![Method::POST])
}
```

Plusieurs routes partageant le **même compteur** :

```rust
.rate_limit_many(5, 60, vec![Method::POST], vec![
    ("/upload-image".into(), "upload_image".into(), view!(upload_image_submit)),
    ("/inscription".into(),  "inscription".into(),  view!(soumission_inscription)),
])
```

> Le `spawn_cleanup` est appelé automatiquement dans cette intégration : les entrées expirées sont
> purgées périodiquement, mais cela ne constitue pas un plafond strict de mémoire.
> Les entrées expirées sont aussi retirées lorsqu'elles sont consultées. Le nettoyage périodique
> retire celles qui ne reviennent plus.

---

## Approche handler — logique fine

Pour une logique par utilisateur, par action, ou avec une clé custom :

```rust
use runique::prelude::*;

static LIMITER: LazyLock<RateLimiter> = LazyLock::new(|| {
    RateLimiter::new()
        .max_requests(10)
        .retry_after(60)
});

pub async fn login(/* ... */) -> impl IntoResponse {
    if !LIMITER.is_allowed(&ip) {
        return StatusCode::TOO_MANY_REQUESTS.into_response();
    }
    // ...
}
```

---

## Quand utiliser laquelle ?

| Cas                                           | Approche                      |
| --------------------------------------------- | ----------------------------- |
| Route publique exposée, limite globale par IP | Déclarative (`.rate_limit()`) |
| Limite par utilisateur connecté               | Handler                       |
| Logique différente selon le contexte          | Handler                       |
| Plusieurs routes, même quota partagé          | `.rate_limit_many()`          |

---

## Configuration

```rust
RateLimiter::new().max_requests(5).retry_after(60)    // 5 requêtes par minute
RateLimiter::new().max_requests(3).retry_after(300)   // 3 requêtes par 5 minutes
RateLimiter::new().max_requests(100).retry_after(60)  // 100 requêtes par minute
```

---

## Comportement

- La clé de limitation est l'**adresse IP** de la requête — l'extension `ClientIp` posée par le middleware `trusted_proxies` (toujours actif) si présente, sinon l'adresse TCP brute du pair (`ConnectInfo`)
- Le support de `X-Forwarded-For` passe par ce mécanisme `trusted_proxies` (validation contre une liste de CIDR de confiance), pas par une lecture directe du header ici — `X-Real-IP` n'est lu nulle part dans le code
- Fenêtre **fixe** : le compteur repart à zéro après `retry_after` secondes
- Réponse `429 Too Many Requests` quand la limite est dépassée, avec header `Retry-After: <secondes>`

> **⚠️ Sécurité :** la fiabilité de la clé IP dépend entièrement de la configuration de `trusted_proxies` (voir [Trusted Proxies](/docs/fr/middleware/trusted-proxies)) — c'est elle qui détermine si `X-Forwarded-For` est validé ou ignoré. Sans reverse proxy de confiance correctement déclaré, un attaquant peut forger ce header pour changer de clé de rate limiting à volonté.

---

## API

### `RateLimiter::new()`

Crée un rate limiter avec les valeurs par défaut (60 req / 60 s).

### `.max_requests(max: u32)`

Nombre de requêtes autorisées dans la fenêtre.

### `.retry_after(secs: u64)`

Durée de la fenêtre en secondes.

### `is_allowed(key: &str) -> bool`

Retourne `true` si la clé est sous la limite, `false` sinon.

### `retry_after_secs(key: &str) -> u64`

Secondes restantes avant réinitialisation de la fenêtre pour cette clé. Retourne `0` si la fenêtre est déjà expirée ou si la clé est inconnue. Utilisé pour remplir le header `Retry-After` dans les réponses 429.
Une entrée arrivée à expiration est supprimée dès qu'elle est consultée.

### `.only_methods(methods: Vec<Method>)`

Restreint le rate limiting aux méthodes HTTP spécifiées. Les requêtes avec d'autres méthodes passent librement sans être comptabilisées.

```rust
use axum::http::Method;

RateLimiter::new()
    .max_requests(5)
    .retry_after(60)
    .only_methods(vec![Method::POST])
```

Cas d'usage : protéger une route de login contre le brute force sur les soumissions POST uniquement, tout en laissant passer GET (affichage du formulaire) librement.

> Sans `.only_methods()`, toutes les méthodes HTTP sont comptabilisées.

---

### `.spawn_cleanup(period: Duration)`

Lance une tâche de nettoyage en arrière-plan qui purge périodiquement les entrées expirées. Pour une intégration manuelle dans un handler ou un middleware, appelez-la une fois pendant qu'un runtime Tokio est actif. La période effective est bornée entre 1 et 60 secondes.

```rust
let limiter = RateLimiter::new().max_requests(5).retry_after(60);
limiter.spawn_cleanup(Duration::from_secs(60));
let limiter = Arc::new(limiter);
```

> Avec l'approche déclarative (`.rate_limit()` / `.rate_limit_many()`), `spawn_cleanup` est appelé automatiquement.
> Le builder admin le démarre également quand un rate limiter est configuré.

---

## Plusieurs instances

Un `RateLimiter` compte dans la mémoire de son processus, tout comme `LoginGuard`. Avec plusieurs instances de l'application, chacune garde ses propres compteurs : une limite de 5 tentatives par minute devient 5 par instance, donc 15 derrière trois instances. Si la limite doit tenir sur l'ensemble, appliquez-la aussi devant les instances (proxy inverse, répartiteur de charge), ou envoyez chaque visiteur toujours vers la même instance.

---

## Limite actuelle : pas de plafond mémoire

**La limite.** `RateLimiter` et `LoginGuard` gardent une entrée par clé dans la mémoire du processus : une par adresse IP pour le limiteur, une par nom d'utilisateur saisi pour `LoginGuard`. Les entrées expirées sont retirées quand on les consulte et par le nettoyage périodique (toutes les 1 à 60 secondes), mais **aucun nombre maximal d'entrées n'est imposé**. Un afflux de clés différentes (beaucoup d'adresses IP, des noms d'utilisateur inventés) fait grossir la mémoire jusqu'à leur expiration.

**Pourquoi.** Un plafond demande de décider quoi faire quand il est atteint, et chaque réponse simple profite à l'attaquant : refuser les nouvelles clés bloque la connexion de tout le monde, évincer les plus anciennes efface un verrouillage en cours, ne plus compter laisse passer la force brute. `LoginGuard` compte aussi volontairement les noms qui n'existent pas : ne compter que les comptes réels ferait du verrouillage un moyen de découvrir quels noms d'utilisateur existent. Une gestion du plafond qui évite ces pièges est prévue dans une version 4.x.

**En attendant :**

- placez l'application derrière un proxy inverse qui **limite lui-même les requêtes**. Un proxy qui se contente de transmettre ne protège pas ces tables. Avec nginx, `limit_req` garde ses compteurs dans une zone de taille fixe, ce qui borne la mémoire :

  ```nginx
  limit_req_zone $binary_remote_addr zone=login:10m rate=10r/m;

  location = /login {
      limit_req zone=login burst=5 nodelay;
      proxy_pass http://127.0.0.1:3000;
  }
  ```

  Faites de même pour la connexion de l'admin, à son propre chemin ;
- combinez `LoginGuard` et un limiteur par IP sur la route de connexion (voir [LoginGuard](/docs/fr/auth/login-guard)) : le limiteur plafonne le nombre de noms qu'une même adresse peut essayer ;
- gardez des durées raisonnables (`retry_after`, `lockout_secs`) : plus elles sont longues, plus les entrées restent en mémoire ;
- surveillez la mémoire du processus en production.

---

← [**Builder & configuration**](/docs/fr/middleware/builder) | [**Flash Messages**](/docs/fr/flash) →
