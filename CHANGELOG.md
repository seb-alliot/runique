## [Unreleased] - En cours

### Added
- Migration pipeline complet : `entities/*.rs` → `makemigrations` → fichiers sea-orm → `cargo run -p migration`
- Support de tous les types de colonnes (string, int, float, bool, datetime, binary, json, etc.)
- Primary key, foreign keys, indexes, nullable, unique dans le DSL `model!`
- Tests sea-orm end-to-end sur Postgres, MariaDB, SQLite via Docker
- Couverture de tests : 76.66% fonctions (baseline 59.35%)
- 1356 tests unitaires et d'intégration, 0 échec

### Changed
- Vue admin : champ `password` corrigé visuellement
- `runique start` corrigé

---

## [1.1.30] - 2026-02-09

### Changed
- Form system stabilized with multiple internal improvements.
- Builder updated with a new, more flexible middleware system.

### Security
- CSRF protection is now transparently enforced in all forms by default.

### Upcoming
- Initial work and design phase for a basic admin view.


## [1.1.30] - 2026-02-09

### Modifié
- Stabilisation du système de formulaires avec plusieurs améliorations internes.
- Mise à jour du builder avec un nouveau système de middleware plus flexible.

### Sécurité
- La protection CSRF est désormais imposée de manière transparente sur tous les formulaires.

### À venir
- Début de réflexion et de conception pour une vue d’administration basique.
