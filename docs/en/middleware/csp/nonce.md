# CSP Nonce

The nonce is a random token generated per request, injected into the CSP header and available in Tera templates. It allows only explicitly tagged inline scripts and styles, blocking any code injected by an attacker.

---

## How it works

1. On each request, `CspNonce::generate()` produces a random token
2. The token is injected into the request extensions
3. The CSP header is built with `'nonce-{value}'` in `script-src` and `style-src`
4. The `csp_nonce` variable is available in all Tera templates
5. `'unsafe-inline'` is automatically removed from `script-src` and `style-src` when the nonce is active

```text
Content-Security-Policy: script-src 'self' 'nonce-r4nd0m...'; style-src 'self' 'nonce-r4nd0m...'
```

---

## Usage in templates

### Runique tag (recommended)

```html
<script {% csp %}>
    console.log("Script secured by nonce");
</script>

<style {% csp %}>
    body { margin: 0; }
</style>
```

The `{% csp %}` tag renders `nonce="r4nd0m..."` directly.

### Direct variable

```html
<script nonce="{{ csp_nonce }}">
    console.log("Alternative");
</script>
```

### Passing the nonce to JavaScript

```html
<script {% csp %}>
    // Store the nonce for dynamically created scripts if needed
    window.__nonce = "{{ csp_nonce }}";
</script>
```

---

## External scripts

Scripts loaded from an allowed URL in `script-src` do not need a nonce:

```html
<!-- Allowed if 'self' or the domain is in script-src -->
<script src="/static/js/app.js"></script>

<!-- Requires adding https://cdn.example.com via .scripts(...) in the builder -->
<script src="https://cdn.example.com/lib.js"></script>
```

---

## Disabling the nonce

Not recommended. If your application cannot use a nonce (e.g. client-side generated templates):

```rust
.middleware(|m| {
    m.with_csp(|c| c.with_nonce(false))
})
```

Without a nonce, inline scripts are blocked unless `'unsafe-inline'` is added to `script-src` — which neutralizes CSP protection against XSS.

---

## False positives in developer tools

### Firefox DevTools — `sandbox eval code`

With a strict CSP (nonce active), Firefox may display an error like this in the console:

```text
Content-Security-Policy: Page's settings blocked the loading of a resource
(script-src-elem) [...] sandbox eval code:17
```

**This is not a bug in your application.** It comes from Firefox's internal sandbox (DevTools, inspector, console) attempting to run `eval()` code on its own behalf and hitting the CSP.

- The error disappears when DevTools is closed
- Chrome does not produce this false positive
- Your application and scripts work correctly

---

## Back

- [CSP — Overview](/docs/en/middleware/csp)
