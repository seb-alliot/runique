# Upload de fichier

## Formulaire d'upload

```rust
pub struct ImageForm {
    pub form: Forms,
}

impl RuniqueForm for ImageForm {
    fn register_fields(form: &mut Forms) {
        let config = StaticConfig::from_env();
        form.field(
            &FileField::image("image")
                .label("Image")
                .upload_to(&config)
                .required()
                .max_size_mb(5)
                .max_files(1)
                .max_dimensions(1920, 1080)
                .allowed_extensions(vec!["jpg", "png", "webp", "avif"])
        );
    }
    impl_form_access!();
}
```

---

## Handler d'upload

```rust
pub async fn upload_image(
    mut request: Request,
    Prisme(mut form): Prisme<ImageForm>,
) -> AppResult<Response> {
    let template = "forms/upload_image.html";

    if request.is_get() {
        context_update!(request => {
            "title" => "Uploader un fichier",
            "image_form" => &form,
        });
        return request.render(template);
    }

    if request.is_post() {
        if form.is_valid().await {
            success!(request.notices => "Fichier uploadé avec succès !");
            return Ok(Redirect::to("/").into_response());
        }

        context_update!(request => {
            "title" => "Erreur",
            "image_form" => &form,
            "messages" => flash_now!(error => "Veuillez corriger les erreurs"),
        });
        return request.render(template);
    }

    request.render(template)
}
```

---

## Template d'upload

```html
{% extends "base.html" %}

{% block content %}
    <h1>{{ title }}</h1>
    {% messages %}

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
| [Formulaires](https://github.com/seb-alliot/runique/blob/main/docs/fr/exemple/formulaires/formulaires.md) | CRUD avec formulaires |
| [Autres exemples](https://github.com/seb-alliot/runique/blob/main/docs/fr/exemple/autres/autres.md) | Messages, API REST, template de base |

## Retour au sommaire

- [Exemples](https://github.com/seb-alliot/runique/blob/main/docs/fr/exemple/10-examples.md)
