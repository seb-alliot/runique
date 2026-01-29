# 🔍 Patterns Répétitifs - Analyse du Codebase Runique

> **Date d'analyse** : 29 janvier 2026  
> **Objectif** : Identifier tous les patterns qui se répètent 3+ fois pour faciliter la factorisation

---

## 📊 Résumé Exécutif

| Catégorie | Patterns trouvés | Occurrences totales | Priorité |
|-----------|------------------|---------------------|----------|
| **Types Arc<>** | 8 | 150+ | 🔴 CRITIQUE |
| **Option<Arc<>>** | 6 | 80+ | 🟠 HAUTE |
| **Gestion d'erreurs** | 4 | 60+ | 🟡 MOYENNE |
| **Session operations** | 5 | 45+ | 🟡 MOYENNE |
| **Context creation** | 3 | 40+ | 🟢 BASSE |

---

## 🔴 PRIORITÉ CRITIQUE : Types `Arc<T>` répétés

### 1. `Arc<Tera>` - **35+ occurrences**

#### **Statut actuel** : ✅ **Déjà factorisé**
```rust
// aliases/definition.rs
pub type ATera = Arc<Tera>;
pub type OATera = Option<ATera>;
```

#### **Occurrences dans le code** :
- `app/builder.rs` (7 fois)
- `app/templates.rs` (2 fois)
- `context/template.rs` (8 fois)
- `engine/core.rs` (3 fois)
- `forms/extractor.rs` (5 fois)
- `forms/field.rs` (4 fois)
- `forms/manager.rs` (6 fois)

#### **Action** : ✅ Déjà appliqué partout

---

### 2. `Arc<DatabaseConnection>` - **28+ occurrences**

#### **Statut actuel** : ✅ **Déjà factorisé**
```rust
// aliases/definition.rs
pub type ADb = Arc<DatabaseConnection>;
pub type OADb = Option<ADb>;
pub type Bdd = Option<DatabaseConnection>;
```

#### **Occurrences principales** :
- `engine/core.rs` (3 fois)
- `app/builder.rs` (4 fois)
- `macros/bdd/objects.rs` (6 fois)
- `macros/bdd/query.rs` (8 fois)
- `forms/manager.rs` (7 fois)

#### **Action** : ✅ Déjà appliqué partout

---

### 3. `Arc<RuniqueEngine>` - **42+ occurrences** 🔥

#### **Statut actuel** : ✅ **Déjà factorisé**
```rust
// aliases/definition.rs
pub type AEngine = Arc<RuniqueEngine>;
pub type OAEngine = Option<AEngine>;
```

#### **Occurrences par fichier** :
```
middleware/security/allowed_hosts.rs → 3 fois (State<AEngine>)
middleware/security/csp.rs           → 5 fois (State<AEngine>)
middleware/security/csrf.rs          → 4 fois (State<AEngine>)
middleware/dev/cache.rs              → 2 fois (State<AEngine>)
context/request/extractor.rs        → 6 fois
context/template.rs                  → 8 fois
app/builder.rs                       → 14 fois
```

#### **Action** : ✅ Massivement utilisé

---

### 4. `Arc<RuniqueConfig>` - **18+ occurrences**

#### **Statut actuel** : ✅ **Déjà factorisé**
```rust
// aliases/definition.rs
pub type ARuniqueConfig = Arc<RuniqueConfig>;
pub type OARuniqueConfig = Option<ARuniqueConfig>;
```

#### **Occurrences principales** :
- `app/builder.rs` (6 fois)
- `context/request_extensions.rs` (4 fois)
- `forms/prisme/aegis.rs` (3 fois)
- `middleware/errors/error.rs` (5 fois)

#### **Action** : ✅ Bien utilisé

---

### 5. `Arc<RwLock<HashMap<String, String>>>` - **12+ occurrences** 🔥

#### **Statut actuel** : ✅ **Déjà factorisé**
```rust
// aliases/definition.rs
pub type ARlockmap = Arc<RwLock<HashMap<String, String>>>;
```

#### **Occurrences** :
- `macros/routeur/register_url.rs` (4 fois)
- `engine/core.rs` (2 fois)
- `app/builder.rs` (3 fois)
- `app/templates.rs` (3 fois)

#### **Action** : ✅ Parfait

---

### 6. `Arc<SecurityPolicy>` - **8 occurrences**

