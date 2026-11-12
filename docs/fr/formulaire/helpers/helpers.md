# Helpers de conversion typée

[← Trait RuniqueForm](/docs/fr/formulaire/trait)

---

Les valeurs de formulaire sont stockées en `String`. Utilisez les helpers typés `cleaned_*` pour les convertir de façon sécurisée (whitelistée) et typée.

## API recommandée — `cleaned_*()`

Méthodes du trait `RuniqueForm`. Elles sont whitelistées (champ non déclaré → `None`) et lisent, dans l'ordre : la valeur du champ, puis, si elle est vide, un paramètre de route, puis de query string, de même nom. Ce repli sur l'URL n'est pas passé par la validation du champ.

### Basiques et Numériques

```rust
self.cleaned::<String>("username")      // Option<String> (None si vide)
self.cleaned::<i32>("age")              // Option<i32>
self.cleaned::<i64>("count")            // Option<i64>
self.cleaned::<u32>("quantity")         // Option<u32>
self.cleaned::<u64>("id")               // Option<u64>
self.cleaned::<f32>("ratio")            // Option<f32> (gère , → .)
self.cleaned::<f64>("price")            // Option<f64> (gère , → .)
self.cleaned::<bool>("active")          // Option<bool> (true/1/on → true)
```

### Date / Heure / UUID

`NaiveDate`, `NaiveDateTime`, `DateTime`, `Utc` et `Uuid` viennent du prelude ; `NaiveTime` s'importe à part (`use runique::chrono::NaiveTime;`).

```rust
self.cleaned::<NaiveDate>("birthday")           // Option<NaiveDate> (format YYYY-MM-DD)
self.cleaned::<NaiveTime>("meeting_time")       // Option<NaiveTime> (format HH:MM)
self.cleaned::<NaiveDateTime>("event_start")    // Option<NaiveDateTime> (YYYY-MM-DDTHH:MM)
self.cleaned::<DateTime<Utc>>("created_at")       // Option<DateTime<Utc>> (RFC3339)

self.cleaned::<Uuid>("external_id")              // Option<Uuid>
```

### Enums

```rust
self.cleaned_enum::<Statut>("statut")         // Option<Statut> — enum texte ou entier
self.cleaned_enums::<Genre>("genres")         // Vec<Genre> — champ liste (checkbox / multichoice)
```

`cleaned_enum` accepte tout enum généré par `model!{}`, texte ou entier (`i8` à `i64`).
`cleaned_enums` lit les valeurs cochées d'un champ liste, dans l'ordre soumis ; il renvoie un
`Vec` vide si rien n'est coché. Une valeur qui n'est pas une variante est ignorée (et journalisée).

> **💡** Les lectures flottantes (`cleaned::<f32>`, `cleaned::<f64>`) convertissent automatiquement la virgule en point (`19,99` → `19.99`) pour simplifier la saisie.
> 
> **Note de sécurité :** Toutes les variantes `cleaned_*` retournent `Option`. Pour une valeur par défaut : `.unwrap_or_default()` ou `.unwrap_or(0)`.

---

## Utilisation dans `save()`

Dans vos méthodes `save()`, utilisez exclusivement `cleaned_*` pour extraire les données :

```rust
impl RegisterForm {
    pub async fn save(&self, db: &DatabaseConnection) -> Result<users::Model, DbErr> {
        let model = users::ActiveModel {
            username: Set(self.cleaned::<String>("username").unwrap_or_default()),
            email: Set(self.cleaned::<String>("email").unwrap_or_default()),
            age: Set(self.cleaned::<i32>("age").unwrap_or(0)),
            website: Set(self.cleaned::<String>("website")), // Option<String>
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
let id = request.get_path("id");         // Option<&str>

// Paramètre de query string : ?page=2
let page = request.get_query("page");    // Option<&str>
```

### `cleaned` ne lit jamais l'URL

`cleaned::<T>` rend la valeur d'un **champ** du formulaire, et rien d'autre : ce que le visiteur a envoyé dans ce formulaire, passé par les règles du champ. Un paramètre de chemin ou de query string du même nom n'est pas lu. Une valeur de l'URL se lit par `request.get_path` / `request.get_query`, et vous savez alors qu'elle est brute.

Un formulaire de recherche envoyé en `GET` n'est pas concerné : ses champs sont remplis depuis la query string comme ceux d'un `POST` le sont depuis le corps.

Exemple — pré-remplir un champ depuis un lien (`GET /edit?title=Mon+Article`) :

```rust
if request.method.is_safe()
    && let Some(title) = request.get_query("title").map(str::to_owned)
{
    form.add_value("title", &title); // `form` déclaré `let mut`
}
```

La valeur est seulement affichée dans le champ : elle sera validée quand le visiteur enverra le formulaire.

---

