# Refactoring Pipeline Middlewares Runique

## 🎯 Objectif

Refactoriser `build_pipeline` en une architecture à **3 structures logiques** utilisant des **Box dynamiques** pour garantir l'ordre d'exécution des middlewares et permettre l'extensibilité.

---

## 📋 Architecture Actuelle (Code qui fonctionne)

### Ordre d'exécution des middlewares (du premier au dernier exécuté)

```
1. Extensions     → Injecte Tera/Config/Engine dans la requête
2. Session        → Crée/récupère la session
3. CSRF           → Lit la session + utilise engine.config.secret_key
4. Optionnels     → CSP, Hosts, Cache, Auth (selon MiddlewareConfig)
5. Statiques      → Hors pipeline (ServeDir)
```

### Code actuel dans `build_pipeline`

```rust
// ÉTAPE 3 (Premier exécuté) - Extensions
app_router = app_router.layer(axum::middleware::from_fn(inject_extensions));

// ÉTAPE 2 - Session  
app_router = app_router.layer(session_layer);

// ÉTAPE 1 (Dernier exécuté) - Security (attach_middlewares)
app_router = RuniqueEngine::attach_middlewares(engine, app_router);

// Hors pile - Statiques
app_router = static_runique(app_router, config);
```

---

## 🏗️ Nouvelle Architecture : 3 Structures Logiques

### Structure 1 : `PipelineFoundation`

**Responsabilité** : Contient les éléments de base avant application des middlewares

```rust
struct PipelineFoundation {
    router: Router,
    config: Arc<RuniqueConfig>,
    engine: Arc<RuniqueEngine>,
    tera: Arc<Tera>,
}
```

**Contient** :
- Le router avec les routes de l'application
- La configuration globale
- L'engine Runique
- Le moteur de templates Tera

---

### Structure 2 : `CoreMiddlewares<S>`

**Responsabilité** : Définit les middlewares obligatoires et optionnels avec leur ordre d'exécution garanti

```rust
struct CoreMiddlewares<S: SessionStore> {
    config: MiddlewareConfig,
    extensions_layer: ExtensionsLayer,
    session_layer: SessionManagerLayer<S>,  
    csrf_layer: CsrfLayer,
    optional_layers: Vec<Box<dyn RuniqueLayer>>,
}
```

**Points clés** :
- **Champs obligatoires** : `extensions`, `session`, `csrf` → Garantis par le type system
- **MiddlewareConfig** : Contrôle l'activation des middlewares optionnels (CSP, Hosts, Cache, etc.)
- **Vec de Box dynamiques** : Permet l'ajout de middlewares personnalisés par les développeurs

---

### Structure 3 : `StaticAssets`

**Responsabilité** : Gestion des fichiers statiques (hors pipeline de middlewares)

```rust
struct StaticAssets {
    static_url: String,
    static_dir: String,
    media_url: String,
    media_dir: String,
    runique_static_url: Option<String>,
    runique_static_dir: Option<String>,
}
```

**Contient** :
- Configuration des chemins pour `/static`, `/media`, `/static-runique`
- Appliqué via `ServeDir` **après** tous les middlewares

---

## 🔄 Ordre d'Application (IMPORTANT : Tower inverse)

### ⚠️ Piège de Tower/Axum

```rust
// ORDRE DANS LE CODE (lecture de haut en bas ↓)
router
    .layer(A)  // ← Écrit en 1er
    .layer(B)  // ← Écrit en 2ème  
    .layer(C)  // ← Écrit en 3ème

// ORDRE D'EXÉCUTION RÉEL (inverse ↑)
// Requête → C → B → A → Handler
```

### ✅ Application correcte des CoreMiddlewares

```rust
fn apply_core_middlewares<S>(
    router: Router, 
    core: CoreMiddlewares<S>,
    foundation: PipelineFoundation,
) -> Router {
    let mut router = router;
    
    // ORDRE INVERSÉ pour respecter l'ordre d'exécution logique
    
    // DERNIER dans le code = DERNIER exécuté (optionnels)
    for optional in core.optional_layers {
        router = optional.apply(router, foundation.engine.clone());
    }
    
    // 3ème dans code = 3ème exécuté (CSRF)
    router = router.layer(/* csrf_layer */);
    
    // 2ème dans code = 2ème exécuté (Session)
    router = router.layer(core.session_layer);
    
    // 1er dans code = 1er exécuté (Extensions)
    router = router.layer(/* extensions_layer */);
    
    router
}
```

---

## 🎨 Trait RuniqueLayer (pour l'extensibilité)

### Définition du trait

```rust
pub trait RuniqueLayer {
    fn apply(&self, router: Router, engine: Arc<RuniqueEngine>) -> Router;
}
```

