# Trait RuniqueForm

[← Extraction de formulaire](/docs/fr/formulaire/prisme)

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
>
> **Note de sécurité :** `get_form()` donne accès à la structure interne. Pour l'extraction des données nettoyées, passez toujours par les méthodes `cleaned_*` du trait (voir plus bas). Les méthodes d'extraction directe sur `Forms` sont réservées à l'usage interne du framework.

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
register_fields()            → déclare les champs
        ↓
req.form()                   → construit l'instance
        ↓
is_valid()                   → pipeline de validation
    ↓  validate() par champ  (requis, format, longueur…)
    ↓  clean_field(name)     [optionnel — règle métier unitaire]
    ↓  clean()               [optionnel — validation croisée]
    ↓  finalize()            (hash Argon2, transformations finales)
        ↓
form.save(&db) / delete(&db)  → écriture, sur le formulaire validé
        ↓
database_error()             → [optionnel] gestion d'erreur DB
        ↓
clear()                      → [optionnel] vide le formulaire après traitement
```

**Référence des méthodes :**

**`register_fields(form)`** — Déclare les champs du formulaire.

**`from_form(form)`** — Construit l'instance depuis un `Forms`.

**`get_form()` / `get_form_mut()`** — Accesseurs de bas niveau vers le `Forms` interne (utile pour le rendu ou la configuration dynamique des champs). Ne pas utiliser pour extraire les données.

**`clean_field(name)`** *(optionnel)* — Validation métier par champ individuel. Retourne `bool`. Appelée après `validate()` pour chaque champ.

**`clean()`** *(optionnel)* — Validation croisée entre plusieurs champs. Retourne `Result<(), StrMap>`. Appelée une fois que tous les champs sont valides.

**`is_valid()`** — Orchestre le pipeline complet. Peut être appelé sur GET comme sur POST : retourne `false` sans poser d'erreurs si aucune donnée n'a été soumise (premier affichage), valide normalement sinon.

**`save(&db)` / `delete(&db)`** — Sur le formulaire validé rendu par `try_form`, pour un formulaire relié à une entité : écrit la ligne, rend `Ok(form)` ou `Err(form)`. Voir « Écriture » plus bas.

**`database_error(&err)`** — Analyse une erreur DB et la positionne sur le bon champ. `ValidationForm::save` l'appelle déjà : le `Err(form)` revient avec l'erreur. À appeler soi-même seulement après une écriture maison qui renvoie un `DbErr`.

**`clear()`** — Vide toutes les valeurs des champs (hors CSRF et nom du formulaire) et remet `submitted` à `false`. À appeler après avoir lu les données nettoyées, avant un redirect ou un re-rendu vide.

---

## Accès aux données — `cleaned_*()`

Après `is_valid()`, accédez aux données via les méthodes `cleaned_*` du trait :

```rust
self.cleaned::<String>("username")      // Option<String>  (None si vide)
self.cleaned::<i32>("age")              // Option<i32>
self.cleaned::<f64>("price")            // Option<f64>  (gère , → .)
self.cleaned::<bool>("active")          // Option<bool>
// … voir la page Helpers pour la liste complète
```

Ces méthodes sont **whitelistées** : elles retournent `None` si le champ n'est pas déclaré dans `register_fields()`. Elles couvrent aussi les paramètres de route et de query string.

> **`cleaned::<String>` retourne `Option<String>`**. Pour obtenir une `String` vide par défaut : `self.cleaned::<String>("x").unwrap_or_default()`.

---

## Surcharge des champs — affichage et contraintes

Ces méthodes permettent de modifier le comportement d'un champ **après** `register_fields()`, sans toucher à sa déclaration. Elles sont chainables et disponibles sur le trait `RuniqueForm` directement.

### Affichage

```rust
form.label("titre", "Titre de l'article")       // surcharge le libellé
    .placeholder("titre", "Ex : Mon article")
    .required("titre", false)                    // relâche la contrainte required
    .readonly("titre", true)
    .disabled("brouillon", true)
    .attr("photo", "accept", "image/*");         // attribut HTML arbitraire
```

### Taille maximale d'un `FileField`

Le modèle peut déclarer un plafond immuable via `.max_size()` dans `register_fields()`. La surcharge côté formulaire ne peut pas le dépasser :

```rust
// Dans register_fields — plafond modèle fixé à 5 MB
form.field(&FileField::image("photo").max_size(FileSize::mb(5)));

