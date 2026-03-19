# Helpers de conversion typée

[← Trait RuniqueForm](https://github.com/seb-alliot/runique/blob/main/docs/fr/formulaire/trait/trait.md)

---

Les valeurs de formulaire sont stockées en `String`. Plutôt que de parser manuellement, utilisez les helpers typés sur `Forms` :

## Conversions directes

```rust
form.get_string("username")     // -> String ("" si vide)
form.get_i32("age")              // -> i32 (0 par défaut)
form.get_i64("count")            // -> i64 (0 par défaut)
form.get_u32("quantity")         // -> u32 (0 par défaut)
form.get_u64("id")               // -> u64 (0 par défaut)
form.get_f32("ratio")            // -> f32 (gère , → .)
form.get_f64("price")            // -> f64 (gère , → .)
form.get_bool("active")          // -> bool (true/1/on → true)
```

## Conversions Option

```rust
form.get_option("bio")           // -> Option<String> (None si vide)
form.get_option_i32("age")       // -> Option<i32>
form.get_option_i64("score")     // -> Option<i64>
form.get_option_f64("note")      // -> Option<f64> (gère , → .)
form.get_option_bool("news")     // -> Option<bool>
```

## Conversions Date / Heure

```rust
form.get_naive_date("birthday")           // -> NaiveDate (default si vide)
form.get_naive_time("meeting_time")       // -> NaiveTime (default si vide)
form.get_naive_datetime("event_start")    // -> NaiveDateTime (default si vide)
form.get_datetime_utc("created_at")       // -> DateTime<Utc> (Utc::now() si vide)

form.get_option_naive_date("birthday")        // -> Option<NaiveDate>
form.get_option_naive_time("meeting_time")    // -> Option<NaiveTime>
form.get_option_naive_datetime("event_start") // -> Option<NaiveDateTime>
form.get_option_datetime_utc("created_at")    // -> Option<DateTime<Utc>>
```

## Conversions UUID

```rust
form.get_uuid("external_id")         // -> Uuid (Uuid::nil() si vide)
form.get_option_uuid("external_id")  // -> Option<Uuid>
```

## Utilisation dans save()

```rust
impl RegisterForm {
    pub async fn save(&self, db: &DatabaseConnection) -> Result<users::Model, DbErr> {
        let model = users::ActiveModel {
            username: Set(self.form.get_string("username")),
            email: Set(self.form.get_string("email")),
            password: Set(self.form.get_string("password")),
            age: Set(self.form.get_i32("age")),
            website: Set(self.form.get_option("website")),  // Option<String>
            ..Default::default()
        };
        model.insert(db).await
    }
}
```

> **💡** Les helpers float (`get_f32`, `get_f64`, `get_option_f64`) convertissent automatiquement la virgule en point (`19,99` → `19.99`) pour les locales françaises.

---

## Accès aux paramètres d'URL

### Depuis `Request` — accès brut

```rust
// Paramètre de route : /article/{id}
let id = request.path_param("id");       // Option<&str>

// Paramètre de query string : ?page=2
let page = request.from_url("page");     // Option<&str>
```

### Depuis le formulaire — `cleaned_*()` whitelisté et typé

Les variantes `cleaned_*` couvrent **toutes les sources** (POST, path param, query param) dans cet ordre de priorité. Elles retournent `None` si le champ n'est pas déclaré dans le formulaire.

```rust
form.cleaned_string("search")   // Option<String>
form.cleaned_i32("page")        // Option<i32>
form.cleaned_i64("id")          // Option<i64>
form.cleaned_u32("quantity")    // Option<u32>
form.cleaned_u64("ref")         // Option<u64>
form.cleaned_f32("ratio")       // Option<f32>  (gère , → .)
form.cleaned_f64("price")       // Option<f64>  (gère , → .)
form.cleaned_bool("active")     // Option<bool> (true/1/on → true)

// Champ non déclaré → None garanti, aucune fuite possible
form.cleaned_string("is_admin") // None
```

Cas concret — pré-remplir un champ depuis l'URL (`GET /edit?title=Mon+Article`) :

```rust
pub async fn edit(
    mut request: Request,
    Prisme(mut form): Prisme<ArticleForm>,
) -> AppResult<Response> {
    if request.is_get() {
        if let Some(t) = form.cleaned_string("title") {
            form.get_form_mut().add_value("title", &t);
        }
    }

    if request.is_post() && form.is_valid().await {
        form.save(&request.engine.db).await?;
        return Ok(Redirect::to("/articles").into_response());
    }

    context_update!(request => { "form" => &form });
    request.render("edit.html")
}
```

> **Sécurité** — `cleaned_*()` est liée au schéma du formulaire : un attaquant ne peut pas injecter un paramètre URL arbitraire (`?is_admin=true`) qui ne soit pas un champ déclaré. Fonctionne aussi bien avec `#[form(...)]` qu'avec les formulaires classiques.

---

← [**Trait RuniqueForm**](https://github.com/seb-alliot/runique/blob/main/docs/fr/formulaire/trait/trait.md) | [**Types de champs**](https://github.com/seb-alliot/runique/blob/main/docs/fr/formulaire/champs/champs.md) →
