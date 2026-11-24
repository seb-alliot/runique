# File upload

## Upload form

The minimum to write:

```rust
use runique::prelude::*;

#[derive(Serialize)]
#[serde(transparent)]
pub struct ImageForm {
    pub form: Forms,
}

impl FormAccess for ImageForm {
    fn register_fields(form: &mut Forms) {
        form.field(&MediaField::image("image").label("Image"));
    }
    impl_form_access!();
}

impl RuniqueForm for ImageForm {}
```

- The struct field must be named `form`: `impl_form_access!()` relies on that name.
- `impl RuniqueForm for ImageForm {}` stays empty, but `request.form()` requires it.
- `Serialize` is what lets the form go to the template (`"image_form" => &form`). Runique does not ask for `Debug` or `Clone`.
- `#[serde(transparent)]` is optional for `{% form.image_form %}`, which reads the form either directly or under the `form` key. It gives direct access in the template: `{{ image_form.errors }}` instead of `{{ image_form.form.errors }}`. To serialize other fields next to `form`, put `#[serde(flatten)]` on the `form` field instead.

### Field options

Every setting is optional. Without it, the field takes the value in the right-hand column:

```rust
form.field(
    &MediaField::image("image")
        .label("Image")
        .upload_to_env()        // → {MEDIA_ROOT}/image/
        .max_size(FileSize::mb(5))
        .max_files(1)
        .max_dimensions(1920, 1080)
        .allowed_extensions(vec!["jpg", "png", "webp", "avif"])
);
```

| Method | Without it |
| --- | --- |
| `.label(...)` | empty label: it is not derived from the field name |
| `.upload_to(...)` / `.upload_to_env()` | file at the root of `MEDIA_ROOT` |
| `.max_size(...)` | 10 MB per file |
| `.max_files(n)` | the browser offers a single file, but the server does not check the count |
| `.max_dimensions(w, h)` | no dimension limit (the image signature is still checked) |
| `.allowed_extensions(...)` | jpg, jpeg, png, gif, webp, avif |
| `.required()` | optional field |

---

## Configuring the upload path

The folder is always **relative to `MEDIA_ROOT`**: the file is moved there on validation, and the database stores the relative path (`avatars/photo.png`), served under `/media/`. Three forms:

```rust
// 1 — explicit subfolder → {MEDIA_ROOT}/avatars/
MediaField::image("avatar").upload_to("avatars")

// 2 — subfolder named after the field → {MEDIA_ROOT}/img/
MediaField::image("img").upload_to_env()

// 3 — root of MEDIA_ROOT (same as no upload_to)
let config = StaticConfig::from_env();
MediaField::image("img").upload_to(&config)
```

Don't include `MEDIA_ROOT` in the path: `upload_to("media/avatars")` with `MEDIA_ROOT=media/` would give `media/media/avatars/`.

`.env` configuration:

```env
MEDIA_ROOT=media/
```

---

## Available field types

```rust
MediaField::image("img")     // jpg jpeg png gif webp avif
FileField::document("doc")  // pdf doc docx txt odt
FileField::any("f")         // any extension, except active ones (.svg, .html, .js…)

// Custom extensions:
FileField::any("data").allowed_extensions(vec!["csv", "json"])
```

`FileField::any` does not check the content. The framework always refuses the active extensions a browser runs: `.svg`, `.svgz`, `.html`, `.htm`, `.xhtml`, `.xht`, `.shtml`, `.xml`, `.js`, `.mjs`. What the file is, and how it is served, is the application's responsibility. For a closed list, use `image()`, `document()`, or `allowed_extensions`.

---

## Upload handler

```rust
pub async fn upload_image(mut request: Request) -> AppResult<Response> {
    let form: ImageForm = request.form("upload");
    let template = "forms/upload_image.html";

    let validated = match ValidationForm::try_form(form, &request).await {
        Ok(validated) => validated,
        Err(form) => {
            context_update!(request => { "title" => "Upload an image", "image_form" => &form });
            return request.render(template);
        }
    };

    // A form without a model: no `save()`, so the file is published here.
    // Once validated, it still sits in the request's staging folder, removed
    // when the request ends.
    let mut form = validated.into_form();
    let path = form.cleaned::<String>("image").unwrap_or_default(); // relative to MEDIA_ROOT
    if let Err(error) = form.forms_mut().publish_uploads().await {
        form.forms_mut().errors.push(error);
        context_update!(request => { "title" => "Upload an image", "image_form" => &form });
        return request.render(template);
    }
    form.forms_mut().complete_uploads().await;

    // `path`: store it wherever the application needs it.
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

`{% form.image_form %}` puts the CSRF token before the fields. In a hand-written form, put `{% csrf %}` **before** the first `<input type="file">`: no file is written to disk until the token has been read and checked, and a file arriving before it gets the request refused (403). A JavaScript upload can send the token in the `X-CSRF-Token` header instead.

---

## See also

| Section | Description |
| --- | --- |
| [Forms](/docs/en/exemple/forms) | CRUD with forms |
| [Other examples](/docs/en/exemple/others) | Messages, REST API, base template |

## Back to summary

- [Examples](/docs/en/exemple)
