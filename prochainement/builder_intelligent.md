
---

# 📋 Architecture du Builder Intelligent - Runique Framework

**CONFIDENTIEL - NE PAS PARTAGER**

---

## 🎯 Concept Clé : Builder Hybride

**Innovation** : Premier framework web à combiner flexibilité d'écriture et rigueur d'exécution via un pipeline de validation.

### Philosophie
```
Flexibilité (Staging) + Validation (Pipeline) = Builder Intelligent
```

Inspiré du système **Prisme** des formulaires :
- Collecte flexible des données
- Validation stricte
- Signal OK
- Finalisation/Construction

---

## 🏗️ Architecture en 3 Couches

### 1. CoreStaging (Composants obligatoires)
```rust
struct CoreStaging {
    db: Option<DatabaseConnection>,
    tera: Option<Tera>,
    url_registry: ARlockmap,
}

impl CoreStaging {
    fn validate(&self) -> Result<(), BuildError> {
        #[cfg(feature = "orm")]
        if self.db.is_none() {
            return Err(BuildError::MissingDatabase);
        }
        Ok(())
    }

    fn is_ready(&self) -> bool {
        // Vérifie que tous les composants obligatoires sont présents
    }

    fn build_engine(&self, config: &RuniqueConfig) -> Result<AEngine, BuildError> {
        // Construit l'engine avec les composants validés
    }
}
```

### 2. MiddlewareStaging (Middlewares configurables)
```rust
struct MiddlewareStaging {
    // CSRF toujours activé (non-configurable)
    csrf: CsrfConfig,

    // Configurables
    session: Option<SessionBackend>,
    csp: Option<SecurityPolicy>,
    hosts: Option<HostPolicy>,

    // Custom
    custom: Vec<BoxedMiddleware>,
}

impl MiddlewareStaging {
    fn validate(&self) -> Result<(), BuildError> {
        // Valide la cohérence des middlewares
    }

    fn apply_to_router(self, router: Router, engine: &AEngine) -> Router {
        // Applique les middlewares dans l'ORDRE CORRECT
        // peu importe l'ordre d'appel par le dev

        let mut app = router;

        // Ordre strict d'application :
        // 1. Host validation (premier rempart)
        // 2. CSRF (sécurité)
        // 3. Session
        // 4. CSP + Security headers
        // 5. Custom middlewares
        // 6. Error handler (dernier, attrape tout)

        app
    }
}
```

### 3. StaticStaging (Fichiers statiques)
```rust
struct StaticStaging {
    enabled: bool,
}

impl StaticStaging {
    fn attach_to_router(self, router: Router, config: &RuniqueConfig) -> Router {
        // Attache les routes de fichiers statiques
    }
}
```

---

## 🔄 Pipeline de Construction (comme Prisme)

```rust
pub struct RuniqueAppBuilder {
    config: RuniqueConfig,
    core: CoreStaging,
    middleware: MiddlewareStaging,
    statics: StaticStaging,
    router: Option<Router>,
}

impl RuniqueAppBuilder {
    // === PHASE 1 : COLLECTE FLEXIBLE ===

    pub fn with_database(mut self, db: DatabaseConnection) -> Self {
        self.core.db = Some(db);  // Juste stocke
        self
    }

    pub fn middleware(mut self, f: impl FnOnce(MiddlewareStaging) -> MiddlewareStaging) -> Self {
        self.middleware = f(self.middleware);  // Juste stocke
        self
    }

    pub fn statics(mut self) -> Self {
        self.statics.enabled = true;  // Juste stocke
        self
    }

    pub fn routes(mut self, router: Router) -> Self {
        self.router = Some(router);  // Juste stocke
        self
    }

    // === PHASE 2 : VALIDATION + CONSTRUCTION ===

    pub async fn build(self) -> Result<RuniqueApp, BuildError> {
        // 1. VALIDATION (comme Forms::is_valid)
        self.validate()?;

        // 2. SIGNAL : Tous les composants sont OK
        if !self.all_ready() {
            return Err(BuildError::NotReady);
        }

        // 3. CONSTRUCTION dans l'ordre STRICT
        // (peu importe l'ordre d'appel par le dev)

        // A. Core (Engine)
        let tera = self.core.tera.ok_or(BuildError::MissingTera)?;
        let config = new(self.config);
        let engine = self.core.build_engine(&config, tera)?;

        // B. Router + Middlewares (ordre garanti)
        let router = self.router.unwrap_or_else(|| Router::new());
        let router = self.middleware.apply_to_router(router, &engine);

        // C. Static files
        let router = self.statics.attach_to_router(router, &config);

        Ok(RuniqueApp { engine, router })
    }

    // === VALIDATION INTERNE ===

    fn validate(&self) -> Result<(), BuildError> {
        // Validation individuelle (comme field.validate())
        self.core.validate()?;
        self.middleware.validate()?;
        self.statics.validate()?;

        // Validation croisée (dépendances entre composants)
        self.cross_validate()?;

        Ok(())
    }

    fn cross_validate(&self) -> Result<(), BuildError> {
        // Vérifie les dépendances entre composants
        // Ex: Si CSP strict, vérifier que session est configurée
        Ok(())
    }

    fn all_ready(&self) -> bool {
        self.core.is_ready()
            && self.middleware.is_ready()
            && self.statics.is_ready()
    }
}
```

