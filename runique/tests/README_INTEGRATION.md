# 🧪 Tests d'intégration pour Runique

Suite de tests d'intégration pour le framework Runique.

## 📁 Structure

```
tests/
├── integration_tests.rs  # Tests d'intégration principaux
└── README.md            # Documentation des tests
```

## 🧪 Exécuter les tests

### Tests d'intégration
```bash
cargo test --test integration_tests
```

### Tests unitaires de la librairie
```bash
cargo test --lib
```

### Tous les tests
```bash
cargo test --all
```

## 📊 Statistiques des tests

| Type | Nombre | État |
|------|--------|------|
| Tests d'intégration | 16 | ✅ Passent |
| Tests unitaires | 20 | ✅ Passent |
| **Total** | **36** | **✅ Tous passent** |

## 🧪 Tests disponibles

### Tests des formulaires (8 tests)

- `test_text_field_creation` - Création d'un champ texte
- `test_email_field` - Création d'un champ email
- `test_password_field` - Création d'un champ mot de passe
- `test_textarea_field` - Création d'une zone de texte
- `test_richtext_field` - Création d'un champ texte enrichi
- `test_url_field` - Création d'un champ URL
- `test_numeric_field_integer` - Création d'un champ numérique entier
- `test_numeric_field_decimal` - Création d'un champ numérique décimal

### Tests du pattern builder (1 test)

- `test_text_field_builder` - Test du pattern builder pour les champs

### Tests des champs requis (1 test)

- `test_field_required` - Test des champs obligatoires

### Tests de la gestion des formulaires (4 tests)

- `test_forms_new` - Création d'un formulaire avec token CSRF
- `test_forms_add_field` - Ajout d'un champ à un formulaire
- `test_forms_fill_data` - Remplissage d'un formulaire avec des données
- `test_complex_form_creation` - Création d'un formulaire complexe

### Tests de configuration (2 tests)

- `test_prelude_exports` - Vérification que les structures sont disponibles
- `test_field_types_available` - Vérification que tous les types sont disponibles

## ✅ Points validés

- ✅ Framework compile sans erreurs
- ✅ Tous les types de formulaires fonctionnent
- ✅ Le pattern builder fonctionne correctement
- ✅ Les formulaires acceptent plusieurs champs
- ✅ La validation basique fonctionne
- ✅ L'intégration avec SeaORM fonctionne

## 📝 Exemple d'utilisation

```rust
#[test]
fn example_test() {
    use runique::prelude::*;

    let mut form = Forms::new("csrf_token");
    form.field(&TextField::text("username")
        .label("Nom d'utilisateur"));

    assert!(form.fields.contains_key("username"));
}
```

## 🔧 Commandes utiles

Run un test spécifique :
```bash
cargo test --test integration_tests test_text_field_creation -- --nocapture
```

Run avec backtrace :
```bash
RUST_BACKTRACE=1 cargo test --test integration_tests
```

Run en mode verbose :
```bash
cargo test --test integration_tests -- --nocapture
```
