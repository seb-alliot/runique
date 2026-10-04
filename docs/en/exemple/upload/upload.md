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
                .upload_to_env()        // → {MEDIA_ROOT}/image/
                .max_size(FileSize::mb(5))
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

The folder is always **relative to `MEDIA_ROOT`**: the file is moved there on validation, and the database stores the relative path (`avatars/photo.png`), served under `/media/`. Three forms:

```rust
// 1 — explicit subfolder → {MEDIA_ROOT}/avatars/
FileField::image("avatar").upload_to("avatars")

// 2 — subfolder named after the field → {MEDIA_ROOT}/img/
FileField::image("img").upload_to_env()

// 3 — root of MEDIA_ROOT (same as no upload_to)
let config = StaticConfig::from_env();
FileField::image("img").upload_to(&config)
```

Don't include `MEDIA_ROOT` in the path: `upload_to("media/avatars")` with `MEDIA_ROOT=media/` would give `media/media/avatars/`.

`.env` configuration:

```env
MEDIA_ROOT=media/
```

---

## Available field types

```rust
FileField::image("img")     // jpg jpeg png gif webp avif
FileField::document("doc")  // pdf doc docx txt odt
FileField::any("f")         // no extension filter

// Custom extensions:
FileField::any("data").allowed_extensions(vec!["csv", "json"])
```

---

## Upload handler

```rust
pub async fn upload_image(mut request: Request) -> AppResult<Response> {
    let form: ImageForm = request.form();
    let template = "forms/upload_image.html";

    if let Err(form) = ValidationForm::try_new(form, &request).await {
        context_update!(request => {
            "title" => "Error",
            "image_form" => &form,
        });
        return request.render(template);
    }

    success!(request.notices => "File uploaded successfully!");
    Ok(Redirect::to("/").into_response())
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
