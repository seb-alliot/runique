# 🤝 Guide de contribution - Rusti Framework

Merci de votre intérêt pour contribuer à Rusti ! Ce guide vous aidera à bien démarrer.

## Table des matières

1. [Code de conduite](#code-de-conduite)
2. [Comment contribuer](#comment-contribuer)
3. [Configuration de l'environnement](#configuration-de-lenvironnement)
4. [Workflow de contribution](#workflow-de-contribution)
5. [Standards de code](#standards-de-code)
6. [Tests](#tests)
7. [Documentation](#documentation)

---

## Code de conduite

Nous nous engageons à créer une communauté accueillante et inclusive. En participant à ce projet, vous acceptez de :

- Respecter tous les contributeurs
- Accepter les critiques constructives
- Vous concentrer sur ce qui est le mieux pour la communauté
- Faire preuve d'empathie envers les autres

---

## Comment contribuer

###  Signaler un bug

1. Vérifiez que le bug n'est pas déjà signalé dans les [Issues](https://github.com/votre-repo/rusti/issues)
2. Ouvrez une nouvelle issue avec le template "Bug Report"
3. Fournissez un exemple minimal reproductible
4. Incluez les informations système (OS, version de Rust, etc.)

### ✨ Proposer une fonctionnalité

1. Ouvrez une issue avec le template "Feature Request"
2. Expliquez le problème que vous voulez résoudre
3. Décrivez votre solution proposée
4. Discutez avec la communauté avant de coder

### 📝 Améliorer la documentation

La documentation est aussi importante que le code !

- Corrections de typos
- Clarifications
- Nouveaux exemples
- Traductions

---

## Configuration de l'environnement

### Prérequis

- Rust 1.70 ou supérieur
- Git
- PostgreSQL, MySQL ou SQLite (pour les tests DB)

### Installation

```bash
# Cloner le dépôt
git clone https://github.com/votre-repo/rusti.git
cd rusti

# Installer les dépendances
cargo build

# Lancer les tests
cargo test

# Vérifier le formatage
cargo fmt --check

# Lancer clippy
cargo clippy --all-features -- -D warnings
```

### Structure du projet

```
rusti/
├── rusti/                  # Framework core
│   ├── src/
│   │   ├── lib.rs
│   │   ├── app.rs
│   │   ├── settings.rs
│   │   ├── middleware/
│   │   ├── database/
│   │   └── ...
│   ├── templates/          # Templates internes
│   ├── static/             # Assets du framework
│   └── tests/
│
├── examples/
│   └── demo-app/          # Application exemple
│
├── docs/                  # Documentation
│   ├── README.md
│   ├── GETTING_STARTED.md
│   └── ...
│
└── Cargo.toml            # Workspace root
```

---

## Workflow de contribution

### 1. Fork et clone

```bash
# Fork sur GitHub puis :
git clone https://github.com/VOTRE-USERNAME/rusti.git
cd rusti
git remote add upstream https://github.com/votre-repo/rusti.git
```

### 2. Créer une branche

```bash
# Feature
git checkout -b feature/ma-super-fonctionnalite

# Bugfix
git checkout -b fix/correction-du-bug

# Documentation
git checkout -b docs/amelioration-docs
```

### 3. Développer

```bash
# Faire vos modifications

# Tester
cargo test

# Formatter
cargo fmt

# Linter
cargo clippy --all-features -- -D warnings
```

### 4. Committer

Utilisez des messages de commit clairs :

```bash
# ✅ Bon
git commit -m "feat: ajouter support WebSocket"
git commit -m "fix: corriger validation CSRF"
git commit -m "docs: améliorer exemples ORM"

# ❌ Mauvais
git commit -m "update"
git commit -m "fix stuff"
git commit -m "WIP"
```

**Format des commits :**
- `feat:` Nouvelle fonctionnalité
- `fix:` Correction de bug
- `docs:` Documentation
- `style:` Formatage, pas de changement de code
- `refactor:` Refactoring
- `test:` Ajout/modification de tests
- `chore:` Maintenance (dépendances, etc.)

### 5. Push et Pull Request

```bash
# Push vers votre fork
git push origin feature/ma-super-fonctionnalite

# Créer une Pull Request sur GitHub
```

**Template de Pull Request :**

```markdown
## Description
Brève description des changements

## Type de changement
- [ ] Bug fix
- [ ] Nouvelle fonctionnalité
- [ ] Breaking change
- [ ] Documentation

## Tests
- [ ] Tests unitaires ajoutés/modifiés
- [ ] Tests d'intégration ajoutés/modifiés
- [ ] Tous les tests passent

## Checklist
- [ ] Code formaté (`cargo fmt`)
- [ ] Pas d'avertissements clippy
- [ ] Documentation mise à jour
- [ ] CHANGELOG.md mis à jour (si applicable)
```

---

## Standards de code

### Style Rust

Suivez les conventions Rust standards :

```rust
// ✅ Bon
pub struct RustiApp {
    router: Router,
    config: Arc<Settings>,
}

impl RustiApp {
    pub fn new(settings: Settings) -> Result<Self> {
        // ...
    }
}

// ❌ Mauvais
pub struct rustiApp {
    Router: Router,
    CONFIG: Arc<Settings>,
}
```

### Documentation

Documentez toutes les fonctions publiques :

```rust
/// Crée une nouvelle instance de RustiApp
///
/// # Exemples
///
/// ```rust
/// use rusti::{RustiApp, Settings};
///
/// let app = RustiApp::new(Settings::default_values())?;
/// ```
///
/// # Erreurs
///
/// Retourne une erreur si la configuration est invalide
pub fn new(settings: Settings) -> Result<Self> {
    // ...
}
```

### Gestion d'erreur

Utilisez `Result` et des types d'erreur appropriés :

```rust
// ✅ Bon
pub fn connect(&self) -> Result<DatabaseConnection, DbErr> {
    // ...
}

// ❌ Mauvais
pub fn connect(&self) -> DatabaseConnection {
    // panic! si erreur
}
```

### Tests

Écrivez des tests pour chaque fonctionnalité :

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settings_builder() {
        let settings = Settings::builder()
            .debug(true)
            .build();

        assert!(settings.debug);
    }

    #[tokio::test]
    async fn test_app_creation() {
        let settings = Settings::default_values();
        let app = RustiApp::new(settings).await;

        assert!(app.is_ok());
    }
}
```

---

## Tests

### Lancer tous les tests

```bash
# Tests unitaires et d'intégration
cargo test

# Tests d'une feature spécifique
cargo test --features postgres

# Tests avec output détaillé
cargo test -- --nocapture

# Tests en parallèle
cargo test -- --test-threads=4
```

### Coverage

```bash
# Installer tarpaulin
cargo install cargo-tarpaulin

# Générer le rapport
cargo tarpaulin --out Html --output-dir coverage
```

### Benchmarks

```bash
# Installer criterion
cargo install cargo-criterion

# Lancer les benchmarks
cargo bench
```

---

## Documentation

### Documentation du code

```bash
# Générer la documentation
cargo doc

# Ouvrir dans le navigateur
cargo doc --open

# Avec les dépendances privées
cargo doc --document-private-items
```

### Documentation Markdown

Les fichiers de documentation se trouvent dans `docs/` :

- Utilisez des titres clairs
- Incluez des exemples de code
- Ajoutez des liens entre les documents
- Gardez un ton accessible

### Exemples

Les exemples dans `examples/` doivent :

- Être fonctionnels (`cargo run` doit marcher)
- Être bien commentés
- Couvrir un cas d'usage réel
- Inclure un README.md

---

## Revue de code

Toutes les Pull Requests sont revues par les mainteneurs. Soyez patient et ouvert aux suggestions.

### Critères de revue

- ✅ Code propre et bien structuré
- ✅ Tests passent
- ✅ Documentation à jour
- ✅ Pas de breaking changes non documentés
- ✅ Performance acceptable
- ✅ Sécurité respectée

### Après la revue

- Répondez aux commentaires
- Effectuez les modifications demandées
- Marquez les conversations comme résolues
- Demandez une nouvelle revue

---

## Premiers pas

### Issues "good first issue"

Cherchez les issues marquées `good first issue` pour commencer :
- Bugs simples
- Améliorations de documentation
- Petites fonctionnalités

### Mentors

N'hésitez pas à demander de l'aide :
- Commentez sur l'issue
- Rejoignez les discussions GitHub
- Posez des questions (il n'y a pas de question stupide !)

---

## Ressources

### Documentation Rust
- [The Rust Book](https://doc.rust-lang.org/book/)
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/)
- [Async Book](https://rust-lang.github.io/async-book/)

### Dépendances principales
- [Axum](https://docs.rs/axum/)
- [Tokio](https://tokio.rs/)
- [SeaORM](https://www.sea-ql.org/SeaORM/)
- [Tera](https://keats.github.io/tera/)

### Outils utiles
- [rust-analyzer](https://rust-analyzer.github.io/) - LSP pour IDE
- [cargo-watch](https://github.com/watchexec/cargo-watch) - Auto-reload
- [cargo-edit](https://github.com/killercup/cargo-edit) - Gérer les dépendances

---

## Questions ?

- 💬 [GitHub Discussions](https://github.com/votre-repo/rusti/discussions)
- 🐛 [Issues](https://github.com/votre-repo/rusti/issues)
- 📧 Email : [votre-email@example.com]

---

## Remerciements

Merci de contribuer à Rusti ! Chaque contribution, aussi petite soit-elle, aide à améliorer le framework.

**Ensemble, construisons le meilleur framework web pour Rust ! 🦀**
