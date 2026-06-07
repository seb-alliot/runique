# Assets & Media

## Static Files and Media

| Variable | Default | Description |
|----------|---------|-------------|
| `STATICFILES_DIRS` | `static` | Static files directory |
| `STATIC_URL` | `/static` | URL prefix for static files |
| `MEDIA_ROOT` | `media` | Uploaded media files directory |
| `MEDIA_URL` | `/media` | URL prefix for media files |
| `TEMPLATES_DIR` | `templates` | Tera templates directory (comma-separated list allowed) |
| `STATICFILES` | `default_storage` | Storage backend |
| `RUNIQUE_MAX_UPLOAD_MB` | `100` | Global maximum size for file uploads (MB) |
| `RUNIQUE_MAX_TEXT_FIELD_KB` | `1024` | Maximum size of a multipart text field (KB) |

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
| [Application & Server](/docs/en/env/application) | DEBUG, IP_SERVER, PORT, DB, Redirects |
| [Security & sessions](/docs/en/env/security) | ALLOWED_HOSTS, CSP, Middlewares, Sessions |

## Back to summary

- [Environment Variables](/docs/en/env)
