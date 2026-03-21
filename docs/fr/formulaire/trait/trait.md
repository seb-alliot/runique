# Trait RuniqueForm

[← Extracteur Prisme](/docs/fr/formulaire/prisme)

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
        ↓
clear()                 → [optionnel] vide le formulaire après traitement
```

**Référence des méthodes :**

**`register_fields(form)`** — Déclare les champs du formulaire.

**`from_form(form)`** — Construit l'instance depuis un `Forms`.

**`get_form()` / `get_form_mut()`** — Accesseurs vers le `Forms` interne.

**`clean_field(name)`** *(optionnel)* — Validation métier par champ individuel. Retourne `bool`. Appelée après `validate()` pour chaque champ.

**`clean()`** *(optionnel)* — Validation croisée entre plusieurs champs. Retourne `Result<(), StrMap>`. Appelée une fois que tous les champs sont valides.

**`is_valid()`** — Orchestre le pipeline complet. Peut être appelé sur GET comme sur POST : retourne `false` sans poser d'erreurs si aucune donnée n'a été soumise (premier affichage), valide normalement sinon.

**`is_submitted()`** — Retourne `true` si le formulaire a reçu des données (POST, ou GET avec query params non vides).

**`database_error(&err)`** — Analyse une erreur DB et la positionne sur le bon champ.

**`clear()`** — Vide toutes les valeurs des champs (hors CSRF) et remet `submitted` à `false`. À appeler après avoir lu les données nettoyées, avant un redirect ou un re-rendu vide.

**`build(tera, csrf_token)`** — Construit un formulaire vide.

**`build_with_data(data, tera, csrf)`** — Construit, remplit et valide.

---

## `is_valid()` — appel sur GET et POST

`is_valid()` est conçu pour être appelé indifféremment sur GET et POST :

- **Premier GET (formulaire vide)** — `is_valid()` retourne `false`, aucune erreur n'est posée sur les champs. Le template affiche un formulaire propre.
- **GET avec query params (formulaire de recherche)** — `is_valid()` valide normalement. Permet de faire des recherches via GET sans code supplémentaire.
- **POST** — comportement standard : valide, pose les erreurs sur les champs si invalide.

```rust
// Handler GET+POST unifié — fonctionne sans if/else sur la méthode
pub async fn search(
    mut request: Request,
    Prisme(mut form): Prisme<SearchForm>,
) -> AppResult<Response> {
    if form.is_valid().await {
        let query = form.get_string("q");
        // lancer la recherche...
    }
    // Premier GET : is_valid() == false, aucune erreur → formulaire vide
    // GET soumis invalide : is_valid() == false, erreurs affichées
    context_update!(request => { "search_form" => &form });
    request.render("search.html")
}
```

> **`is_submitted()`** est disponible si tu as besoin de distinguer explicitement "premier affichage" de "formulaire soumis sans données valides".

---

## Pipeline de validation `is_valid()`

L'appel `form.is_valid().await` déclenche **4 étapes dans l'ordre** (uniquement si le formulaire est soumis) :

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

> **⚠️ Important** : Les champs `Password` sont **hachés automatiquement** lors de `finalize()` par défaut (Argon2), sauf si `password_init` est appelé dans `main.rs` avec `PasswordConfig::Manual`, `Delegated` ou `Custom`.
> Utilisez `clean()` pour toute comparaison de mots de passe en clair — c'est la seule étape où ils sont encore lisibles.

---

## `clear()` — vider le formulaire après traitement

`clear()` vide toutes les valeurs des champs (hors token CSRF) et remet `submitted` à `false`.

Accessible partout où `self` est `&mut Self` — dans un handler ou dans une méthode du formulaire lui-même.

### Depuis un handler

```rust
if form.is_valid().await {
    let path = form.cleaned_string("image"); // 1. lire avant clear
    // sauvegarder en DB...
    form.clear();                            // 2. vider
    success!(request.notices => "Fichier uploadé !");
    context_update!(request => { "image_form" => &form });
    return request.render(template);         // 3. re-rendre avec formulaire vide
}
```

### Depuis le formulaire lui-même (`save(&mut self)`)

Passer `save` en `&mut self` permet d'encapsuler le clear directement — le handler n'a rien à faire :

```rust
impl BlogForm {
    pub async fn save(
        &mut self,
        db: &DatabaseConnection,
    ) -> Result<blog::Model, DbErr> {
        let record = blog::ActiveModel {
            title: Set(self.form.get_string("title")),
            // ...
            ..Default::default()
        };
        let result = record.insert(db).await;
        if result.is_ok() {
            self.clear(); // vide automatiquement après succès
        }
        result
    }
}
```

### Où `clear()` ne peut pas être appelé

- Dans une méthode `&self` (lecture seule) — ne compile pas
- Dans `clean()` ou `clean_field()` — s'exécutent **pendant** `is_valid()`, avant que les données soient lues par `save()` ; appeler `clear()` ici viderait le formulaire avant la sauvegarde

> **💡 Avec redirect (PRG)** : si le handler redirige après succès (`Redirect::to(...)`), `clear()` n'est pas nécessaire — la nouvelle requête GET crée automatiquement une instance fraîche et vide.

---

← [**Extracteur Prisme**](/docs/fr/formulaire/prisme) | [**Helpers de conversion**](/docs/fr/formulaire/helpers) →
