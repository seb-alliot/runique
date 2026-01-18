# Runique

**Un framework web Rust inspiré de Django**

[![Version](https://img.shields.io/badge/version-0.1.86-blue.svg)](https://crates.io/crates/runique)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE-MIT)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org)

---

⚠️ **Statut : Développement actif (v0.1.x)**

L'API peut changer entre les versions mineures. La documentation complète sera mise à jour après la refonte de la base du framework (v0.2.0).

---

## 🚀 Installation
```toml
[dependencies]
runique = { version = "0.1", features = ["sqlite"] }
```

**Features disponibles :** `sqlite`, `postgres`, `mysql`, `mariadb`, `all-databases`

---

## 🎯 Fonctionnalités principales

- 🎨 **Architecture Django-like** - API familière avec routage déclaratif
- 📝 **Système de formulaires** - Génération et validation automatiques
- 🔐 **Sécurité intégrée** - CSRF, CSP, sanitization, validation ALLOWED_HOSTS
- 💾 **ORM style Django** - Basé sur SeaORM avec API intuitive
- 🎨 **Templates Tera** - Prétraitement avec syntaxe Django
- ⚡ **Performances Rust** - Async/await natif avec Tokio

---

## 🏁 Démarrage rapide

### Installation du CLI
```bash
cargo install runique
```

### Créer un nouveau projet
```bash
runique new mon_app
cd mon_app
cargo run
```

Le CLI génère une structure complète avec :
- Modèle utilisateur avec authentification
- Formulaires d'inscription et de connexion
- Templates avec design responsive
- Configuration base de données
- Migrations prêtes

---

## 📦 Exemple minimal
```rust
use runique::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let settings = Settings::default_values();

    RuniqueApp::new(settings).await?
        .with_default_middleware()
        .run()
        .await?;

    Ok(())
}
```

---

## 🔧 Configuration (.env)
```env
# Serveur
IP_SERVER=127.0.0.1
PORT=3000
SECRET_KEY=votre-cle-secrete
DEBUG=true

# Base de données (SQLite par défaut)
DB_ENGINE=sqlite
DB_NAME=app.db
```

---

## 📚 Documentation

La documentation complète sera disponible après la stabilisation de l'API (v0.2.0).

En attendant :
- Consultez les exemples dans le dossier `examples/`
- Utilisez `cargo doc --open` pour la documentation API
- Rejoignez notre Discord pour obtenir de l'aide

---

## 🛠️ Développement
```bash
# Tests
cargo test

# Formatage
cargo fmt

# Linting
cargo clippy
```

---

## 🤝 Contribuer

Les contributions sont bienvenues ! Ouvrez une issue ou soumettez une PR.

---

## 📄 Licence

MIT - Voir LICENSE-MIT pour plus de détails.

---

## 📧 Contact

- **GitHub** : seb-alliot/runique
- **Discord** : discord.gg/Y5zW7rbt
- **Email** : alliotsebastien04@gmail.com

---

**Construit avec ❤️ et 🦀**