# Types de champs

[← Helpers de conversion](/docs/fr/formulaire/helpers)

---

## TextField — Champs texte

Le `TextField` gère 6 formats spéciaux via l'enum `SpecialFormat` :

```rust
// Texte simple
form.field(&TextField::text("username").label("Nom").required());

// Email — validé via `validator::ValidateEmail`
form.field(&TextField::email("email").label("Email").required());

// URL — validée via `validator::ValidateUrl`
form.field(&TextField::url("website").label("Site web"));

// Formulaire d'inscription / modification — hachage automatique dans finalize()
form.field(
    &TextField::password("password")
        .label("Mot de passe")
        .required()
        .min_length(8, "Min 8 caractères"),
);

// Formulaire de connexion (login) — .no_hash() obligatoire
// Sans .no_hash(), finalize() hache le mot de passe soumis avant la comparaison
// → verify(hash, stored_hash) échoue toujours silencieusement
form.field(
    &TextField::password("password")
        .label("Mot de passe")
        .no_hash()
        .required(),
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
| --- | --- | --- |
| `Email` | `validator::ValidateEmail` | Conversion en lowercase |
| `Url` | `validator::ValidateUrl` | — |
| `Password` | Standard | Hachage auto dans `finalize()` si config `Auto` et pas `.no_hash()`, valeur vidée au `render()` |
| `RichText` | Standard | Sanitisation XSS (`sanitize()`) avant validation |
| `Csrf` | Token session | — |

**Utilitaires mot de passe :**

Le hachage et la vérification sont délégués à `PasswordConfig`, initialisé au démarrage via `password_init()` :

```rust
use runique::prelude::{hash, verify};

// Hacher manuellement (ex: création de compte hors formulaire)
let hashed = hash("mon_mot_de_passe")?;

// Vérifier un mot de passe clair contre un hash stocké en DB (ex: connexion)
let ok = verify("mdp_clair", &user.password_hash);
if !ok {
    // mot de passe incorrect
}
```

> Le hachage automatique dans `finalize()` détecte si la valeur commence déjà par `$argon2` pour éviter un double hachage. Dans un formulaire de **connexion**, n'utilisez pas `is_valid()` pour vérifier le mot de passe — récupérez d'abord l'utilisateur en DB, puis appelez `verify()` manuellement.
>
> Voir → [Configuration des mots de passe](/docs/fr/configuration/password) pour tous les modes (`Auto`, `Manual`, `Delegated`, `Custom`) et la configuration dans `main.rs`.

---

## NumericField — Champs numériques

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

**Options :** `.min(val, msg)`, `.max(val, msg)`, `.step(val)` (Type `range` uniquement), `.digits(min, max)`, `.label(l)`, `.placeholder(p)`

---

## BooleanField — Cases à cocher / Radio simple

```rust
// Checkbox simple
form.field(
    &BooleanField::new("accept_terms")
        .label("J'accepte les conditions")
        .required(),
);
```

> **Note sur le `.required()`** : Sur un `BooleanField`, `.required()` signifie que le champ est obligatoire au sens base de données (`NOT NULL`). Cela ne force pas l'utilisateur à cocher la case pour valider le formulaire (une checkbox non cochée envoie `false`, ce qui est une valeur valide pour un booléen non null). Pour forcer une acceptation (ex: CGU), utilisez une validation personnalisée ou vérifiez la valeur manuellement.

// Radio simple (oui/non)
form.field(&BooleanField::radio("newsletter").label("Newsletter"));

// Pré-coché
form.field(&BooleanField::new("remember_me").label("Se souvenir").checked());
```

---

## ChoiceField — Select / Dropdown

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

---

## RadioField — Boutons radio

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

---

## CheckboxField — Checkboxes multiples

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

---

## DateField, TimeField, DateTimeField — Date / Heure

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

---

## DurationField — Durée

```rust
form.field(
    &DurationField::new("timeout")
        .label("Délai (secondes)")
        .min_seconds(60, "Minimum 1 minute")
        .max_seconds(3600, "Maximum 1 heure"),
);
```

---

## FileField — Upload de fichiers

```rust
// Image avec contraintes complètes — dossier explicite
form.field(
    &FileField::image("avatar")
        .label("Photo de profil")
        .upload_to("uploads/avatars")   // → uploads/avatars/
        .max_size(5)
        .max_files(1)
        .max_dimensions(1920, 1080)
        .allowed_extensions(vec!["png", "jpg", "jpeg", "webp", "avif"]),
);

// Image — dossier automatique depuis MEDIA_ROOT (.env)
// Les fichiers vont dans {MEDIA_ROOT}/{nom_du_champ}/  ex: media/photo/
form.field(
    &FileField::image("photo")
        .label("Photo")
        .upload_to_env()
        .max_size(5),
);

// Sans upload_to — fichiers stockés directement dans MEDIA_ROOT
form.field(
    &FileField::image("image")
        .label("Image")
        .max_size(5),
);

// Document
form.field(
    &FileField::document("cv")
        .label("CV")
        .upload_to("uploads/cv")
        .max_size(10),
);

// Fichier quelconque (multi-fichiers)
form.field(
    &FileField::any("attachments")
        .label("Pièces jointes")
        .max_files(5),
);
```

**Destination des fichiers :**

| Méthode | Destination |
| --- | --- |
| `.upload_to("uploads/images")` | `uploads/images/` (chemin exact) |
| `.upload_to_env()` | `{MEDIA_ROOT}/{nom_du_champ}/` (depuis `.env`) |
| *(aucune)* | `MEDIA_ROOT` directement (pas de sous-dossier) |