**Permet** :
- D'unifier l'interface de tous les middlewares
- De stocker différents types dans `Vec<Box<dyn RuniqueLayer>>`
- Aux développeurs d'ajouter leurs propres middlewares

### Exemple d'implémentation

```rust
struct CspLayer {
    engine: Arc<RuniqueEngine>,
}

impl RuniqueLayer for CspLayer {
    fn apply(&self, router: Router, engine: Arc<RuniqueEngine>) -> Router {
        router.layer(axum::middleware::from_fn_with_state(
            engine,
            csp_middleware
        ))
    }
}
```

---

## 🔧 Construction du Pipeline

### Builder Pattern

```rust
impl CoreMiddlewares<S> {
    fn from_config(
        config: MiddlewareConfig,
        engine: Arc<RuniqueEngine>,
        session_layer: SessionManagerLayer<S>,
        tera: Arc<Tera>,
        app_config: Arc<RuniqueConfig>,
    ) -> Self {
        let mut optional = Vec::new();
        
        // Ajouter les middlewares selon la config
        if config.enable_csp {
            optional.push(Box::new(CspLayer::new(engine.clone())) as Box<dyn RuniqueLayer>);
        }
        
        if config.enable_host_validation {
            optional.push(Box::new(HostValidationLayer::new(engine.clone())) as Box<dyn RuniqueLayer>);
        }
        
        if config.enable_cache {
            optional.push(Box::new(CacheLayer::new(engine.clone())) as Box<dyn RuniqueLayer>);
        }
        
        // etc...
        
        Self {
            config,
            extensions_layer: ExtensionsLayer::new(tera, app_config, engine.clone()),
            session_layer,
            csrf_layer: CsrfLayer::new(engine),
            optional_layers: optional,
        }
    }
}
```

### Méthode build_pipeline refactorée

```rust
fn build_pipeline<S: SessionStore + Clone + Send + Sync + 'static>(
    foundation: PipelineFoundation,
    core: CoreMiddlewares<S>,
    statics: StaticAssets,
) -> Router {
    // Partie 2 : Appliquer middlewares (ordre Tower inversé)
    let router = apply_core_middlewares(foundation.router, core, foundation);
    
    // Partie 3 : Ajouter statiques (hors pile)
    apply_static_assets(router, statics)
}
```

---

## 🎁 Avantages de cette Architecture

### ✅ Type Safety
- Middlewares obligatoires (CSRF, Session, Extensions) garantis par les champs de la struct
- Impossible de construire un pipeline incomplet

### ✅ Ordre Garanti
- L'ordre d'exécution est défini dans `apply_core_middlewares`
- Pas de risque d'inversion accidentelle

### ✅ Extensibilité
- Les développeurs peuvent ajouter leurs middlewares via `Vec<Box<dyn RuniqueLayer>>`
- API simple : implémenter `RuniqueLayer` et ajouter à `optional_layers`

### ✅ Clarté
- Séparation logique : Foundation → Core → Statics
- Code auto-documenté

### ✅ Flexibilité
- `MiddlewareConfig` contrôle l'activation des middlewares fournis par Runique
- Position des middlewares optionnels contrôlable

---

## 📝 Points d'Attention

### 1. Dépendances entre middlewares

```
Extensions → Injecte engine/config/tera
    ↓
Session → Utilise les extensions
    ↓
CSRF → Lit la session ET utilise engine.config.secret_key
```

**CSRF dépend de** :
- Session (pour lire `user_id`)
- Engine (pour `secret_key`)

**Donc Extensions DOIT être avant Session/CSRF**

### 2. Type-state pour forcer la construction complète (optionnel)

```rust
struct CoreMiddlewareBuilder<State> {
    csrf: Option<CsrfLayer>,
    session: Option<SessionManagerLayer<S>>,
    extensions: Option<ExtensionsLayer>,
    _state: PhantomData<State>,
}

// Impossible de .build() sans tous les champs obligatoires
```

### 3. MiddlewareConfig vs présence/absence

- **MiddlewareConfig** : Active/désactive les middlewares fournis par Runique
- **Présence dans Vec** : Middlewares personnalisés ajoutés par l'utilisateur

---

## 🚀 Prochaines Étapes

1. ✅ **Lire le cours PDF** sur Box<dyn Trait> et l'encapsulation
2. ✅ **Créer le trait RuniqueLayer**
3. ✅ **Créer les 3 structures** : PipelineFoundation, CoreMiddlewares, StaticAssets
4. ✅ **Implémenter les wrappers** pour chaque middleware (CsrfLayer, CspLayer, etc.)
5. ✅ **Refactoriser build_pipeline** pour utiliser cette nouvelle architecture
6. ✅ **Tester** que l'ordre d'exécution est respecté
7. ✅ **Documenter** l'API pour les développeurs qui veulent ajouter des middlewares

