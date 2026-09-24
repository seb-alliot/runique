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

`ValidationForm::try_new(form, &request)` replaces the `if request.is_get() {...} if request.is_post() {...}` boilerplate: it dispatches on the HTTP method itself (`allow_get`/`allow_post`), validates, and returns `Ok(ValidationForm<F>)` (a type-level proof the form has been validated) or `Err(F)` (the form with its field errors, ready to re-render).

```rust
pub async fn register(mut request: Request) -> AppResult<Response> {
    let form: RegisterForm = request.form();
    let template = "profile/register_form.html";

    let mut validated = match ValidationForm::try_new(form, &request).await {
        Ok(validated) => validated,
        Err(form) => {
            // GET (nothing submitted): blank form, no flash.
            // Submitted but invalid: re-render with the error flash.
            if request.method.is_safe() {
                context_update!(request => {
                    "title" => "Sign up",
                    "register_form" => &form,
                });
            } else {
                context_update!(request => {
                    "title" => "Error",
                    "register_form" => &form,
                    "messages" => flash_now!(error => "Please correct the errors"),
                });
            }
            return request.render(template);
        }
    };

    match validated.save(&request.engine.db).await {
        Ok(_) => {
            success!(request.notices => "Registration successful!");
            return Ok(Redirect::to("/").into_response());
        }
        Err(err) => validated.database_error(&err),
    }

    context_update!(request => {
        "title" => "Error",
        "register_form" => &*validated,
        "messages" => flash_now!(error => "Please correct the errors"),
    });
    request.render(template)
}
```

> **💡** `validated` (type `ValidationForm<RegisterForm>`) implements `Deref<Target = RegisterForm>`: `&*validated` gives access to the form to serialize it into the context. `database_error()` stays callable directly on `ValidationForm` — no need for `into_form()` first to record a save error.

---

## Edit form — PATCH mode

In `PATCH` mode, `fill()` automatically relaxes the `required` constraint on `Password` fields. This allows an edit form where the password is optional: if left empty, the existing hash is preserved.

```rust
pub async fn edit_profile(mut request: Request) -> AppResult<Response> {
    let form: EditProfileForm = request.form();
    let template = "profile/edit.html";
    let user = get_current_user(&request).await?;

    let validated = match ValidationForm::try_new(form, &request).await {
        Ok(validated) => validated,
        Err(form) => {
            context_update!(request => {
                "title" => "Edit profile",
                "edit_form" => &form,
            });
            return request.render(template);
        }
    };

    // In PATCH mode: the password field is no longer automatically required
    let new_password = validated.cleaned_string("password");

    let mut active: users::ActiveModel = user.into();
    active.username = Set(validated.cleaned_string("username").unwrap_or_default());

    // If the password field is filled → new hash; otherwise → unchanged
    if let Some(pwd) = new_password {
        active.password = Set(pwd); // already hashed by finalize()
    }

    active.update(&request.engine.db).await?;
    success!(request.notices => "Profile updated!");
    Ok(Redirect::to("/profile").into_response())
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
let form: MyForm = request.form();

//  Correct
let mut form: MyForm = request.form();
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