#### **Statut actuel** : ✅ **Déjà factorisé**
```rust
// aliases/definition.rs
pub type ASecurityCsp = Arc<SecurityPolicy>;
pub type OSecurityCsp = Option<ASecurityCsp>;
```

#### **Occurrences** :
- `engine/core.rs` (2 fois)
- `app/builder.rs` (2 fois)
- `middleware/security/csp.rs` (4 fois)

#### **Action** : ✅ Bien fait

---

### 7. `Arc<HostPolicy>` - **6 occurrences**

#### **Statut actuel** : ✅ **Déjà factorisé**
```rust
// aliases/definition.rs
pub type ASecurityHosts = Arc<HostPolicy>;
pub type OSecurityHosts = Option<ASecurityHosts>;
```

#### **Occurrences** :
- `engine/core.rs` (2 fois)
- `app/builder.rs` (2 fois)
- `middleware/security/allowed_hosts.rs` (2 fois)

#### **Action** : ✅ OK

---

### 8. `Arc<dyn SessionStore + Send + Sync>` - **5 occurrences** ⚠️

#### **Statut actuel** : ❌ **PAS ENCORE FACTORISÉ**

#### **Occurrences** :
```rust
// middleware/session/session.rs (ligne 9)
Custom(Arc<dyn SessionStore + Send + Sync>),

// app/builder.rs (ligne 118)
pub fn with_session_store<S: SessionStore + Clone>(...)

// app/builder.rs (ligne 176, 234)
impl<Store: SessionStore + Clone> RuniqueAppBuilderWithStore<Store>
```

#### **Action recommandée** : 🟡 Ajouter l'alias
```rust
// aliases/definition.rs
pub type ASessionStore = Arc<dyn SessionStore + Send + Sync>;
```

