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
| `RUNIQUE_MAX_UPLOAD_MB` | unset | Largest uploaded file (MB). Unset: a request is capped at 2 MB (axum's default). Set: the whole request is capped at this value + 1 MB for the text fields |
| `RUNIQUE_MAX_TEXT_FIELD_KB` | `1024` | Maximum size of a multipart text field (KB) |

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
| `OG_IMAGE` | `/runique/static/favicon_runique.ico` | Default Open Graph image |

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