// Dans le handler — surcharge autorisée si ≤ plafond
form.max_size("photo", FileSize::mb(3))?;   // Ok
form.max_size("photo", FileSize::mb(6))?;   // Err — dépasse le plafond du modèle
```

> **Séparation des responsabilités** : le modèle contrôle la contrainte maximale, le formulaire peut l'affiner vers le bas. Même principe que Django `ModelForm` pour les `max_length`.

---

## Nom du formulaire — plusieurs formulaires sur une page

`try_form(form, &request, "nom")` donne un nom au formulaire. Ce nom est rendu dans un champ caché `runique_form` (en premier, avant le CSRF), et préfixe les `id` HTML des champs (`id_nom-champ`). Au POST, le formulaire n'est validé que si la requête renvoie **ce même nom** : sur une page qui affiche plusieurs formulaires, seul celui qui a été envoyé est validé, les autres reviennent en `Err` sans aucune erreur posée.

- Tout handler qui **affiche** le formulaire passe par `try_form`, GET compris : un formulaire jamais nommé échoue au rendu (erreur Tera sur le nom manquant), plutôt que d'afficher un formulaire dont l'envoi ne serait jamais validé.
- Le nom n'est jamais rempli à partir de la requête : seul `try_form` le pose.
- Un formulaire écrit à la main (sans `{% form.x %}`) doit porter lui-même le champ caché `runique_form` avec le bon nom.

```rust
let username_form = ValidationForm::try_form(request.form::<UsernameForm>(), &request, "username").await;
let email_form = ValidationForm::try_form(request.form::<EmailForm>(), &request, "email").await;
```

---

## `is_valid()` — appel sur GET et POST

`is_valid()` valide indifféremment sur GET et POST, mais **ne regarde jamais la méthode HTTP** — c'est `ValidationForm::try_form(form, &request, "my_form")` qui décide, via `allow_get`/`allow_post`, si la validation doit même être tentée. Par défaut `allow_get` est `false` (protection CSRF : GET est la seule méthode exemptée, un formulaire mutant ne doit jamais s'auto-valider dessus). Un formulaire de recherche en lecture seule est le cas qui doit **explicitement** lever cette garde :

- **Premier GET (formulaire vide)** — `allow_get` retombe sur `is_submitted()` → `false`, `try_form` renvoie `Err` sans tenter la validation, aucune erreur posée. Le template affiche un formulaire propre.
- **GET avec query params (formulaire de recherche)** — `is_submitted()` → `true`, `try_form` valide et renvoie `Ok`. Permet de faire des recherches via GET sans code supplémentaire.
- **POST** — comportement standard : valide, pose les erreurs sur les champs si invalide (nécessite aussi un `allow_post` qui l'autorise — le défaut couvre déjà ce cas).

```rust
impl RuniqueForm for SearchForm {
    // ...
    fn allow_get(&self, _request: &Request) -> bool {
        self.is_submitted()
    }
}

// Handler GET+POST unifié — fonctionne sans if/else sur la méthode
pub async fn search(mut request: Request) -> AppResult<Response> {
    let form: SearchForm = request.form();
    let form = match ValidationForm::try_form(form, &request, "my_form").await {
        Ok(validated) => {
            let query = validated.cleaned::<String>("q").unwrap_or_default();
            // lancer la recherche...
            validated.into_form()
        }
        // Premier GET ou GET/POST invalide : aucune erreur si rien n'a été
        // soumis, erreurs de champ posées sinon.
        Err(form) => form,
    };
    context_update!(request => { "search_form" => &form });
    request.render("search.html")
}
```

> Pour distinguer explicitement "premier affichage" de "formulaire soumis sans données valides", utilise `request.method.is_safe()` (GET/HEAD).

---

## Pipeline de validation `is_valid()`

L'appel `form.is_valid().await` déclenche **4 étapes dans l'ordre** (uniquement si le formulaire est soumis) :

1. **Validation des champs** — Chaque champ exécute son `validate()` : requis, longueur, format (email via `validator`, URL via `validator`, JSON via `serde_json`, UUID via `uuid`, IP via `std::net::IpAddr`…)
2. **`clean_field(name)`** — Validation métier par champ, appelée pour chaque champ après l'étape 1 (uniquement si la validation standard a réussi)
3. **`clean()`** — Validation croisée sur l'ensemble du formulaire (ex: `mdp1 == mdp2`) ; les mots de passe sont encore en clair à cette étape
4. **`finalize()`** — Transformations finales (hachage automatique des champs `Password` si la config globale est en mode `Auto`)

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
            let val = self.cleaned::<String>("username").unwrap_or_default();
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
        let mdp1 = self.cleaned::<String>("password").unwrap_or_default();
        let mdp2 = self.cleaned::<String>("password_confirm").unwrap_or_default();

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

> **⚠️ Important** : Les champs `Password` sont hachés lors de `finalize()` uniquement si la config globale est `PasswordConfig::Auto` (mode par défaut) **et** que `.no_hash()` n'a pas été appliqué au champ. En mode `Manual`, `Delegated` ou `Custom`, `finalize()` ne hache pas — c'est à l'application de le faire.
> Utilisez `clean()` pour toute comparaison de mots de passe en clair — c'est la seule étape où ils sont encore lisibles.
>
> **Formulaire de connexion (login)** : le champ password ne doit **pas** être haché — il sera comparé au hash stocké en base. Désactivez le hachage automatique avec `.no_hash()` : `TextField::password("password").no_hash().required()`

---

## Écriture — `save` et `delete`

Un formulaire relié à une entité (`model!{}`, `extend!{}`, `#[form(schema = …)]`) s'écrit depuis le formulaire validé que rend `ValidationForm::try_form`. Runique parcourt ses champs et écrit la ligne : aucune méthode d'écriture à implémenter. Un formulaire sans entité n'a ni `save` ni `delete` : son handler fait le travail après `try_form`.

