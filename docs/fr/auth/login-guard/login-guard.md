# LoginGuard — Protection contre le brute-force

`LoginGuard` suit les tentatives de connexion échouées **par username**, indépendamment de l'adresse IP.

Contrairement au rate limiting IP, il n'y a pas de faux positifs liés au NAT ou aux proxies partagés : chaque compte est suivi individuellement.

`LoginGuard` s'utilise **dans le handler**, pas comme middleware — c'est la seule façon d'accéder au username soumis dans le formulaire sans consommer le body avant l'extraction via `request.form()`.

---

## Usage complet avec `effective_key`

`effective_key` détermine automatiquement la bonne clé selon que l'utilisateur a soumis un username ou non :

- Username soumis → clé par compte (protection ciblée)
- Username vide → `"anonym:{ip}"` (protection par IP, compteurs indépendants)

```rust
use axum::Extension;
use runique::middleware::security::trusted_proxies::ClientIp;
use runique::prelude::*;
use std::sync::LazyLock;

static GUARD: LazyLock<LoginGuard> = LazyLock::new(|| {
    LoginGuard::new()
        .max_attempts(5)
        .lockout_secs(300)
});

pub async fn sign_in(
    client_ip: Option<Extension<ClientIp>>,
    mut request: Request,
) -> AppResult<Response> {
    let form: LoginForm = request.form();
    let validated = match ValidationForm::try_form(form, &request, "login").await {
        Ok(validated) => validated,
        Err(form) => {
            context_update!(request => { "login_form" => &form });
            return request.render("login.html");
        }
    };
    let username = validated.cleaned::<String>("username").unwrap_or_default();
    let password = validated.cleaned::<String>("password").unwrap_or_default();
    // L'IP cliente réelle, résolue par le middleware des proxies de confiance
    let ip = client_ip.map(|Extension(ClientIp(ip))| ip.to_string()).unwrap_or_default();

    // username soumis → "alice"  |  username vide → "anonym:1.2.3.4"
    let key = LoginGuard::effective_key(&username, &ip);

    if GUARD.is_locked(&key) {
        let remaining = GUARD.remaining_lockout_secs(&key).unwrap_or(0);
        return Ok((StatusCode::TOO_MANY_REQUESTS, format!("Réessayez dans {remaining}s")).into_response());
    }

    if let Some(user) = authenticate_user(&request.engine.db, &username, &password).await
        && login(&request.session, &user, None, false).await.is_ok()
    {
        GUARD.record_success(&key);
        return Ok(Redirect::to("/dashboard").into_response());
    }

    GUARD.record_failure(&key);
    context_update!(request => {
        "login_form" => &*validated,
        "messages" => flash_now!(error => "Identifiants invalides"),
    });
    request.render("login.html")
}
```

---

## Pourquoi dans le handler et pas en middleware ?

Un middleware s'exécute avant le handler et ne peut pas lire le body sans le consommer.
`request.form()` extrait le formulaire une seule fois — le username n'est disponible qu'après cette extraction.

La session connaît l'état d'authentification, mais au moment du login l'utilisateur n'est pas encore connecté : `request.user` vaut `None` sur cette route, ce qui ne protège pas le compte ciblé.

| Source | Disponible en middleware | Fiable pour LoginGuard |
| --- | --- | --- |
| Adresse IP | ✅ | ✅ (utilisée par `effective_key` pour anonym) |
| Utilisateur connecté (`request.user`) | ✅ | ❌ (`None` sur `/login`, retombe sur anonym) |
| Username (form body) | ❌ (consomme le body) | ✅ (via `request.form()` dans le handler) |

---

## Configuration

```rust
LoginGuard::new().max_attempts(5).lockout_secs(300)    // 5 échecs → blocage 5 minutes
LoginGuard::new().max_attempts(3).lockout_secs(900)    // 3 échecs → blocage 15 minutes
LoginGuard::new().max_attempts(10).lockout_secs(3600)  // 10 échecs → blocage 1 heure
```

---

## API

### `LoginGuard::new()`

Crée un LoginGuard avec les valeurs par défaut (5 tentatives / 300 s).

### `.max_attempts(max: u32)`

Nombre d'échecs avant verrouillage du compte.

### `.lockout_secs(secs: u64)`

Durée du verrouillage en secondes.

### `LoginGuard::effective_key(username, ip) -> Cow<str>`

Retourne la clé à utiliser selon le contexte :

- Username non vide → clé par username (protection compte ciblé)
- Username vide ou absent → `"anonym:{ip}"` (protection anonyme par IP)

