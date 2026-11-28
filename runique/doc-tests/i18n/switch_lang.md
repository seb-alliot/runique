# i18n — `switch_lang`

## Enum `Lang` — Debug et code

```rust
use runique::translation::Lang;

assert_eq!(format!("{:?}", Lang::Fr), "Fr");
assert_eq!(format!("{:?}", Lang::En), "En");
assert_eq!(format!("{:?}", Lang::It), "It");
assert_eq!(Lang::Fr.code(), "fr");
assert_eq!(Lang::En.code(), "en");
assert_eq!(Lang::It.code(), "it");
```

## Conversion `Lang::from(&str)`

```rust
use runique::translation::Lang;

// Français
assert_eq!(Lang::from("fr"),    Lang::Fr);
assert_eq!(Lang::from("fr-FR"), Lang::Fr);
assert_eq!(Lang::from("fr-CA"), Lang::Fr);
assert_eq!(Lang::from("fr-BE"), Lang::Fr);
assert_eq!(Lang::from("fr-CH"), Lang::Fr);

// Anglais
assert_eq!(Lang::from("en"),    Lang::En);
assert_eq!(Lang::from("en-US"), Lang::En);
assert_eq!(Lang::from("en-GB"), Lang::En);
assert_eq!(Lang::from("en-CA"), Lang::En);

// Italien
assert_eq!(Lang::from("it"),    Lang::It);
assert_eq!(Lang::from("it-IT"), Lang::It);
assert_eq!(Lang::from("it-CH"), Lang::It);

// Allemand
assert_eq!(Lang::from("de"),    Lang::De);
assert_eq!(Lang::from("de-DE"), Lang::De);
assert_eq!(Lang::from("de-AT"), Lang::De);
assert_eq!(Lang::from("de-CH"), Lang::De);

// Inconnu → anglais par défaut
assert_eq!(Lang::from("xx"),    Lang::En);
```

## `Lang::get` — clé simple

```rust
use runique::translation::Lang;

assert_eq!(Lang::En.get("forms.error.required"), "This field is required");
assert_eq!(Lang::Fr.get("forms.error.required"), "Ce champ est obligatoire");
assert_eq!(Lang::It.get("forms.error.required"), "Questo campo è obbligatorio");
```

## `Lang::get` — clé imbriquée

```rust
use runique::translation::Lang;

assert_eq!(Lang::En.get("app.label.not_found"), "Page Not Found");
assert_eq!(Lang::Fr.get("app.label.not_found"), "Page non trouvée");
assert_eq!(Lang::It.get("app.label.not_found"), "Pagina non trovata");
```

## `Lang::get` — CSRF

```rust
use runique::translation::Lang;

assert_eq!(Lang::En.get("csrf.error.missing"), "CSRF token missing");
assert_eq!(Lang::Fr.get("csrf.error.missing"), "Token CSRF manquant");
assert_eq!(Lang::It.get("csrf.error.missing"), "Token CSRF mancante");
```

## `Lang::get` — clé manquante (fallback sur la clé brute)

```rust
use runique::translation::Lang;

assert_eq!(Lang::En.get("inexistant.cle"), "inexistant.cle");
assert_eq!(Lang::Fr.get("inexistant.cle"), "inexistant.cle");
assert_eq!(Lang::It.get("inexistant.cle"), "inexistant.cle");
```

## `Lang::format` — avec paramètres

```rust
use runique::translation::Lang;

assert_eq!(Lang::En.format("forms.error.too_short", &[3]), "Too short (min 3)");
assert_eq!(Lang::Fr.format("forms.error.too_short", &[3]), "Trop court (min 3)");
assert_eq!(Lang::It.format("forms.error.too_short", &[3]), "Troppo corto (min 3)");

assert_eq!(Lang::En.format("forms.error.too_long", &[100]), "Too long (max 100)");
assert_eq!(Lang::Fr.format("forms.error.too_long", &[100]), "Trop long (max 100)");
assert_eq!(Lang::It.format("forms.error.too_long", &[100]), "Troppo lungo (max 100)");
```

## `current_lang` / `t`

The language of the site comes from `.with_lang("fr")` on the builder, else from `LANG` in the `.env`, else it is English.

```rust
use runique::translation::{Lang, current_lang, t};

assert_eq!(current_lang(), Lang::En);
assert_eq!(t("forms.error.required"), "This field is required");
```
