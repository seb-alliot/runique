# Helpers de conversion typée

[← Trait RuniqueForm](/docs/fr/formulaire/trait)

---

Les valeurs de formulaire sont stockées en `String`. Utilisez les helpers typés `cleaned_*` pour les convertir de façon sécurisée (whitelistée) et typée.

## API recommandée — `cleaned_*()`

Méthodes du trait `RuniqueForm`. Elles sont whitelistées (champ non déclaré → `None`) et couvrent par priorité : POST > Paramètre de route > Query string.

### Basiques et Numériques

```rust
self.cleaned_string("username")      // Option<String> (None si vide)
self.cleaned_i32("age")              // Option<i32>
self.cleaned_i64("count")            // Option<i64>
self.cleaned_u32("quantity")         // Option<u32>
self.cleaned_u64("id")               // Option<u64>
self.cleaned_f32("ratio")            // Option<f32> (gère , → .)
self.cleaned_f64("price")            // Option<f64> (gère , → .)
self.cleaned_bool("active")          // Option<bool> (true/1/on → true)
```

### Date / Heure / UUID

```rust
self.cleaned_naive_date("birthday")           // Option<NaiveDate> (format YYYY-MM-DD)
self.cleaned_naive_time("meeting_time")       // Option<NaiveTime> (format HH:MM)
self.cleaned_naive_datetime("event_start")    // Option<NaiveDateTime> (YYYY-MM-DDTHH:MM)
self.cleaned_datetime_utc("created_at")       // Option<DateTime<Utc>> (RFC3339)

self.cleaned_uuid("external_id")              // Option<Uuid>
```

> **💡** Les helpers float (`cleaned_f32`, `cleaned_f64`) convertissent automatiquement la virgule en point (`19,99` → `19.99`) pour simplifier la saisie.
> 
> **Note de sécurité :** Toutes les variantes `cleaned_*` retournent `Option`. Pour une valeur par défaut : `.unwrap_or_default()` ou `.unwrap_or(0)`.

---

## Utilisation dans `save()`

Dans vos méthodes `save()`, utilisez exclusivement `cleaned_*` pour extraire les données :

```rust
impl RegisterForm {
    pub async fn save(&self, db: &DatabaseConnection) -> Result<users::Model, DbErr> {
        let model = users::ActiveModel {
            username: Set(self.cleaned_string("username").unwrap_or_default()),
            email: Set(self.cleaned_string("email").unwrap_or_default()),
            age: Set(self.cleaned_i32("age").unwrap_or(0)),
            website: Set(self.cleaned_string("website")), // Option<String>
            ..Default::default()
        };
        model.insert(db).await
    }
}
```

---

## Accès aux paramètres d'URL

### Depuis `Request` (Accès brut)

```rust
// Paramètre de route : /article/{id}
let id = request.path_param("id");       // Option<&str>

// Paramètre de query string : ?page=2
let page = request.from_url("page");     // Option<&str>
```

### Depuis le formulaire (Recommandé)

Les variantes `cleaned_*` fusionnent automatiquement les données du corps de la requête et de l'URL.

Exemple — pré-remplir un champ depuis l'URL (`GET /edit?title=Mon+Article`) :

```rust
if request.is_get() {
    if let Some(t) = form.cleaned_string("title") {
        form.get_form_mut().add_value("title", &t);
    }
}
```

---

[← Trait RuniqueForm](/docs/fr/formulaire/trait) | [**Types de champs**](/docs/fr/formulaire/champs) →