Garantit que deux IPs anonymes différentes ont des compteurs indépendants — bloquer `"anonym:1.2.3.4"` n'affecte pas `"anonym:5.6.7.8"`.

> **Pourquoi pas une clé `"anonym"` globale ?** Une seule tentative abusive suffirait à verrouiller tous les utilisateurs anonymes du monde entier. La clé par IP isole chaque attaquant.

### `record_failure(key)`

Incrémente le compteur d'échecs pour cette clé. À appeler après chaque authentification échouée.

### `record_success(key)`

Réinitialise le compteur. À appeler après une connexion réussie.

### `is_locked(key) -> bool`

Retourne `true` si le nombre d'échecs dépasse `max_attempts` et que le délai de verrouillage n'est pas écoulé.
Une entrée arrivée à expiration est supprimée lorsqu'elle est consultée. Pour les clés qui ne sont plus consultées, le nettoyage périodique les retire.

### `attempts(key) -> u32`

Nombre d'échecs en cours pour cette clé.

### `remaining_lockout_secs(key) -> Option<u64>`

Secondes restantes avant déverrouillage. `None` si non verrouillé.

### Nettoyage périodique

Le builder admin démarre automatiquement le nettoyage du `LoginGuard` configuré. Pour un usage autonome, appelez `spawn_cleanup` une fois pendant qu'un runtime Tokio est actif ; la période effective est bornée entre 1 et 60 secondes. Cette purge limite la rétention des entrées expirées mais ne fixe pas de plafond strict au nombre d'entrées.

---

## Combinaison avec le rate limiting IP

Les deux mécanismes couvrent des vecteurs d'attaque différents et sont complémentaires :

| | `RateLimiter` | `LoginGuard` |
| --- | --- | --- |
| Clé | Adresse IP | Username ou `anonym:{ip}` |
| Où | Middleware (automatique) | Handler (manuel) |
| Cible | Toutes les routes | Login uniquement |
| Protège contre | Volume d'attaque par IP | Brute-force par compte |
| Faux positifs | Possible (NAT) | Aucun (par compte) |

```rust
static IP_LIMITER: LazyLock<RateLimiter> = LazyLock::new(|| RateLimiter::new().max_requests(10).retry_after(60));
static GUARD:      LazyLock<LoginGuard>  = LazyLock::new(|| LoginGuard::new().max_attempts(5).lockout_secs(300));

// Dans le handler de connexion ci-dessus, avant l'authentification :

// 1. Volume par IP → RateLimiter (en middleware sur la route, ou à la main)
if !IP_LIMITER.is_allowed(&ip) { /* 429 */ }

// 2. Brute-force par compte ou IP anonyme → LoginGuard
let key = LoginGuard::effective_key(&username, &ip);
if GUARD.is_locked(&key) { /* 429 */ }

// 3. Authentification : authenticate_user, puis record_success / record_failure
```

---

## Limite connue : un compte peut être bloqué par quelqu'un d'autre

Le verrouillage se fait par **nom d'utilisateur**, pas par IP : c'est ce qui empêche un attaquant de répartir ses essais sur beaucoup d'adresses. Le revers : quiconque connaît un nom d'utilisateur (celui d'un admin, par exemple) peut bloquer ce compte en échouant exprès, sans connaître son mot de passe, et le maintenir bloqué en recommençant.

Pour le limiter :

- gardez le limiteur par IP devant (section ci-dessus) : il plafonne le nombre d'essais par adresse, ce qui rend le blocage en boucle bien plus coûteux pour l'attaquant ;
- bloquez les adresses abusives avant l'application, au niveau du proxy inverse ;
- choisissez des noms d'utilisateur d'admin qui ne sont pas publics (ni `admin`, ni le nom affiché sur le site).

Les compteurs vivent dans la mémoire de chaque instance : voir [Rate limiting — plusieurs instances](/docs/fr/middleware/rate-limit).

---

## Limite actuelle : pas de plafond mémoire

`LoginGuard` garde une entrée par nom d'utilisateur saisi, y compris les noms qui n'existent pas (sinon le verrouillage révélerait quels comptes existent), sans nombre maximal d'entrées. Des noms inventés envoyés en masse font donc grossir la mémoire jusqu'à leur expiration. Le pourquoi et les mesures à prendre en attendant une gestion du plafond (proxy inverse qui limite les requêtes, limiteur par IP devant) : voir [Rate limiting — limite actuelle](/docs/fr/middleware/rate-limit).

---

← [**Middlewares de protection**](/docs/fr/auth/middleware) | [**Exemple complet**](/docs/fr/auth/exemple) →
