# Upload de fichier

## Formulaire d'upload

```rust
use runique::prelude::*;
use serde::Serialize;

#[derive(Serialize, Debug, Clone)]
#[serde(transparent)]
pub struct ImageForm {
    pub form: Forms,
}

impl FormAccess for ImageForm {
    fn register_fields(form: &mut Forms) {
        form.field(
            &MediaField::image("image")
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

impl RuniqueForm for ImageForm {}
```

---

## Configurer le chemin d'upload

Le dossier est toujours **relatif à `MEDIA_ROOT`** : le fichier y est déplacé à la validation, et la base stocke le chemin relatif (`avatars/photo.png`), servi sous `/media/`. Trois formes :

```rust
// 1 — sous-dossier explicite → {MEDIA_ROOT}/avatars/
MediaField::image("avatar").upload_to("avatars")

// 2 — sous-dossier au nom du champ → {MEDIA_ROOT}/img/
MediaField::image("img").upload_to_env()

// 3 — racine de MEDIA_ROOT (comme sans upload_to)
let config = StaticConfig::from_env();
MediaField::image("img").upload_to(&config)
```

Ne pas inclure `MEDIA_ROOT` dans le chemin : `upload_to("media/avatars")` avec `MEDIA_ROOT=media/` donnerait `media/media/avatars/`.

Le chemin `.env` :

```env
MEDIA_ROOT=media/
```

---

## Extensions disponibles

```rust
MediaField::image("img")     // jpg jpeg png gif webp avif
FileField::document("doc")  // pdf doc docx txt odt
FileField::any("f")         // toute extension, sauf les actives (.svg, .html, .js…)

// Extensions personnalisées :
FileField::any("data").allowed_extensions(vec!["csv", "json"])
```

`FileField::any` ne filtre pas le contenu. Le framework refuse toujours les extensions actives, qu'un navigateur exécute : `.svg`, `.svgz`, `.html`, `.htm`, `.xhtml`, `.xht`, `.shtml`, `.xml`, `.js`, `.mjs`. Ce que le fichier est, et comment il est servi, est à la charge de l'application. Pour une liste fermée, utiliser `image()`, `document()`, ou `allowed_extensions`.

---

## Handler d'upload

```rust
pub async fn upload_image(mut request: Request) -> AppResult<Response> {
    let form: ImageForm = request.form("upload");
    let template = "forms/upload_image.html";

    let validated = match ValidationForm::try_form(form, &request).await {
        Ok(validated) => validated,
        Err(form) => {
            context_update!(request => { "title" => "Envoyer une image", "image_form" => &form });
            return request.render(template);
        }
    };

    // Formulaire sans modèle : pas de `save()`, donc le fichier est publié ici.
    // Validé, il est encore dans le dossier temporaire de la requête, supprimé
    // à la fin de celle-ci.
    let mut form = validated.into_form();
    let chemin = form.cleaned::<String>("image").unwrap_or_default(); // relatif à MEDIA_ROOT
    if let Err(erreur) = form.forms_mut().publish_uploads().await {
        form.forms_mut().errors.push(erreur);
        context_update!(request => { "title" => "Envoyer une image", "image_form" => &form });
        return request.render(template);
    }
    form.forms_mut().complete_uploads().await;

    // `chemin` : à enregistrer là où l'application en a besoin.
    success!(request.notices => "Fichier uploadé avec succès !");
    Ok(Redirect::to("/").into_response())
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

`{% form.image_form %}` place le jeton CSRF avant les champs. Dans un formulaire écrit à la main, mettez `{% csrf %}` **avant** le premier `<input type="file">` : aucun fichier n'est écrit sur le disque tant que le jeton n'a pas été lu et vérifié, et un fichier qui arrive avant lui fait refuser la requête (403). Un envoi en JavaScript peut passer le jeton dans l'en-tête `X-CSRF-Token` à la place.

---

## Voir aussi

| Section | Description |
| --- | --- |
| [Formulaires](/docs/fr/exemple/formulaires) | CRUD avec formulaires |
| [Autres exemples](/docs/fr/exemple/autres) | Messages, API REST, template de base |

## Retour au sommaire

- [Exemples](/docs/fr/exemple)
