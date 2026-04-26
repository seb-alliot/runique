# Network Deployment

## Supported HTTP protocols

Runique natively supports **HTTP/1.1** and **HTTP/2** via Axum/Hyper.

### HTTP/2 and the `Host` header

HTTP/2 does not send a `Host` header — it uses the `:authority` pseudo-header instead.
Runique handles this automatically: the `allowed_hosts` middleware and the HTTPS redirect
middleware first read the `Host` header, then fall back to `request.uri().authority()`
when absent.

This behavior covers HTTP/1.1, HTTP/2, and reverse proxies (nginx, Caddy, Cloudflare).

### HTTP/3

HTTP/3 runs over **QUIC** (UDP) and is not natively supported by Axum/Hyper at this time.
To benefit from it, two options:

| Option | Description |
|---|---|
| **Cloudflare** (recommended) | Terminates HTTP/3 on Cloudflare's side, proxies HTTP/2 to Runique. Zero server-side configuration. |
| **Reverse proxy** (Caddy, nginx) | Some reverse proxies support HTTP/3 and proxy HTTP/1.1 or HTTP/2 to Runique. |

Runique directly on the internet = HTTP/2 maximum.

## ACME / Automatic TLS

The `acme` feature allows Runique to manage its own Let's Encrypt certificates
without a reverse proxy.

```toml
# Cargo.toml
runique = { features = ["acme"] }
```

```env
# .env
ACME_ENABLED=true
ACME_DOMAIN=mydomain.com
ACME_EMAIL=admin@mydomain.com
ACME_CERTS_DIR=/absolute/path/to/certs   # default: ./certs
```

> `ACME_CERTS_DIR` should be an **absolute path** in production. A relative path
> depends on the systemd `WorkingDirectory` — if not set correctly, the certificate
> is not found and the server crashes on every restart.
>
> If `ACME_ENABLED=true` but the `acme` feature is not compiled in, Runique
> prints a warning at startup.

### Required ports

| Port | Usage |
|---|---|
| 80 | Let's Encrypt HTTP-01 challenge + HTTPS redirect |
| 443 | HTTPS (TLS) |

To listen on these ports without root, use `CAP_NET_BIND_SERVICE`:

```ini
# /etc/systemd/system/runique.service
[Service]
CapabilityBoundingSet=CAP_NET_BIND_SERVICE
AmbientCapabilities=CAP_NET_BIND_SERVICE
```

## Behind a reverse proxy

If a reverse proxy (nginx, Caddy, Cloudflare) handles TLS, Runique runs over HTTP
on an internal port (e.g. 3000) and does not require the `acme` feature.

```env
ACME_ENABLED=false
PORT=3000
IP_SERVER=127.0.0.1
```

For HTTPS redirection, let the proxy handle it and disable `ENFORCE_HTTPS` on
Runique's side to avoid a double redirect.
