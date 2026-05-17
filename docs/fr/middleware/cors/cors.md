# CORS

## Ce que ça fait

CORS (Cross-Origin Resource Sharing) contrôle quelles origines peuvent envoyer des requêtes navigateur vers votre API.
Sans en-têtes CORS, le navigateur bloque les requêtes cross-origin par défaut.

Runique **n'active pas** CORS par défaut. À activer uniquement si un frontend sur un domaine différent doit appeler votre API.

---

## Configuration de base

```rust
.middleware(|m| {
    m.with_cors(|c| {
        c.origin("https://app.example.com")
         .origin("https://www.example.com")
    })
})
```

---

## Autoriser toutes les origines

```rust
m.with_cors(|c| c.any_origin())
```

> Génère `Access-Control-Allow-Origin: *`.
> À n'utiliser que pour les APIs **entièrement publiques et en lecture seule**.

---

## Avec credentials

Les cookies et l'en-tête `Authorization` nécessitent un opt-in explicite :

```rust
m.with_cors(|c| {
    c.origin("https://app.example.com")
     .allow_credentials(true)
})
```

> **Contrainte de sécurité** : `any_origin()` et `.allow_credentials(true)` ne peuvent pas être combinés.
> Runique rejette cette configuration au démarrage avec un `BuildError`.

---

## Durée du cache preflight

```rust
m.with_cors(|c| {
    c.origin("https://app.example.com")
     .max_age(3600)  // secondes, défaut : 3600
})
```

---

## Webhooks Stripe / services tiers

Stripe et les services similaires envoient leurs POST directement depuis leurs serveurs — **pas depuis un navigateur**.
CORS ne s'applique pas aux appels serveur-à-serveur.
Pour les webhooks Stripe, utilisez `.csrf_exempt()` (et vérifiez l'en-tête `Stripe-Signature` dans votre handler).

---

## Voir aussi

| Section | Description |
| --- | --- |
| [CSRF](/docs/fr/middleware/csrf) | Protection CSRF et chemins exemptés |
| [Hosts & cache](/docs/fr/middleware/hosts-cache) | Validation du Host header |
| [Builder](/docs/fr/middleware/builder) | Configuration du builder |

## Retour au sommaire

- [Middleware & Sécurité](/docs/fr/middleware)
