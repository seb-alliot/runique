# Exemple complet & pièges courants

[← Rendu dans les templates](/docs/fr/formulaire/templates)

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
            username: Set(self.cleaned_string("username").unwrap_or_default()),
            email: Set(self.cleaned_string("email").unwrap_or_default()),
            // Le mot de passe est déjà haché en Argon2 après is_valid()
            password: Set(self.cleaned_string("password").unwrap_or_default()),
            ..Default::default()
        };
        model.insert(db).await
    }
}
```

### Handler GET/POST

`ValidationForm::try_new(form, &request)` remplace le boilerplate `if request.is_get() {...} if request.is_post() {...}` : il dispatche lui-même sur la méthode HTTP (`allow_get`/`allow_post`), valide, et renvoie `Ok(ValidationForm<F>)` (prouve au niveau du type que le formulaire est validé) ou `Err(F)` (formulaire avec ses erreurs de champ, à ré-afficher).

```rust
pub async fn inscription(mut request: Request) -> AppResult<Response> {
    let form: RegisterForm = request.form();
    let template = "profile/register_form.html";

    let mut validated = match ValidationForm::try_new(form, &request).await {
        Ok(validated) => validated,
        Err(form) => {
            // GET (rien soumis) : formulaire vierge, pas de flash.
            // Soumis mais invalide : ré-affichage avec le flash d'erreur.
            if request.method.is_safe() {
                context_update!(request => {
                    "title" => "Inscription",
                    "register_form" => &form,
                });
            } else {
                context_update!(request => {
                    "title" => "Erreur",
                    "register_form" => &form,
                    "messages" => flash_now!(error => "Veuillez corriger les erreurs"),
                });
            }
            return request.render(template);
        }
    };

    match validated.save(&request.engine.db).await {
        Ok(_) => {
            success!(request.notices => "Inscription réussie !");
            return Ok(Redirect::to("/").into_response());
        }
        Err(err) => validated.database_error(&err),
    }

    context_update!(request => {
        "title" => "Erreur",
        "register_form" => &*validated,
        "messages" => flash_now!(error => "Veuillez corriger les erreurs"),
    });
    request.render(template)
}
```

> **💡** `validated` (type `ValidationForm<RegisterForm>`) implémente `Deref<Target = RegisterForm>` : `&*validated` donne accès au formulaire pour le sérialiser dans le contexte. `database_error()` reste appelable sur `ValidationForm` directement — pas besoin de `into_form()` pour poser une erreur de sauvegarde après coup.

---

## Formulaire d'édition — mode PATCH

En mode `PATCH`, `fill()` relâche automatiquement le `required` sur les champs `Password`. Cela permet de proposer un formulaire d'édition où le mot de passe est optionnel : s'il est laissé vide, l'ancien hash est conservé.

```rust
pub async fn modifier_profil(mut request: Request) -> AppResult<Response> {
    let form: EditProfileForm = request.form();
    let template = "profile/edit.html";
    let user = get_current_user(&request).await?;

    let validated = match ValidationForm::try_new(form, &request).await {
        Ok(validated) => validated,
        Err(form) => {
            context_update!(request => {
                "title" => "Modifier le profil",
                "edit_form" => &form,
            });
            return request.render(template);
        }
    };

    // En PATCH : le champ password n'est plus requis automatiquement
    let new_password = validated.cleaned_string("password");

    let mut active: users::ActiveModel = user.into();
    active.username = Set(validated.cleaned_string("username").unwrap_or_default());

    // Si le champ password est rempli → nouveau hash ; sinon → inchangé
    if let Some(pwd) = new_password {
        active.password = Set(pwd); // déjà haché par finalize()
    }

    active.update(&request.engine.db).await?;
    success!(request.notices => "Profil mis à jour !");
    Ok(Redirect::to("/profil").into_response())
}
```

> **💡** Le mode PATCH est détecté automatiquement par `fill()` via la méthode HTTP. Aucune configuration supplémentaire n'est nécessaire.

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
//  Ne peut pas appeler is_valid()
let form: MyForm = request.form();

//  Correct
let mut form: MyForm = request.form();
```

### 3. Comparer des mots de passe après `is_valid()`

```rust
/// main.rs ->
/// avec cette configuration ->
password_init(PasswordConfig::auto_with(Manual::Argon2));

// Après is_valid(), les mots de passe sont hachés !
let mdp = form.cleaned_string("password").unwrap_or_default();
// mdp == "$argon2id$v=19$m=..." 😱

// Comparer dans clean(), AVANT la finalisation
async fn clean(&mut self) -> Result<(), StrMap> {
    let mdp1 = self.cleaned_string("password").unwrap_or_default();
    let mdp2 = self.cleaned_string("password_confirm").unwrap_or_default();
    if mdp1 != mdp2 { /* erreur */ }
    Ok(())
}
```

---

← [**Rendu dans les templates**](/docs/fr/formulaire/templates) | [**Formulaires**](/docs/fr/formulaire) →
