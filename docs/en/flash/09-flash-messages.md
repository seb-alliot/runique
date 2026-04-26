# Flash Messages

Runique provides a flash message system for user notifications. There are **two types** of messages:

1. **Redirect messages** (`success!`, `error!`, `info!`, `warning!`) — stored in the session and displayed after a redirect
2. **Immediate messages** (`flash_now!`) — displayed on the current request without using the session

---

## Table of Contents

| Section | Content |
| --- | --- |
| [Macros](/docs/en/flash/macros) | `success!`, `error!`, `info!`, `warning!`, `flash_now!`, differences, when to use |
| [Handlers](/docs/en/flash/handlers) | Usage in handlers, flash behavior (single read) |
| [Templates](/docs/en/flash/templates) | `{% messages %}` tag, placement, customization |

---

## Next Steps

← [**Middleware & Security**](/docs/en/middleware) | [**Practical Examples**](/docs/en/exemple) →

