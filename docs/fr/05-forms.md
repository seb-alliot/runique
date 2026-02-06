# 📋 Formulaires

## Vue d'ensemble

Runique fournit un système de formulaires puissant inspiré de Django. Il existe **deux approches** :

1. **Manuelle** — Définir les champs via le trait `RuniqueForm`
2. **Automatique** — Dériver un formulaire depuis un modèle SeaORM avec `#[derive(DeriveModelForm)]`

Les formulaires sont extraits automatiquement des requêtes via l'extracteur **Prisme**, gèrent la validation (y compris via le crate `validator` pour les emails/URLs), le CSRF, le hachage Argon2 des mots de passe, et peuvent être sauvegardés directement en base de données.

---

## Extracteur Prisme

`Prisme<T>` est un extracteur Axum qui orchestre un pipeline complet en coulisses :

1. **Sentinel** — Vérifie les règles d'accès (login, rôles) via `GuardRules`
2. **Aegis** — Extraction unique du body (multipart, urlencoded, json) normalisée en `HashMap`
3. **CSRF Gate** — Vérifie le token CSRF dans les données parsées
4. **Construction** — Crée le formulaire `T`, remplit les champs, lance la validation

```rust
use runique::prelude::*;

pub async fn inscription(
    mut request: Request,
    Prisme(mut form): Prisme<RegisterForm>,
) -> AppResult<Response> {
    if request.is_post() {
        if form.is_valid().await {
            // Formulaire valide → traiter
        }
    }
    // ...
}
```

> **💡** Le développeur écrit juste `Prisme(mut form)` — tout le pipeline sécurité est transparent.

---

## Approche manuelle : trait RuniqueForm

### Structure de base

Chaque formulaire contient un champ `form: Forms` et implémente le trait `RuniqueForm` :

```rust
use runique::prelude::*;

#[derive(Serialize, Debug, Clone)]
#[serde(transparent)]
pub struct UsernameForm {
    pub form: Forms,
}

impl RuniqueForm for UsernameForm {
    fn register_fields(form: &mut Forms) {
        form.field(
            &TextField::text("username")
                .label("Nom d'utilisateur")
                .required()
                .placeholder("Entrez un nom d'utilisateur"),
        );
    }

    impl_form_access!();
}
```

> **💡 `impl_form_access!()`** génère automatiquement `from_form()`, `get_form()` et `get_form_mut()`. Si votre champ ne s'appelle pas `form`, passez le nom en argument : `impl_form_access!(formulaire);`

<details>
<summary>Équivalent sans macro (pour référence)</summary>

```rust
fn from_form(form: Forms) -> Self {
    Self { form }
}
fn get_form(&self) -> &Forms {
    &self.form
}
fn get_form_mut(&mut self) -> &mut Forms {
    &mut self.form
}
```
</details>

### Méthodes du trait RuniqueForm

| Méthode | Rôle |
|---------|------|
| `register_fields(form)` | Déclare les champs du formulaire |
| `from_form(form)` | Construit l'instance depuis un `Forms` |
| `get_form()` / `get_form_mut()` | Accesseurs vers le `Forms` interne |
| `clean()` | Logique métier croisée (ex: `mdp1 == mdp2`) — **optionnel** |
| `is_valid()` | Pipeline complet : validation champs → `clean()` → `finalize()` |
| `database_error(&err)` | Injecte une erreur DB sur le bon champ |
| `build(tera, csrf_token)` | Construit un formulaire vide |
| `build_with_data(data, tera, csrf)` | Construit, remplit et valide |

### Pipeline de validation `is_valid()`

L'appel `form.is_valid().await` déclenche **3 étapes dans l'ordre** :

1. **Validation des champs** — Chaque champ exécute son `validate()` : requis, longueur, format (email via `validator`, URL via `validator`, JSON via `serde_json`, UUID via `uuid`, IP via `std::net::IpAddr`…)
2. **`clean()`** — Logique métier personnalisée (les mots de passe sont encore en clair à cette étape, ce qui permet de comparer `mdp1 == mdp2`)
3. **`finalize()`** — Transformations finales (hachage Argon2 automatique des champs `Password`)

```rust
#[async_trait::async_trait]
impl RuniqueForm for RegisterForm {
    // ...

    async fn clean(&mut self) -> Result<(), StrMap> {
        let mdp1 = self.form.get_string("password");
        let mdp2 = self.form.get_string("password_confirm");

        if mdp1 != mdp2 {
            let mut errors = StrMap::new();
            errors.insert(
                "password_confirm".to_string(),
                "Les mots de passe ne correspondent pas".to_string(),
            );
            return Err(errors);
        }
        Ok(())
    }
}
```