## Colonnes posées par le code — `add_value` hors champs

Un nom passé à `add_value` qui n'est pas un champ du formulaire est gardé pour `save()` : s'il nomme une colonne de l'entité, `save()` l'écrit, converti au type de la colonne ; sinon il est ignoré. Le visiteur ne peut jamais l'envoyer : ce n'est pas un champ, `fill()` ne le lit pas.

C'est la façon d'imposer une valeur que le visiteur ne doit pas choisir : l'auteur d'un commentaire, l'article auquel il appartient, le compte propriétaire d'une ligne.

### Exemple : un commentaire sur un article

L'entité `commentaire` a trois colonnes : `texte`, `auteur_id`, `article_id`. Le formulaire ne déclare que `texte`.

```rust
#[form(schema = commentaire, fields = [texte])]
pub struct CommentaireForm;
impl RuniqueForm for CommentaireForm {}

// Route : "/article/{article_id}/commenter"
pub async fn commenter(request: Request) -> AppResult<Response> {
    let Some(user) = request.user.clone() else {
        return Ok(Redirect::to("/login").into_response());
    };
    let article_id = request.get_path_as::<Pk>("article_id").unwrap_or_default();

    // `afficher(form, &request)` : ta fonction qui rend le template avec le formulaire.

    // 1. Le formulaire est créé et rempli avec ce que le visiteur a envoyé.
    let mut form: CommentaireForm = request.form("commentaire");

    // 2. Le code pose les colonnes que le visiteur ne choisit pas.
    form.add_value("auteur_id", &user.id.to_string());
    form.add_value("article_id", &article_id.to_string());

    // 3. Validation.
    match ValidationForm::try_form(form, &request).await {
        // 4. Écriture : texte (visiteur) + auteur_id et article_id (code).
        Ok(form) => match form.save(&request.engine.db).await {
            Ok(_) => Ok(Redirect::to("/merci").into_response()),
            Err(form) => afficher(form, &request),
        },
        Err(form) => afficher(form, &request),
    }
}
```

Si le visiteur ajoute `auteur_id=1` ou `article_id=99` à sa requête, ces valeurs sont ignorées : aucun champ ne les lit, et `save()` n'écrit que ce que le code a posé.

Le code doit d'abord vérifier que l'article existe et que le compte a le droit de le commenter : `add_value` impose une valeur, il ne contrôle pas qu'elle est légitime.

### L'ordre : `request.form` → `add_value` → `try_form` → `save`

| Étape | Pourquoi à cette place |
|-------|------------------------|
| `request.form("nom")` | Crée le formulaire et le remplit avec la requête. Avant, il n'existe pas : rien sur quoi appeler `add_value`. |
| `add_value(...)` | Le formulaire existe et est encore modifiable. Les valeurs posées ici passent la validation avec le reste. |
| `ValidationForm::try_form(form, &request)` | Valide, puis rend un `ValidationForm` **en lecture seule** : ce qui est écrit est exactement ce qui a été validé. |
| `form.save(db)` | Écrit les champs, puis les colonnes posées par le code. |

Pourquoi pas un autre ordre :

- **`add_value` après `try_form`** : ne compile pas. `ValidationForm` ne donne qu'un accès en lecture au formulaire ; c'est voulu, sinon une valeur pourrait changer entre la validation et l'écriture.

  ```rust
  if let Ok(form) = ValidationForm::try_form(form, &request).await {
      form.add_value("auteur_id", &id); // erreur de compilation : `form` n'est pas modifiable
  }
  ```

- **`add_value` avant `request.form`** : impossible, le formulaire n'existe pas encore. `request.form("nom")` est le seul moyen de le créer (hors `build_form_for_test`, réservé aux tests) : il lui donne son nom, son jeton CSRF et les données envoyées.

### Valeurs en texte

`add_value` prend un `&str`, comme ce que le navigateur envoie : la conversion vers le type de la colonne a lieu à un seul endroit, dans `save()`, avec les mêmes règles que pour un champ.

```rust
form.add_value("auteur_id", &user.id.to_string()); // entier ou UUID
form.add_value("publie", "true");                  // booléen : true/false, 1/0, on/off
form.add_value("date", "2026-11-07");              // date : AAAA-MM-JJ
```

### Mauvaises pratiques

**Prendre la valeur dans la requête.** Le visiteur choisit de nouveau la valeur : c'est exactement ce que `add_value` devait empêcher.

```rust
// ❌ le visiteur envoie ?auteur_id=1 et commente au nom d'un autre
if let Some(id) = request.get_query("auteur_id") {
    form.add_value("auteur_id", id);
}

// ✅ l'auteur vient du compte connecté, relu en base à chaque requête
form.add_value("auteur_id", &user.id.to_string());
```

