# Trusted Proxies

## Ce que ça fait

Extrait la vraie IP cliente depuis l'en-tête `X-Forwarded-For` lorsque la requête transite par un reverse proxy de confiance.

Sans ce middleware, `X-Forwarded-For` est une entrée utilisateur non fiable — un attaquant peut forger n'importe quelle IP. Runique valide la chaîne : ce n'est que si l'IP de connexion directe est un proxy de confiance que l'en-tête XFF est parcouru de droite à gauche pour trouver la première adresse non fiable.

**Actif par défaut.** Slot `2` — s'exécute immédiatement après les Extensions, avant tout autre middleware.

---

## Algorithme

1. Lire l'IP de connexion directe (`ConnectInfo<SocketAddr>`).
2. Si elle n'est **pas** dans la liste de confiance → la retourner comme IP cliente réelle (XFF ignoré).
3. Si elle **est** de confiance → parser `X-Forwarded-For`, parcourir de droite à gauche :
   - Ignorer les entrées qui sont des proxies de confiance.
   - Retourner la première entrée non fiable comme IP cliente réelle.
4. Si toutes les entrées sont de confiance → retourner la plus à gauche (déclaration du client).

Le résultat est injecté dans les extensions de la requête sous la forme `ClientIp(IpAddr)`.

---

## Liste de confiance par défaut

Réseaux privés RFC 1918 et adresses de loopback :

| CIDR | Description |
| --- | --- |
| `127.0.0.0/8` | Loopback IPv4 |
| `10.0.0.0/8` | Réseau privé classe A |
| `172.16.0.0/12` | Réseau privé classe B |
| `192.168.0.0/16` | Réseau privé classe C |
| `::1/128` | Loopback IPv6 |
| `fc00::/7` | Local unique IPv6 |

---

## Configuration via le builder

```rust
.middleware(|m| {
    m.with_trusted_proxies(|t| {
        // Partir des défauts réseau privé et ajouter l'IP d'un CDN
        t.proxy("203.0.113.42")
         .cidr("198.51.100.0/24")
    })
})
```

Pour désactiver complètement le traitement XFF (serveur direct, sans proxy) :

```rust
.middleware(|m| {
    m.with_trusted_proxies(|t| t.none())
})
```

---

## Méthodes disponibles

| Méthode | Description |
| --- | --- |
| `.private_networks()` | Réinitialiser au preset RFC 1918 + loopback (le défaut) |
| `.proxy("1.2.3.4")` | Faire confiance à une IP exacte |
| `.cidr("10.0.0.0/8")` | Faire confiance à une plage CIDR |
| `.none()` | Vider la liste (XFF ignoré) |

Les méthodes sont cumulatives. `.none()` vide la liste ; les appels suivants ajoutent à la liste vide.

---

## Accéder à l'IP cliente dans les handlers

```rust
use axum::Extension;
use runique::middleware::ClientIp;

pub async fn ma_vue(
    Extension(client_ip): Extension<ClientIp>,
    engine: Arc<RuniqueEngine>,
    req: Request,
) -> Response {
    let ip = client_ip.0; // IpAddr
    // ...
}
```

---

## Garder la configuration par défaut

Ne pas appeler `.with_trusted_proxies` — le preset RFC 1918 s'applique automatiquement.

---

## Voir aussi

| Section | Description |
| --- | --- |
| [Permissions-Policy](/docs/fr/middleware/permissions-policy) | Restrictions des API navigateur |
| [Validation des hôtes](/docs/fr/middleware/host) | Hôtes autorisés |
| [Builder](/docs/fr/middleware/builder) | Configuration du builder |

## Retour au sommaire

- [Middleware & Sécurité](/docs/fr/middleware)
