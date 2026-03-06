## [Unreleased]

### Changed
- Les sessions anonymes expirent désormais après 5 minutes d'inactivité (configurable).
- Lorsqu'un utilisateur s'authentifie, la session est automatiquement prolongée à 24h (configurable).
- Middleware slot 55 : upgrade dynamique du TTL de session après login, sans impact sur la logique CSRF ou les handlers applicatifs.

### Dev
- Ajout des méthodes `with_session_duration` et `with_anonymous_session_duration` dans le builder pour personnaliser les TTL.

## [1.1.35] - 2026-02-09

### Changed
- Form system stabilized with multiple internal improvements.
- Builder updated with a new, more flexible middleware system.

### Security
- CSRF protection is now transparently enforced in all forms by default.

### Upcoming
- Initial work and design phase for a basic admin view.


## [1.1.35] - 2026-02-09

### Modifié
- Stabilisation du système de formulaires avec plusieurs améliorations internes.
- Mise à jour du builder avec un nouveau système de middleware plus flexible.

### Sécurité
- La protection CSRF est désormais imposée de manière transparente sur tous les formulaires.

### À venir
- Début de réflexion et de conception pour une vue d’administration basique.
