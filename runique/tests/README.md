# 🧪 Tests du Framework Runique

Suite de tests complète couvrant toutes les fonctionnalités principales du framework Runique.

## 📁 Structure des Tests

```
tests/
├── common.rs              # Utilities et helpers partagés
├── forms_test.rs          # Tests du système de formulaires (Prisme extractor)
├── orm_test.rs            # Tests de l'ORM (Objects manager, impl_objects!)
├── config_test.rs         # Tests de configuration
├── flash_messages_test.rs # Tests des messages flash (Message extractor)
├── routes_test.rs         # Tests du routage (Axum Router)
├── middleware_test.rs     # Tests des middlewares (CSRF, CSP, etc.)
├── prelude_test.rs        # Tests que tous les types sont dans le prelude
└── README.md              # Ce fichier
```

## 🧪 Exécuter les Tests

### Tous les tests
```bash
cargo test
```

### Un fichier de test spécifique
```bash
cargo test --test forms_test
cargo test --test macros_test
cargo test --test orm_test
```

### Un test spécifique
```bash
cargo test test_text_field_creation
cargo test test_forms_new
cargo test test_context_macro_empty
```

### Avec output
```bash
cargo test -- --nocapture
cargo test -- --show-output
```

## 📋 Couverture des Tests

### ✅ Formulaires (`forms_test.rs`)
- [x] Prisme extractor
- [x] RuniqueForm derive macro
- [x] Validation des champs
- [x] Génération HTML des formulaires
- [x] CSRF token validation

### ✅ ORM (`orm_test.rs`)
- [x] impl_objects! macro
- [x] Objects manager (.all(), .filter(), etc.)
- [x] SeaORM integration
- [x] Relations

### ✅ Flash Messages (`flash_messages_test.rs`)
- [x] Message extractor
- [x] success(), error(), info(), warning() methods
- [x] message.level (Success/Error/Info/Warning)
- [x] {% messages %} template tag

### ✅ Formulaires (`forms_test.rs`)
- [x] `TextField` - text, email, password, textarea, richtext, url
- [x] `NumericField` - integer, decimal
- [x] `BooleanField`
- [x] Validations - required, min_length, max_length
- [x] `Forms` manager - new, add_field, fill_data
- [x] Chaîning de méthodes

### ✅ ORM (`orm_test.rs`)
- [x] `Objects<E>` manager
- [x] Méthodes chainables (filter, exclude, limit, offset, etc.)
- [x] RuniqueQueryBuilder
- [x] Django-style queries

### ✅ Configuration (`config_test.rs`)
- [x] `RuniqueConfig`
- [x] `ServerConfig`
- [x] `SecurityConfig`
- [x] Chargement depuis `.env`

### ✅ Messages Flash (`flash_messages_test.rs`)
- [x] `MessageLevel` (Success, Error, Warning, Info)
- [x] `FlashMessage`
- [x] `Message` type
- [x] Création et gestion

### ✅ Routage (`routes_test.rs`)
- [x] `urlpatterns!` macro
- [x] `view!` macro
- [x] Méthodes HTTP (GET, POST, PUT, DELETE, PATCH, OPTIONS)
- [x] `register_name_url` pour URL naming
- [x] `reverse` et `reverse_with_parameters`

### ✅ Middlewares (`middleware_test.rs`)
- [x] CSRF middleware
- [x] CSP middleware
- [x] Sanitizer middleware
- [x] Session middleware
- [x] AllowedHosts middleware

### ✅ Prelude (`prelude_test.rs`)
- [x] Types de formulaires disponibles
- [x] Types de contexte disponibles
- [x] Types de messages flash disponibles
- [x] Types ORM disponibles
- [x] Types de sérialisation
- [x] Types de concurrence

## 🧩 Utilities Communes

Le fichier `common.rs` fournit des helpers réutilisables:

```rust
// Créer un formulaire de test simple
let form = create_test_form("csrf_token");

// Créer un formulaire complexe avec plusieurs champs
let form = create_complex_form("csrf_token");

// Remplir un formulaire avec des données
fill_form(&mut form, &[("field", "value")]);
```

## 🔧 Ajouter Nouveaux Tests

### Template pour un nouveau test
```rust
#[test]
fn test_my_feature() {
    // Arrange - Préparer les données

    // Act - Exécuter le code à tester

    // Assert - Vérifier les résultats
    assert!(true);
}
```

### Tests asynchrones
```rust
#[tokio::test]
async fn test_async_feature() -> Result<(), Box<dyn std::error::Error>> {
    // Code asynchrone ici
    Ok(())
}
```

## ℹ️ Notes Importantes

1. **Tests d'intégration avec DB**: Pour tester l'ORM avec une vraie DB, utiliser SQLite en mémoire:
   ```rust
   let db = sea_orm::Database::connect("sqlite::memory:").await?;
   ```

2. **Tests de handlers Web**: Utiliser `axum-test` ou similaire pour tester les handlers Axum

3. **Tests de templates**: Utiliser `tera` directement pour tester le rendu

4. **Mocking**: Pour les dépendances externes, considérer `mockito` ou `wiremock`

## 📚 Resources

- [Rust Testing Documentation](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Tokio Testing](https://tokio.rs/tokio/tutorial/select#testing)
- [Axum Examples](https://github.com/tokio-rs/axum/tree/main/examples)

## 🎯 Objectifs Futurs

- [ ] Tests d'intégration complets avec DB SQLite
- [ ] Tests des handlers web avec `axum-test`
- [ ] Tests de rendu Tera
- [ ] Benchmarks de performance
- [ ] Tests de couverture (coverage reporting)
