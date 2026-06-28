# Diagrammes Runique — rétro-ingénierie UML + Merise

Objectif : représenter Runique en diagrammes (UML + Merise) pour **révéler les bugs latents
et inconnus** en suivant les flux de données de bout en bout.

Source de vérité : le code (`runique/`, `derive_form/`). Diagrammes en **Mermaid**
(rendu natif GitHub/markdown, versionnable, sans outil binaire).

> Périmètre : framework uniquement. `demo-app/` n'est ni lu ni modifié.

## Arborescence

```
diagramme/
├── README.md                ← ce fichier (index)
├── merise/
│   └── modele-donnees.md     ← MCD + MLD des tables eihwaz_* + anomalies données
├── uml/                      ← diagrammes de classes par couche
│   ├── app/                  ← builder, staging, RuniqueApp
│   ├── engine/               ← RuniqueEngine, état runtime
│   ├── context/              ← Request, RuniqueContext (dans uml/app pour l'instant)
│   ├── forms/                ← Forms, champs, Prisme, ModelForm
│   ├── admin/                ← registry, resource, handlers, permissions
│   ├── auth/                 ← session, guard, reset password
│   ├── migration/            ← ColumnDef, ModelSchema, diff
│   └── middleware/           ← slots, session store, sécurité
├── flux/                     ← diagrammes de séquence / flux de données
└── anomalies.md              ← synthèse consolidée des bugs latents
```

## Méthode

Chaque diagramme est suivi d'une section **« Anomalies / flux suspects »**. Les trouvailles
sont consolidées dans [anomalies.md](anomalies.md) avec sévérité (🔴 bloquant / 🟠 sérieux /
🟡 mineur) et localisation `fichier:ligne`.

## Avancement

- [x] Merise — modèle de données ([merise/modele-donnees.md](merise/modele-donnees.md))
- [x] UML classes — toutes les couches
- [x] Flux — requête/CSRF/upload, login/session, makemigrations, admin CRUD, reset, contexte
- [x] Synthèse anomalies ([anomalies.md](anomalies.md))

### Diagrammes UML produits

- [uml/engine/engine-et-contexte.md](uml/engine/engine-et-contexte.md)
- [uml/context/request-pipeline.md](uml/context/request-pipeline.md)
- [uml/app/builder-staging.md](uml/app/builder-staging.md)
- [uml/forms/formulaires.md](uml/forms/formulaires.md)
- [uml/admin/admin-resource-permissions.md](uml/admin/admin-resource-permissions.md)
- [uml/auth/authentification.md](uml/auth/authentification.md)
- [uml/middleware/sessions.md](uml/middleware/sessions.md)
- [uml/middleware/securite.md](uml/middleware/securite.md)
- [uml/migration/schema-et-diff.md](uml/migration/schema-et-diff.md)
- [uml/transverse/utilitaires.md](uml/transverse/utilitaires.md) (errors, flash, mailer, password, i18n)
- [uml/derive_form/proc-macro.md](uml/derive_form/proc-macro.md) (AST, parser, generateur, registre)
- [uml/config/configuration.md](uml/config/configuration.md) (RuniqueConfig, db)
- [uml/forms/fields-complets.md](uml/forms/fields-complets.md) (tous les champs + options + validator)
- [uml/migration/types-builder-et-parsed.md](uml/migration/types-builder-et-parsed.md) (PK/FK/index/relation/hooks + Parsed/Changes)
- [uml/admin/admin-complements.md](uml/admin/admin-complements.md) (daemon, helper, forms admin, history)
- [uml/app/staging-configs-et-build-errors.md](uml/app/staging-configs-et-build-errors.md) (BuildError, config builders)
- [uml/macros/macros.md](uml/macros/macros.md) (bdd Objects/search!, routeur, context, forms, template)
- [uml/utils/tracing-securite-tokens.md](uml/utils/tracing-securite-tokens.md) (RuniqueLog, TraceResult, CSRF/CSP)
- [uml/context/extensions-et-middleware-config.md](uml/context/extensions-et-middleware-config.md) (RequestExtensions, MiddlewareConfig, Tera)

> **Reprise de session** : [ETAT-AVANCEMENT.md](ETAT-AVANCEMENT.md) — ce qui est fait, les
> faux positifs vérifiés, et la liste exhaustive de ce qui reste (macros, utils, compléments).

### Flux produits

- [flux/requete-csrf-upload.md](flux/requete-csrf-upload.md)
- [flux/auth-session-et-makemigrations.md](flux/auth-session-et-makemigrations.md)
- [flux/admin-crud-reset-makemigrations.md](flux/admin-crud-reset-makemigrations.md)

### Couverture des modules (lib runique)

| Module | Couvert par |
|--------|-------------|
| app / staging | uml/app |
| engine | uml/engine |
| context | uml/context + uml/engine |
| forms (+ prisme, fields) | uml/forms + flux/requete-csrf-upload |
| admin (registry, resource, handlers, permissions) | uml/admin + flux/admin-crud |
| auth (session, guard, reset) | uml/auth + flux/auth-session + flux/admin-crud-reset |
| migration | uml/migration + flux/makemigrations |
| middleware (session) | uml/middleware/sessions |
| middleware (sécurité) | uml/middleware/securite |
| tables eihwaz_* | merise/modele-donnees |

Modules transverses non diagrammés (utilitaires sans flux de données propre) :
`utils/{trad, mailer, password, cli, constante}`, `flash`, `errors` — à traiter si besoin.