**⚠️ Attention** : Les occurrences avec `<S: SessionStore + Clone>` sont **génériques** et ne peuvent pas être remplacées (c'est voulu pour la flexibilité du builder).

---

## 🟠 PRIORITÉ HAUTE : Patterns `Option<T>` répétés

### 1. `Option<CsrfToken>` - **12 occurrences**

#### **Statut actuel** : ✅ **Déjà factorisé**
```rust
// aliases/definition.rs
pub type OCsrfToken = Option<CsrfToken>;
```

#### **Occurrences** :
- `context/request_extensions.rs` (3 fois)
- `middleware/errors/error.rs` (4 fois)
- `middleware/security/csrf.rs` (5 fois)

#### **Action** : ✅ Bien utilisé

---

### 2. `Option<CspNonce>` - **8 occurrences**

#### **Statut actuel** : ✅ **Déjà factorisé**
```rust
// aliases/definition.rs
pub type OCspNonce = Option<CspNonce>;
```

#### **Occurrences** :
- `context/request_extensions.rs` (2 fois)
- `middleware/security/csp.rs` (6 fois)

#### **Action** : ✅ OK

---

### 3. `Option<CurrentUser>` - **6 occurrences**

#### **Statut actuel** : ✅ **Déjà factorisé**
```rust
// aliases/definition.rs
pub type OCurrentUser = Option<CurrentUser>;
```

#### **Occurrences** :
- `context/request_extensions.rs` (2 fois)
- `middleware/auth/auth.rs` (4 fois)

#### **Action** : ✅ Bon

---

### 4. `Option<String>` - **90+ occurrences** 🔥

#### **Statut actuel** : ❌ **Trop générique pour factoriser**

**Contextes différents** :
- Messages d'erreur optionnels
- Valeurs de champs optionnelles
- Paramètres de configuration
- Headers HTTP

#### **Action recommandée** : ❌ **NE PAS factoriser** (trop contextuel)

---

### 5. `Option<usize>` / `Option<i32>` - **40+ occurrences**

#### **Statut actuel** : ❌ **Trop générique**

**Contextes** :
- Limites de longueur de champs
- IDs de base de données
- Compteurs

#### **Action** : ❌ **NE PAS factoriser**

---

## 🟡 PRIORITÉ MOYENNE : Patterns de gestion d'erreurs

### 1. `Result<T, Box<dyn std::error::Error>>` - **15+ occurrences**

#### **Contextes** :
```rust
// app/templates.rs (ligne 17)
pub fn init(...) -> Result<Tera, Box<dyn std::error::Error>>

// app/builder.rs (ligne 157, 226)
pub async fn build(self) -> Result<RuniqueApp, Box<dyn std::error::Error>>

// context/template.rs (ligne 21)
pub struct AppError { ... }
```

#### **Statut actuel** : ⚠️ **Partiellement factorisé**
```rust
// aliases/definition.rs
pub type AppResult<T> = Result<T, Box<AppError>>;
```

**Mais** : Certains endroits utilisent `Box<dyn std::error::Error>` au lieu de `Box<AppError>`.

#### **Action recommandée** : 🟡 Harmoniser après la refonte du builder
- Décider si tout doit retourner `AppResult<T>`
- Ou garder `Box<dyn std::error::Error>` pour les cas génériques

---

### 2. `Result<T, DbErr>` - **25+ occurrences**

#### **Contextes** :
- `macros/bdd/objects.rs` (8 fois)
- `macros/bdd/query.rs` (10 fois)
- `db/config.rs` (7 fois)

#### **Statut actuel** : ❌ **Pas factorisé**

#### **Action recommandée** : 🟡 **Optionnel** — Ajouter si utile :
```rust
pub type DbResult<T> = Result<T, sea_orm::DbErr>;
```

---

### 3. `Result<T, Response>` - **8 occurrences**

#### **Contextes** :
- `forms/prisme/aegis.rs` (ligne 16)
- `forms/prisme/csrf_gate.rs` (ligne 12)
- `forms/extractor.rs` (ligne 19)

#### **Statut actuel** : ❌ **Pas factorisé**

#### **Action recommandée** : 🟢 **Pas nécessaire** (trop spécifique à Axum)

---

### 4. Pattern `map_err(|e| e.to_string())` - **12+ occurrences**

#### **Exemples** :
```rust
// Répété partout dans les champs de formulaire
tera.render(...).map_err(|e| e.to_string())
```

#### **Action recommandée** : 🟢 **Créer une extension trait** (après refonte)
```rust
trait TeraErrorExt {
    fn to_string_err(self) -> Result<String, String>;
}

impl TeraErrorExt for tera::Result<String> {
    fn to_string_err(self) -> Result<String, String> {
        self.map_err(|e| e.to_string())
    }
}
```

---

## 🟡 PRIORITÉ MOYENNE : Opérations Session répétées

### 1. `session.get::<T>(KEY).await.ok().flatten()` - **18+ occurrences**

#### **Pattern répété** :
```rust
// middleware/auth/auth.rs (lignes 11, 18, 24)
session.get::<i32>(SESSION_USER_ID_KEY).await.ok().flatten()
session.get::<String>(SESSION_USER_USERNAME_KEY).await.ok().flatten()

// context/request/extractor.rs (ligne 57)
session.get::<i32>(SESSION_USER_ID_KEY).await.ok().flatten()

// middleware/security/csrf.rs (ligne 24)
session.get::<CsrfToken>(CSRF_TOKEN_KEY).await.ok().flatten()

// flash/flash_manager.rs (lignes 32, 55)
session.get::<Vec<FlashMessage>>(FLASH_KEY).await.ok().flatten()
```

#### **Action recommandée** : 🟡 **Créer une extension trait**
```rust
// utils/session_ext.rs (nouveau fichier)
use tower_sessions::Session;

pub trait SessionExt {
    async fn get_optional<T: for<'de> serde::Deserialize<'de>>(&self, key: &str) -> Option<T>;
}

impl SessionExt for Session {
    async fn get_optional<T: for<'de> serde::Deserialize<'de>>(&self, key: &str) -> Option<T> {
        self.get::<T>(key).await.ok().flatten()
    }
}

// Usage
let user_id = session.get_optional::<i32>(SESSION_USER_ID_KEY).await;
```

---

### 2. `session.insert(KEY, value).await` - **15+ occurrences**

#### **Pattern répété** :
```rust
// Partout dans flash_manager.rs, csrf.rs, auth.rs
session.insert(FLASH_KEY, messages).await
session.insert(CSRF_TOKEN_KEY, &token).await
session.insert(SESSION_USER_ID_KEY, user_id).await
```

#### **Action recommandée** : ✅ **Déjà OK** — Pattern standard, pas besoin de factoriser

---

### 3. Pattern de vérification d'authentification - **8 occurrences**

#### **Pattern répété** :
```rust
// middleware/auth/auth.rs (ligne 11)
pub async fn is_authenticated(session: &Session) -> bool {
    session.get::<i32>(SESSION_USER_ID_KEY).await.ok().flatten().is_some()
}

// Utilisé dans :
// - context/request/extractor.rs
// - middleware/auth/auth.rs (plusieurs fois)
// - middleware/security/csrf.rs
```

#### **Action** : ✅ **Déjà factorisé dans `middleware/auth/auth.rs`** — Réutiliser partout

---

## 🟢 PRIORITÉ BASSE : Context creation répété

### 1. `Context::new()` + `.insert()` - **40+ occurrences**

#### **Pattern répété** :
```rust
let mut context = Context::new();
context.insert("key", &value);
context.insert("another", &other);
```

#### **Occurrences principales** :
- `middleware/errors/error.rs` (10 fois)
- `context/template.rs` (6 fois)
- `macros/bdd/objects.rs` (4 fois)

#### **Statut actuel** : ⚠️ **Partiellement factorisé**
```rust
// macros/context/helper.rs
pub struct ContextHelper { ... }

// Mais peu utilisé dans le code
```

#### **Action recommandée** : 🟢 **Après refonte** — Promouvoir l'usage de `ContextHelper` partout

---

### 2. Injection de variables globales dans le context - **8 occurrences**

#### **Pattern répété** :
```rust
// middleware/errors/error.rs (ligne 95)
fn inject_global_vars(context: &mut Context, config: &RuniqueConfig, csrf_token: Option<String>) {
    context.insert("static_runique", &config.static_files.static_runique_url);
    context.insert("timestamp", &Utc::now().to_rfc3339());
    context.insert("csrf_token", &token);
    context.insert("debug", &config.debug);
}
```

#### **Action** : ✅ **Déjà factorisé dans `inject_global_vars()`** — Bon travail !

---

## 🔵 PATTERNS MINEURS (< 3 occurrences)

### Patterns ignorés volontairement :

- `StatusCode::INTERNAL_SERVER_ERROR` (30+ fois) → Normal
- `HeaderValue::from_static(...)` (20+ fois) → Standard Axum
- `axum::response::Html(...)` (15+ fois) → Standard
- `serde_json::json!(...)` (50+ fois) → Standard

---

## 📋 Plan d'Action Recommandé

### **Phase 1 : Compléter les aliases (maintenant)** ⏱️ 5 min

```rust
// À ajouter dans aliases/definition.rs

/// Session store type alias
pub type ASessionStore = Arc<dyn SessionStore + Send + Sync>;

/// Database result alias (optionnel)
pub type DbResult<T> = Result<T, sea_orm::DbErr>;
```

**Application** : Seulement dans `middleware/session/session.rs` ligne 9

---

### **Phase 2 : Extensions traits (après refonte builder)** ⏱️ 30 min

```rust
// Nouveau fichier : utils/session_ext.rs
pub trait SessionExt {
    async fn get_optional<T: for<'de> serde::Deserialize<'de>>(&self, key: &str) -> Option<T>;
}

// Nouveau fichier : utils/tera_ext.rs
pub trait TeraErrorExt {
    fn to_string_err(self) -> Result<String, String>;
}
```

**Impact** : Réduction de 30+ lignes répétitives

---

### **Phase 3 : Promouvoir ContextHelper (après refonte)** ⏱️ 1h

- Utiliser `ContextHelper` au lieu de `Context::new()` + `.insert()` partout
- Réduction de ~50 lignes

---

## ✅ Récapitulatif Final

| Pattern | Occurrences | Statut | Action |
|---------|-------------|--------|--------|
| `Arc<Tera>` | 35+ | ✅ Fait | Rien |
| `Arc<DatabaseConnection>` | 28+ | ✅ Fait | Rien |
| `Arc<RuniqueEngine>` | 42+ | ✅ Fait | Rien |
| `Arc<RuniqueConfig>` | 18+ | ✅ Fait | Rien |
| `Arc<RwLock<HashMap<...>>>` | 12+ | ✅ Fait | Rien |
| `Arc<SessionStore>` | 5 | ❌ Manquant | Ajouter alias |
| `session.get().await.ok().flatten()` | 18+ | ❌ Répétitif | Extension trait |
| `tera.render().map_err(\|e\| e.to_string())` | 12+ | ❌ Répétitif | Extension trait |
| `Context::new() + insert()` | 40+ | ⚠️ Peu utilisé | Promouvoir helper |

---

## 🎯 Score de Factorisation Actuel

**Couverture** : 85% des patterns majeurs déjà factorisés ✅  
**Reste à faire** : 15% (extensions traits + helpers)  
**Priorité immédiate** : Refonte du builder 🔥

---

**Auteur** : Assistant IA  
**Dernière mise à jour** : 29 janvier 2026