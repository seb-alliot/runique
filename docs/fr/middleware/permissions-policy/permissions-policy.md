# Permissions-Policy

## Ce que ça fait

Contrôle quelles APIs navigateur sont disponibles pour la page et les frames embarquées.
Envoyé en tant qu'en-tête HTTP `Permissions-Policy` sur chaque réponse.

Runique ajoute cet en-tête automatiquement. Le défaut est un preset sécurisé qui refuse
les APIs sensibles. Surchargez les directives individuelles via le builder.

---

## Preset par défaut

**Refusés (toutes origines) :**

| Feature | Catégorie |
| --- | --- |
| `accelerometer` | Capteur |
| `ambient-light-sensor` | Capteur |
| `bluetooth` | Matériel |
| `camera` | Matériel |
| `gyroscope` | Capteur |
| `hid` | Matériel |
| `magnetometer` | Capteur |
| `microphone` | Matériel |
| `midi` | Matériel |
| `serial` | Matériel |
| `usb` | Matériel |
| `geolocation` | Localisation |
| `idle-detection` | Vie privée |
| `display-capture` | Capture écran |
| `payment` | Paiements |
| `interest-cohort` | Fingerprinting (désactive FLoC) |
| `local-fonts` | Fingerprinting |
| `sync-xhr` | Legacy / déprécié |
| `xr-spatial-tracking` | XR |
| `window-management` | Multi-fenêtre |

**Autorisés pour la même origine (`(self)`) :**

| Feature | Notes |
| --- | --- |
| `fullscreen` | Besoin UX standard |
| `picture-in-picture` | Besoin UX standard |
| `publickey-credentials-create` | WebAuthn / passkeys |
| `publickey-credentials-get` | WebAuthn / passkeys |

---

## Configuration via le builder

```rust
.middleware(|m| {
    m.with_permissions_policy(|p| {
        p.deny("geolocation")
         .allow_self("fullscreen")
         .allow("payment", vec!["https://pay.example.com"])
    })
})
```

---

## Méthodes disponibles

| Méthode | Valeur header | Description |
| --- | --- | --- |
| `.deny("feature")` | `feature=()` | Refus total |
| `.allow_self("feature")` | `feature=(self)` | Même origine uniquement |
| `.allow_any("feature")` | `feature=*` | Toutes origines |
| `.allow("feature", vec!["https://…"])` | `feature=("url1" "url2")` | Origines explicites |

Les méthodes surchargent le défaut pour cette directive. Les directives non mentionnées conservent leur valeur par défaut.

---

## Conserver le défaut

Ne pas appeler `.with_permissions_policy` — le preset sécurisé s'applique automatiquement.

---

## Voir aussi

| Section | Description |
| --- | --- |
| [CSP & headers](/docs/fr/middleware/csp) | Content Security Policy |
| [Builder](/docs/fr/middleware/builder) | Configuration du builder |

## Retour au sommaire

- [Middleware & Sécurité](/docs/fr/middleware)
