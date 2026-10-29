# Diagrammes Runique — rétro-ingénierie UML + Merise

Objectif : représenter Runique en diagrammes (UML + Merise) pour **révéler les bugs latents
et inconnus** en suivant les flux de données de bout en bout.

Source de vérité : le code (`runique/`, `derive_form/`, `runique_dsl/`). Diagrammes en **Mermaid**
(rendu natif GitHub/markdown, versionnable, sans outil binaire).

> Périmètre : framework uniquement. `demo-app/` n'est ni lu ni modifié.
> Version cible des diagrammes admin/auth/merise : **4.0** (refonte gate + ModelAdmin).

## Arborescence

```
diagramme/
├── README.md                ← ce fichier (index)
├── ETAT-AVANCEMENT.md       ← journal de reprise
├── anomalies.md             ← synthèse bugs latents
├── merise/
│   └── modele-donnees.md     ← MCD + MLD eihwaz_* (4.0)
├── uml/                      ← diagrammes de classes par couche
│   ├── app/                  ← builder, staging, RuniqueApp
│   ├── engine/               ← RuniqueEngine
│   ├── context/              ← Request, pipeline, extensions
│   ├── forms/                ← Forms, champs, Prisme, ValidationForm
│   ├── admin/                ← ModelAdmin, AdminSite, gate UI, templates
│   ├── auth/                 ← session, CurrentUser, Rang, gate, reset
│   ├── migration/            ← ColumnDef, ModelSchema, diff, DSL
│   ├── middleware/           ← sessions, sécurité
│   ├── config/               ← RuniqueConfig, db
│   ├── macros/               ← bdd, routeur, forms, template
│   ├── derive_form/          ← proc-macro + runique_dsl
│   ├── transverse/           ← errors, flash, mailer…
│   └── utils/                ← tracing, tokens
└── flux/                     ← séquences bout-en-bout
```

## Méthode

Chaque diagramme est suivi d'une section **« Anomalies / flux suspects »**. Les trouvailles
sont consolidées dans [anomalies.md](anomalies.md) avec sévérité (🔴 bloquant / 🟠 sérieux /
🟡 mineur) et localisation.

## Avancement

- [x] Merise — modèle de données 4.0 ([merise/modele-donnees.md](merise/modele-donnees.md))
- [x] UML classes — toutes les couches (admin/auth réécrits pour 4.0)
- [x] Flux — requête/CSRF/upload, login/session, makemigrations, admin CRUD gate, reset
- [x] Synthèse anomalies ([anomalies.md](anomalies.md))

### Diagrammes UML

- [uml/engine/engine-et-contexte.md](uml/engine/engine-et-contexte.md)
- [uml/context/request-pipeline.md](uml/context/request-pipeline.md)
- [uml/context/extensions-et-middleware-config.md](uml/context/extensions-et-middleware-config.md)
- [uml/app/builder-staging.md](uml/app/builder-staging.md)
- [uml/app/staging-configs-et-build-errors.md](uml/app/staging-configs-et-build-errors.md)
- [uml/forms/formulaires.md](uml/forms/formulaires.md)
- [uml/forms/fields-complets.md](uml/forms/fields-complets.md)
- [uml/admin/admin-resource-permissions.md](uml/admin/admin-resource-permissions.md) ← **4.0 ModelAdmin + gate**
- [uml/admin/admin-complements.md](uml/admin/admin-complements.md) ← **4.0 templates / history / MW**
- [uml/auth/authentification.md](uml/auth/authentification.md) ← **4.0 Rang + CurrentUser + gate**
- [uml/middleware/sessions.md](uml/middleware/sessions.md)
- [uml/middleware/securite.md](uml/middleware/securite.md)
- [uml/migration/schema-et-diff.md](uml/migration/schema-et-diff.md)
- [uml/migration/types-builder-et-parsed.md](uml/migration/types-builder-et-parsed.md)
- [uml/transverse/utilitaires.md](uml/transverse/utilitaires.md)
- [uml/derive_form/proc-macro.md](uml/derive_form/proc-macro.md)
- [uml/config/configuration.md](uml/config/configuration.md)
- [uml/macros/macros.md](uml/macros/macros.md)
- [uml/utils/tracing-securite-tokens.md](uml/utils/tracing-securite-tokens.md)

> **Reprise de session** : [ETAT-AVANCEMENT.md](ETAT-AVANCEMENT.md)

### Flux

- [flux/requete-csrf-upload.md](flux/requete-csrf-upload.md)
- [flux/auth-session-et-makemigrations.md](flux/auth-session-et-makemigrations.md)
- [flux/admin-crud-reset-makemigrations.md](flux/admin-crud-reset-makemigrations.md) ← **4.0 gate**

### Couverture des modules (lib runique)

| Module | Couvert par |
|--------|-------------|
| app / staging (`AdminStaging.site`) | uml/app |
| engine | uml/engine |
| context | uml/context |
| forms (+ prisme, ValidationForm) | uml/forms + flux/requete-csrf-upload |
| **admin 4.0** (builder, site, router, template, config, middleware) | uml/admin + flux/admin-crud |
| **auth 4.0** (gate, ModelMeta, Rang, permissions, history, migrations_table) | uml/auth + merise |
| migration + runique_dsl | uml/migration + flux/makemigrations |
| middleware session / sécurité | uml/middleware |
| tables eihwaz_* | merise/modele-donnees |

Hors-scope assumé : `composant-bin/` (scaffolding `new_project`) ; `cli/` couvert en flux.
