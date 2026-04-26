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

## See also

| Section | Description |
| --- | --- |
| [Application & Server](/docs/en/env/application) | DEBUG, IP_SERVER, PORT, DB, Redirects |
| [Security & sessions](/docs/en/env/security) | ALLOWED_HOSTS, CSP, Middlewares, Sessions |

## Back to summary

- [Environment Variables](/docs/en/env)
