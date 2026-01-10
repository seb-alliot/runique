# Bienvenue dans Runique 1.0.86 - Première Release

## À propos de ce document

Ce document explique le contenu de la **première version stable publique** de Runique Framework.


---

## Contenu de la release 1.0.86

### Fonctionnalités principales

| Fonctionnalité | Description | Status |
|----------------|-------------|--------|
| **API Django-like** | Syntaxe familière inspirée de Django | Stable |
| **Performances Axum** | Basé sur Axum et Tokio | Stable |
| **Sécurité intégrée** | CSRF, CSP, Sessions, Validation | Stable |
| **Templates Tera** | Moteur avec balises personnalisées | Stable |
| **ORM Django-like** | SeaORM avec API `Entity::objects` | Stable |
| **Formulaires** | `DeriveModelForm` auto-génération | Stable |
| **Configuration flexible** | Builder pattern + `.env` | Stable |
| **Debug avancé** | Pages d'erreur détaillées | Stable |
| **Flash messages** | Messages entre requêtes | Stable |
| **Reverse routing** | URLs générées automatiquement | Stable |

---

## Démarrage rapide

### 1. Installation
```toml
[dependencies]
runique = "1.0.86"
tokio = { version = "1", features = ["full"] }
```

### 2. Hello World
```rust
use runique::prelude::*;

async fn hello() -> &'static str {
    "Hello, Runique 1.0!"
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let settings = Settings::default_values();

    RuniqueApp::new(settings).await?
        .routes(Router::new().route("/", get(hello)))
        .run()
        .await?;

    Ok(())
}
```

**Lancez :**
```bash
cargo run
```

Ouvrez http://127.0.0.1:3000

---

## Documentation complète

Runique 1.0 inclut **89 pages** de documentation professionnelle.

### Guides principaux

| Document | Pages | Description |
|----------|-------|-------------|
| **README.md** | 11 | Vue d'ensemble et installation |
| **GETTING_STARTED.md** | 13 | Tutorial complet pas à pas |
| **TEMPLATES.md** | 11 | Système de templates et balises |
| **DATABASE.md** | 15 | ORM Django-like et configuration |
| **FORMULAIRE.md** | 18 | Système de formulaires complet |
| **CONFIGURATION.md** | 12 | Configuration avancée et production |
| **CSP.md** | 4 | Content Security Policy |
| **CONTRIBUTING.md** | 9 | Guide de contribution |

### Parcours d'apprentissage

**Débutant (2-3 heures) :**
1. README.md - Comprendre Runique
2. GETTING_STARTED.md - Créer votre première app
3. TEMPLATES.md - Maîtriser les templates

**Intermédiaire (4-6 heures) :**
1. DATABASE.md - ORM et base de données
2. FORMULAIRE.md - Validation de formulaires
3. CONFIGURATION.md - Configuration avancée

**Avancé :**
1. CSP.md - Sécurité avancée
2. CONTRIBUTING.md - Contribuer au framework

---

## Tests et qualité

### Couverture de tests : 75%

**50+ tests** couvrant toutes les fonctionnalités :

| Module | Nombre de tests | Description |
|--------|-----------------|-------------|
| `allowed_hosts` | 9 | Validation des hôtes autorisés |
| `csrf` | 5 | Protection CSRF |
| `csp` | 6 | Content Security Policy |
| `routing` | 7 | Système de routing |
| `forms` | 17 | Validation de formulaires |
| `sanitization` | 5 | Sanitisation XSS |
| `utils` | 5 | Fonctions utilitaires |
| `login` | 4 | Authentification |
| `settings` | 9 | Configuration |

**Lancer les tests :**
```bash
cargo test
```

---

## Architecture

### Diagrammes de séquence

**18 diagrammes** documentant l'architecture complète :

1. Request lifecycle
2. Middleware pipeline
3. Template preprocessing
4. CSRF protection
5. Flash messages
6. Database ORM
7. Form validation
8. Security headers
9. Session management
10. Routing system
11. Error handling
12. Static files
13. CSP implementation
14. Authentication flow
15. Configuration loading
16. Template rendering
17. Sanitization process
18. Response generation

