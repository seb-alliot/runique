# Déploiement réseau

## Protocoles HTTP supportés

Runique supporte nativement **HTTP/1.1** et **HTTP/2** via Axum/Hyper.

### HTTP/2 et le header `Host`

HTTP/2 n'envoie pas de header `Host` — il utilise le pseudo-header `:authority`.
Runique gère ce cas automatiquement : le middleware `allowed_hosts` et le middleware
HTTPS redirect lisent d'abord le header `Host`, puis tombent en fallback sur
`request.uri().authority()` si absent.

Ce comportement couvre HTTP/1.1, HTTP/2, et les proxies inverses (nginx, Caddy, Cloudflare).

### HTTP/3

HTTP/3 repose sur **QUIC** (UDP) et n'est pas supporté nativement par Axum/Hyper
à ce jour. Pour en bénéficier, deux options :

| Option | Description |
|---|---|
| **Cloudflare** (recommandé) | Termine HTTP/3 côté Cloudflare, proxifie en HTTP/2 vers Runique. Zéro configuration côté serveur. |
| **Reverse proxy** (Caddy, nginx) | Certains reverse proxies supportent HTTP/3 et proxifient en HTTP/1.1 ou HTTP/2 vers Runique. |

Runique direct sur Internet = HTTP/2 max.

## ACME / TLS automatique

La feature `acme` permet à Runique de gérer ses propres certificats Let's Encrypt
sans reverse proxy.

```toml
# Cargo.toml
runique = { features = ["acme"] }
```

```env
# .env
ACME_ENABLED=true
ACME_DOMAIN=mondomaine.fr
ACME_EMAIL=admin@mondomaine.fr
ACME_CERTS_DIR=/chemin/absolu/vers/certs   # défaut : ./certs
```

> `ACME_CERTS_DIR` doit être un **chemin absolu** en production. Un chemin relatif
> dépend du `WorkingDirectory` systemd — s'il n'est pas correctement configuré,
> le certificat n'est pas trouvé et le serveur crash à chaque redémarrage.
>
> Si `ACME_ENABLED=true` mais que la feature `acme` n'est pas compilée, Runique
> affiche un avertissement au démarrage.

### Ports requis

| Port | Usage |
|---|---|
| 80 | Challenge HTTP-01 Let's Encrypt + redirection HTTPS |
| 443 | HTTPS (TLS) |

Pour écouter sur ces ports sans root, utiliser `CAP_NET_BIND_SERVICE` :

```ini
# /etc/systemd/system/runique.service
[Service]
CapabilityBoundingSet=CAP_NET_BIND_SERVICE
AmbientCapabilities=CAP_NET_BIND_SERVICE
```

## Derrière un reverse proxy

Si un reverse proxy (nginx, Caddy, Cloudflare) gère TLS, Runique tourne en HTTP
sur un port interne (ex. 3000) et ne nécessite pas la feature `acme`.

```env
ACME_ENABLED=false
PORT=3000
IP_SERVER=127.0.0.1
```

Pour la redirection HTTPS, laisser le proxy la gérer et désactiver `ENFORCE_HTTPS`
côté Runique (évite une double redirection).
