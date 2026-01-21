# Corrections nécessaires pour demo-app

## Problèmes identifiés

### 1. Macros non accessibles
Les macros (`context!`, `success!`, `flash_now!`, `urlpatterns!`, `impl_objects!`) ne sont pas accessibles via `prelude`.

**Solution :** Les macros `#[macro_export]` sont automatiquement exportées à la racine de la crate. Il faut les utiliser avec `runique::context!` ou les importer explicitement :

```rust
use runique::context;
use runique::success;
use runique::flash_now;
// etc.
```

### 2. API RuniqueContext vs TemplateContext
- `RuniqueContext` a un champ `tpl` (pas `engine`)
- `TemplateContext` a un champ `engine` et une méthode `render()`
- Il faut choisir le bon extracteur selon les besoins

### 3. SeaORM entity::prelude non accessible
Ajouté dans lib.rs :
```rust
#[cfg(feature = "orm")]
pub use sea_orm::{
    self,  // Permet d'utiliser sea_orm::
    entity::prelude::*,  // Exporte DeriveEntityModel, etc.
    // ... autres exports
};
```

### 4. NotSet manquant dans blog.rs
Utiliser `Default::default()` au lieu de spécifier chaque champ avec NotSet

### 5. DateTime sans timezone
Remplacer `DateTime` par `DateTime<Utc>` partout

### 6. impl_objects! macro non trouvée
La macro est définie mais pas exportée correctement. Voir point 1.

###7. RuniqueConfig::builder() n'existe pas
Vérifier l'API réelle de RuniqueConfig dans config_struct.rs

### 8. model_derive::ModelForm non généré
`DeriveModelForm` est une macro derive personnalisée qui doit être importée et configurée correctement.

## Actions à effectuer

1. **Dans demo-app/src/views.rs** :
   - Importer les macros explicitement
   - Utiliser `template.engine` pour accéder à l'engine
   - Utiliser les bonnes méthodes de TemplateContext

2. **Dans tous les models** :
   - Importer `impl_objects` explicitement
   - S'assurer que tous les derives SeaORM sont accessibles

3. **Dans demo-app/src/url.rs** :
   - Importer `urlpatterns` et `view` explicitement
   - Retourner correctement le Router

4. **Dans demo-app/src/main.rs** :
   - Vérifier l'API correcte de RuniqueConfig et RuniqueApp

## Recommandation

Créer un fichier `demo-app/src/prelude.rs` qui ré-exporte toutes les macros nécessaires:

```rust
pub use runique::prelude::*;
pub use runique::{context, success, error, info, warning, flash_now};
pub use runique::{urlpatterns, view, impl_objects};
```

Puis dans chaque fichier : `use crate::prelude::*;`
