# Lien avec les formulaires & enjeux techniques

## Lien avec les formulaires via `#[form(...)]`

La macro attribut `#[form(...)]` attend :

- `schema = chemin_fonction` (obligatoire)
- `fields = [..]` (optionnel)
- `exclude = [..]` (optionnel)
- `model = Entity` (optionnel) — lie le formulaire à une entité SeaORM

Elle génère :

- la struct avec `form: Forms`
- `impl ModelForm` (`schema()`, `fields()`, `exclude()`)
- si `model` est présent : `impl FormEntity` + `pub const objects`

Le dev écrit ensuite `impl RuniqueForm` avec `impl_form_access!(model)` :

```rust
use runique::prelude::*;

#[form(schema = user_schema, fields = [username, email])]
pub struct UserForm;

impl RuniqueForm for UserForm {
    impl_form_access!(model);
}
```

### Avec `model` — accès ORM depuis le formulaire

En ajoutant `model = Entity`, le formulaire devient un point d'entrée direct pour les requêtes ORM, sans passer par l'entité.

```rust
#[form(schema = user_schema, model = users::Entity)]
pub struct UserForm;

#[form(schema = blog_schema, model = blog::Entity)]
pub struct BlogForm;

#[form(schema = document_schema, model = document::Entity)]
pub struct DocumentForm;
```

Cela génère automatiquement :

```rust
impl FormEntity for UserForm {
    type Entity = users::Entity;
}
impl UserForm {
    pub const objects: Objects<users::Entity> = Objects::new();
}
```

**Accès ORM direct via le formulaire :**

```rust
// Tous les enregistrements
let users = UserForm::objects.all().all(&db).await?;

// Avec filtre
let user = UserForm::objects
    .filter(users::Column::Email.eq("alice@example.com"))
    .first(&db)
    .await?;

// Via la macro search!
let results = search!(@UserForm => Username eq "alice").all(&db).await?;
let adults  = search!(@UserForm => Age gte 18).all(&db).await?;

// Toutes les syntaxes search! sont supportées
let results = search!(@BlogForm =>
    or(Status eq "published", Status eq "featured"),
    AuthorId eq author_id,
)
.order_by_desc(blog::Column::CreatedAt)
.limit(10)
.all(&db)
.await?;
```

> `model` est compatible avec `fields` et `exclude` — ils peuvent être combinés librement.

```rust
#[form(schema = user_schema, fields = [username, email], model = users::Entity)]
pub struct UserForm;
```

### Avec validation métier (`clean`)

Overrider `clean` directement dans `impl RuniqueForm` — comme Django.
`#[async_trait]` est requis uniquement quand on override une méthode async :

```rust
#[form(schema = user_schema, fields = [username, email, password])]
pub struct RegisterForm;

#[async_trait]
impl RuniqueForm for RegisterForm {
    impl_form_access!(model);

    async fn clean(&mut self) -> Result<(), StrMap> {
        let mut errors = StrMap::new();
        if self.cleaned_string("username").unwrap_or_default().len() < 3 {
            errors.insert("username".to_string(), "Minimum 3 caractères".to_string());
        }
        if !self.cleaned_string("email").unwrap_or_default().contains('@') {
            errors.insert("email".to_string(), "Email invalide".to_string());
        }
        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }
}
```

> `is_valid()` appelle automatiquement `clean` après la validation structurelle.
> Les erreurs retournées sont attachées aux champs et affichées inline dans le template.

---

## Enjeux techniques

### Avantages

- Contrat unique modèle/schéma centralisé
- Génération cohérente migration + formulaire
- Réduction de duplication de définition de champs
- `clean` est l'override officiel du trait — uniforme entre formulaires manuels et basés modèle

### Points d'attention

- DSL stricte : erreur de syntaxe = erreur de macro au build
- `fields`/`exclude` mal alignés avec le schéma => erreurs de génération/exécution
- `#[async_trait]` requis sur `impl RuniqueForm` uniquement quand on override `clean` ou `clean_field`

### Limitation connue — surcharge de champ non prise en charge

> **La surcharge individuelle d'un champ auto-généré par `#[form(...)]` ou `model!` n'est pas encore prise en charge.**

Il n'est pas possible aujourd'hui de personnaliser un seul champ (ex: ajouter `.max_size(5)` ou changer le label) sans réécrire l'intégralité de `register_fields` à la main, ce qui annule le bénéfice de la macro.

**Contournement actuel :** écrire un formulaire manuel complet et déclarer les champs explicitement.

```rust
// ❌ Pas encore possible
#[form(schema = article_schema)]
pub struct ArticleForm;

impl RuniqueForm for ArticleForm {
    impl_form_access!(model);
    // surcharger juste le champ image → impossible sans tout réécrire
}

// ✅ Contournement : formulaire manuel
impl RuniqueForm for ArticleForm {
    impl_form_access!();
    fn register_fields(form: &mut Forms) {
        form.field(&TextField::text("titre").label("Titre").required());
        form.field(
            &FileField::image("image")
                .upload_to("media/articles")
                .max_size(5),
        );
    }
}
```

Cette limitation sera levée en **v2.0** avec la refactorisation du système de champs en widgets, qui permettra de déclarer et surcharger n'importe quel champ directement depuis le modèle.

---

## Voir aussi

| Section | Description |
| --- | --- |
| [DSL & AST](/docs/fr/model/dsl) | Syntaxe `model!`, types, options |
| [Génération & ModelSchema](/docs/fr/model/generation) | Code généré |
| [Requêtes CRUD](/docs/fr/orm/requetes) | Référence complète `search!` |

## Retour au sommaire

- [Models](/docs/fr/model)
