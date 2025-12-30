# Cours 5 : Configuration et Settings

## 🎯 Objectif

Créer un système de configuration flexible avec support des variables d'environnement et builder pattern.

## 📚 Concepts de base

### Architecture

```
Settings (struct principal)
  ├── ServerSettings
  ├── Valeurs par défaut
  ├── Variables d'environnement
  └── Builder pattern
```

## 🔧 Implémentation étape par étape

### Étape 1 : Structure de base

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub server: ServerSettings,
    pub debug: bool,
    pub allowed_hosts: Vec<String>,
    pub templates_dir: Vec<String>,
    pub staticfiles_dirs: String,
    // ... autres champs
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerSettings {
    pub ip_server: String,
    pub domain_server: String,
    pub port: u16,
    pub secret_key: String,
}
```

### Étape 2 : Valeurs par défaut

```rust
impl Settings {
    pub fn default_values() -> Self {
        Self {
            server: ServerSettings::from_env(),
            debug: cfg!(debug_assertions),  // true en mode debug
            allowed_hosts: vec!["localhost".to_string(), "127.0.0.1".to_string()],
            templates_dir: vec!["templates".to_string()],
            staticfiles_dirs: "static".to_string(),
            // ...
        }
    }
}
```

### Étape 3 : Chargement depuis .env

```rust
impl ServerSettings {
    pub fn from_env() -> Self {
        use dotenvy::dotenv;
        use std::env;

        // Charger le fichier .env
        dotenv().ok();

        // Lire les variables avec valeurs par défaut
        let ip = env::var("IP_SERVER")
            .unwrap_or_else(|_| "127.0.0.1".to_string());

        let port = env::var("PORT")
            .unwrap_or_else(|_| "3000".to_string())
            .parse()
            .unwrap_or(3000);

        let secret_key = env::var("SECRET_KEY")
            .unwrap_or_else(|_| "change_me_in_production".to_string());

        Self {
            ip_server: ip.clone(),
            domain_server: format!("{}:{}", ip, port),
            port,
            secret_key,
        }
    }
}
```

### Étape 4 : Builder Pattern

Le builder permet une configuration fluide :

```rust
pub struct SettingsBuilder {
    settings: Settings,
}

impl SettingsBuilder {
    pub fn new() -> Self {
        Self {
            settings: Settings::default_values(),
        }
    }

    pub fn debug(mut self, debug: bool) -> Self {
        self.settings.debug = debug;
        self
    }

    pub fn templates_dir(mut self, dirs: Vec<String>) -> Self {
        self.settings.templates_dir = dirs;
        self
    }

    pub fn server(mut self, ip: &str, port: u16, secret_key: &str) -> Self {
        self.settings.server.ip_server = ip.to_string();
        self.settings.server.port = port;
        self.settings.server.secret_key = secret_key.to_string();
        self.settings.server.domain_server = format!("{}:{}", ip, port);
        self
    }

    pub fn build(self) -> Settings {
        self.settings
    }
}
```

**Utilisation :**
```rust
let settings = Settings::builder()
    .debug(true)
    .templates_dir(vec!["templates".to_string()])
    .server("127.0.0.1", 3000, "my_secret_key")
    .build();
```

### Étape 5 : Parsing de listes depuis .env

```rust
impl ServerSettings {
    pub fn parse_allowed_hosts_from_env() -> Vec<String> {
        use std::env;

        env::var("ALLOWED_HOSTS")
            .unwrap_or_else(|_| "localhost,127.0.0.1".to_string())
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    }
}
```

### Étape 6 : Validation

```rust
impl Settings {
    pub fn validate_allowed_hosts(&self) {
        if !self.debug {
            // En production, ALLOWED_HOSTS ne peut pas être vide
            if self.allowed_hosts.is_empty() {
                panic!(
                    "ALLOWED_HOSTS ne peut pas être vide en production!\n\
                    Ajoutez vos domaines dans le fichier .env:\n\
                    ALLOWED_HOSTS=exemple.com,www.exemple.com"
                );
            }

            // Avertir si seulement localhost
            let only_local = self.allowed_hosts.iter().all(|h| {
                h == "localhost" || h == "127.0.0.1" || h == "::1"
            });

            if only_local {
                eprintln!(
                    "AVERTISSEMENT: ALLOWED_HOSTS contient uniquement des hôtes locaux."
                );
            }
        }
    }
}
```

## 🎓 Exercices

### Exercice 1 : Configuration hiérarchique

Créez un système qui charge la config dans cet ordre :
1. Fichier de config (YAML/TOML)
2. Variables d'environnement
3. Valeurs par défaut

### Exercice 2 : Validation avancée

Ajoutez des validations pour :
- Port dans une plage valide (1-65535)
- Secret key d'une longueur minimale
- Chemins de dossiers existants

### Exercice 3 : Hot reload

Implémentez un système qui recharge la config sans redémarrer :
```rust
impl Settings {
    pub fn watch_for_changes(&self) {
        // Surveiller le fichier .env et recharger si modifié
    }
}
```

## 💡 Bonnes pratiques

1. **Séparation** : Séparez la config serveur de la config application
2. **Validation** : Validez la config au démarrage
3. **Sécurité** : Ne jamais commiter les secrets
4. **Flexibilité** : Supportez plusieurs sources (env, fichiers, builder)

## 🔗 Ressources

- [dotenvy](https://docs.rs/dotenvy/)
- [Builder Pattern](https://doc.rust-lang.org/1.0.0/style/ownership/builders.html)
