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

// Phone — chiffres/espaces/tirets/parenthèses, '+' en tête uniquement, 7 à 15 chiffres
form.field(&TextField::phone("telephone").label("Téléphone"));
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
| `Phone` | Caractères autorisés (chiffres/espaces/`-`/`()`), `+` en première position uniquement, 7 à 15 chiffres | — |

**Utilitaires mot de passe :**

Le hachage et la vérification sont délégués à `PasswordConfig`, initialisé au démarrage via `password_init()` :

```rust
use runique::prelude::{hash, verify};

// Hacher manuellement (ex: création de compte hors formulaire)
let hashed = hash("mon_mot_de_passe")?;

// Vérifier un mot de passe clair contre un hash stocké en DB (ex: connexion)
let ok = verify("mdp_clair", &user.password);
if !ok {
    // mot de passe incorrect
}
```

> `finalize()` appelle `is_already_hashed()` avant de hacher : si la valeur est déjà un hash reconnu (Argon2 : `$argon2id$`/`$argon2i$`/`$argon2d$`, bcrypt : `$2`, scrypt : `$scrypt$`), le hachage est ignoré. Cela couvre le cas de l'édition admin où un hash existant est rechargé dans le formulaire.
>
> Dans un formulaire de **connexion**, la valeur soumise est toujours du texte brut — `is_already_hashed()` retourne `false` et `finalize()` hacherait le mot de passe avant la comparaison, ce qui ferait échouer `verify()`. C'est pourquoi `.no_hash()` est obligatoire sur les champs password d'un formulaire de connexion. Appelez `is_valid()` normalement (validation des champs requis), puis vérifiez le mot de passe séparément via `verify()` après avoir récupéré l'utilisateur en base.
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

> **Note sur le `.required()`** : sur un `BooleanField`, `.required()` signifie **« doit être cochée »** (CGU, consentement) : une case non cochée est une erreur. Une case répond toujours (`true` ou `false`), donc une colonne NOT NULL n'a besoin d'aucun `required` : dans le DSL, `actif: bool` suffit, et `cgu: bool [required]` force à cocher. `required` avec `default: true` est refusé à la compilation : une case pré-cochée ne serait pas un consentement. La case n'accepte que la valeur qu'elle envoie (`"true"`, ou celle de `.checked_value(..)`) : toute autre valeur est une erreur.

```rust
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

## FileField et MediaField — Upload de fichiers

Deux champs partagent le même mécanisme d'upload (dossier temporaire, déplacement entre l'écriture et le commit, taille, nombre, destination) :

- **`MediaField`** — images, audio et vidéo : `MediaField::image()`, `::audio()`, `::video()`. Le **contenu** de chaque fichier est vérifié (sa signature), pas seulement son extension : un fichier texte renommé en `.mp4` est refusé.
- **`FileField`** — documents et autres fichiers : `FileField::document()`, `::any()`. Seuls l'extension, la taille et le nombre sont vérifiés.

| Constructeur | Extensions acceptées par défaut | Contenu vérifié |
| --- | --- | --- |
| `MediaField::image` | jpg, jpeg, png, gif, webp, avif | signature + dimensions (`max_dimensions`) |
| `MediaField::audio` | mp3, ogg, oga, wav, m4a, flac, aac | signature (MP3, Ogg, WAV, FLAC, MP4/M4A, AAC) |
| `MediaField::video` | mp4, webm, mov, ogv | signature (MP4/MOV, WebM, Ogg) |
| `FileField::document` | pdf, doc, docx, txt, odt | non |
| `FileField::any` | toutes sauf `.svg` | non |

La taille maximale par défaut est de 10 Mo pour tous ; pour l'audio et la vidéo, réglez `.max_size()`. La limite globale du corps de la requête (`RUNIQUE_MAX_UPLOAD_MB`) s'applique aussi.

```rust
// Image avec contraintes complètes — dossier explicite
form.field(
    &MediaField::image("avatar")
        .label("Photo de profil")
        .upload_to("uploads/avatars")   // → {MEDIA_ROOT}/uploads/avatars/
        .max_size(FileSize::mb(5))
        .max_files(1)
        .max_dimensions(1920, 1080)
        .allowed_extensions(vec!["png", "jpg", "jpeg", "webp", "avif"]),
);

