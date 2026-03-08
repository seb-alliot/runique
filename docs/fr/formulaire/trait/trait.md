# Trait RuniqueForm

[← Extracteur Prisme](https://github.com/seb-alliot/runique/blob/main/docs/fr/formulaire/prisme/prisme.md)

---

## Structure de base

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

> **💡 `impl_form_access!()`** génère automatiquement `from_form()`, `get_form()` et `get_form_mut()`. Si votre champ ne s'appelle pas `form`, passez le nom en argument : `impl_form_access!(formulaire)`.

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

---

## Méthodes du trait RuniqueForm

**Cycle de vie du formulaire (ordre d'appel) :**

```text
register_fields()       → déclare les champs
        ↓
build() / build_with_data()  → construit l'instance
        ↓
is_valid()              → pipeline de validation
    ↓  validate() par champ (requis, format, longueur…)
    ↓  clean_field(name) par champ  [optionnel — règle métier unitaire]
    ↓  clean()                      [optionnel — validation croisée]
    ↓  finalize()                   (hash Argon2, transformations finales)
        ↓
save() / database_error()    → persistance ou gestion d'erreur DB
```

**Référence des méthodes :**

**`register_fields(form)`** — Déclare les champs du formulaire.

**`from_form(form)`** — Construit l'instance depuis un `Forms`.

**`get_form()` / `get_form_mut()`** — Accesseurs vers le `Forms` interne.

**`clean_field(name)`** *(optionnel)* — Validation métier par champ individuel. Retourne `bool`. Appelée après `validate()` pour chaque champ.

**`clean()`** *(optionnel)* — Validation croisée entre plusieurs champs. Retourne `Result<(), StrMap>`. Appelée une fois que tous les champs sont valides.

**`is_valid()`** — Orchestre le pipeline complet. Ignoré si le formulaire n'a pas reçu de données (évite les erreurs sur GET).

**`database_error(&err)`** — Analyse une erreur DB et la positionne sur le bon champ.

**`build(tera, csrf_token)`** — Construit un formulaire vide.

**`build_with_data(data, tera, csrf)`** — Construit, remplit et valide.

---

## Pipeline de validation `is_valid()`

L'appel `form.is_valid().await` déclenche **4 étapes dans l'ordre** :

1. **Validation des champs** — Chaque champ exécute son `validate()` : requis, longueur, format (email via `validator`, URL via `validator`, JSON via `serde_json`, UUID via `uuid`, IP via `std::net::IpAddr`…)
2. **`clean_field(name)`** — Validation métier par champ, appelée pour chaque champ après l'étape 1 (uniquement si la validation standard a réussi)
3. **`clean()`** — Validation croisée sur l'ensemble du formulaire (ex: `mdp1 == mdp2`) ; les mots de passe sont encore en clair à cette étape
4. **`finalize()`** — Transformations finales (hachage Argon2 automatique des champs `Password`)

---

## `clean_field` — validation métier par champ

`clean_field` est appelée pour chaque champ après sa validation standard. Elle permet d'implémenter une règle métier sur un champ précis (unicité, format personnalisé, valeur interdite…).

- Retourne `true` si le champ est valide, `false` sinon
- En cas d'échec, poser l'erreur manuellement sur le champ via `set_error()`
- **N'est pas invoquée** si le champ requis est déjà vide (la validation standard échoue d'abord)

```rust
#[async_trait::async_trait]
impl RuniqueForm for UsernameForm {
    // ...

    async fn clean_field(&mut self, name: &str) -> bool {
        if name == "username" {
            let val = self.get_form().get_string("username");
            if val.to_lowercase().contains("admin") {
                if let Some(f) = self.get_form_mut().fields.get_mut("username") {
                    f.set_error("Le nom 'admin' est réservé".to_string());
                }
                return false;
            }
        }
        true
    }
}
```

> **💡** `clean_field` est idéale pour les règles isolées sur un champ : valeur interdite, format personnalisé, vérification d'unicité légère. Pour les règles qui impliquent plusieurs champs à la fois, utilisez `clean()`.
>
> **⚠️ Ne pas appeler `clean_field` depuis `clean`** : le pipeline garantit que `clean_field` s'est déjà exécuté pour chaque champ avant l'appel de `clean`. Rappeler `clean_field` depuis `clean` serait redondant et risquerait de poser une erreur en double sur un champ. De plus, `clean` n'est invoquée que si tous les `clean_field` ont retourné `true` — depuis `clean`, tous les champs sont déjà individuellement valides.

---

## `clean` — validation croisée

`clean` est appelée une fois que **tous** les champs ont passé leur validation (standard + `clean_field`). Elle permet de croiser les valeurs de plusieurs champs.

- Retourne `Ok(())` si le formulaire est valide
- Retourne `Err(StrMap)` avec une map `{ "nom_du_champ" => "message d'erreur" }` en cas d'échec

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

> **⚠️ Important** : Après `is_valid()`, les champs `Password` sont **automatiquement hachés**.
> Utilisez `clean()` pour toute comparaison de mots de passe en clair — c'est la seule étape où ils sont encore lisibles.

---

← [**Extracteur Prisme**](https://github.com/seb-alliot/runique/blob/main/docs/fr/formulaire/prisme/prisme.md) | [**Helpers de conversion**](https://github.com/seb-alliot/runique/blob/main/docs/fr/formulaire/helpers/helpers.md) →