---

## 📊 Comparaison avec les autres frameworks

| Framework | Ordre libre | Validation | Réorganisation | Pipeline |
|-----------|-------------|------------|----------------|----------|
| Actix | ✅ | ❌ | ❌ | ❌ |
| Rocket | ✅ | ❌ | ❌ | ❌ |
| Axum | ✅ | ❌ | ❌ | ❌ |
| Django | ✅ | ❌ | ❌ | ❌ |
| Rails | ✅ | ❌ | ❌ | ❌ |
| **Runique** | ✅ | ✅ | ✅ | ✅ |

---

## 🎯 Avantages de cette approche

### 1. **UX Supérieure**
```rust
// Le dev écrit naturellement
RuniqueApp::builder(config)
    .statics()              // Ce qu'il pense en premier
    .routes(router)         // Ce qu'il code après
    .with_database(db)      // Ce qu'il configure ensuite
    .middleware(...)        // Ce qu'il ajoute à la fin
    .build().await?         // ✅ Marche ! Ordre correct automatique
```

### 2. **Sécurité garantie**
- Validation complète avant construction
- Impossible d'oublier un composant obligatoire
- Messages d'erreur clairs : `BuildError::MissingDatabase`

### 3. **Déterminisme**
- Exécution toujours dans le même ordre
- Pas de surprise selon l'ordre d'appel
- Comportement prévisible

### 4. **Cohérence avec Runique**
- Même philosophie que Prisme (formulaires)
- Pattern reconnaissable dans tout le framework

### 5. **Innovation unique**
- Aucun autre framework ne fait ça
- Argument de vente différenciant

---

## 🔒 Gestion des erreurs

```rust
#[derive(Debug, thiserror::Error)]
pub enum BuildError {
    #[error("Database connection is required when 'orm' feature is enabled")]
    MissingDatabase,

    #[error("Template engine initialization failed: {0}")]
    TemplateError(#[from] tera::Error),

    #[error("Core components not ready: {0}")]
    CoreNotReady(String),

    #[error("Middleware configuration invalid: {0}")]
    MiddlewareInvalid(String),

    #[error("Component dependency not satisfied: {0}")]
    DependencyError(String),
}
```

---

## 📝 Exemples d'usage

### Cas 1 : Configuration standard
```rust
RuniqueApp::builder(config)
    .with_database(db)
    .routes(router)
    .build().await?
```

### Cas 2 : Configuration complète
```rust
RuniqueApp::builder(config)
    .with_database(db)
    .middleware(|m| {
        m.with_session(SessionBackend::Memory)
         .with_csp(SecurityPolicy::strict())
         .add_custom(MyMiddleware)
    })
    .routes(router)
    .statics()
    .build().await?
```

### Cas 3 : Ordre chaotique (marche quand même !)
```rust
RuniqueApp::builder(config)
    .statics()                      // ← En premier
    .routes(router)                 // ← Ensuite
    .middleware(|m| m.with_csp(...)) // ← Après
    .with_database(db)              // ← À la fin
    .build().await?                 // ✅ Exécute dans le bon ordre
```