> **⚠️ Important** : Après `is_valid()`, les champs `Password` sont **automatiquement hachés en Argon2**. Utilisez `clean()` pour toute comparaison de mots de passe en clair.

---

## Helpers de conversion typée

Les valeurs de formulaire sont stockées en `String`. Plutôt que de parser manuellement, utilisez les helpers typés sur `Forms` :

### Conversions directes

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

### Conversions Option (None si vide)

```rust
form.get_option("bio")           // -> Option<String> (None si vide)
form.get_option_i32("age")       // -> Option<i32>
form.get_option_i64("score")     // -> Option<i64>
form.get_option_f64("note")      // -> Option<f64> (gère , → .)
form.get_option_bool("news")     // -> Option<bool>
```

### Utilisation dans save()

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

## Types de champs

### TextField — Champs texte

Le `TextField` gère 6 formats spéciaux via l'enum `SpecialFormat` :

```rust
// Texte simple
form.field(&TextField::text("username").label("Nom").required());

// Email — validé via `validator::ValidateEmail`
form.field(&TextField::email("email").label("Email").required());

// URL — validée via `validator::ValidateUrl`
form.field(&TextField::url("website").label("Site web"));

// Mot de passe — hachage Argon2 automatique dans finalize(), jamais ré-affiché en HTML
form.field(
    &TextField::password("password")
        .label("Mot de passe")
        .required()
        .min_length(8, "Min 8 caractères"),
);

// Textarea
form.field(&TextField::textarea("summary").label("Résumé"));

// RichText — sanitisation XSS automatique avant validation
form.field(&TextField::richtext("content").label("Contenu"));
```

**Options du builder :**

```rust
TextField::text("nom")
    .label("Mon champ")              // Label affiché
    .placeholder("Entrez...")        // Placeholder
    .required()                       // Obligatoire (message par défaut)
    .min_length(3, "Trop court")     // Longueur min avec message
    .max_length(100, "Trop long")    // Longueur max avec message
    .readonly("Lecture seule")       // Lecture seule
    .disabled("Désactivé")          // Désactivé
```

**Comportements automatiques par format :**

| Format | Validation | Transformation |
|--------|-----------|----------------|
| `Email` | `validator::ValidateEmail` | Conversion en lowercase |
| `Url` | `validator::ValidateUrl` | — |
| `Password` | Standard | Hachage Argon2 dans `finalize()`, valeur vidée au `render()` |
| `RichText` | Standard | Sanitisation XSS (`sanitize()`) avant validation |
| `Csrf` | Token session | — |

**Utilitaires mot de passe :**

```rust
// Hacher manuellement
let hash = field.hash_password()?;

// Vérifier un mot de passe
let ok = TextField::verify_password("mdp_clair", "$argon2...");
```

> Le hachage automatique détecte si la valeur commence déjà par `$argon2` pour éviter un double hachage.

### NumericField — Champs numériques

5 variantes via l'enum `NumericConfig` :

```rust
// Entier avec bornes
form.field(
    &NumericField::integer("age")
        .label("Âge")
        .min(0.0, "Min 0")
        .max(150.0, "Max 150"),
);

// Nombre flottant
form.field(&NumericField::float("price").label("Prix"));

// Nombre décimal avec précision
form.field(
    &NumericField::decimal("amount")
        .label("Montant")
        .digits(2, 4),  // min 2, max 4 chiffres après la virgule
);

// Pourcentage (0–100 par défaut)
form.field(&NumericField::percent("rate").label("Taux"));

// Range slider avec min, max, valeur par défaut
form.field(
    &NumericField::range("volume", 0.0, 100.0, 50.0)
        .label("Volume")
        .step(5.0),
);
```

**Options : `.min(val, msg)`, `.max(val, msg)`, `.step(val)`, `.digits(min, max)`, `.label(l)`, `.placeholder(p)`**

### BooleanField — Cases à cocher / Radio simple

```rust
// Checkbox simple
form.field(
    &BooleanField::new("accept_terms")
        .label("J'accepte les conditions")
        .required(),
);

// Radio simple (oui/non)
form.field(&BooleanField::radio("newsletter").label("Newsletter"));

// Pré-coché
form.field(&BooleanField::new("remember_me").label("Se souvenir").checked());
```

### ChoiceField — Select / Dropdown