```rust
match ValidationForm::try_form(form, &request, "article").await {
    Ok(form) => match form.save(&request.engine.db).await {
        Ok(_) => {
            success!(request.notices => "Article enregistré");
            Ok(Redirect::to("/articles").into_response())
        }
        Err(form) => {
            context_update!(request => { "article_form" => &form });
            request.render(template)
        }
    },
    Err(form) => {
        context_update!(request => { "article_form" => &form });
        request.render(template)
    }
}
```

`save` et `delete` rendent le formulaire dans les deux cas : `Ok(form)` une fois écrit (rediriger, ou `clear()` puis ré-afficher), `Err(form)` avec la raison posée sur ses champs ou ses erreurs.

### La ligne vient de l'URL

La clé est cherchée dans les paramètres de la route, par le nom donné à `try_form` :

1. `{<nom>_<clé>}` — par exemple `{article_id}` pour le formulaire `"article"` ;
2. sinon `{<clé>}` — par exemple `{id}` ;
3. sinon aucune : `save` crée la ligne.

Avec une clé, `save` met à jour cette ligne ; `delete` exige une clé. Une clé qui ne se lit pas au type de la clé du modèle, ou une ligne absente, donne « Ressource introuvable » et rien n'est écrit. Deux formulaires sur une page pointent chacun leur ligne par leur propre nom.

### Ce qui est écrit

- **Hors admin** : SeaORM écrit (`insert` en création, `ActiveModel::save()` en mise à jour) ; le comportement du modèle (`ActiveModelBehavior::before_save` / `after_save`, `auto_now`) s'exécute. Les champs verrouillés (`readonly`, `disabled`) et les colonnes secrètes (mot de passe) ne sont jamais écrits. Vérifier que l'utilisateur a le droit de toucher la ligne est le rôle du handler, comme pour `search!`.
- **Dans l'admin** (panneau et routes déclarées `admin`) : la porte écrit, au nom du compte vérifié — droits, propriétaire, historique, dans une transaction.

### Où placer la logique métier

- Valeurs à vérifier ou ajuster : `clean_field`, `clean`.
- Valeurs venues de la requête (compte connecté, choix) : `register_dynamic_fields`.
- Effets extérieurs (email, cache) et écritures dans d'autres tables : dans le handler, après un `save` réussi.

---

## `clear()` — vider le formulaire après traitement

`clear()` vide toutes les valeurs des champs (hors token CSRF) et remet `submitted` à `false`.

Accessible partout où `self` est `&mut Self` — dans un handler ou dans une méthode du formulaire lui-même.

### Depuis un handler

```rust
if form.is_valid().await {
    let path = form.cleaned::<String>("image").unwrap_or_default(); // 1. lire avant clear
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
            title: Set(self.cleaned::<String>("title").unwrap_or_default()),
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

← [**Extraction de formulaire**](/docs/fr/formulaire/prisme) | [**Helpers de conversion**](/docs/fr/formulaire/helpers) →
