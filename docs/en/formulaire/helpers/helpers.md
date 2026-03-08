# Typed conversion helpers

[← RuniqueForm trait](https://github.com/seb-alliot/runique/blob/main/docs/en/formulaire/trait/trait.md)

---

Form values are stored as `String`. Instead of parsing manually, use the typed helpers on `Forms`:

## Direct conversions

```rust
form.get_string("username")     // -> String ("" if empty)
form.get_i32("age")              // -> i32 (0 by default)
form.get_i64("count")            // -> i64 (0 by default)
form.get_u32("quantity")         // -> u32 (0 by default)
form.get_u64("id")               // -> u64 (0 by default)
form.get_f32("ratio")            // -> f32 (handles , → .)
form.get_f64("price")            // -> f64 (handles , → .)
form.get_bool("active")          // -> bool (true/1/on → true)
```

## Option conversions

```rust
form.get_option("bio")           // -> Option<String> (None if empty)
form.get_option_i32("age")       // -> Option<i32>
form.get_option_i64("score")     // -> Option<i64>
form.get_option_f64("note")      // -> Option<f64> (handles , → .)
form.get_option_bool("news")     // -> Option<bool>
```

## Date / Time conversions

```rust
form.get_naive_date("birthday")           // -> NaiveDate (default if empty)
form.get_naive_time("meeting_time")       // -> NaiveTime (default if empty)
form.get_naive_datetime("event_start")    // -> NaiveDateTime (default if empty)
form.get_datetime_utc("created_at")       // -> DateTime<Utc> (Utc::now() if empty)

form.get_option_naive_date("birthday")        // -> Option<NaiveDate>
form.get_option_naive_time("meeting_time")    // -> Option<NaiveTime>
form.get_option_naive_datetime("event_start") // -> Option<NaiveDateTime>
form.get_option_datetime_utc("created_at")    // -> Option<DateTime<Utc>>
```

## UUID conversions

```rust
form.get_uuid("external_id")         // -> Uuid (Uuid::nil() if empty)
form.get_option_uuid("external_id")  // -> Option<Uuid>
```

## Usage in save()

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

> **💡** Float helpers (`get_f32`, `get_f64`, `get_option_f64`) automatically convert commas to dots (`19,99` → `19.99`) for French locales.

---

← [**RuniqueForm trait**](https://github.com/seb-alliot/runique/blob/main/docs/en/formulaire/trait/trait.md) | [**Field types**](https://github.com/seb-alliot/runique/blob/main/docs/en/formulaire/fields/fields.md) →