```rust
use runique::forms::fields::choice::ChoiceOption;

let choices = vec![
    ChoiceOption::new("fr", "France"),
    ChoiceOption::new("be", "Belgique"),
    ChoiceOption::new("ch", "Suisse"),
];

// Select simple
form.field(
    &ChoiceField::new("country")
        .label("Pays")
        .choices(choices.clone())
        .required(),
);

// Select multiple
form.field(
    &ChoiceField::new("languages")
        .label("Langues")
        .choices(choices)
        .multiple(),
);
```

> La validation vérifie automatiquement que la valeur soumise fait partie des choix déclarés.

### RadioField — Boutons radio

```rust
form.field(
    &RadioField::new("gender")
        .label("Genre")
        .choices(vec![
            ChoiceOption::new("m", "Masculin"),
            ChoiceOption::new("f", "Féminin"),
            ChoiceOption::new("o", "Autre"),
        ])
        .required(),
);
```

### CheckboxField — Checkboxes multiples

```rust
form.field(
    &CheckboxField::new("hobbies")
        .label("Loisirs")
        .choices(vec![
            ChoiceOption::new("sport", "Sport"),
            ChoiceOption::new("musique", "Musique"),
            ChoiceOption::new("lecture", "Lecture"),
        ]),
);
```

> Les valeurs soumises sont au format `"val1,val2,val3"`. La validation vérifie que chaque valeur existe dans les choix.

### DateField, TimeField, DateTimeField — Date / Heure

```rust
use chrono::NaiveDate;

// Date (format: YYYY-MM-DD)
form.field(
    &DateField::new("birthday")
        .label("Date de naissance")
        .min(NaiveDate::from_ymd_opt(1900, 1, 1).unwrap(), "Trop ancien")
        .max(NaiveDate::from_ymd_opt(2010, 12, 31).unwrap(), "Trop récent"),
);

// Heure (format: HH:MM)
form.field(&TimeField::new("meeting_time").label("Heure du RDV"));

// Date + Heure (format: YYYY-MM-DDTHH:MM)
form.field(&DateTimeField::new("event_start").label("Début de l'événement"));
```

### DurationField — Durée

```rust
form.field(
    &DurationField::new("timeout")
        .label("Délai (secondes)")
        .min_seconds(60, "Minimum 1 minute")
        .max_seconds(3600, "Maximum 1 heure"),
);
```

### FileField — Upload de fichiers

```rust
use runique::config::StaticConfig;

let config = StaticConfig::from_env();

// Image avec contraintes complètes
form.field(
    &FileField::image("avatar")
        .label("Photo de profil")
        .upload_to(&config)
        .max_size_mb(5)
        .max_files(1)
        .max_dimensions(1920, 1080)
        .allowed_extensions(vec!["png", "jpg", "jpeg", "webp"]),
);

// Document
form.field(
    &FileField::document("cv")
        .label("CV")
        .max_size_mb(10),
);

// Fichier quelconque (multi-fichiers)
form.field(
    &FileField::any("attachments")
        .label("Pièces jointes")
        .max_files(5),
);
```

> **Sécurité** : les fichiers `.svg` sont **toujours refusés** par défaut (risque XSS). La validation d'image utilise le crate `image` pour vérifier le format réel du fichier.

### Fichiers JS associés

Vous pouvez ajouter des fichiers JavaScript spécifiques à un formulaire :

```rust
fn register_fields(form: &mut Forms) {
    // ... champs ...
    form.add_js(&["js/mon_script.js", "js/autre.js"]);
}
```

Les fichiers JS sont inclus automatiquement dans le rendu HTML du formulaire.

### ColorField — Sélecteur de couleur

```rust
form.field(
    &ColorField::new("theme_color")
        .label("Couleur du thème")
        .default_color("#3498db"),  // Valide le format #RGB ou #RRGGBB
);
```

### SlugField — Slug URL-friendly

```rust
form.field(
    &SlugField::new("slug")
        .label("Slug")
        .placeholder("mon-article-url")
        .allow_unicode(),  // Optionnel : autorise les caractères unicode
);
```

> Validation : lettres, chiffres, tirets, underscores uniquement. Ne peut pas commencer ou finir par un tiret.

### UUIDField

```rust
form.field(
    &UUIDField::new("external_id")
        .label("ID externe")
        .placeholder("xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx"),
);
```

### JSONField — Textarea avec validation JSON

```rust
form.field(
    &JSONField::new("metadata")
        .label("Métadonnées")
        .placeholder(r#"{"clé": "valeur"}"#)
        .rows(10),  // Nombre de lignes du textarea
);
```

### IPAddressField — Adresse IP

