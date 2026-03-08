# File upload

## Upload form

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

## Upload handler

```rust
pub async fn upload_image(
    mut request: Request,
    Prisme(mut form): Prisme<ImageForm>,
) -> AppResult<Response> {
    let template = "forms/upload_image.html";

    if request.is_get() {
        context_update!(request => {
            "title" => "Upload a file",
            "image_form" => &form,
        });
        return request.render(template);
    }

    if request.is_post() {
        if form.is_valid().await {
            success!(request.notices => "File uploaded successfully!");
            return Ok(Redirect::to("/").into_response());
        }

        context_update!(request => {
            "title" => "Error",
            "image_form" => &form,
            "messages" => flash_now!(error => "Please fix the errors"),
        });
        return request.render(template);
    }

    request.render(template)
}
```

---

## Upload template

```html
{% extends "base.html" %}

{% block content %}
    <h1>{{ title }}</h1>
    {% messages %}

    <form method="post" enctype="multipart/form-data">
        {% form.image_form %}
        <button type="submit">Upload</button>
    </form>
{% endblock %}
```

---

## See also

| Section | Description |
| --- | --- |
| [Forms](https://github.com/seb-alliot/runique/blob/main/docs/en/exemple/forms/forms.md) | CRUD with forms |
| [Other examples](https://github.com/seb-alliot/runique/blob/main/docs/en/exemple/others/others.md) | Messages, REST API, base template |

## Back to summary

- [Examples](https://github.com/seb-alliot/runique/blob/main/docs/en/exemple/10-examples.md)