### Cas 4 : Oubli d'un composant
```rust
RuniqueApp::builder(config)
    .routes(router)
    .build().await?  // ❌ Err(BuildError::MissingDatabase)
```

---

## 🚀 Plan d'implémentation

### Phase 1 : Structures de base (2h)
- [ ] Créer `CoreStaging`
- [ ] Créer `MiddlewareStaging`
- [ ] Créer `StaticStaging`
- [ ] Créer `BuildError` enum

### Phase 2 : Validation (2h)
- [ ] Implémenter `validate()` pour chaque staging
- [ ] Implémenter `cross_validate()`
- [ ] Implémenter `is_ready()` pour chaque staging

### Phase 3 : Construction ordonnée (2h)
- [ ] Implémenter `CoreStaging::build_engine()`
- [ ] Implémenter `MiddlewareStaging::apply_to_router()`
- [ ] Implémenter `StaticStaging::attach_to_router()`

### Phase 4 : Tests (1h)
- [ ] Test ordre chaotique → exécution correcte
- [ ] Test composant manquant → erreur claire
- [ ] Test validation croisée

**Total estimé : 6-8h**

---

## 💎 Communication / Marketing

### Slogan potentiel
> "Runique : Le seul framework avec un Builder Intelligent qui valide et réorganise votre configuration automatiquement"

### Arguments de vente
1. **Flexibilité totale** : Écrivez dans l'ordre qui vous convient
2. **Sécurité garantie** : Validation automatique avant démarrage
3. **Zéro surprise** : Exécution déterministe dans l'ordre optimal
4. **Messages clairs** : Erreurs explicites avec solutions
5. **Innovation unique** : Aucun autre framework ne fait ça

---

## 📚 Documentation utilisateur

```markdown
# Builder Intelligent

Runique introduit le concept de **Builder Intelligent** : vous configurez
votre application dans l'ordre qui vous semble logique, et Runique valide
puis réorganise automatiquement pour garantir un démarrage optimal.

## Exemple

```rust
// Ordre d'écriture libre
RuniqueApp::builder(config)
    .routes(router)          // Vous codez d'abord les routes
    .with_database(db)       // Puis configurez la DB
    .middleware(|m| {        // Enfin ajoutez les middlewares
        m.with_csp(...)
    })
    .build().await?          // ✅ Runique exécute dans l'ordre optimal

// Ordre d'exécution garanti :
// 1. Validation de tous les composants
// 2. Construction du Core (DB, Engine)
// 3. Application des Middlewares (ordre correct)
// 4. Configuration des Routes
// 5. Démarrage
```

## Avantages

- **Productivité** : Écrivez naturellement sans vous soucier de l'ordre
- **Sécurité** : Impossible d'oublier un composant critique
- **Clarté** : Messages d'erreur explicites si configuration incomplète
```

---

## ⚠️ Notes importantes

1. **CSRF toujours activé** : Non-désactivable, intégré au système de formulaires
2. **Session obligatoire** mais customisable (Memory par défaut, Redis/Custom possible)
3. **Ordre d'application des middlewares** : Fixe et optimisé (Host → CSRF → Session → CSP → Custom → Error)
4. **Validation runtime, pas compile-time** : Choix délibéré pour la flexibilité

---

## 🔮 Évolutions futures possibles

### Option 1 : Mode strict optionnel (bien plus tard)
```rust
RuniqueApp::builder_strict(config)  // Type system force l'ordre
    .with_database(db)              // Obligatoire en premier
    .routes(router)                 // Puis routes
    .build().await?
```

Mais **pas prioritaire** - le builder hybride suffit largement.

### Option 2 : Presets
```rust
RuniqueApp::builder(config)
    .preset(Preset::Api)        // Configuration pré-définie pour API
    .with_database(db)
    .build().await?
```

---

## ✅ Décisions architecturales

| Question | Décision | Raison |
|----------|----------|--------|
| Ordre libre ou imposé ? | **Libre** | Meilleure UX |
| Validation compile-time ou runtime ? | **Runtime** | Flexibilité + messages clairs |
| Typestate ou Staging ? | **Staging** | Plus simple, plus maintenable |
| Une API ou deux ? | **Une seule** | Pipeline suffit, pas besoin de `builder_strict()` |
| Inspiration ? | **Prisme (formulaires)** | Cohérence dans Runique |

---