```rust
// IPv4 + IPv6
form.field(&IPAddressField::new("server_ip").label("IP du serveur"));

// IPv4 uniquement
form.field(&IPAddressField::new("gateway").label("Passerelle").ipv4_only());

// IPv6 uniquement
form.field(&IPAddressField::new("ipv6").label("Adresse IPv6").ipv6_only());
```

---

## Récapitulatif des types de champs

| Struct | Constructeurs | Validation spéciale |
|--------|-------------|---------------------|
| `TextField` | `text()`, `email()`, `url()`, `password()`, `textarea()`, `richtext()` | Email/URL via `validator`, Argon2, sanitisation XSS |
| `NumericField` | `integer()`, `float()`, `decimal()`, `percent()`, `range()` | Bornes min/max, précision décimale |
| `BooleanField` | `new()`, `radio()` | Requis = doit être coché |
| `ChoiceField` | `new()` + `.multiple()` | Valeur dans les choix déclarés |
| `RadioField` | `new()` | Valeur dans les choix déclarés |
| `CheckboxField` | `new()` | Toutes les valeurs dans les choix |
| `DateField` | `new()` | Format `YYYY-MM-DD`, bornes min/max |
| `TimeField` | `new()` | Format `HH:MM`, bornes min/max |
| `DateTimeField` | `new()` | Format `YYYY-MM-DDTHH:MM`, bornes min/max |
| `DurationField` | `new()` | Secondes, bornes min/max |
| `FileField` | `image()`, `document()`, `any()` | Extensions, taille, dimensions, anti-SVG |
| `ColorField` | `new()` | Format `#RRGGBB` ou `#RGB` |
| `SlugField` | `new()` | ASCII/unicode, pas de tiret en début/fin |
| `UUIDField` | `new()` | Format UUID valide |
| `JSONField` | `new()` | JSON valide via `serde_json` |
| `IPAddressField` | `new()` + `.ipv4_only()` / `.ipv6_only()` | IPv4/IPv6 via `std::net::IpAddr` |

---

## Approche automatique : DeriveModelForm

Pour les cas simples, dérivez directement un formulaire depuis un modèle SeaORM :

```rust
use runique::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub username: String,
    pub email: String,
    pub password: String,
    pub bio: Option<String>,
    pub age: Option<i32>,
    pub is_active: bool,
    pub created_at: DateTime,
}

// Génère automatiquement : pub struct ModelForm { pub form: Forms }
#[derive(DeriveModelForm)]
pub struct Model;
```

### Champs auto-exclus

`DeriveModelForm` exclut automatiquement :
- `id` (clé primaire)
- `csrf_token`
- `created_at`, `updated_at`
- `is_active`, `deleted_at`
- Tout champ marqué `#[sea_orm(primary_key)]`

### Détection automatique des types

| Règle | Type de champ généré | Helper dans `to_active_model()` |
|-------|---------------------|---|
| Nom contient `email` | `TextField::email()` | `get_string()` |
| Nom contient `password` / `pwd` | `TextField::password()` | `get_string()` |
| Nom contient `url` / `link` / `website` | `TextField::url()` | `get_string()` |
| `String` + nom `description` / `bio` / `content` / `message` | `TextField::textarea()` | `get_string()` |
| `String` (autre) | `TextField::text()` | `get_string()` |
| `i32` | `NumericField::integer()` | `get_i32()` |
| `i64` | `NumericField::integer()` | `get_i64()` |
| `u32` | `NumericField::integer()` | `get_u32()` |
| `u64` | `NumericField::integer()` | `get_u64()` |
| `f32` | `NumericField::float()` | `get_f32()` |
| `f64` | `NumericField::float()` | `get_f64()` |
| `bool` | `BooleanField::new()` | `get_bool()` |
| `Option<T>` | Champ **non required** | `get_option()` |
| Non-`Option<T>` | Champ **required** | Type correspondant |

### Attributs de personnalisation

```rust
#[derive(DeriveModelForm)]
#[exclude(bio, age)]  // Exclure des champs supplémentaires
pub struct Model;
```

---

## Erreurs de base de données

La méthode `database_error()` analyse automatiquement les erreurs DB pour remonter l'erreur au bon champ :

```rust
match form.save(&request.engine.db).await {
    Ok(_) => { /* succès */ }
    Err(err) => {
        form.database_error(&err);
        // L'erreur est positionnée sur le champ concerné
    }
}
```

**Formats d'erreur supportés :**
- **PostgreSQL** : `UNIQUE constraint`, `Key (field)=(value)`
- **SQLite** : `UNIQUE constraint failed: table.field`
- **MySQL** : `Duplicate entry ... for key 'table.field'`

