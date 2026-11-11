# Assets & Media

## Static Files and Media

| Variable | Default | Description |
|----------|---------|-------------|
| `STATICFILES_DIRS` | `static` | Static files directory |
| `STATIC_URL` | `/static` | URL prefix that folder is served under |
| `MEDIA_ROOT` | `{BASE_DIR}/media`, else `{current folder}/media` | Folder of the files users upload. Created at startup when missing; startup fails if it can't be |
| `MEDIA_URL` | `/media` | URL prefix that folder is served under |
| `TEMPLATES_DIR` | `templates` | Tera template folder(s), comma-separated |
| `STATICFILES` | `default_storage` | Reserved: read but not used yet |
| `RUNIQUE_MAX_UPLOAD_MB` | `100` | Largest size of **one** uploaded file (MB), app-wide. See *Upload size* below |
| `RUNIQUE_MAX_BODY_MB` | `100`, never under `RUNIQUE_MAX_UPLOAD_MB` | Largest size of **one request** carrying files (MB): all its files together, plus 1 MB for the text fields. See *Upload size* below |
| `RUNIQUE_MAX_TEXT_FIELD_KB` | `1024` | Maximum size of a multipart text field (KB) |

---

### Upload size

A file upload goes through three limits, from the narrowest to the widest:

| Limit | Applies to | Default | Set |
| --- | --- | --- | --- |
| the field's `.max_size()` | each file of that field | 10 MB | in the form: `.max_size(FileSize::mb(50))` |
| `RUNIQUE_MAX_UPLOAD_MB` | each file, whatever the field | 100 MB | in the `.env` |
| `RUNIQUE_MAX_BODY_MB` | the whole request: all its files + 1 MB of text fields | 100 MB (+ 1 MB) | in the `.env` |

A field's `.max_size()` can't go past `RUNIQUE_MAX_UPLOAD_MB` or `RUNIQUE_MAX_BODY_MB`: a field declared higher is brought down to them (also through `form.max_size(...)` in the handler), and the hint shown under the field gives that real limit.

A body **without files** (urlencoded form, JSON) read by Runique is capped at **2 MB**, whatever these variables say. Files are written to disk as they arrive; those bodies are read into memory, and 100 MB of memory per request would be open to any visitor.

#### Changing them

```env
# Files up to 200 MB, a request up to 300 MB (+ 1 MB)
RUNIQUE_MAX_UPLOAD_MB=200
RUNIQUE_MAX_BODY_MB=300
```

Both values are read at startup: **restart the app** after changing them. A non-numeric value is ignored and the default applies.

#### Depending on what you set

| `.env` | One file up to | One request up to | For |
| --- | --- | --- | --- |
| nothing | 100 MB | 101 MB | images, documents, short videos: the usual case |
| `RUNIQUE_MAX_UPLOAD_MB=500` | 500 MB | 501 MB (follows the file) | one large file per upload (video) |
| `RUNIQUE_MAX_BODY_MB=300` | 100 MB | 301 MB | several files in one form |
| `RUNIQUE_MAX_UPLOAD_MB=10` + `RUNIQUE_MAX_BODY_MB=30` | 10 MB | 31 MB | 3 PDFs of 10 MB, say, and nothing larger |
| `RUNIQUE_MAX_UPLOAD_MB=200` + `RUNIQUE_MAX_BODY_MB=50` | 200 MB | 201 MB | a request smaller than one file is raised to the per-file cap |

Either way, each field keeps its `.max_size()` (10 MB by default): raising the variables doesn't raise the fields. A 200 MB video needs `.max_size(FileSize::mb(200))` on the field **and** `RUNIQUE_MAX_UPLOAD_MB=200`.

A form with several files adds their sizes up: with the defaults, two 60 MB files (each under 100 MB) make a 120 MB request, refused. Raise `RUNIQUE_MAX_BODY_MB` to the expected total.

Lowering the values under the defaults shrinks what a visitor can have written to disk per request; a higher `.max_size()` then can't be reached. These limits hold **one** request, not the total volume: rate-limit the upload routes too (`.rate_limit(...)` in `urlpatterns!`).

#### When a limit is exceeded

| Over | What happens |
| --- | --- |
| a file over its field's `.max_size()` | The file is received, then refused by the field: the form shows again with the error under the field, nothing is saved |
| a file over `RUNIQUE_MAX_UPLOAD_MB` | The request is refused during the upload (413), the partial file is deleted; the handler isn't called |
| a request over `RUNIQUE_MAX_BODY_MB` + 1 MB | Same (413) |
| a body without files over 2 MB | The request is refused (413) |

For a refusal during the upload (413), the server answers before the browser is done sending: most browsers don't show the response but a connection error. The message ("file too large") can be read by an HTTP client (API, `curl`) and in the log. To warn the visitor, give `.max_size()` a value under the global limits: the refusal then comes from the field, with its message in the page.

An axum extractor (`Json`, `Bytes`, `Multipart`…) used directly in a handler follows `RUNIQUE_MAX_BODY_MB`, not Runique's 2 MB. To cap it further, put `axum::extract::DefaultBodyLimit::max(...)` on its route.

---

### Relative paths and prefixes

- A relative path (`static`, `templates`, `media`) is read from the **folder the app runs in**: run `cargo run` from the project root, and set `WorkingDirectory` (systemd) or `BASE_DIR` in production.
- Changing `STATIC_URL` or `MEDIA_URL` moves the public URLs: `{% static %}` and `{% media %}` follow automatically. A path written by hand in a template (`/static/...`) doesn't.
- Runique only serves the files when the builder turns it on (`.statics()`, or `.static_files(...)`). Behind an Nginx serving `/static/` and `/media/` itself, the prefixes on both sides must match.
- A template folder that can't be found doesn't block startup: the page fails at render time with "template not found".

### Runique's internal resources

| Variable | Default | Description |
|----------|---------|-------------|
| `STATIC_RUNIQUE_PATH` | `<runique sources>/static` | Internal static files (admin, error pages, default home page) |
| `STATIC_RUNIQUE_URL` | `/runique/static` | URL prefix they're served under |
| `MEDIA_RUNIQUE_PATH` | `<runique sources>/media` | Internal media |
| `MEDIA_RUNIQUE_URL` | `/runique/media` | Internal media URL prefix |
| `TEMPLATES_RUNIQUE` | `<runique sources>/templates` | Internal templates |
| `OG_IMAGE` | `/runique/static/runique_320.avif` | Default Open Graph image |

`<runique sources>` is the crate's folder at compile time (`CARGO_MANIFEST_DIR`).

---

## Docker deployment

`STATIC_RUNIQUE_PATH` points to Runique's own admin static files (CSS, JS). Its default value is resolved at compile time via `CARGO_MANIFEST_DIR` and is invalid inside a Docker container at runtime.

Set it explicitly in your `.env`:

```env
STATIC_RUNIQUE_PATH=/app/runique/static
```

Then copy the corresponding directory from the Runique source into the image during the Docker build.

After a `runique` dependency version bump, always rebuild the image:

```bash
docker compose build
docker compose up -d
```

`docker compose up -d` alone reuses the old image and the old assets.

---

## See also

| Section | Description |
| --- | --- |
| [Application & Server](/docs/en/env/application) | RUNIQUE_ENV, DEBUG, IP_SERVER, PORT, DB |
| [Security & sessions](/docs/en/env/security) | Middlewares, CSP, Sessions |

## Back to summary

- [Environment Variables](/docs/en/env)
