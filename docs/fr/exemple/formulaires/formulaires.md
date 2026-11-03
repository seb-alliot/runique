# Formulaires : inscription, connexion, recherche

Les exemples ci-dessous reprennent le code du projet créé par `runique new` (compilé à chaque passage de la CI). Voir [Formulaires](/docs/fr/formulaire) pour l'API complète.

---

## Formulaire manuel : connexion

Un formulaire manuel déclare ses champs à la main. À utiliser quand il ne correspond à aucune table (connexion, recherche, contact).

```rust
// src/forms/login.rs
use runique::prelude::*;
use serde::Serialize;

#[derive(Serialize, Debug, Clone)]
#[serde(transparent)]
pub struct LoginForm {
    pub form: Forms,
}

impl FormAccess for LoginForm {
    fn register_fields(form: &mut Forms) {
        form.field(&TextField::text("username").label("Nom d'utilisateur").required());
        form.field(&TextField::password("password").label("Mot de passe").required());
    }
    impl_form_access!();
}

impl RuniqueForm for LoginForm {}
```

`Serialize` (transparent) est nécessaire pour passer le formulaire au template. Ne rangez pas ces formulaires dans `src/entities/` : `runique makemigrations` y cherche les modèles.

### Handler de connexion

```rust
pub async fn sign_in(mut request: Request) -> AppResult<Response> {
    let form: LoginForm = request.form();
    if request.user.is_some() {
        return Ok(Redirect::to("/").into_response());
    }

    let template = "login.html";
    let validated = match ValidationForm::try_form(form, &request, "login").await {
        Ok(validated) => validated,
        Err(form) => {
            context_update!(request => {
                "title" => "Connexion",
                "login_form" => &form,
            });
            return request.render(template);
        }
    };

    let username = validated.cleaned::<String>("username").unwrap_or_default();
    let password = validated.cleaned::<String>("password").unwrap_or_default();
    // Vérifie le mot de passe même pour un nom inconnu (même durée dans les
    // deux cas) et refuse un compte inactif ou jamais activé.
    if let Some(user) = authenticate_user(&request.engine.db, &username, &password).await
        && login(&request.session, &user, None, false).await.is_ok()
    {
        success!(request.notices => format!("Bon retour {} !", user.username));
        return Ok(Redirect::to("/").into_response());
    }

    context_update!(request => {
        "title" => "Connexion",
        "login_form" => &*validated,
        "messages" => flash_now!(error => "Nom d'utilisateur ou mot de passe invalide."),
    });
    request.render(template)
}
```

`ValidationForm::try_form(form, &request, "login")` nomme le formulaire et ne le valide que si la requête renvoie ce nom : un GET, ou l'envoi d'un autre formulaire de la même page, revient en `Err(form)` sans aucune erreur de champ. Le jeton CSRF est vérifié à la lecture du formulaire.

---

## Formulaire basé sur un modèle : inscription

`#[form(schema = …, fields = […])]` génère la struct et `FormAccess` à partir du modèle ; vous n'écrivez que `RuniqueForm`, avec ses hooks.

```rust
// src/forms/signup.rs
use runique::prelude::*;

#[form(schema = runique_users, fields = [username, email, password])]
pub struct SignupForm;

#[async_trait]
impl RuniqueForm for SignupForm {
    async fn clean(&mut self) -> Result<(), StrMap> {
        let username = self.cleaned::<String>("username").unwrap_or_default();
        let mut errors = StrMap::new();
        if username.len() < 5 {
            errors.insert("username".to_string(), "Au moins 5 caractères.".to_string());
        }
        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }
}
```

`#[async_trait]` n'est utile que si vous surchargez un hook asynchrone (`clean`, `clean_field`) ; sans hook, `impl RuniqueForm for SignupForm {}` suffit.

### Enregistrer un compte

`validated.save(&db)` enregistre la ligne, mot de passe compris (son hash). Mais un compte naît **inactif** : il ne peut pas se connecter avant d'être activé. Le projet généré écrit donc son propre `save`, qui crée le compte puis l'active aussitôt :

```rust
impl SignupForm {
    pub async fn save(&self, db: &ADb) -> Result<runique_users::Model, DbErr> {
        use runique::prelude::runique_users::ActiveModel;
        let now = chrono::Utc::now().naive_utc();
        let user = ActiveModel {
            username: Set(self.cleaned::<String>("username").unwrap_or_default()),
            email: Set(self.cleaned::<String>("email").unwrap_or_default()),
            // Après is_valid(), le champ password contient déjà le hash.
            password: Set(self.cleaned::<String>("password").unwrap_or_default()),
            is_active: Set(false),
            is_superuser: Set(false),
            is_dirigeant: Set(false),
            is_admin: Set(false),
            created_at: Set(Some(now)),
            updated_at: Set(Some(now)),
            ..Default::default()
        }
        .insert(db)
        .await?;

        // Un compte est créé inactif. L'activer est une décision métier : ici
        // tout de suite ; pour confirmer l'email d'abord, envoyer un lien et
        // appeler `activate_account` à son ouverture.
        BuiltinUserEntity::activate_account(db, user.id)
            .await?
            .ok_or_else(|| DbErr::Custom("compte déjà activé".into()))
    }
}
```

