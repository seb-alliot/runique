# Trusted Proxies

## What it does

Extracts the real client IP from the `X-Forwarded-For` header when the request comes through a trusted reverse proxy.

Without this middleware, `X-Forwarded-For` is untrusted user input — an attacker can forge any IP. Runique validates the chain: only if the direct connection IP is a trusted proxy does it walk the XFF header from right to left and return the first untrusted address.

**Active by default.** Slot `2` — runs immediately after Extensions, before any other middleware.

---

## Algorithm

1. Read the direct connection IP (`ConnectInfo<SocketAddr>`).
2. If it is **not** in the trusted list → return it as the real client IP (XFF is ignored).
3. If it **is** trusted → parse `X-Forwarded-For`, walk from right to left:
   - Skip entries that are trusted proxies.
   - Return the first untrusted entry as the real client IP.
4. If all entries are trusted → return the leftmost (the client's own claim).

The result is injected into request extensions as `ClientIp(IpAddr)`.

---

## Default trusted list

RFC 1918 private networks and loopback addresses:

| CIDR | Description |
| --- | --- |
| `127.0.0.0/8` | IPv4 loopback |
| `10.0.0.0/8` | Class A private |
| `172.16.0.0/12` | Class B private |
| `192.168.0.0/16` | Class C private |
| `::1/128` | IPv6 loopback |
| `fc00::/7` | IPv6 unique local |

**Exception — edge TLS (ACME).** When Runique terminates TLS itself (`ACME_ENABLED=true`) and you did not configure `with_trusted_proxies`, the default automatically switches to `none()`: when Runique is the TLS entry point there is **by construction** no proxy in front, so `X-Forwarded-*` must never be believed. An explicit configuration still takes precedence.

---

## Security — when to use `.none()`

The "private ranges" default is safe **behind a reverse proxy**. It becomes a risk if the application is **directly exposed** (no proxy) while keeping that default.

The danger: trusting a private network is only safe if **only your proxy** lives in that network. If an attacker shares the trusted private range — neighboring VPC, sidecar container, another machine on the same LAN — their connection IP falls inside `10.0.0.0/8` (etc.), so they are treated as a trusted proxy and can **forge `X-Forwarded-For`** to spoof any client IP. This corrupts everything that relies on the IP: rate limiting, lockout, logs.

**Rule:**

- Behind a reverse proxy (nginx, Caddy) on a private network → keep the default, or declare the exact proxy IP with `.proxy(...)` to be strict.
- Behind a public-IP proxy (Cloudflare…) → `.none()` then `.proxy(...)`/`.cidr(...)` with the proxy's ranges.
- Directly exposed with no proxy (Runique does the TLS) → `.none()` (automatic in ACME mode).

---

## Configuration via the builder

```rust
.middleware(|m| {
    m.with_trusted_proxies(|t| {
        // Start from the private network defaults and add a CDN IP
        t.proxy("203.0.113.42")
         .cidr("198.51.100.0/24")
    })
})
```

To disable XFF processing entirely (direct server, no proxy):

```rust
.middleware(|m| {
    m.with_trusted_proxies(|t| t.none())
})
```

---

## Available methods

| Method | Description |
| --- | --- |
| `.private_networks()` | Reset to RFC 1918 + loopback (the default) |
| `.proxy("1.2.3.4")` | Trust an exact IP |
| `.cidr("10.0.0.0/8")` | Trust a CIDR range |
| `.none()` | Clear all trusted entries (XFF ignored) |

Methods are cumulative. `.none()` clears the list; subsequent calls add to the empty list.

---

## Accessing the client IP in handlers

```rust
use axum::Extension;
use runique::middleware::ClientIp;

pub async fn my_handler(
    Extension(client_ip): Extension<ClientIp>,
    engine: Arc<RuniqueEngine>,
    req: Request,
) -> Response {
    let ip = client_ip.0; // IpAddr
    // ...
}
```

---

## Keeping the default

Do not call `.with_trusted_proxies` — the RFC 1918 preset applies automatically.

## See also

| Section | Description |
| --- | --- |
| [Permissions-Policy](/docs/en/middleware/permissions-policy) | Browser API restrictions |
| [Host validation](/docs/en/middleware/hosts-cache) | Allowed hosts |
| [Builder](/docs/en/middleware/builder) | Builder configuration |

## Back to summary

- [Middleware & Security](/docs/en/middleware)