**Localisation :** `informations/diagramme sequence Runique/`

---

## Cours d'implémentation

Des cours détaillés dans `informations/cours/` expliquent :

- Architecture et design patterns
- Comment implémenter chaque fonctionnalité
- Bonnes pratiques Rust
- Sécurité et performance

**Idéal pour :**
- Comprendre l'implémentation interne
- Contribuer au framework
- Créer vos propres extensions

---

## Sécurité

### Score Mozilla Observatory : A+ (115/100)

**Headers de sécurité implémentés :**
- Content-Security-Policy (avec nonces dynamiques)
- X-Content-Type-Options: nosniff
- X-Frame-Options: DENY
- X-XSS-Protection: 1; mode=block
- Referrer-Policy: strict-origin-when-cross-origin
- Permissions-Policy
- Strict-Transport-Security (production)

**Fonctionnalités de sécurité :**
- Protection CSRF avec tokens HMAC-SHA256
- Validation ALLOWED_HOSTS avec wildcards sécurisés
- Sanitisation XSS automatique
- Hash Argon2id pour les mots de passe
- Sessions sécurisées (tower-sessions)
- Validation constante des tokens

---

## Configuration

### Bases de données supportées

| Base de données | Feature | URL de connexion |
|----------------|---------|------------------|
| **SQLite** | `sqlite` (défaut) | `sqlite://database.db` |
| **PostgreSQL** | `postgres` | `postgres://user:pass@host/db` |
| **MySQL** | `mysql` | `mysql://user:pass@host/db` |
| **MariaDB** | `mariadb` | `mariadb://user:pass@host/db` |

**Installation :**
```toml
# PostgreSQL
runique = { version = "1.0", features = ["postgres"] }

# MySQL
runique = { version = "1.0", features = ["mysql"] }

# Toutes les bases
runique = { version = "1.0", features = ["all-databases"] }
```

---

## Statistiques du projet

| Métrique | Valeur |
|----------|--------|
| **Lignes de code** | ~15,000 LOC |
| **Documentation** | ~89 pages |
| **Tests** | 50+ tests |
| **Couverture** | 75% |
| **Modules** | 20+ modules |
| **Diagrammes** | 18 diagrammes |

---

## Comparaison avec Django

| Concept Django | Équivalent Runique | Notes |
|----------------|------------------|-------|
| `settings.py` | `Settings::builder()` | Configuration |
| `urls.py` | `urlpatterns! { ... }` | Routing |
| `views.py` | Handlers Axum | Logique métier |
| `models.py` | SeaORM entities | ORM |
| `forms.py` | `#[derive(DeriveModelForm)]` | **Auto-généré** |
| `{% url 'name' %}` | `{% link "name" %}` | Reverse routing |
| `{% static 'file' %}` | `{% static "file" %}` | Fichiers statiques |
| `messages.success()` | `message.success().await` | Flash messages |
| `{% csrf_token %}` | `{% csrf %}` | Protection CSRF |
| `Model.objects.filter()` | `Entity::objects.filter()` | Queries ORM |

**Avantage clé :** Pas besoin de fichier `forms.py` séparé - les formulaires sont auto-générés depuis les modèles.

---

## Fonctionnalités marquantes

### 1. ORM Django-like avec SeaORM
```rust
// Définir le modèle
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub username: String,
    pub email: String,
    pub age: i32,
}

// Activer l'API Django-like
impl_objects!(Entity);

// Utiliser comme Django
let adults = User::objects
    .filter(users::Column::Age.gte(18))
    .exclude(users::Column::Email.like("%@banned.com"))
    .order_by_desc(users::Column::CreatedAt)
    .limit(10)
    .all(&db)
    .await?;
```

### 2. Formulaires auto-générés
```rust
// Pas besoin de forms.py
#[derive(DeriveModelForm, DeriveEntityModel)]
#[sea_orm(table_name = "users")]
pub struct Model {
    pub id: i32,
    pub username: String,
    pub email: String,
}

// UserForm est automatiquement généré avec validate() et save()
let mut form = UserForm::new();
if form.validate(&raw_data) {
    form.save(&db).await?;
}
```

