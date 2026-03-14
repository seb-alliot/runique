# CSP Nonce

The nonce is a random token generated per request, injected into the CSP header and available in Tera templates. It allows only explicitly tagged inline scripts and styles, blocking any code injected by an attacker.

---

## How it works

1. On each request, `CspNonce::generate()` produces a random token
2. The token is injected into the request extensions
3. The CSP header is built with `'nonce-{value}'` in `script-src` and `style-src`
4. The `csp_nonce` variable is available in all Tera templates
5. `'unsafe-inline'` is automatically removed from `script-src` and `style-src` when the nonce is active

```
Content-Security-Policy: script-src 'self' 'nonce-r4nd0m...'; style-src 'self' 'nonce-r4nd0m...'
```

---

## Usage in templates

### Tera tag (recommended)

```html
<script {% csp_nonce %}>
    console.log("Script secured by nonce");
</script>

<style {% csp_nonce %}>
    body { margin: 0; }
</style>
```

The `{% csp_nonce %}` tag renders `nonce="r4nd0m..."` directly.

### Direct variable

```html
<script nonce="{{ csp_nonce }}">
    console.log("Alternative");
</script>
```

### Passing the nonce to JavaScript

```html
<script {% csp_nonce %}>
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

<!-- Requires adding https://cdn.example.com to RUNIQUE_POLICY_CSP_SCRIPTS -->
<script src="https://cdn.example.com/lib.js"></script>
```

---

## Disabling the nonce

Not recommended. If your application cannot use a nonce (e.g. client-side generated templates):

```env
RUNIQUE_POLICY_CSP_STRICT_NONCE=false
```

Without a nonce, inline scripts are blocked unless `'unsafe-inline'` is added to `script-src` — which neutralizes CSP protection against XSS.

---

## Back

- [CSP — Overview](https://github.com/seb-alliot/runique/blob/main/docs/en/middleware/csp/csp.md)
