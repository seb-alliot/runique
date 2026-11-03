# Lien avec les formulaires & enjeux techniques

## Lien avec les formulaires via `#[form(...)]`

La macro attribut `#[form(...)]` attend :

- `schema = module_du_modèle` (obligatoire) ;
- **et une seule** de ces trois déclarations, qui dit quels champs le formulaire prend :

| Déclaration | Champs du formulaire |
| --- | --- |
| `fields = [a, b]` | seulement ceux-là, dans cet ordre |
| `exclude = [a, b]` | tous, sauf ceux-là |
| `fields = All` | tous |

Sans l'une des trois, ou avec `fields` et `exclude` ensemble, le code **ne compile pas** : la déclaration est incomplète. Un nom qui n'est pas une colonne du modèle (`fields = [username, roles]`) ne compile pas non plus, et l'erreur pointe sur le nom.

`fields = All` et `exclude` prennent aussi les colonnes ajoutées plus tard au modèle. Pour les comptes (`runique_users`), « tous » comprend les rangs (`is_admin`, `is_dirigeant`, `is_superuser`) et `is_active` : à réserver à un formulaire dont vous maîtrisez l'usage. Sur un formulaire public (inscription, profil), listez les champs.

Elle génère :

- la struct avec `form: Forms`
- `impl ModelForm` (`schema()`, `fields()`, `exclude()`)
- `impl FormAccess` (champs et accesseurs — pas de `impl_form_access!(model)` à écrire)
- `impl FormEntity` + `pub const objects` (lien automatique vers `<schema>::Entity`)

Le dev écrit ensuite seulement `impl RuniqueForm` (vide, ou avec ses hooks) :

```rust
use runique::prelude::*;

#[form(schema = users, fields = [username, email])]
pub struct UserForm;

impl RuniqueForm for UserForm {}
```

### Accès ORM depuis le formulaire

`#[form(schema = …)]` relie automatiquement le formulaire à l'entité de son module de modèle (`schema = blog` → `blog::Entity`), ce qui donne `Form::objects` et `search!(@Form => …)` : le formulaire devient un point d'entrée direct pour les requêtes ORM, sans passer par l'entité.

```rust
#[form(schema = users, fields = All)]
pub struct UserForm;

#[form(schema = blog, fields = All)]
pub struct BlogForm;

#[form(schema = document, fields = All)]
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

> `fields` et `exclude` se combinent librement avec le lien automatique vers l'entité.

```rust
#[form(schema = users, fields = [username, email])]
pub struct UserForm;
```

### Avec validation métier (`clean`)

Overrider `clean` directement dans `impl RuniqueForm` — comme Django.
`#[async_trait]` est requis uniquement quand on override une méthode async :

```rust
#[form(schema = users, fields = [username, email, password])]
pub struct RegisterForm;

#[async_trait]
impl RuniqueForm for RegisterForm {

    async fn clean(&mut self) -> Result<(), StrMap> {
        let mut errors = StrMap::new();
        if self.cleaned::<String>("username").unwrap_or_default().len() < 3 {
            errors.insert("username".to_string(), "Minimum 3 caractères".to_string());
        }
        if !self.cleaned::<String>("email").unwrap_or_default().contains('@') {
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

### Personnaliser un champ — `customize()`

Un formulaire `#[form]` se personnalise champ par champ sans réécrire `register_fields` : le hook `customize()` s'exécute juste après l'enregistrement des champs du modèle, à chaque construction du formulaire. Redéclarer un champ sous le même nom le remplace.

```rust
#[form(schema = article, fields = All)]
pub struct ArticleForm;

impl RuniqueForm for ArticleForm {
    fn customize(form: &mut Forms) {
        // Remplace le seul champ `image`, les autres restent ceux du modèle
        form.field(
            &FileField::image("image")
                .label("Illustration")
                .upload_to("articles")
                .max_size(FileSize::mb(5)),
        );
    }
}
```

Les limites déclarées dans le modèle (longueur, taille de fichier…) sont réappliquées après `customize()` : un formulaire peut les resserrer, jamais les dépasser.

## Voir aussi

| Section | Description |
| --- | --- |
| [DSL & AST](/docs/fr/model/dsl) | Syntaxe `model!`, types, options |
| [Génération & ModelSchema](/docs/fr/model/generation) | Code généré |
| [Requêtes CRUD](/docs/fr/orm/requetes) | Référence complète `search!` |

## Retour au sommaire

- [Models](/docs/fr/model)
