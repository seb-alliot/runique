# Anti-Bot Honeypot

Runique intègre un middleware honeypot qui protège automatiquement les formulaires contre les bots simples — sans aucune modification de vos handlers.

## Fonctionnement

1. À la première visite, le middleware génère un nom de champ hexadécimal aléatoire de 16 caractères et le stocke en session.
2. À chaque requête, ce nom est injecté comme extension Axum (`HoneypotFieldName`).
3. `Request::form()` le lit et ajoute le champ au HTML rendu (invisible via `hp.css`).
4. Sur toute méthode autre que GET et HEAD (POST, PUT, PATCH, DELETE…), si le champ est non vide → `force_invalid = true` → `form.is_valid()` retourne `false` immédiatement.

Le handler voit un échec de validation normal — aucun cas particulier à gérer.

## Activation

```rust
.middleware(|m| m.with_anti_bot())
```

C'est le seul changement requis dans `main.rs`.

## Rendu du template

Le champ honeypot est injecté automatiquement :

- **`{{ form | form }}`** (rendu complet) : honeypot ajouté après le dernier champ
- **Champ par champ** (`{{ form | form(field="nom") }}`) : honeypot ajouté après le dernier champ

La clé `honeypot_html` est aussi disponible directement dans le contexte Tera pour un placement manuel :

```html
{{ form.honeypot_html | safe }}
```

## Propriétés de sécurité

| Propriété | Valeur |
| --- | --- |
| Nom du champ | Hex-16 aléatoire, lié à la session |
| Rotation | Nouveau nom par session (persiste entre GET et POST) |
| Visibilité | Caché via CSS externe (`hp.css`) — CSP-safe, pas de style inline |
| Résistance aux bots | Bloque les form-fillers qui remplissent tous les champs |
| Impact utilisateur | Aucun — le champ est invisible et ignoré par les navigateurs |

Le nom du champ n'a aucun préfixe reconnaissable — un bot ne peut pas le sauter par correspondance de motif.

## Contrainte de test local

Le cookie de session utilise `Secure=true` en production (`RUNIQUE_ENV=prod`). Sur une adresse en `http` qui n'est pas `localhost` (IP du réseau local, nom de machine), le navigateur n'envoie pas un cookie Secure : la session, et avec elle le nom du champ piège, est recréée à chaque requête, et le piège ne se déclenche jamais. Chrome et Firefox acceptent un cookie Secure sur `http://localhost`, d'autres navigateurs non.

**Tester en local avec `RUNIQUE_ENV=dev`.** Sur le VPS en HTTPS, le middleware fonctionne correctement sans aucune modification.

## Slot

`65` — juste après CSRF (60), avant le handler. Nécessite que le middleware de session (slot 50) soit actif.

## Retour au sommaire

- [Middleware & Sécurité](/docs/fr/middleware)