Si le champ est identifié, l'erreur apparaît sur ce champ (ex: « Ce email est déjà utilisé »).
Sinon, elle est ajoutée aux erreurs globales.

---

## Rendu dans les templates

### Formulaire complet

```html
<form method="post">
    {% form.inscription_form %}
    <button type="submit">S'inscrire</button>
</form>
```

Rend automatiquement : tous les champs, les labels, les erreurs de validation, le token CSRF et les scripts JS.

### Champ par champ

```html
<form method="post">
    {% csrf %}
    <div class="row">
        <div class="col-6">{% form.inscription_form.username %}</div>
        <div class="col-6">{% form.inscription_form.email %}</div>
    </div>
    {% form.inscription_form.password %}
    <button type="submit">S'inscrire</button>
</form>
```

### Erreurs globales

```html
{% if inscription_form.global_errors %}
    <div class="alert alert-danger">
        {% for msg in inscription_form.global_errors %}
            <p>{{ msg }}</p>
        {% endfor %}
    </div>
{% endif %}
```

### Données de champ en JSON

Les formulaires sérialisent automatiquement `data`, `errors`, `global_errors`, `html`, `rendered_fields`, `fields` et `js_files`.

---

## Exemple complet : inscription avec sauvegarde

```rust
use runique::prelude::*;

#[derive(Serialize, Debug, Clone)]
#[serde(transparent)]
pub struct RegisterForm {
    pub form: Forms,
}

impl RuniqueForm for RegisterForm {
    fn register_fields(form: &mut Forms) {
        form.field(
            &TextField::text("username")
                .label("Nom d'utilisateur")
                .required(),
        );

        form.field(
            &TextField::email("email")
                .label("Email")
                .required(),
        );

        form.field(
            &TextField::password("password")
                .label("Mot de passe")
                .required()
                .min_length(8, "Minimum 8 caractères"),
        );
    }

    impl_form_access!();
}

impl RegisterForm {
    pub async fn save(&self, db: &DatabaseConnection) -> Result<users::Model, DbErr> {
        use sea_orm::Set;
        let model = users::ActiveModel {
            username: Set(self.form.get_string("username")),
            email: Set(self.form.get_string("email")),
            // Le mot de passe est déjà haché en Argon2 après is_valid()
            password: Set(self.form.get_string("password")),
            ..Default::default()
        };
        model.insert(db).await
    }
}
```

### Handler GET/POST

```rust
pub async fn inscription(
    mut request: Request,
    Prisme(mut form): Prisme<RegisterForm>,
) -> AppResult<Response> {
    let template = "profile/register_form.html";

    if request.is_get() {
        context_update!(request => {
            "title" => "Inscription",
            "register_form" => &form,
        });
        return request.render(template);
    }

    if request.is_post() {
        if form.is_valid().await {
            match form.save(&request.engine.db).await {
                Ok(_) => {
                    success!(request.notices => "Inscription réussie !");
                    return Ok(Redirect::to("/").into_response());
                }
                Err(err) => {
                    form.database_error(&err);
                }
            }
        }

        context_update!(request => {
            "title" => "Erreur",
            "register_form" => &form,
            "messages" => flash_now!(error => "Veuillez corriger les erreurs"),
        });
        return request.render(template);
    }

    request.render(template)
}
```

---

## ⚠️ Pièges courants

### 1. Collision de noms de variables template

Si votre template utilise `{% form.user %}`, la variable `user` dans le contexte **doit** être un formulaire, pas un Model SeaORM :

```rust
// ❌ ERREUR — db_user est un Model, pas un formulaire
context_update!(request => { "user" => &db_user });

// ✅ CORRECT — séparer les noms
context_update!(request => {
    "user_form" => &form,
    "found_user" => &db_user,
});
```

### 2. Oublier le `mut` sur form

```rust
// ❌ Ne peut pas appeler is_valid()
Prisme(form): Prisme<MyForm>

// ✅ Correct
Prisme(mut form): Prisme<MyForm>
```

### 3. Comparer des mots de passe après `is_valid()`

```rust
// ❌ Après is_valid(), les mots de passe sont hachés !
let mdp = form.get_form().get_string("password");
// mdp == "$argon2id$v=19$m=..." 😱

// ✅ Comparer dans clean(), AVANT la finalisation
async fn clean(&mut self) -> Result<(), StrMap> {
    let mdp1 = self.form.get_string("password");
    let mdp2 = self.form.get_string("password_confirm");
    if mdp1 != mdp2 { /* erreur */ }
    Ok(())
}
```

---

## Prochaines étapes

← [**Routage**](04-routing.md) | [**Templates**](06-templates.md) →
