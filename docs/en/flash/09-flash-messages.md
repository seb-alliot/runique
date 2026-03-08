# Flash Messages

Runique provides a flash message system for user notifications. There are **two types** of messages:

1. **Redirect messages** (`success!`, `error!`, `info!`, `warning!`) — stored in the session and displayed after a redirect
2. **Immediate messages** (`flash_now!`) — displayed on the current request without using the session

---

## Table of Contents

| Section | Content |
| --- | --- |
| [Macros](https://github.com/seb-alliot/runique/blob/main/docs/en/flash/macros/macros.md) | `success!`, `error!`, `info!`, `warning!`, `flash_now!`, differences, when to use |
| [Handlers](https://github.com/seb-alliot/runique/blob/main/docs/en/flash/handlers/handlers.md) | Usage in handlers, flash behavior (single read) |
| [Templates](https://github.com/seb-alliot/runique/blob/main/docs/en/flash/templates/templates.md) | `{% messages %}` tag, placement, customization |

---

## Next Steps

← [**Middleware & Security**](https://github.com/seb-alliot/runique/blob/main/docs/en/middleware/08-middleware.md) | [**Practical Examples**](https://github.com/seb-alliot/runique/blob/main/docs/en/exemple/10-examples.md) →

