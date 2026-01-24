# 🚀 Runique Framework

Le cœur du framework web Runique.

## 📁 Structure

```
runique/
├── src/
│   ├── lib.rs              # Point d'entrée principal
│   ├── app/                # Constructeur d'application
│   ├── config/             # Configuration
│   ├── context/            # Contexte des requêtes
│   ├── db/                 # Base de données
│   ├── engine/             # Moteur principal
│   ├── flash/              # Messages flash
│   ├── forms/              # Système de formulaires
│   ├── middleware/         # Middlewares
│   ├── macros/             # Macros utiles
│   └── utils/              # Utilitaires
├── tests/                  # Tests d'intégration
└── Cargo.toml
```

## 🧪 Tests

```bash
# Tests unitaires
cargo test --lib

# Tests d'intégration
cargo test --test integration_tests

# Tous les tests
cargo test --all
```

Résultats : **36/36 tests passent** ✅

## 📦 Modules principaux

### `src/forms/` - Système de formulaires
Types de champs, validation, CSRF automatique.

### `src/middleware/` - Middlewares de sécurité
CSRF protection, CSP, headers de sécurité.

### `src/macros/` - Macros utiles
`context!`, `success!`, `error!`, `impl_objects!`, etc.

### `src/db/` - Configuration base de données
Connexion, migrations, configuration.

## 🔧 Compilation

```bash
# Vérifier
cargo check

# Compiler
cargo build

# Release
cargo build --release
```

## 📚 Documentation

- [Installation](../docs/en/01-installation.md)
- [Architecture](../docs/en/02-architecture.md)
- [Guide complet](../docs/en/README.md)

## 🚀 Démarrage

```rust
use runique::prelude::*;

#[tokio::main]
async fn main() {
    let settings = Settings::from_env();
    let app = RuniqueApp::builder(settings)
        .with_routes(routes)
        .build()
        .await;

    app.run().await;
}
```

## 📊 État

- ✅ Compilation sans erreurs
- ✅ 36 tests passant
- ✅ Documentation complète
- ✅ Production ready

---

**Pour en savoir plus** : [Documentation complète](../docs/en/README.md)
