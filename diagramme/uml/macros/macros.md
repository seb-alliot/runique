# UML — macros (bdd, routeur, context, forms, template)

Module surtout déclaratif (`macro_rules!`) + quelques types support génériques.

## bdd — ORM ergonomique (`Objects` / `search!`)

[`macros/bdd/`](../../../runique/src/macros/bdd/)

```mermaid
classDiagram
    class Objects~E~ {
        +PhantomData~E~
        +new() const
        +all/filter/get/create… (API Django-like)
    }
    class RuniqueQueryBuilder~E~ {
        +Select~E~ query
        +all(db) / all_from_engine(engine)
        +filter / order_by / limit …
    }
    class Queryable { <<trait>> }
    Objects ..> RuniqueQueryBuilder : construit
    RuniqueQueryBuilder ..> Queryable
```

Macros associées :
- `impl_objects!` — génère `Model::objects` (point d'entrée `Objects<E>`).
- `search!{}` / `search_cond!` / `search_munch!` / `search_apply_op!` — DSL de filtres
  composables → `Condition` SeaORM (op `eq/like/gt/in/...`). Voir [filter.rs](../../../runique/src/macros/bdd/filter.rs).

## routeur — routing déclaratif

[`macros/routeur/`](../../../runique/src/macros/routeur/)

```mermaid
flowchart LR
    UP["urlpatterns!{ \"/path\" => view }"] --> RT[Router Axum]
    UP --> REG[register_url: nom → path reverse]
    V["view!"] --> H[handler GET/POST]
    REG --> TAG["{% url %}` Tera"]
```

- `urlpatterns!{}` — déclare les routes (`"/x" => view`, `[get:…, post:…]`), peuple le
  registre d'URL nommées (reverse `{% url %}`).
- `view!` ([get_post.rs](../../../runique/src/macros/routeur/get_post.rs)) — sépare handlers GET/POST.
- `register_url` / `router_ext` — extension du Router + enregistrement reverse.

## context / forms / template — macros

```mermaid
flowchart TB
    subgraph context
      CS[context_update! / context!]
      FL["success!/error!/info!/warning! (flash)"]
      IE[impl_error! → From pour AppError]
    end
    subgraph forms
      EK[define_enum_kind! → FieldKind + From~Field~ GenericField]
      IFA["impl_form_access!(model) → register_fields+customize hook"]
    end
    subgraph template
      CT[const_template! / tags Tera static·media·url]
    end
```

## Anomalies / flux suspects

### 🟢 macros = génération concrète (pas d'avalage)
`search!`/`impl_objects!`/`urlpatterns!` génèrent du code concret. Les `let _ = write!` des
expansions écrivent dans des `String` (infaillible) — bénins (déjà classés).

### 🟡 Rappel F1 — `customize` câblé seulement sur `impl_form_access!(model)` — ✅ VÉRIFIÉ (voulu)
Les forms non-`model` (arms `()`/`($field)`) ne déclenchent pas `customize` — comportement
voulu. Arm `(model)` dupliqué mort supprimé en 2.1.21. Voir
[../forms/formulaires.md](../forms/formulaires.md).
