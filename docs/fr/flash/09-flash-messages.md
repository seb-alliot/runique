# Flash Messages

Runique fournit un système de messages flash pour les notifications utilisateur. Il existe **deux types** de messages :

1. **Messages de redirection** (`success!`, `error!`, `info!`, `warning!`) — stockés en session, affichés après un redirect
2. **Messages immédiats** (`flash_now!`) — affichés sur la requête courante, sans passer par la session

---

## Table des matières

| Section | Contenu |
| --- | --- |
| [Macros](https://github.com/seb-alliot/runique/blob/main/docs/fr/flash/macros/macros.md) | `success!`, `error!`, `info!`, `warning!`, `flash_now!`, différences, quand utiliser |
| [Handlers](https://github.com/seb-alliot/runique/blob/main/docs/fr/flash/handlers/handlers.md) | Utilisation dans les handlers, comportement flash (une seule lecture) |
| [Templates](https://github.com/seb-alliot/runique/blob/main/docs/fr/flash/templates/templates.md) | Tag `{% messages %}`, placement, personnalisation |

---

## Prochaines étapes

← [**Middleware & Security**](https://github.com/seb-alliot/runique/blob/main/docs/fr/middleware/08-middleware.md) | [**Exemples pratiques**](https://github.com/seb-alliot/runique/blob/main/docs/fr/exemple/10-examples.md) →
