# Cours 1 : Routing et Reverse Routing

## 🎯 Objectif

Apprendre à créer un système de routing nommé avec reverse routing, similaire à Django.

## 📚 Concepts de base

### 1. Qu'est-ce que le reverse routing ?

Le reverse routing permet de générer des URLs à partir du nom d'une route, plutôt que d'écrire l'URL en dur.

**Exemple Django :**
```python
# urls.py
path('user/<int:id>/', views.user_detail, name='user_detail')

# Dans un template
{% url 'user_detail' id=123 %}  # Génère: /user/123/
```

**Notre objectif en Rust :**
```rust
register_name_url("user_detail", "/user/{id}");
let url = reverse_with_parameters("user_detail", &[("id", "123")]);
// url = Some("/user/123".to_string())
```

## 🔧 Implémentation étape par étape

### Étape 1 : Stockage des routes nommées

Nous avons besoin d'une structure de données globale pour stocker les associations nom → chemin.

```rust
use std::collections::HashMap;
use std::sync::RwLock;
use once_cell::sync::Lazy;

// Structure globale thread-safe pour stocker les routes
static NAME_URL: Lazy<RwLock<HashMap<String, String>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));
```

**Explication :**
- `Lazy` : Initialisation paresseuse (créé seulement quand nécessaire)
- `RwLock` : Permet plusieurs lecteurs ou un seul écrivain
- `HashMap<String, String>` : Associe nom de route → chemin

### Étape 2 : Enregistrer une route

```rust
pub fn register_name_url(name: impl Into<String>, path: impl Into<String>) {
    let mut name_url_map = NAME_URL.write().unwrap();
    name_url_map.insert(name.into(), path.into());
}
```

**Comment ça marche :**
1. Obtient un verrou en écriture
2. Insère le nom et le chemin dans la HashMap
3. Le verrou est libéré automatiquement à la fin de la fonction

### Étape 3 : Reverse simple (sans paramètres)

```rust
pub fn reverse(name: &str) -> Option<String> {
    let name_url_map = NAME_URL.read().unwrap();
    name_url_map.get(name).cloned()
}
```

**Explication :**
- `read()` : Verrou en lecture (plusieurs lecteurs possibles)
- `get(name)` : Récupère le chemin associé au nom
- `cloned()` : Clone la String (Option<String>)
- Retourne `None` si la route n'existe pas

### Étape 4 : Reverse avec paramètres

```rust
pub fn reverse_with_parameters(
    name: &str,
    parameters: &[(&str, &str)]
) -> Option<String> {
    // 1. Récupérer le chemin de base
    let path = reverse(name)?;  // Retourne None si route inexistante

    // 2. Remplacer les placeholders {key} par les valeurs
    Some(
        parameters.iter().fold(path, |acc, (key, value)| {
            acc.replace(&format!("{{{}}}", key), value)
        })
    )
}
```

**Explication :**
- `fold()` : Itère sur les paramètres et remplace chaque `{key}` par sa valeur
- `format!("{{{}}}", key)` : Crée `{id}` à partir de `id`
- `replace()` : Remplace toutes les occurrences

**Exemple :**
```rust
register_name_url("user", "/user/{id}");
let url = reverse_with_parameters("user", &[("id", "123")]);
// url = Some("/user/123".to_string())
```

### Étape 5 : Macro urlpatterns!

La macro permet une syntaxe plus lisible :

```rust
#[macro_export]
macro_rules! urlpatterns {
    // Version avec noms
    (
        $($path:expr => $handler:expr, name = $name:expr) ,* $(,)?
    ) => {{
        let mut router = $crate::Router::new();

        $(
            // Enregistrer le nom de la route
            $crate::register_name_url($name, $path);
            // Ajouter la route au router
            router = router.route($path, $handler);
        )*
        router
    }};

    // Version sans noms
    (
        $($path:expr => $handler:expr) , * $(,)?
    ) => {{
        let mut router = $crate::Router::new();
        $(
            router = router.route($path, $handler);
        )*
        router
    }};
}
```

**Explication de la macro :**
- `$($path:expr => ...)` : Répétition (peut être plusieurs routes)
- `$(,)?` : Virgule optionnelle à la fin
- `$()` : Répétition dans le bloc de code
- `*` : Zéro ou plusieurs répétitions

## 🧪 Tests

```rust
#[test]
fn test_register_and_reverse() {
    register_name_url("home", "/");
    assert_eq!(reverse("home"), Some("/".to_string()));
}

#[test]
fn test_reverse_with_params() {
    register_name_url("user", "/user/{id}");
    let url = reverse_with_parameters("user", &[("id", "123")]);
    assert_eq!(url, Some("/user/123".to_string()));
}
```

## 🎓 Exercices

### Exercice 1 : Implémenter reverse avec plusieurs paramètres

Modifiez `reverse_with_parameters` pour gérer plusieurs paramètres dans l'ordre.

**Indice :** Utilisez `fold()` ou une boucle.

### Exercice 2 : Ajouter la validation des paramètres

Vérifiez que tous les placeholders `{key}` dans le chemin ont un paramètre correspondant.

### Exercice 3 : Support des query parameters

Ajoutez une fonction `reverse_with_query` qui génère des URLs avec query string :
```rust
reverse_with_query("search", &[("q", "rust")])
// → Some("/search?q=rust".to_string())
```

## 💡 Bonnes pratiques

1. **Thread-safety** : Utilisez `RwLock` pour les accès concurrents
2. **Option plutôt que panic** : Retournez `Option` pour les routes inexistantes
3. **Clonage minimal** : Clonez seulement quand nécessaire
4. **Documentation** : Documentez les fonctions publiques

## 🔗 Ressources

- [Rust Book - Macros](https://doc.rust-lang.org/book/ch19-06-macros.html)
- [once_cell documentation](https://docs.rs/once_cell/)
- [RwLock documentation](https://doc.rust-lang.org/std/sync/struct.RwLock.html)
