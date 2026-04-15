# Full example & common pitfalls

[← Template rendering](/docs/en/formulaire/templates)

---

## Full example: signup with persistence

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
                .label("Username")
                .required(),
        );

        form.field(
            &TextField::email("email")
                .label("Email")
                .required(),
        );

        form.field(
            &TextField::password("password")
                .label("Password")
                .required()
                .min_length(8, "Minimum 8 characters"),
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
            // The password is already Argon2-hashed after is_valid()
            password: Set(self.cleaned_string("password").unwrap_or_default()),
            ..Default::default()
        };
        model.insert(db).await
    }
}
```

### GET/POST handler

```rust
pub async fn register(
    mut request: Request,
    Prisme(mut form): Prisme<RegisterForm>,
) -> AppResult<Response> {
    let template = "profile/register_form.html";

    if request.is_get() {
        context_update!(request => {
            "title" => "Sign up",
            "register_form" => &form,
        });
        return request.render(template);
    }

    if request.is_post() {
        if form.is_valid().await {
            match form.save(&request.engine.db).await {
                Ok(_) => {
                    success!(request.notices => "Registration successful!");
                    return Ok(Redirect::to("/").into_response());
                }
                Err(err) => {
                    form.database_error(&err);
                }
            }
        }

        context_update!(request => {
            "title" => "Error",
            "register_form" => &form,
            "messages" => flash_now!(error => "Please correct the errors"),
        });
        return request.render(template);
    }

    request.render(template)
}
```

---

## Edit form — PATCH mode

In `PATCH` mode, `fill()` automatically relaxes the `required` constraint on `Password` fields. This allows an edit form where the password is optional: if left empty, the existing hash is preserved.

```rust
pub async fn edit_profile(
    mut request: Request,
    Prisme(mut form): Prisme<EditProfileForm>,
) -> AppResult<Response> {
    let template = "profile/edit.html";
    let user = get_current_user(&request).await?;

    if request.is_get() {
        context_update!(request => {
            "title" => "Edit profile",
            "edit_form" => &form,
        });
        return request.render(template);
    }

    // In PATCH mode: the password field is no longer automatically required
    if request.is_patch() {
        if form.is_valid().await {
            let new_password = form.cleaned_string("password");

            let mut active: users::ActiveModel = user.into();
            active.username = Set(form.cleaned_string("username").unwrap_or_default());

            // If the password field is filled → new hash; otherwise → unchanged
            if let Some(pwd) = new_password {
                active.password = Set(pwd); // already hashed by finalize()
            }

            active.update(&request.engine.db).await?;
            success!(request.notices => "Profile updated!");
            return Ok(Redirect::to("/profile").into_response());
        }

        context_update!(request => {
            "title" => "Error",
            "edit_form" => &form,
        });
        return request.render(template);
    }

    request.render(template)
}
```

> **💡** PATCH mode is detected automatically by `fill()` via the HTTP method. No additional configuration is needed.

---

## ⚠️ Common pitfalls

### 1. Template variable name collision

If your template uses `{% form.user %}`, the `user` variable in the context **must** be a form, not a SeaORM Model:

```rust
// ❌ ERROR — db_user is a Model, not a form
context_update!(request => { "user" => &db_user });

// ✅ CORRECT — separate names
context_update!(request => {
    "user_form" => &form,
    "found_user" => &db_user,
});
```

### 2. Forgetting `mut` on form

```rust
//  Cannot call is_valid()
Prisme(form): Prisme<MyForm>

//  Correct
Prisme(mut form): Prisme<MyForm>
```

### 3. Comparing passwords after `is_valid()`

```rust
/// main.rs ->
/// with this configuration ->
password_init(PasswordConfig::auto_with(Manual::Argon2));

// After is_valid(), passwords are hashed!
let pwd = form.cleaned_string("password").unwrap_or_default();
// pwd == "$argon2id$v=19$m=..." 😱

// Compare in clean(), BEFORE finalization
async fn clean(&mut self) -> Result<(), StrMap> {
    let pwd1 = self.cleaned_string("password").unwrap_or_default();
    let pwd2 = self.cleaned_string("password_confirm").unwrap_or_default();
    if pwd1 != pwd2 { /* error */ }
    Ok(())
}
```

---

← [**Template rendering**](/docs/en/formulaire/templates) | [**Forms**](/docs/en/formulaire) →
