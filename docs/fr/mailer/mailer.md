# Mailer — envoi d'emails

[← Retour](/docs/fr/)

---

## Configuration

Le mailer s'initialise dans le builder de l'application. Deux modes disponibles :

### Via variables d'environnement (recommandé)

```rust
RuniqueAppBuilder::new(config)
    .with_mailer_from_env()
    // ...
```

Variables `.env` :

```env
# Backend : "smtp" (défaut) ou "console" (dev — affiche dans le terminal)
EMAIL_BACKEND=smtp

SMTP_HOST=smtp.example.com
SMTP_PORT=587
SMTP_USER=user@example.com
SMTP_PASS=secret
SMTP_FROM=noreply@example.com   # optionnel, défaut = SMTP_USER
SMTP_STARTTLS=true              # optionnel, défaut = true
```

### Manuellement

```rust
use runique::prelude::MailerConfig;

RuniqueAppBuilder::new(config)
    .with_mailer(MailerConfig {
        backend: Default::default(),  // Smtp
        host: "smtp.example.com".to_string(),
        port: 587,
        username: "user@example.com".to_string(),
        password: "secret".to_string(),
        from: "noreply@example.com".to_string(),
        starttls: true,
    })
```

---

## Backends

| Backend | Comportement |
| --- | --- |
| `smtp` | Envoi réel via SMTP (production) |
| `console` | Affiche l'email dans le terminal (développement) |

En dev, `EMAIL_BACKEND=console` évite d'avoir un serveur SMTP local. L'email complet (from, to, subject, body) s'affiche dans les logs.

---

## Envoyer un email

### Raccourci — texte brut

```rust
use runique::prelude::dispatch_email;

dispatch_email("user@example.com", "Bienvenue", "Votre compte a été créé.").await?;
```

### Builder complet — HTML, templates, reply-to

```rust
use runique::prelude::Email;

Email::new()
    .to("user@example.com")
    .subject("Bienvenue sur la plateforme")
    .html("<h1>Bonjour !</h1><p>Votre compte est actif.</p>")
    .reply_to("support@example.com")
    .send()
    .await?;
```

### Avec un template Tera

```rust
use runique::prelude::Email;
use runique::context;

let ctx = context! { "username" => "Alice", "url" => "https://example.com/confirm/abc" };

Email::new()
    .to("alice@example.com")
    .subject("Confirmez votre adresse")
    .template(&tera, "emails/confirmation.html", ctx.into())?
    .send()
    .await?;
```

Le template est un fichier Tera standard dans ton dossier `templates/` :

```html
{# templates/emails/confirmation.html #}
<h1>Bonjour {{ username }} !</h1>
<p><a href="{{ url }}">Confirmez votre adresse email</a></p>
```

---

## Méthodes du builder `Email`

| Méthode | Description |
| --- | --- |
| `.to(address)` | Destinataire |
| `.subject(text)` | Objet |
| `.html(body)` | Corps HTML |
| `.text(body)` | Corps texte brut |
| `.reply_to(address)` | En-tête Reply-To |
| `.template(&tera, name, ctx)` | Rend un template Tera comme corps HTML. Retourne `Result`. |
| `.send().await` | Envoie l'email. Retourne `Result<(), String>`. |

> **Priorité** : si `.html()` et `.template()` sont tous les deux appelés, le dernier gagne. `.text()` est utilisé uniquement si aucun corps HTML n'est défini.

---

## Utilisation dans un handler

```rust
use runique::prelude::dispatch_email;

pub async fn register(mut request: Request) -> AppResult<Response> {
    let mut form: RegisterForm = request.form();

    if form.is_valid().await {
        let email = form.cleaned_string("email").unwrap_or_default();
        let username = form.cleaned_string("username").unwrap_or_default();

        // ... sauvegarder en DB ...

        if let Err(e) = dispatch_email(
            &email,
            "Bienvenue !",
            &format!("Bonjour {username}, votre compte est actif."),
        )
        .await
        {
            tracing::warn!("Erreur email : {e}");
        }

        return request.redirect("/");
    }

    context_update!(request => { "form" => &form });
    request.render("register.html")
}
```

> **Erreurs d'envoi** : `.send()` et `dispatch_email` retournent `Result<(), String>`. En production, logue l'erreur et continue — un email raté ne doit pas planter la requête.

---

← [**Retour**](/docs/fr/)
