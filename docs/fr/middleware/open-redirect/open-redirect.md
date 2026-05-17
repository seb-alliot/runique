# Protection Open Redirect

## Ce que ça fait

Intercepte toutes les réponses 3xx et valide l'en-tête `Location`.
Si la destination de la redirection est un hôte externe absent de la liste autorisée, le middleware retourne HTTP 400.

Protège contre les attaques de phishing où un attaquant forge un lien du type
`https://votresite.fr/login?next=https://evil.com` qui redirige silencieusement l'utilisateur ailleurs.

---

## Fonctionnement

Une destination de redirection est considérée sûre si :

- C'est un **chemin relatif** (`/tableau-de-bord`, `../profil`) — toujours sûr
- L'hôte est **localhost ou loopback** (`localhost`, `127.x.x.x`, `[::1]`, IPv6 mappé IPv4) — toujours sûr
- L'hôte correspond à une entrée de **`with_allowed_hosts`** (exact ou sous-domaine wildcard)

Toute autre URL absolue est bloquée avec HTTP 400.

---

## Configuration

Pas de configuration dédiée — le middleware lit `with_allowed_hosts` automatiquement :

```rust
.middleware(|m| {
    m.with_allowed_hosts(|h| {
        h.enabled(true)
         .host("monsite.fr")
         .host(".monsite.fr")  // monsite.fr + tous ses sous-domaines
    })
})
```

Le middleware open redirect est **toujours actif** et utilise la même liste d'hôtes.

---

## URLs protocol-relative

Les URLs commençant par `//` (ex : `//evil.com/chemin`) sont traitées comme absolues et soumises à la même vérification.
Elles sont bloquées sauf si l'hôte est dans la liste autorisée.

---

## Voir aussi

| Section | Description |
| --- | --- |
| [Hosts & cache](/docs/fr/middleware/hosts-cache) | Configuration des hôtes autorisés |
| [CSP & headers](/docs/fr/middleware/csp) | Content Security Policy |
| [Builder](/docs/fr/middleware/builder) | Configuration du builder |

## Retour au sommaire

- [Middleware & Sécurité](/docs/fr/middleware)
