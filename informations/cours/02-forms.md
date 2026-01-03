# Cours 2 : Système de Formulaires

## 🎯 Objectif

Créer un système de validation de formulaires type-safe inspiré de Django Forms.

## 📚 Concepts de base

### Architecture

```
Forms (conteneur)
  ├── errors: HashMap<String, String>
  └── cleaned_data: HashMap<String, Value>
       └── Données validées et typées

RuniqueField (trait)
  ├── CharField
  ├── IntegerField
  ├── EmailField
  └── ... (autres champs)
```

## 🔧 Implémentation étape par étape

### Étape 1 : Structure Forms

```rust
use std::collections::HashMap;
use serde_json::Value;

#[derive(Clone, Debug)]
pub struct Forms {
    pub errors: HashMap<String, String>,
    pub cleaned_data: HashMap<String, Value>,
}

impl Forms {
    pub fn new() -> Self {
        Self {
            errors: HashMap::new(),
            cleaned_data: HashMap::new(),
        }
    }

    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }
}
```

### Étape 2 : Trait RuniqueField

Le trait définit le comportement de tous les champs :

```rust
pub trait RuniqueField {
    type Output;  // Type de sortie après validation

    fn process(&self, raw_value: &str) -> Result<Self::Output, String>;

    fn strip(&self) -> bool {
        true  // Par défaut, on retire les espaces
    }
}
```

**Explication :**
- `Output` : Type associé (String, i64, etc.)
- `process()` : Valide et transforme la valeur
- `strip()` : Indique si on doit retirer les espaces

### Étape 3 : Implémenter CharField

```rust
pub struct CharField {
    pub allow_blank: bool,
}

impl RuniqueField for CharField {
    type Output = String;

    fn process(&self, raw_value: &str) -> Result<Self::Output, String> {
        // 1. Vérifier si vide
        if !self.allow_blank && raw_value.is_empty() {
            return Err("Ce champ ne peut pas être vide".to_string());
        }

        // 2. Sanitizer (nettoyer les entrées malveillantes)
        Ok(sanitize(raw_value))
    }
}
```

### Étape 4 : Implémenter IntegerField

```rust
pub struct IntegerField;

impl RuniqueField for IntegerField {
    type Output = i64;

    fn process(&self, raw_value: &str) -> Result<Self::Output, String> {
        raw_value
            .parse::<i64>()
            .map_err(|_| "Entrez un nombre entier".to_string())
    }
}
```

### Étape 5 : Méthode field() dans Forms

```rust
impl Forms {
    pub fn field<F: RuniqueField>(
        &mut self,
        name: &str,
        field: &F,
        raw_value: &str
    ) -> Option<F::Output>
    where
        F::Output: Serialize + Clone
    {
        // 1. Retirer les espaces si nécessaire
        let value_to_process = if field.strip() {
            raw_value.trim()
        } else {
            raw_value
        };

        // 2. Valider avec le champ
        match field.process(value_to_process) {
            Ok(value) => {
                // 3. Stocker dans cleaned_data
                if let Ok(json_val) = serde_json::to_value(value.clone()) {
                    self.cleaned_data.insert(name.to_string(), json_val);
                }
                Some(value)
            },
            Err(e) => {
                // 4. Stocker l'erreur
                self.errors.insert(name.to_string(), e);
                None
            }
        }
    }
}
```

### Étape 6 : require() et optional()

```rust
impl Forms {
    // Champ obligatoire
    pub fn require<F: RuniqueField>(
        &mut self,
        name: &str,
        field: &F,
        raw_data: &HashMap<String, String>
    ) where F::Output: Serialize + Clone {
        match raw_data.get(name) {
            Some(value) => {
                self.field(name, field, value);
            },
            None => {
                self.errors.insert(name.to_string(), "Requis".to_string());
            }
        }
    }

    // Champ optionnel
    pub fn optional<F: RuniqueField>(
        &mut self,
        name: &str,
        field: &F,
        raw_data: &HashMap<String, String>
    ) where F::Output: Serialize + Clone {
        if let Some(value) = raw_data.get(name) {
            self.field(name, field, value);
        }
        // Si absent, pas d'erreur
    }
}
```

### Étape 7 : Récupérer les valeurs typées

```rust
impl Forms {
    pub fn get_value<T: DeserializeOwned>(
        &self,
        field_name: &str
    ) -> Option<T> {
        self.cleaned_data
            .get(field_name)
            .and_then(|value| serde_json::from_value(value.clone()).ok())
    }
}
```

**Exemple d'utilisation :**
```rust
let age: Option<i64> = form.get_value("age");
let name: Option<String> = form.get_value("name");
```

### Étape 8 : Trait FormulaireTrait

Pour créer des formulaires personnalisés :

```rust
pub trait FormulaireTrait: Send + Sync + 'static {
    fn new() -> Self;
    fn validate(&mut self, raw_data: &HashMap<String, String>) -> bool;
}
```

**Exemple d'implémentation :**
```rust
struct UserForm {
    form: Forms,
}

impl FormulaireTrait for UserForm {
    fn new() -> Self {
        Self { form: Forms::new() }
    }

    fn validate(&mut self, raw_data: &HashMap<String, String>) -> bool {
        self.form.require("name", &CharField { allow_blank: false }, raw_data);
        self.form.optional("age", &IntegerField, raw_data);
        self.form.is_valid()
    }
}
```

## 🧪 Exemple complet

```rust
// 1. Créer un formulaire
let mut form = Forms::new();
let mut raw_data = HashMap::new();
raw_data.insert("name".to_string(), "John".to_string());
raw_data.insert("age".to_string(), "25".to_string());

// 2. Valider
form.require("name", &CharField { allow_blank: false }, &raw_data);
form.optional("age", &IntegerField, &raw_data);

// 3. Vérifier
if form.is_valid() {
    let name: String = form.get_value("name").unwrap();
    let age: Option<i64> = form.get_value("age");
    println!("Name: {}, Age: {:?}", name, age);
} else {
    for (field, error) in &form.errors {
        println!("{}: {}", field, error);
    }
}
```

## 🎓 Exercices

### Exercice 1 : Implémenter EmailField

Créez un champ qui valide les emails :
- Doit contenir `@`
- Doit contenir `.`
- Longueur minimale

### Exercice 2 : Ajouter des validateurs personnalisés

Permettez d'ajouter des fonctions de validation supplémentaires :
```rust
form.field_with_validator("password", &CharField, |value| {
    if value.len() < 8 {
        Err("Trop court".to_string())
    } else {
        Ok(value)
    }
});
```

### Exercice 3 : Implémenter clean() comme Django

Ajoutez une méthode `clean()` qui permet la validation croisée :
```rust
fn clean(&mut self) {
    let password: Option<String> = self.form.get_value("password");
    let confirm: Option<String> = self.form.get_value("password_confirm");

    if password != confirm {
        self.form.errors.insert(
            "password_confirm".to_string(),
            "Les mots de passe ne correspondent pas".to_string()
        );
    }
}
```

## 💡 Bonnes pratiques

1. **Type safety** : Utilisez les types associés pour la sécurité des types
2. **Séparation des responsabilités** : Forms gère les données, Fields gèrent la validation
3. **Immutabilité** : Les champs sont immutables, Forms est mutable
4. **Erreurs descriptives** : Messages d'erreur clairs pour l'utilisateur

## 🔗 Ressources

- [Serde documentation](https://serde.rs/)
- [Django Forms](https://docs.djangoproject.com/en/stable/topics/forms/)