### Handler d'inscription

```rust
pub async fn sign_up(mut request: Request) -> AppResult<Response> {
    let form: SignupForm = request.form();
    if request.user.is_some() {
        return Ok(Redirect::to("/").into_response());
    }

    let template = "signup.html";
    let mut form = match ValidationForm::try_form(form, &request, "signup").await {
        Ok(validated) => validated.into_form(),
        Err(form) => {
            context_update!(request => {
                "title" => "Inscription",
                "signup_form" => &form,
            });
            return request.render(template);
        }
    };

    match form.save(&request.engine.db).await {
        Ok(user) => {
            if login(&request.session, &user, None, false).await.is_ok() {
                success!(request.notices => format!("Bienvenue {} !", user.username));
            }
            Ok(Redirect::to("/").into_response())
        }
        Err(err) => {
            // Un nom ou un email déjà pris s'affiche sur le champ concerné.
            form.forms_mut().database_error(&err);
            context_update!(request => {
                "title" => "Inscription",
                "signup_form" => &form,
                "messages" => flash_now!(error => "L'inscription a échoué."),
            });
            request.render(template)
        }
    }
}
```

### Template

```html
{% extends "base.html" %}

{% block content %}
  <h1>{{ title }}</h1>
  {% messages %}
  <form method="post" action='{% link "signup" %}'>
    {% form.signup_form %}
    <button type="submit">S'inscrire</button>
  </form>
{% endblock %}
```

`{% form.signup_form %}` écrit les champs, leurs erreurs, le jeton CSRF et le nom du formulaire.

---

## Déconnexion

Un POST, jamais un GET. La vue ne lit aucun formulaire, elle vérifie donc le jeton CSRF elle-même :

```rust
pub async fn sign_out(request: Request) -> AppResult<Response> {
    if !request.prisme.csrf_valid {
        return Ok(StatusCode::FORBIDDEN.into_response());
    }
    logout(&request.session, None).await?;
    Ok(Redirect::to("/").into_response())
}
```

```rust
// url.rs
"/logout" => post(sign_out), name = "logout",
```

```html
<form method="post" action='{% link "logout" %}'>
  {% csrf %}
  <button type="submit">Se déconnecter</button>
</form>
```

---

## Formulaire de recherche (GET)

Par défaut, un formulaire ne se valide jamais sur un GET : les GET ne portent pas de jeton CSRF, un lien forgé pourrait sinon déclencher ses effets. Un formulaire de recherche, qui ne fait que lire, l'autorise explicitement :

```rust
#[derive(Serialize, Debug, Clone)]
#[serde(transparent)]
pub struct SearchForm {
    pub form: Forms,
}

impl FormAccess for SearchForm {
    fn register_fields(form: &mut Forms) {
        form.field(&TextField::text("username").label("Utilisateur").required());
    }
    impl_form_access!();
}

impl RuniqueForm for SearchForm {
    fn allow_get(&self, _request: &Request) -> bool {
        self.is_submitted()
    }
}
```

```rust
pub async fn search_user(mut request: Request) -> AppResult<Response> {
    let form: SearchForm = request.form();
    let template = "search.html";

    let form = match ValidationForm::try_form(form, &request, "search").await {
        Ok(validated) => {
            let username = validated.cleaned::<String>("username").unwrap_or_default();
            let found = runique_users::Entity::find()
                .filter(runique_users::Column::Username.eq(&username))
                .one(&*request.engine.db)
                .await?;
            context_update!(request => { "found_user" => &found.map(|u| u.username) });
            validated.into_form()
        }
        Err(form) => form,
    };

    context_update!(request => { "title" => "Recherche", "search_form" => &form });
    request.render(template)
}
```

```html
<form method="get">
  {% form.search_form %}
  <button type="submit">Chercher</button>
</form>
{% if found_user %}<p>Trouvé : {{ found_user }}</p>{% endif %}
```

Ne passez jamais un modèle de compte entier au template : il contient le hash du mot de passe. Ne gardez que les colonnes à afficher.

---

## Voir aussi

| Section | Description |
| --- | --- |
| [Application minimale](/docs/fr/exemple/minimal) | Point de départ simple |
| [Upload](/docs/fr/exemple/upload) | Upload de fichier |

## Retour au sommaire

- [Exemples](/docs/fr/exemple)
