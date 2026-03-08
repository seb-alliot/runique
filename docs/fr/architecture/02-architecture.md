# Architecture

Runique est organisée en **modules fonctionnels** basés sur la responsabilité :

```text
runique/src/
├── app/                    #  App Builder, Templates & Builder Intelligent
│   ├── builder.rs          #  RuniqueAppBuilder avec slots
│   ├── error_build.rs      #  Erreurs de build
│   ├── templates.rs        #  TemplateLoader (Tera)
│   └── staging/            #  Staging structs
│       ├── core_staging.rs
│   │   ├── middleware_staging.rs
│   │   └── static_staging.rs
│   └── error_build.rs      #  BuildError & CheckReport
├── config/                 #  Configuration & Settings
├── context/                #  Request Context & Tera tools
│   ├── request.rs          #  Struct Request (extracteur)
│   └── tera/               #  Filtres et fonctions Tera
├── db/                     #  ORM & Database
├── engine/                 #  RuniqueEngine
├── errors/                 #  Gestion des erreurs
├── flash/                  #  Messages flash
├── forms/                  #  Système de formulaires
├── macros/                 #  Macros utilitaires
│   ├── context_macro/      #  context!, context_update!
│   ├── flash_message/      #  success!, error!, info!, warning!, flash_now!
│   └── router/             #  urlpatterns!, view!, impl_objects!
├── middleware/             #  Middleware (Sécurité)
│   └── security/           #  CSRF, CSP, Host, Cache, Error Handler
├── utils/                  #  Utilitaires
├── lib.rs
└── prelude.rs
```

---

## Table des matières

| Section | Contenu |
| --- | --- |
| [Concepts clés](https://github.com/seb-alliot/runique/blob/main/docs/fr/architecture/concepts/concepts.md) | `RuniqueEngine`, `Request`, `Prisme<T>` |
| [Macros](https://github.com/seb-alliot/runique/blob/main/docs/fr/architecture/macros/macros.md) | Macros de contexte, flash, routage, erreur |
| [Tags & filtres Tera](https://github.com/seb-alliot/runique/blob/main/docs/fr/architecture/tera/tera.md) | Tags Django-like, filtres, fonctions |
| [Stack middleware](https://github.com/seb-alliot/runique/blob/main/docs/fr/architecture/middleware/middleware.md) | Ordre des slots, injection de dépendances |
| [Lifecycle d'une requête](https://github.com/seb-alliot/runique/blob/main/docs/fr/architecture/lifecycle/lifecycle.md) | Cycle de vie, bonnes pratiques |

---

## Prochaines étapes

← [Installation](https://github.com/seb-alliot/runique/blob/main/docs/fr/installation/01-installation.md) | [**Configuration**](https://github.com/seb-alliot/runique/blob/main/docs/fr/configuration/03-configuration.md) →
