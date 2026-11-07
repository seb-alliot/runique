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

impl FormAccess for RegisterForm {
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

impl RuniqueForm for RegisterForm {}

impl RegisterForm {
    pub async fn save(&self, db: &DatabaseConnection) -> Result<users::Model, DbErr> {
        use sea_orm::Set;
        let model = users::ActiveModel {
            username: Set(self.cleaned::<String>("username").unwrap_or_default()),
            email: Set(self.cleaned::<String>("email").unwrap_or_default()),
            // Auto mode: after is_valid(), the field's value is already the hash.
            // In Manual or Custom mode, hash it here with hash().
            password: Set(self.cleaned::<String>("password").unwrap_or_default()),
            ..Default::default()
        };
        model.insert(db).await
    }
}
```

### GET/POST handler

`ValidationForm::try_form(form, &request)` replaces the `if request.is_get() {...} if request.is_post() {...}` boilerplate: it dispatches on the HTTP method itself (`allow_get`/`allow_post`), validates, and returns `Ok(ValidationForm<F>)` (a type-level proof the form has been validated) or `Err(F)` (the form with its field errors, ready to re-render).

```rust
pub async fn register(mut request: Request) -> AppResult<Response> {
    let form: RegisterForm = request.form("my_form");
    let template = "profile/register_form.html";

    let mut validated = match ValidationForm::try_form(form, &request).await {
        Ok(validated) => validated,
        Err(form) => {
            // Not sent (first display): blank form, no flash.
            // Sent but invalid: re-render with the error flash.
            if !form.is_submitted() {
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

    // `RegisterForm::save` (the form's own method, not `ValidationForm::save`):
    // `Ok` is the model, `Err` is a `DbErr` to report with `database_error`.
    match validated.save(&request.engine.db).await {
        Ok(_user) => {
            success!(request.notices => "Registration successful!");
            return Ok(Redirect::to("/").into_response());
        }
        Err(err) => {
            validated.database_error(&err);
        }
    }

    context_update!(request => {
        "title" => "Error",
        "register_form" => &*validated,
        "messages" => flash_now!(error => "Please correct the errors"),
    });
    request.render(template)
}
```

> **💡** This `save` is the one written on `RegisterForm`: it returns the model, and `database_error(&err)` reports the `DbErr`. For an entity form, `ValidationForm::save` returns the form (`Ok(form)` / `Err(form)`) with the database error already on it. `Deref` allows `&*validated` in the context.

---

## Edit form — PATCH mode

On a `PATCH` or `PUT` request, `fill()` automatically relaxes the `required` constraint on `Password` fields. This allows an edit form where the password is optional: if left empty, the existing hash is preserved.

An HTML form only sends `GET` or `POST`: this mode applies to a request sent as `PATCH` / `PUT` by JavaScript (`fetch`). For a regular `POST` form, declare the field without `.required()`.

```rust
pub async fn edit_profile(mut request: Request) -> AppResult<Response> {
    let form: EditProfileForm = request.form("my_form");
    let template = "profile/edit.html";
    let Some(current) = request.user.clone() else {
        return Ok(Redirect::to("/login").into_response());
    };
    let Some(user) = users::Entity::find_by_id(current.id).one(request.db()).await? else {
        return Ok(Redirect::to("/login").into_response());
    };

    let validated = match ValidationForm::try_form(form, &request).await {
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
    let new_password = validated.cleaned::<String>("password");

    let mut active: users::ActiveModel = user.into();
    active.username = Set(validated.cleaned::<String>("username").unwrap_or_default());

    // If the password field is filled → new hash; otherwise → unchanged
    if let Some(pwd) = new_password {
        active.password = Set(pwd); // already hashed by finalize()
    }

    active.update(&request.engine.db).await?;
    success!(request.notices => "Profile updated!");
    Ok(Redirect::to("/profile").into_response())
}
```

> **💡** `fill()` reads the request's HTTP method: nothing else to configure for a `PATCH` / `PUT` request.

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

### 2. Validating without naming the form

```rust
// Avoid: is_valid() alone skips try_form's checks. It does not check that the
// request sends the form's name back (an add_value is enough to make it look
// sent), ignores allow_get / allow_post, and gives no ValidationForm: no save()
// or delete() after it.
let mut form: MyForm = request.form("my_form");
if form.is_valid().await { /* ... */ }

// Correct: request.form("name") names the form, try_form only validates the one submitted
let form: MyForm = request.form("my_form");
match ValidationForm::try_form(form, &request).await { /* ... */ }
```

### 3. Comparing passwords after `is_valid()`

```rust
/// main.rs ->
/// with this configuration ->
password_init(PasswordConfig::auto_with(Manual::Argon2));

// After is_valid(), passwords are hashed!
let pwd = form.cleaned::<String>("password").unwrap_or_default();
// pwd == "$argon2id$v=19$m=..." 😱

// Compare in clean(), BEFORE finalization
async fn clean(&mut self) -> Result<(), StrMap> {
    let pwd1 = self.cleaned::<String>("password").unwrap_or_default();
    let pwd2 = self.cleaned::<String>("password_confirm").unwrap_or_default();
    if pwd1 != pwd2 { /* error */ }
    Ok(())
}
```

---

← [**Template rendering**](/docs/en/formulaire/templates) | [**Forms**](/docs/en/formulaire) →
