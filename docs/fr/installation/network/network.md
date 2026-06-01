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

### Limitation — un seul site par machine

ACME requiert l'**exclusivité du port 80**. Si plusieurs applications tournent sur le même serveur, une seule peut utiliser ACME — les autres ne peuvent pas obtenir leurs certificats simultanément.

Dans un contexte multi-site, utiliser un reverse proxy (nginx, Caddy) qui gère lui-même Let's Encrypt, et faire tourner chaque instance Runique sur un port interne distinct avec `ACME_ENABLED=false`.

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

## Nginx — configuration recommandée en production

### TLS et durcissement

```nginx
ssl_protocols TLSv1.2 TLSv1.3;
ssl_ciphers ECDHE-ECDSA-AES128-GCM-SHA256:ECDHE-RSA-AES128-GCM-SHA256:ECDHE-ECDSA-AES256-GCM-SHA384:ECDHE-RSA-AES256-GCM-SHA384:ECDHE-ECDSA-CHACHA20-POLY1305:ECDHE-RSA-CHACHA20-POLY1305;
ssl_prefer_server_ciphers off;
ssl_session_timeout 1d;
ssl_session_cache shared:SSL:10m;
server_tokens off;
```

### Headers de sécurité sur les fichiers media

Runique injecte ses headers de sécurité (CSP, HSTS, `X-Content-Type-Options`, etc.)
dans chaque réponse qu'il génère. Mais les fichiers servis directement par Nginx via
`alias` (bloc `location /media/`) **contournent Runique entièrement** — Nginx doit
y ajouter les headers lui-même.

```nginx
location /media/ {
    alias /var/www/monprojet/media/;
    add_header Cache-Control "public, max-age=31536000, immutable";
    add_header Strict-Transport-Security "max-age=31536000; includeSubDomains; preload" always;
    add_header X-Content-Type-Options "nosniff" always;
}
```

> Le flag `always` est obligatoire : sans lui, Nginx n'envoie ces headers que sur
> les réponses 2xx/3xx, pas sur les erreurs 4xx/5xx.

### Exemple complet (multi-site)

```nginx
server {
    listen 80;
    server_name mondomaine.fr www.mondomaine.fr;
    return 301 https://$host$request_uri;
}

server {
    listen 443 ssl;
    server_name mondomaine.fr www.mondomaine.fr;

    ssl_certificate     /etc/letsencrypt/live/mondomaine.fr/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/mondomaine.fr/privkey.pem;

    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers ECDHE-ECDSA-AES128-GCM-SHA256:ECDHE-RSA-AES128-GCM-SHA256:ECDHE-ECDSA-AES256-GCM-SHA384:ECDHE-RSA-AES256-GCM-SHA384:ECDHE-ECDSA-CHACHA20-POLY1305:ECDHE-RSA-CHACHA20-POLY1305;
    ssl_prefer_server_ciphers off;
    ssl_session_timeout 1d;
    ssl_session_cache shared:SSL:10m;
    server_tokens off;

    client_max_body_size 10M;

    location / {
        proxy_pass http://127.0.0.1:3000;
        proxy_set_header Host              $host;
        proxy_set_header X-Real-IP         $remote_addr;
        proxy_set_header X-Forwarded-For   $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }

    location /media/ {
        alias /var/www/monprojet/media/;
        add_header Cache-Control "public, max-age=31536000, immutable";
        add_header Strict-Transport-Security "max-age=31536000; includeSubDomains; preload" always;
        add_header X-Content-Type-Options "nosniff" always;
    }
}
```
