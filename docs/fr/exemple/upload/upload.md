# Upload de fichier

## Formulaire d'upload

```rust
pub struct ImageForm {
    pub form: Forms,
}

impl RuniqueForm for ImageForm {
    fn register_fields(form: &mut Forms) {
        form.field(
            &FileField::image("image")
                .label("Image")
                .upload_to_env()        // lit MEDIA_ROOT depuis .env
                .max_size(5)
                .max_files(1)
                .max_dimensions(1920, 1080)
                .allowed_extensions(vec!["jpg", "png", "webp", "avif"])
        );
    }
    impl_form_access!();
}
```

---

## Configurer le chemin d'upload

`upload_to` accepte trois formes :

```rust
// 1 — chemin direct
FileField::image("avatar").upload_to("media/avatars")

// 2 — lit MEDIA_ROOT depuis .env (recommandé)
FileField::image("img").upload_to_env()

// 3 — depuis une StaticConfig existante
let config = StaticConfig::from_env();
FileField::image("img").upload_to(&config)
```

Le chemin `.env` :

```env
MEDIA_ROOT=media/
```

---

## Extensions disponibles

```rust
FileField::image("img")     // jpg jpeg png gif webp avif
FileField::document("doc")  // pdf doc docx odt
FileField::any("f")         // pas de filtre

// Extensions personnalisées :
FileField::any("data").allowed_extensions(vec!["csv", "json"])
```

---

## Handler d'upload

```rust
pub async fn upload_image(mut request: Request) -> AppResult<Response> {
    let mut form: ImageForm = request.form();
    let template = "forms/upload_image.html";

    if request.is_get() {
        context_update!(request => {
            "title" => "Uploader un fichier",
            "image_form" => &form,
        });
        return request.render(template);
    }

    if request.is_post() && form.is_valid().await {
        success!(request.notices => "Fichier uploadé avec succès !");
        return Ok(Redirect::to("/").into_response());
    }

    context_update!(request => {
        "title" => "Erreur",
        "image_form" => &form,
    });
    request.render(template)
}
```

---

## Template d'upload

```html
{% extends "base.html" %}

{% block content %}
    <h1>{{ title }}</h1>

    <form method="post" enctype="multipart/form-data">
        {% form.image_form %}
        <button type="submit">Uploader</button>
    </form>
{% endblock %}
```

---

## Voir aussi

| Section | Description |
| --- | --- |
| [Formulaires](/docs/fr/exemple/formulaires) | CRUD avec formulaires |
| [Autres exemples](/docs/fr/exemple/autres) | Messages, API REST, template de base |

## Retour au sommaire

- [Exemples](/docs/fr/exemple)
