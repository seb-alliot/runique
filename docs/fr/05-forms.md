# 📝 Formulaires

## Définir un Formulaire

Utiliser la dérivation `RuniqueForm`:

```rust
use runique::derive_form::RuniqueForm;
use serde::{Deserialize, Serialize};

#[derive(RuniqueForm, Debug, Clone, Serialize, Deserialize)]
pub struct LoginForm {
    #[field(label = "Email", required, input_type = "email")]
    pub email: String,

    #[field(label = "Mot de passe", required, input_type = "password")]
    pub password: String,

    #[field(label = "Se souvenir de moi?", input_type = "checkbox")]
    pub remember: Option<bool>,
}

#[derive(RuniqueForm, Debug, Clone, Serialize, Deserialize)]
pub struct RegisterForm {
    #[field(label = "Nom d'utilisateur", required, min_length = 3, max_length = 50)]
    pub username: String,

    #[field(label = "Email", required, input_type = "email")]
    pub email: String,

    #[field(label = "Mot de passe", required, min_length = 8)]
    pub password: String,

    #[field(label = "Confirmer le mot de passe", required)]
    pub confirm_password: String,

    #[field(label = "J'accepte les conditions", required, input_type = "checkbox")]
    pub accept_terms: bool,
}
```

---

## Utiliser dans un Handler

### Extraction automatique (Prisme)

```rust
use demo_app::forms::LoginForm;
use runique::forms::Prisme;

async fn login_form(
    template: TemplateContext,
) -> Response {
    let form = template.form::<LoginForm>();
    template.render("login.html", &context! {
        "form" => form
    })
}

async fn login_submit(
    mut ctx: RuniqueContext,
    template: TemplateContext,
    Prisme(mut form): Prisme<LoginForm>,
) -> Response {
    // Validation automatique
    if !form.is_valid().await {
        return template.render("login.html", &context! {
            "form" => form,
            "has_errors" => true
        });
    }

    // Authentifier l'utilisateur
    if let Ok(Some(user)) = authenticate(&form.email, &form.password).await {
        success!(ctx.flash => "Bienvenue!");
        ctx.session.insert("user_id", user.id).unwrap();
        return Redirect::to("/dashboard").into_response();
    }

    error!(ctx.flash => "Email ou mot de passe incorrect");
    template.render("login.html", &context! {
        "form" => form
    })
}
```

### Garde (Sentinel) et rôles

`Prisme` exécute trois étapes : `Sentinel` (règles d’accès), vérification CSRF, puis extraction (`Aegis`). Sans règle fournie, seul CSRF + extraction s’exécutent. Pour imposer login/rôle :

```rust
use runique::forms::utils::prisme::{GuardContext, GuardRules};

pub async fn with_rules<B>(mut req: Request<B>, next: Next<B>) -> impl IntoResponse {
    // Login requis + rôle Admin ou Editor
    req.extensions_mut()
        .insert(GuardRules::login_and_roles(["Admin", "Editor"]));

    // Contexte utilisateur injecté depuis votre couche d’authentification
    req.extensions_mut().insert(GuardContext {
        user_id: Some("123".into()),
        roles: vec!["Admin".into()],
    });

    next.run(req).await
}
```

Si aucune règle n’est fournie, `Sentinel` est un no-op. Le CSRF reste appliqué et l’extraction du formulaire continue normalement.

---

## Rendu des formulaires

Le token CSRF est **toujours inclus automatiquement** dans tous les formulaires créés avec la macro `#[derive(RuniqueForm)]`.

### Rendu complet du formulaire

Utilisez le filtre Tera `form` pour afficher le formulaire complet avec tous les champs et le token CSRF :

```html
{% form.inscription_form %}
```

### Rendu champ-par-champ

Pour un contrôle total sur le layout, accédez aux champs individuellement via le filtre `form` :

```html
{{ csrf(csrf_token=form.fields.csrf_token.value) | safe }}

<input type="text" name="username" value="{{ form.inscription_form.username }}" />
<input type="email" name="email" value="{{ form.inscription_form.email }}" />
```
---

## Validation Personnalisée

```rust
#[derive(RuniqueForm, Debug, Clone)]
pub struct UserForm {
    pub username: String,
    pub email: String,
}

impl UserForm {
    pub async fn is_valid(&mut self) -> bool {
        let mut is_valid = true;

        // Validation de longueur
        if self.username.len() < 3 {
            self.add_error("username", "Min 3 caractères");
            is_valid = false;
        }

        // Validation d'unicité
        if let Ok(Some(_)) = User::find_by_email(&self.email).await {
            self.add_error("email", "Email déjà utilisé");
            is_valid = false;
        }

        is_valid
    }

    pub async fn save(&self, db: &DatabaseConnection) -> Result<User> {
        User::create(self.username.clone(), self.email.clone()).save(db).await
    }
}
```

---

## Rendu dans les Templates

```html
<form method="post" action="/login">
    {% csrf_field %}

    <div class="form-group">
        <label for="email">Email:</label>
        <input
            type="email"
            name="email"
            id="email"
            value="{{ form.email }}"
            {% if form.has_error('email') %}class="error"{% endif %}
        >
        {% if form.has_error('email') %}
            <span class="error">{{ form.get_error('email') }}</span>
        {% endif %}
    </div>

    <div class="form-group">
        <label for="password">Mot de passe:</label>
        <input
            type="password"
            name="password"
            id="password"
        >
        {% if form.has_error('password') %}
            <span class="error">{{ form.get_error('password') }}</span>
        {% endif %}
    </div>

    <button type="submit">Connexion</button>
</form>
```

---

## Extraction de Formulaire

```rust
#[derive(Deserialize, RuniqueForm)]
pub struct SearchForm {
    pub query: String,
    pub category: Option<String>,
}

async fn search(
    ExtractForm(form): ExtractForm<SearchForm>,
) -> Response {
    let results = search_items(&form.query).await;
    Json(json!({ "results" => results }))
}
```

---

## Prochaines étapes

← [**Routage**](./04-routing.md) | [**Templates**](./06-templates.md) →