// Image — dossier automatique depuis MEDIA_ROOT (.env)
// Les fichiers vont dans {MEDIA_ROOT}/{nom_du_champ}/  ex: media/photo/
form.field(
    &MediaField::image("photo")
        .label("Photo")
        .upload_to_env()
        .max_size(FileSize::mb(5)),
);

// Sans upload_to — fichiers stockés directement dans MEDIA_ROOT
form.field(
    &MediaField::image("image")
        .label("Image")
        .max_size(FileSize::mb(5)),
);

// Audio et vidéo : le contenu est vérifié comme pour une image
form.field(
    &MediaField::audio("podcast")
        .label("Épisode")
        .upload_to("podcasts")
        .max_size(FileSize::mb(50)),
);
form.field(
    &MediaField::video("bande_annonce")
        .label("Bande-annonce")
        .upload_to("videos")
        .max_size(FileSize::mb(200)),
);

// Document
form.field(
    &FileField::document("cv")
        .label("CV")
        .upload_to("uploads/cv")
        .max_size(FileSize::mb(10)),
);

// Toute extension, sauf .svg. Le contenu n'est pas vérifié :
// ce que le fichier est, et comment il est servi, est à la charge de l'application.
form.field(
    &FileField::any("attachments")
        .label("Pièces jointes")
        .max_files(5),
);
```

> **Note :** `.max_size()` est une validation côté formulaire uniquement. Si le champ est défini dans un `model!{}` avec `max_size(5 MB)`, la limite du modèle et celle du formulaire sont deux valeurs indépendantes — elles ne sont pas automatiquement synchronisées. Pour un formulaire manuel, il faut les aligner à la main.

**Destination des fichiers :**

| Méthode | Destination |
| --- | --- |
| `.upload_to("uploads/images")` | `{MEDIA_ROOT}/uploads/images/` (relatif à `MEDIA_ROOT`) |
| `.upload_to_env()` | `{MEDIA_ROOT}/{nom_du_champ}/` |
| *(aucune)* | `MEDIA_ROOT` directement (pas de sous-dossier) |

`finalize()` prépare le chemin relatif, mais garde le fichier dans son dossier temporaire, sous `MEDIA_ROOT`, nommé `{uuid}.extension`. Avec `ValidationForm::save`, Runique écrit la ligne, déplace le fichier vers sa destination, puis valide la transaction. Tant que la ligne n'est pas validée, personne d'autre ne la voit : aucun lecteur ne tombe sur un chemin dont le fichier n'est pas encore en place. Une écriture refusée (valeur déjà prise, clé étrangère…) laisse le fichier dans le dossier temporaire, supprimé à la fin de la requête ; seul un échec du commit lui-même retire le fichier déjà déplacé. La destination doit être relative à `MEDIA_ROOT` et ne peut contenir de composant `.` ou `..`, de séparateur Windows ni de préfixe de volume. Le dossier est créé automatiquement s'il n'existe pas encore.

**Le fichier remplacé.** Runique ne supprime jamais le fichier qu'un nouvel envoi remplace : il peut encore vous servir (historique, archive). Pour le supprimer, lisez l'ancien chemin avant le `save`, puis supprimez-le une fois l'écriture faite :

```rust
let ancien = profil::Entity::find_by_id(id).one(&request.engine.db).await?.and_then(|p| p.avatar);
match form.save(&request.engine.db).await {
    Ok(form) => {
        let nouveau = form.forms().fields["avatar"].value().to_string();
        if let Some(ancien) = ancien.filter(|a| *a != nouveau) {
            let _ = tokio::fs::remove_file(std::path::Path::new(&runique::config::static_files::resolve_media_root()).join(ancien)).await;
        }
        Ok(Redirect::to("/profil").into_response())
    }
    Err(form) => { /* le formulaire avec ses erreurs */ }
}
```

**Sans `save`.** Un formulaire sans modèle (simple dépôt de fichier) ou une sauvegarde écrite à la main gère le fichier lui-même : `form.forms_mut().publish_uploads().await` dans votre transaction, après l'écriture et avant le `commit()`, puis `complete_uploads().await` une fois validé, ou `rollback_uploads().await` si le commit échoue. Sans écriture en base, `publish_uploads()` puis `complete_uploads()` suffisent.

**Dans une transaction du handler** ([`request.db_atomic()`](/docs/fr/orm/avance)), le fichier est déplacé au moment du `save`. Si le handler sort sans `commit()`, la ligne est annulée (rollback) mais le fichier reste sur le disque, orphelin.

> **Limite actuelle** : le disque ne participe pas à la transaction. Un arrêt brutal du processus entre le déplacement du fichier et la validation en base, ou un échec du commit qu'on ne peut rattraper, peut laisser un fichier orphelin sous `MEDIA_ROOT` — jamais une ligne qui pointe vers un fichier absent.

> **Sécurité** : les fichiers `.svg` sont **toujours refusés** (risque XSS). `FileField::any` accepte toute autre extension et ne lit pas le contenu : la sécurité de ce champ est à la charge de l'application. Pour un `MediaField`, la validation vérifie le format réel via la signature du fichier (pas seulement l'extension) : image, audio ou vidéo. Avec `max_dimensions`, le crate `image` mesure le fichier : si la mesure échoue, le fichier est refusé, pour tous les formats. Sans cette mesure, le plafond de largeur et de hauteur ne s'appliquerait pas. Les fichiers vides (aucun fichier sélectionné) sont ignorés proprement — le champ `required` fonctionne correctement.

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

Champ invisible dans le formulaire HTML (`<input type="hidden">`). Usage principal : passer des données techniques sans les montrer à l'utilisateur.

```rust
// Champ caché générique (ex: ID d'une entité liée)
form.field(
    &HiddenField::new("entity_id")
        .label("ID entité"),
);
```

> Le champ CSRF est **émis naturellement** par tout `RuniqueForm` (`req.form("nom")` / `form_fields.html` / `{% form.xxx %}`) — pas besoin de `HiddenField::new_csrf()` ni de `{% csrf %}` dans le template. `{% csrf %}` ne sert que pour un `<form>` HTML écrit à la main, hors `RuniqueForm`.

---

## Récapitulatif des types de champs

| Struct           | Constructeurs                                                              | Validation spéciale                                           |
| ---------------- | -------------------------------------------------------------------------- | ------------------------------------------------------------- |
| `TextField`      | `text()`, `email()`, `url()`, `password()`, `textarea()`, `richtext()`     | Email/URL via `validator`, Argon2, sanitisation XSS, `.rows(n)` |
| `NumericField`   | `integer()`, `float()`, `decimal()`, `percent()`, `range()`                | Bornes min/max, précision décimale, `.step(n)` (range uniquement) |
| `BooleanField`   | `new()`, `radio()`                                                         | `required` = doit être cochée ; seule sa valeur est acceptée |
| `ChoiceField`    | `new()` + `.multiple()`                                                    | Valeur dans les choix déclarés                                |
| `RadioField`     | `new()`                                                                    | Valeur dans les choix déclarés                                |
| `CheckboxField`  | `new()`                                                                    | Toutes les valeurs dans les choix                             |
| `DateField`      | `new()`                                                                    | Format `YYYY-MM-DD`, bornes min/max                           |
| `TimeField`      | `new()`                                                                    | Format `HH:MM`, bornes min/max                                |
| `DateTimeField`  | `new()`                                                                    | Format `YYYY-MM-DDTHH:MM`, bornes min/max                     |
| `DurationField`  | `new()`                                                                    | Secondes, bornes min/max                                      |
| `MediaField`     | `image()`, `audio()`, `video()`                                            | Extensions, taille, signature du contenu, dimensions (image), anti-SVG |
| `FileField`      | `document()`, `any()`                                                      | Extensions, taille, anti-SVG                                  |
| `ColorField`     | `new()`                                                                    | Format `#RRGGBB` ou `#RGB`                                    |
| `SlugField`      | `new()`                                                                    | ASCII/unicode, pas de tiret en début/fin                      |
| `UUIDField`      | `new()`                                                                    | Format UUID valide                                            |
| `JSONField`      | `new()`                                                                    | JSON valide via `serde_json`, `.rows(n)`                      |
| `IPAddressField` | `new()` + `.ipv4_only()` / `.ipv6_only()`                                  | IPv4/IPv6 via `std::net::IpAddr`                              |
| `HiddenField`    | `new()`, `new_csrf()`                                                      | Token CSRF si `name == "csrf_token"`                          |

---

← [**Helpers de conversion**](/docs/fr/formulaire/helpers) | [**Erreurs de base de données**](/docs/fr/formulaire/erreurs) →
