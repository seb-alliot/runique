# Nonce CSP

Le nonce est un jeton aléatoire généré par requête, injecté dans le header CSP et disponible dans les templates Tera. Il permet d'autoriser uniquement les scripts et styles inline explicitement balisés, bloquant tout code injecté par un attaquant.

---

## Fonctionnement

1. À chaque requête, `CspNonce::generate()` produit un jeton aléatoire
2. Le jeton est injecté dans les extensions de la requête
3. Le header CSP est construit avec `'nonce-{valeur}'` dans `script-src` et `style-src`
4. La variable `csp_nonce` est disponible dans tous les templates Tera
5. `'unsafe-inline'` est automatiquement retiré de `script-src` et `style-src` quand le nonce est actif

```
Content-Security-Policy: script-src 'self' 'nonce-r4nd0m...'; style-src 'self' 'nonce-r4nd0m...'
```

---

## Usage dans les templates

### Tag Tera (recommandé)

```html
<script {% csp_nonce %}>
    console.log("Script sécurisé par nonce");
</script>

<style {% csp_nonce %}>
    body { margin: 0; }
</style>
```

Le tag `{% csp_nonce %}` génère `nonce="r4nd0m..."` directement.

### Variable directe

```html
<script nonce="{{ csp_nonce }}">
    console.log("Alternative");
</script>
```

### Transmission à du JavaScript

```html
<script {% csp_nonce %}>
    // Stocker le nonce pour les scripts dynamiques si nécessaire
    window.__nonce = "{{ csp_nonce }}";
</script>
```

---

## Scripts externes

Les scripts chargés depuis une URL autorisée dans `script-src` n'ont pas besoin de nonce :

```html
<!-- Autorisé si 'self' ou le domaine est dans script-src -->
<script src="/static/js/app.js"></script>

<!-- Nécessite d'ajouter https://cdn.example.com à RUNIQUE_POLICY_CSP_SCRIPTS -->
<script src="https://cdn.example.com/lib.js"></script>
```

---

## Désactiver le nonce

Non recommandé. Si votre application ne peut pas utiliser de nonce (ex. templates générés côté client) :

```env
RUNIQUE_POLICY_CSP_STRICT_NONCE=false
```

Sans nonce, les scripts inline sont bloqués sauf si `'unsafe-inline'` est ajouté à `script-src` — ce qui neutralise la protection CSP contre le XSS.

---

## Retour

- [CSP — Vue d'ensemble](https://github.com/seb-alliot/runique/blob/main/docs/fr/middleware/csp/csp.md)
