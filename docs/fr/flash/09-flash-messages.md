# Flash Messages

Runique fournit un système de messages flash pour les notifications utilisateur. Il existe **deux types** de messages :

1. **Messages de redirection** (`success!`, `error!`, `info!`, `warning!`) — stockés en session, affichés après un redirect
2. **Messages immédiats** (`flash_now!`) — affichés sur la requête courante, sans passer par la session

---

## Table des matières

| Section | Contenu |
| --- | --- |
| [Macros](/docs/fr/flash/macros) | `success!`, `error!`, `info!`, `warning!`, `flash_now!`, différences, quand utiliser |
| [Handlers](/docs/fr/flash/handlers) | Utilisation dans les handlers, comportement flash (une seule lecture) |
| [Templates](/docs/fr/flash/templates) | Tag `{% messages %}`, placement, personnalisation |

---

## Prochaines étapes

← [**Middleware & Security**](/docs/fr/middleware) | [**Exemples pratiques**](/docs/fr/exemple) →
