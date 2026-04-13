# File upload

## Upload form

```rust
pub struct ImageForm {
    pub form: Forms,
}

impl RuniqueForm for ImageForm {
    fn register_fields(form: &mut Forms) {
        form.field(
            &FileField::image("image")
                .label("Image")
                .upload_to_env()        // reads MEDIA_ROOT from .env
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

## Configuring the upload path

`upload_to` accepts three forms:

```rust
// 1 — direct path
FileField::image("avatar").upload_to("media/avatars")

// 2 — reads MEDIA_ROOT from .env (recommended)
FileField::image("img").upload_to_env()

// 3 — from an existing StaticConfig
let config = StaticConfig::from_env();
FileField::image("img").upload_to(&config)
```

`.env` configuration:

```env
MEDIA_ROOT=media/
```

---

## Available field types

```rust
FileField::image("img")     // jpg jpeg png gif webp avif
FileField::document("doc")  // pdf doc docx odt
FileField::any("f")         // no extension filter

// Custom extensions:
FileField::any("data").allowed_extensions(vec!["csv", "json"])
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

    if request.is_post() && form.is_valid().await {
        success!(request.notices => "File uploaded successfully!");
        return Ok(Redirect::to("/").into_response());
    }

    context_update!(request => {
        "title" => "Error",
        "image_form" => &form,
    });
    request.render(template)
}
```

---

## Upload template

```html
{% extends "base.html" %}

{% block content %}
    <h1>{{ title }}</h1>

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
| [Forms](/docs/en/exemple/forms) | CRUD with forms |
| [Other examples](/docs/en/exemple/others) | Messages, REST API, base template |

## Back to summary

- [Examples](/docs/en/exemple)
