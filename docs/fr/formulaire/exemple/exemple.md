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

```rust
pub async fn inscription(mut request: Request) -> AppResult<Response> {
    let mut form: RegisterForm = request.form();
    let template = "profile/register_form.html";

    if request.is_get() {
        context_update!(request => {
            "title" => "Inscription",
            "register_form" => &form,
        });
        return request.render(template);
    }

    if request.is_post() {
        if form.is_valid().await {
            match form.save(&request.engine.db).await {
                Ok(_) => {
                    success!(request.notices => "Inscription réussie !");
                    return Ok(Redirect::to("/").into_response());
                }
                Err(err) => {
                    form.database_error(&err);
                }
            }
        }

        context_update!(request => {
            "title" => "Erreur",
            "register_form" => &form,
            "messages" => flash_now!(error => "Veuillez corriger les erreurs"),
        });
        return request.render(template);
    }

    request.render(template)
}
```

---

## Formulaire d'édition — mode PATCH

En mode `PATCH`, `fill()` relâche automatiquement le `required` sur les champs `Password`. Cela permet de proposer un formulaire d'édition où le mot de passe est optionnel : s'il est laissé vide, l'ancien hash est conservé.

```rust
pub async fn modifier_profil(mut request: Request) -> AppResult<Response> {
    let mut form: EditProfileForm = request.form();
    let template = "profile/edit.html";
    let user = get_current_user(&request).await?;

    if request.is_get() {
        context_update!(request => {
            "title" => "Modifier le profil",
            "edit_form" => &form,
        });
        return request.render(template);
    }

    // En PATCH : le champ password n''est plus requis automatiquement
    if request.is_patch() {
        if form.is_valid().await {
            let new_password = form.cleaned_string("password");

            let mut active: users::ActiveModel = user.into();
            active.username = Set(form.cleaned_string("username").unwrap_or_default());

            // Si le champ password est rempli → nouveau hash ; sinon → inchangé
            if let Some(pwd) = new_password {
                active.password = Set(pwd); // déjà haché par finalize()
            }

            active.update(&request.engine.db).await?;
            success!(request.notices => "Profil mis à jour !");
            return Ok(Redirect::to("/profil").into_response());
        }

        context_update!(request => {
            "title" => "Erreur",
            "edit_form" => &form,
        });
        return request.render(template);
    }

    request.render(template)
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