### 3. Templates avec balises personnalisées
```html
<!DOCTYPE html>
<html>
<head>
    <link rel="stylesheet" href='{% static "css/main.css" %}'>
</head>
<body>
    <nav>
        <a href='{% link "home" %}'>Accueil</a>
        <a href='{% link "about" %}'>À propos</a>
    </nav>

    {% messages %}

    <form method="post" action="/submit">
        {% csrf %}
        <input type="text" name="message">
        <button type="submit">Envoyer</button>
    </form>
</body>
</html>
```

### 4. Flash Messages
```rust
pub async fn create_post(mut message: Message) -> Response {
    // ... logique de création ...

    let _ = message.success("Article créé avec succès !").await;
    let _ = message.info("N'oubliez pas de le publier").await;

    redirect("/posts")
}
```

---

## Checklist de démarrage

Pour bien démarrer avec Runique 1.0 :

- [ ] Lire le README.md
- [ ] Suivre le GETTING_STARTED.md
- [ ] Créer votre première application
- [ ] Explorer les exemples
- [ ] Consulter les guides selon vos besoins
- [ ] Rejoindre la communauté (GitHub Discussions)
- [ ] Star le projet sur GitHub

---

## Fonctionnalités bonus

### Pages d'erreur élégantes en dev

En mode développement, Runique affiche des pages d'erreur détaillées :
- Stack trace complète
- Informations de requête HTTP
- Source du template avec numéro de ligne
- Liste des templates disponibles
- Variables d'environnement
- Version de Rust utilisée

### Reverse Routing
```rust
// Dans le code
let url = reverse_with_parameters("user_profile", &[
    ("id", "42"),
    ("name", "alice"),
]).unwrap();

// Dans les templates
<a href='{% link "user_profile", id=42, name="alice" %}'>Profil</a>
```

---

## Roadmap (v1.1.0 et au-delà)

### Planifié pour v1.1.0

**Fonctionnalités :**
- Implémentation réelle du rate limiting
- CLI Runique pour scaffolding
- Support WebSocket intégré
- Admin panel auto-généré
- Hot reload en développement
- Système de cache (Redis, Memcached)
- Support i18n/l10n complet

**Améliorations :**
- Pool de connexions configurable via Settings
- Variables dynamiques dans balises templates
- Plus de types de champs pour formulaires
- Benchmarks de performance

---

## Contribution

Runique est open source et accueille les contributions.

**Comment contribuer :**
1. Fork le projet
2. Créer une branche (`git checkout -b feature/AmazingFeature`)
3. Commit vos changements (`git commit -m 'Add AmazingFeature'`)
4. Push vers la branche (`git push origin feature/AmazingFeature`)
5. Ouvrir une Pull Request

**Voir :** [Guide de contribution](informations/documentation_french/CONTRIBUTING.md)

---

## Support et communauté

### Ressources officielles
- Documentation complète
- Issues GitHub
- Discussions GitHub
- Star le projet

### Obtenir de l'aide
1. Consultez d'abord la documentation
2. Cherchez dans les issues existantes
3. Posez votre question dans Discussions
4. Ouvrez une issue si c'est un bug

---

## Remerciements

Runique s'appuie sur l'excellent travail de :
- Django - Inspiration
- Axum - Framework HTTP
- Tokio - Runtime async
- Tera - Moteur de templates
- SeaORM - ORM
- La communauté Rust

---

## Licence

Runique est distribué sous double licence MIT / Apache-2.0.

Vous êtes libre de :
- Utiliser Runique dans des projets commerciaux
- Modifier le code source
- Distribuer votre application
- Contribuer au projet

**Voir :** LICENSE-MIT et LICENSE-APACHE

---

## C'est parti

Runique 1.0.86 est prêt pour la production. Construisez des applications web performantes et sécurisées avec Rust.

**Prochaines étapes :**
1. Lire le Guide de démarrage
2. Créer votre première application
3. Partager votre expérience
4. Rejoindre la communauté

---

**Développé avec passion en Rust par Itsuki**

**Bon développement avec Runique 1.0**