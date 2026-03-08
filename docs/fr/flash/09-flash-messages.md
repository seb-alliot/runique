# 💬 Flash Messages

## Système de Messages

Runique fournit un système de messages flash pour les notifications utilisateur. Il existe **deux types** de messages :

1. **Messages de redirection** (`success!`, `error!`, `info!`, `warning!`) — stockés en session, affichés après un redirect
2. **Messages immédiats** (`flash_now!`) — affichés sur la requête courante, sans passer par la session

---

## Macros de redirection

Ces macros stockent les messages en session via `request.notices`. Ils s'affichent **après la prochaine redirection** (pattern Post/Redirect/Get).

### success! — Message de succès

```rust
success!(request.notices => "Utilisateur créé avec succès !");
success!(request.notices => format!("Bienvenue {} !", username));

// Plusieurs messages en une fois
success!(request.notices => "Créé", "Email envoyé", "Bienvenue !");
```

### error! — Message d'erreur

```rust
error!(request.notices => "Une erreur s'est produite");
error!(request.notices => format!("Erreur : {}", e));
```

### info! — Message informatif

```rust
info!(request.notices => "Veuillez vérifier votre email");
```

### warning! — Avertissement

```rust
warning!(request.notices => "Cette action ne peut pas être annulée");
```

> 💡 Chaque macro appelle `.success()`, `.error()`, `.info()` ou `.warning()` sur `request.notices` (de type `Message`).

---

## Macro flash_now! — Messages immédiats

`flash_now!` crée un `Vec<FlashMessage>` pour affichage **immédiat** dans la requête courante. Idéal pour les cas où il n'y a pas de redirection (par exemple, ré-affichage du formulaire après une erreur de validation).

```rust
// Un seul message
let msgs = flash_now!(error => "Veuillez corriger les erreurs");

// Plusieurs messages
let msgs = flash_now!(warning => "Champ A incorrect", "Champ B manquant");
```

### Types disponibles

| Type | Classe CSS générée |
|------|-------------------|
| `success` | `message-success` |
| `error` | `message-error` |
| `info` | `message-info` |
| `warning` | `message-warning` |

### Injection dans le contexte

`flash_now!` retourne un vecteur à injecter manuellement dans le contexte :

```rust
context_update!(request => {
    "title" => "Erreur de validation",
    "form" => &form,
    "messages" => flash_now!(error => "Veuillez corriger les erreurs"),
});
```

---

## Utilisation dans les handlers

### Pattern avec redirection (messages flash)

```rust
pub async fn soumission_inscription(
    mut request: Request,
    Prisme(mut form): Prisme<RegisterForm>,
) -> AppResult<Response> {
    if request.is_post() {
        if form.is_valid().await {
            let user = form.save(&request.engine.db).await.map_err(|err| {
                form.get_form_mut().database_error(&err);
                AppError::from(err)
            })?;

            // ✅ Message flash → affiché après le redirect
            success!(request.notices => format!(
                "Bienvenue {}, votre compte est créé !",
                user.username
            ));
            return Ok(Redirect::to("/").into_response());
        }

        // ❌ Validation échouée → message immédiat (pas de redirect)
        context_update!(request => {
            "title" => "Erreur de validation",
            "inscription_form" => &form,
            "messages" => flash_now!(error => "Veuillez corriger les erreurs"),
        });
        return request.render("inscription_form.html");
    }

    // GET → afficher le formulaire
    context_update!(request => {
        "title" => "Inscription",
        "inscription_form" => &form,
    });
    request.render("inscription_form.html")
}
```

### Plusieurs types de messages

```rust
pub async fn about(mut request: Request) -> AppResult<Response> {
    success!(request.notices => "Ceci est un message de succès.");
    info!(request.notices => "Ceci est un message d'information.");
    warning!(request.notices => "Ceci est un message d'avertissement.");
    error!(request.notices => "Ceci est un message d'erreur.");

    context_update!(request => {
        "title" => "À propos",
    });
    request.render("about/about.html")
}
```

---

## Affichage dans les templates

### Tag automatique {% messages %}

La balise `{% messages %}` affiche automatiquement tous les messages :

```html
{% messages %}
```

Elle inclut le template interne `message/message_include.html` qui génère :

```html
{% if messages %}
    <div class="flash-messages">
        {% for message in messages %}
        <div class="message message-{{ message.level }}">
            {{ message.content }}
        </div>
        {% endfor %}
    </div>
{% endif %}
```

### Placement recommandé

Placez `{% messages %}` dans votre template de base, juste avant le contenu principal :

```html
<!-- base.html -->
<body>
    <header>...</header>

    {% messages %}

    <main>
        {% block content %}{% endblock %}
    </main>

    <footer>...</footer>
</body>
```

### Personnalisation de l'affichage

Pour personnaliser l'affichage, bouclez manuellement sur `messages` :

```html
{% if messages %}
    {% for msg in messages %}
        <div class="alert alert-{{ msg.level }}" role="alert">
            <strong>
                {% if msg.level == "success" %}✅
                {% elif msg.level == "error" %}❌
                {% elif msg.level == "warning" %}⚠️
                {% elif msg.level == "info" %}ℹ️
                {% endif %}
            </strong>
            {{ msg.content }}
        </div>
    {% endfor %}
{% endif %}
```

---

## Comportement flash (une seule lecture)

Les messages flash stockés en session sont **consommés automatiquement** lors de l'affichage :

```
1. POST /inscription
   → success!("Bienvenue !")
   → Redirect::to("/")

2. GET /
   → Les messages sont lus depuis la session
   → Affichés dans le template
   → Supprimés de la session

3. GET / (reload)
   → Plus de messages (déjà consommés)
```

---

## Différence flash vs flash_now

| | `success!` / `error!` / etc. | `flash_now!` |
|---|---|---|
| **Stockage** | Session | Mémoire (Vec) |
| **Affichage** | Après redirect | Requête courante |
| **Durée de vie** | Jusqu'à la prochaine lecture | Requête unique |
| **Usage typique** | Post/Redirect/Get | Ré-affichage formulaire |
| **Injection contexte** | Automatique | Manuelle (`"messages" => flash_now!(...)`) |

---

## Quand utiliser quoi ?

### ✅ Utilisez les macros flash (session)

```rust
// Après une action réussie avec redirection
success!(request.notices => "Sauvegardé !");
return Ok(Redirect::to("/").into_response());
```

### ✅ Utilisez flash_now! (immédiat)

```rust
// Erreur de validation → ré-afficher la page sans redirect
context_update!(request => {
    "form" => &form,
    "messages" => flash_now!(error => "Formulaire invalide"),
});
return request.render("form.html");
```

---

## Prochaines étapes

← [**Middleware & Security**](https://github.com/seb-alliot/runique/blob/main/docs/fr/08-middleware.md) | [**Practical Examples**](https://github.com/seb-alliot/runique/blob/main/docs/fr/10-examples.md) →