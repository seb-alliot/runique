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

---

## See also

| Section | Description |
| --- | --- |
| [Permissions-Policy](/docs/en/middleware/permissions-policy) | Browser API restrictions |
| [Host validation](/docs/en/middleware/hosts-cache) | Allowed hosts |
| [Builder](/docs/en/middleware/builder) | Builder configuration |

## Back to summary

- [Middleware & Security](/docs/en/middleware)