Le déplacement vers la destination finale s'effectue dans `finalize()`, **uniquement si la validation passe**. Le dossier est créé automatiquement s'il n'existe pas encore.

> **Sécurité** : les fichiers `.svg` sont **toujours refusés** par défaut (risque XSS). La validation d'image utilise le crate `image` pour vérifier le format réel du fichier. Les fichiers vides (aucun fichier sélectionné) sont ignorés proprement — le champ `required` fonctionne correctement.

### Fichiers JS associés

```rust
fn register_fields(form: &mut Forms) {
    // ... champs ...
    form.add_js(&["js/mon_script.js", "js/autre.js"]);
}
```

Les fichiers JS sont inclus automatiquement dans le rendu HTML du formulaire.

---

## ColorField — Sélecteur de couleur

```rust
form.field(
    &ColorField::new("theme_color")
        .label("Couleur du thème")
        .default_color("#3498db"),  // Valide le format #RGB ou #RRGGBB
);
```

---

## SlugField — Slug URL-friendly

```rust
form.field(
    &SlugField::new("slug")
        .label("Slug")
        .placeholder("mon-article-url")
        .allow_unicode(),  // Optionnel : autorise les caractères unicode
);
```

> Validation : lettres, chiffres, tirets, underscores uniquement. Ne peut pas commencer ou finir par un tiret.

---

## UUIDField

```rust
form.field(
    &UUIDField::new("external_id")
        .label("ID externe")
        .placeholder("xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx"),
);
```

---

## JSONField — Textarea avec validation JSON

```rust
form.field(
    &JSONField::new("metadata")
        .label("Métadonnées")
        .placeholder(r#"{"clé": "valeur"}"#)
        .rows(10),  // Nombre de lignes du textarea
);
```

---

## IPAddressField — Adresse IP

```rust
// IPv4 + IPv6
form.field(&IPAddressField::new("server_ip").label("IP du serveur"));

// IPv4 uniquement
form.field(&IPAddressField::new("gateway").label("Passerelle").ipv4_only());

// IPv6 uniquement
form.field(&IPAddressField::new("ipv6").label("Adresse IPv6").ipv6_only());
```

---

## HiddenField — Champ caché

Champ invisible dans le formulaire HTML (`<input type="hidden">`). Deux usages principaux : passer des données techniques sans les montrer à l'utilisateur, ou valider un token CSRF manuellement.

```rust
// Champ caché générique (ex: ID d'une entité liée)
form.field(
    &HiddenField::new("entity_id")
        .label("ID entité"),
);

// Champ CSRF interne (géré automatiquement par Runique — pour usage avancé)
form.field(&HiddenField::new_csrf());
```

> Dans les formulaires Runique standards, le CSRF est géré automatiquement via `{% csrf %}` dans le template. Vous n'avez pas besoin de `HiddenField::new_csrf()` sauf si vous construisez un formulaire entièrement personnalisé.

---

## Récapitulatif des types de champs

| Struct           | Constructeurs                                                              | Validation spéciale                                           |
| ---------------- | -------------------------------------------------------------------------- | ------------------------------------------------------------- |
| `TextField`      | `text()`, `email()`, `url()`, `password()`, `textarea()`, `richtext()`     | Email/URL via `validator`, Argon2, sanitisation XSS, `.rows(n)` |
| `NumericField`   | `integer()`, `float()`, `decimal()`, `percent()`, `range()`                | Bornes min/max, précision décimale, `.step(n)` (range uniquement) |
| `BooleanField`   | `new()`, `radio()`                                                         | Requis = NOT NULL en base de données                          |
| `ChoiceField`    | `new()` + `.multiple()`                                                    | Valeur dans les choix déclarés                                |
| `RadioField`     | `new()`                                                                    | Valeur dans les choix déclarés                                |
| `CheckboxField`  | `new()`                                                                    | Toutes les valeurs dans les choix                             |
| `DateField`      | `new()`                                                                    | Format `YYYY-MM-DD`, bornes min/max                           |
| `TimeField`      | `new()`                                                                    | Format `HH:MM`, bornes min/max                                |
| `DateTimeField`  | `new()`                                                                    | Format `YYYY-MM-DDTHH:MM`, bornes min/max                     |
| `DurationField`  | `new()`                                                                    | Secondes, bornes min/max                                      |
| `FileField`      | `image()`, `document()`, `any()`                                           | Extensions, taille, dimensions, anti-SVG                      |
| `ColorField`     | `new()`                                                                    | Format `#RRGGBB` ou `#RGB`                                    |
| `SlugField`      | `new()`                                                                    | ASCII/unicode, pas de tiret en début/fin                      |
| `UUIDField`      | `new()`                                                                    | Format UUID valide                                            |
| `JSONField`      | `new()`                                                                    | JSON valide via `serde_json`, `.rows(n)`                      |
| `IPAddressField` | `new()` + `.ipv4_only()` / `.ipv6_only()`                                  | IPv4/IPv6 via `std::net::IpAddr`                              |
| `HiddenField`    | `new()`, `new_csrf()`                                                      | Token CSRF si `name == "csrf_token"`                          |

---

← [**Helpers de conversion**](/docs/fr/formulaire/helpers) | [**Erreurs de base de données**](/docs/fr/formulaire/erreurs) →