**Poser un identifiant d'URL sans le vérifier.** `article_id` vient du chemin : le visiteur peut écrire n'importe quel nombre. `add_value` impose la valeur, il ne vérifie pas qu'elle est permise.

```rust
// ❌ commente un article privé, ou qui n'existe pas
form.add_value("article_id", &article_id.to_string());

// ✅ vérifier d'abord l'article et le droit d'y accéder
let Some(article) = search!(article::Entity => Id eq article_id, Publie eq true)
    .first(request.db()).await.ok().flatten()
else {
    return Ok(StatusCode::NOT_FOUND.into_response());
};
form.add_value("article_id", &article.id.to_string());
```

**Modifier une ligne sans vérifier à qui elle est.** Sur `/commentaire/{commentaire_id}`, `save()` du formulaire `"commentaire"` modifie cette ligne. Poser `auteur_id` ne protège pas cette ligne : il la donne au compte connecté.

```rust
// ❌ n'importe quel compte prend le commentaire 42 d'un autre
form.add_value("auteur_id", &user.id.to_string());
form.save(db).await;

// ✅ vérifier d'abord que la ligne appartient au compte
let id = request.get_path_as::<Pk>("commentaire_id").unwrap_or_default();
let a_lui = search!(commentaire::Entity => Id eq id, AuteurId eq user.id)
    .count(request.db()).await.unwrap_or(0) > 0;
if !a_lui {
    return Ok(StatusCode::NOT_FOUND.into_response());
}
```

**Garder la colonne comme champ.** Avec `fields = All`, `auteur_id` est un champ : le visiteur peut l'envoyer. Un `add_value` qui suit le remplace, mais un seul chemin du code qui l'oublie (une branche, un `if`) écrit la valeur du visiteur. Hors des champs, l'oubli n'écrit rien.

```rust
// ❌ auteur_id est un champ : protégé seulement si chaque chemin appelle add_value
#[form(schema = commentaire, fields = All)]

// ✅ seuls les champs que le visiteur remplit
#[form(schema = commentaire, fields = [texte])]
```

**Passer une valeur mise en forme pour l'affichage.** `{:?}` d'une `Option` donne `Some(5)`, que la colonne ne sait pas lire : `save()` échoue.

```rust
// ❌ "Some(5)"
form.add_value("auteur_id", &format!("{:?}", request.user_id().await));

// ✅ la valeur elle-même
let Some(id) = request.user_id().await else { /* non connecté */ };
form.add_value("auteur_id", &id.to_string());
```

**Se tromper de nom.** Un nom qui n'est pas une colonne est ignoré sans erreur (`"auteurId"`, `"Auteur_id"`). La colonne n'est alors pas écrite : `NULL` si elle est nullable, insertion refusée sinon. Un test qui relit la ligne écrite le détecte.

### Règles

- Mêmes règles qu'un champ : jamais la clé d'une ligne modifiée, jamais une colonne secrète dans l'admin, droits de colonne respectés derrière la porte.
- Une valeur illisible pour la colonne (`"abc"` sur un entier) fait échouer `save()` avec une erreur du formulaire.
- Une valeur vide écrit `NULL` sur une colonne nullable ; une colonne non nullable garde sa valeur.
- Un nom qui n'est pas une colonne est ignoré, sans erreur : vérifier l'orthographe (la casse compte).
- Une colonne que le code ne pose pas n'est pas écrite : oublier `add_value` laisse la colonne à `NULL` ou fait refuser l'insertion, jamais une valeur du visiteur.
- `add_value` sur un nom qui **est** un champ remplace la valeur envoyée par le visiteur pour ce champ.

---

## Injecter des choix dynamiques — `field_choices`

`Forms::field_choices` remplace un champ par un `ChoiceField` peuplé depuis une liste `Vec<(String, String)>` de paires `(valeur, libellé)`. La valeur courante est pré-sélectionnée et le flag `required` est préservé.

Utile dans les handlers admin custom pour injecter des options chargées depuis la base :

```rust
use runique::forms::Forms;

// Dans une closure CRUD ou un handler admin personnalisé
let choices: Vec<(String, String)> = themes::Entity::find()
    .all(&db).await?
    .into_iter()
    .map(|t| (t.id.to_string(), t.nom))
    .collect();

form.forms_mut().field_choices("theme_id", "Thème", choices);
```

> Dans l'admin `ModelAdmin`, les champs de formulaire viennent des colonnes déclarées via `.form(|f| f.fields([...]))`. Pour un select peuplé depuis une autre table, utilise `field_choices` dans un handler custom, ou un `ChoiceField` dans ton formulaire. Voir [Vue liste](/docs/fr/admin/liste) et [Builder ModelAdmin](/docs/fr/admin/declaration-macro).

---

[← Trait RuniqueForm](/docs/fr/formulaire/trait) | [**Types de champs**](/docs/fr/formulaire/champs) →
