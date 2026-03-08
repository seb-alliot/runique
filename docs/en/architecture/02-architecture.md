# Architecture

Runique is organized into **functional modules** based on responsibility:

```text
runique/src/
├── app/                    #  App Builder, Templates & Intelligent Builder
│   ├── builder.rs          #  RuniqueAppBuilder with slots
│   ├── error_build.rs      #  Build errors
│   ├── templates.rs        #  TemplateLoader (Tera)
│   └── staging/            #  Staging structs
│       ├── core_staging.rs
│       ├── middleware_staging.rs
│       └── static_staging.rs
│   └── error_build.rs      #  BuildError & CheckReport
├── config/                 #  Configuration & Settings
├── context/                #  Request Context & Tera tools
│   ├── request.rs          #  Request struct (extractor)
│   └── tera/               #  Tera filters and functions
├── db/                     #  ORM & Database
├── engine/                 #  RuniqueEngine
├── errors/                 #  Error handling
├── flash/                  #  Flash messages
├── forms/                  #  Form system
├── macros/                 #  Utility macros
│   ├── context_macro/      #  context!, context_update!
│   ├── flash_message/      #  success!, error!, info!, warning!, flash_now!
│   └── router/             #  urlpatterns!, view!, impl_objects!
├── middleware/             #  Middleware (Security)
│   └── security/           #  CSRF, CSP, Host, Cache, Error Handler
├── utils/                  #  Utilities
├── lib.rs
└── prelude.rs
````

---

## Table of Contents

| Section                                                                                                           | Content                                   |
| ----------------------------------------------------------------------------------------------------------------- | ----------------------------------------- |
| [Core Concepts](https://github.com/seb-alliot/runique/blob/main/docs/en/architecture/concepts/concepts.md)        | `RuniqueEngine`, `Request`, `Prisme<T>`   |
| [Macros](https://github.com/seb-alliot/runique/blob/main/docs/en/architecture/macros/macros.md)                   | Context, flash, routing, and error macros |
| [Tera Tags & Filters](https://github.com/seb-alliot/runique/blob/main/docs/en/architecture/tera/tera.md)          | Django-like tags, filters, functions      |
| [Middleware Stack](https://github.com/seb-alliot/runique/blob/main/docs/en/architecture/middleware/middleware.md) | Slot order and dependency injection       |
| [Request Lifecycle](https://github.com/seb-alliot/runique/blob/main/docs/en/architecture/lifecycle/lifecycle.md)  | Lifecycle and best practices              |

---

## Next Steps

← [Installation](https://github.com/seb-alliot/runique/blob/main/docs/en/installation/01-installation.md) | [**Configuration**](https://github.com/seb-alliot/runique/blob/main/docs/en/configuration/03-configuration.md) →