---

## 💡 Exemple d'Utilisation (API Développeur)

```rust
// Middleware personnalisé de l'utilisateur
struct MyCustomMiddleware {
    config: MyConfig,
}

impl RuniqueLayer for MyCustomMiddleware {
    fn apply(&self, router: Router, engine: Arc<RuniqueEngine>) -> Router {
        router.layer(axum::middleware::from_fn(my_custom_logic))
    }
}

// Dans l'application
let app = RuniqueApp::builder(config)
    .with_database(db)
    .routes(routes)
    .add_optional_middleware(Box::new(MyCustomMiddleware { /* ... */ }))
    .build()
    .await?;
```

---

**Stucture envisagée**

```rust
// 1. Trait pour l'uniformité
trait RuniqueLayer {
    fn apply(self: Box<Self>, router: Router) -> Router;
}

// 2. Pipeline avec positions fixes
struct MiddlewarePipeline<S: SessionStore> {
    // Slots FIXES (ordre garanti)
    slot_extensions: Box<dyn RuniqueLayer>,
    slot_session: Box<dyn RuniqueLayer>,  
    slot_csrf: Box<dyn RuniqueLayer>,
    
    // Slots POSITIONNELS optionnels
    before_core: Vec<Box<dyn RuniqueLayer>>,      // Avant Extensions
    after_csrf: Vec<Box<dyn RuniqueLayer>>,       // Après CSRF
    after_all: Vec<Box<dyn RuniqueLayer>>,        // Tout à la fin
}

// 3. Type-State pour forcer la construction complète
struct PipelineBuilder<State> {
    extensions: Option<Box<dyn RuniqueLayer>>,
    session: Option<Box<dyn RuniqueLayer>>,
    csrf: Option<Box<dyn RuniqueLayer>>,
    before_core: Vec<Box<dyn RuniqueLayer>>,
    after_csrf: Vec<Box<dyn RuniqueLayer>>,
    after_all: Vec<Box<dyn RuniqueLayer>>,
    _state: PhantomData<State>,
}

// États du type-state
struct NoExtensions;
struct HasExtensions;
struct HasSession;
struct HasCsrf;
struct Complete;

// Progression forcée
impl PipelineBuilder<NoExtensions> {
    fn new() -> Self { /* ... */ }
    
    fn with_extensions(self, ext: Box<dyn RuniqueLayer>) 
        -> PipelineBuilder<HasExtensions> 
    { /* ... */ }
}

impl PipelineBuilder<HasExtensions> {
    fn with_session(self, session: Box<dyn RuniqueLayer>) 
        -> PipelineBuilder<HasSession> 
    { /* ... */ }
}

impl PipelineBuilder<HasSession> {
    fn with_csrf(self, csrf: Box<dyn RuniqueLayer>) 
        -> PipelineBuilder<Complete> 
    { /* ... */ }
}

// Méthodes optionnelles disponibles à tous les états
impl<State> PipelineBuilder<State> {
    fn add_before_core(mut self, layer: Box<dyn RuniqueLayer>) -> Self {
        self.before_core.push(layer);
        self
    }
    
    fn add_after_csrf(mut self, layer: Box<dyn RuniqueLayer>) -> Self {
        self.after_csrf.push(layer);
        self
    }
}

// build() UNIQUEMENT sur Complete
impl PipelineBuilder<Complete> {
    fn build(self) -> MiddlewarePipeline {
        MiddlewarePipeline {
            slot_extensions: self.extensions.unwrap(),
            slot_session: self.session.unwrap(),
            slot_csrf: self.csrf.unwrap(),
            before_core: self.before_core,
            after_csrf: self.after_csrf,
            after_all: self.after_all,
        }
    }
}

// 4. Application stricte de l'ordre
impl MiddlewarePipeline {
    fn apply(self, router: Router) -> Router {
        let mut router = router;
        
        // ORDRE INVERSÉ pour Tower
        
        // Dernier exécuté
        for layer in self.after_all.into_iter().rev() {
            router = layer.apply(router);
        }
        
        for layer in self.after_csrf.into_iter().rev() {
            router = layer.apply(router);
        }
        
        router = self.slot_csrf.apply(router);
        router = self.slot_session.apply(router);
        router = self.slot_extensions.apply(router);
        
        // Premier exécuté
        for layer in self.before_core.into_iter().rev() {
            router = layer.apply(router);
        }
        
        router
    }
}
```