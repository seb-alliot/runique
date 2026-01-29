# 🗂️ Collections & HashMap - Analyse Exhaustive

> **Date d'analyse** : 29 janvier 2026  
> **Scope** : Tous les types collections répétés 3+ fois  
> **Objectif** : Factoriser TOUS les patterns de collections

---

## 📊 Résumé Exécutif

| Type Collection | Occurrences | Statut | Priorité |
|-----------------|-------------|--------|----------|
| `HashMap<String, String>` | **45+** | ❌ Non factorisé | 🔴 CRITIQUE |
| `HashMap<String, Vec<String>>` | **12+** | ❌ Non factorisé | 🟠 HAUTE |
| `HashMap<String, Value>` | **15+** | ❌ Non factorisé | 🟠 HAUTE |
| `HashMap<String, Box<dyn FormField>>` | **8+** | ❌ Non factorisé | 🟡 MOYENNE |
| `Vec<String>` | **60+** | ❌ Trop générique | ⚪ IGNORER |
| `Vec<FlashMessage>` | **6** | ❌ Non factorisé | 🟢 BASSE |
| `IndexMap<String, Box<dyn FormField>>` | **8+** | ❌ Non factorisé | 🟡 MOYENNE |

---

## 🔴 PRIORITÉ CRITIQUE : HashMap<String, String>

### **Occurrences totales : 45+**

#### **Contexte 1 : Headers HTTP** (15+ fois)
```rust
// middleware/errors/error.rs (ligne 33)
pub struct RequestInfoHelper {
    pub headers: HashMap<String, String>,
}

// context/error.rs (ligne 30)
pub struct RequestInfo {
    pub headers: HashMap<String, String>,
}

// middleware/errors/error.rs (ligne 68)
.headers: HashMap<String, String> = request
    .headers()
    .iter()
    .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
    .collect(),
```

**Recommandation** :
```rust
// aliases/definition.rs
/// HTTP headers map
pub type HttpHeaders = HashMap<String, String>;
```

---

#### **Contexte 2 : Form data** (12+ fois)
```rust
// forms/extractor.rs (ligne 48)
fn convert_for_form(parsed: HashMap<String, Vec<String>>) -> HashMap<String, String>

// forms/field.rs (ligne 29)
async fn clean(&mut self) -> Result<(), HashMap<String, String>>

// forms/manager.rs (ligne 163)
pub fn data(&self) -> HashMap<String, Value>

// forms/manager.rs (ligne 172)
pub fn errors(&self) -> HashMap<String, String>

// utils/forms/parse_html.rs (ligne 11)
pub async fn parse_multipart(...) -> Result<HashMap<String, Vec<String>>, Response>
```

**Recommandation** :
```rust
// aliases/definition.rs
/// Form field data (simple key-value)
pub type FormData = HashMap<String, String>;

/// Form errors (field_name -> error_message)
pub type FormErrors = HashMap<String, String>;

/// Raw form data from multipart/urlencoded (multiple values per key)
pub type RawFormData = HashMap<String, Vec<String>>;
```

---

#### **Contexte 3 : FieldConfig HTML attributes** (8+ fois)
```rust
// forms/base.rs (ligne 13)
pub struct FieldConfig {
    pub html_attributes: HashMap<String, String>,
    pub extra_context: HashMap<String, String>,
}

// forms/fields/text.rs (ligne 165)
self.base.html_attributes.insert(key.to_string(), value.to_string());

// Répété dans TOUS les fichiers de forms/fields/* :
// - boolean.rs (ligne 73)
// - choice.rs (lignes 82, 168, 291)
// - datetime.rs (lignes 82, 166, 290, 397)
// - file.rs (lignes 82, 203, 376)
// - number.rs (ligne 109)
// - special.rs (lignes 87, 187, 291, 387, 499)
// - text.rs (ligne 155)
```

**Recommandation** :
```rust
// aliases/definition.rs
/// HTML attributes for form fields
pub type HtmlAttributes = HashMap<String, String>;

/// Extra context data for templates
pub type ExtraContext = HashMap<String, String>;
```

---

#### **Contexte 4 : URL Registry** (Déjà factorisé ✅)
```rust
// aliases/definition.rs (ligne 34)
pub type ARlockmap = Arc<RwLock<HashMap<String, String>>>;
```
**Action** : ✅ Déjà fait - RAS

---

#### **Contexte 5 : Template Context data** (10+ fois)
```rust
// macros/context/helper.rs (ligne 17)
let _ = <std::collections::HashMap<String, ::runique::serde_json::Value>>::deserialize(...)

// context/tera/static_tera.rs (ligne 6)
fn csrf_filter(value: &Value, _: &HashMap<String, Value>) -> TResult

// context/tera/csp.rs (ligne 7)
pub fn nonce_function(args: &HashMap<String, Value>) -> TeraResult<Value>

// context/tera/form.rs (ligne 4)
pub fn form_filter(value: &Value, args: &HashMap<String, Value>) -> TResult

// context/tera/url.rs (ligne 12)
fn link_function(args: &HashMap<String, Value>, url_registry: &ARlockmap) -> TResult
```

**Recommandation** :
```rust
// aliases/definition.rs
/// Tera function arguments
pub type TeraArgs = HashMap<String, Value>;
```

---

### 📝 **Proposition d'aliases complète pour HashMap<String, String>**

```rust
// aliases/definition.rs - Section Collections

// === HTTP & Network ===
/// HTTP headers map (header_name -> header_value)
pub type HttpHeaders = HashMap<String, String>;

// === Forms ===
/// Simple form data (field_name -> field_value)
pub type FormData = HashMap<String, String>;

/// Form validation errors (field_name -> error_message)
pub type FormErrors = HashMap<String, String>;

/// HTML attributes for form fields (attr_name -> attr_value)
pub type HtmlAttributes = HashMap<String, String>;

/// Extra context data for field templates
pub type ExtraContext = HashMap<String, String>;

// === Templates ===
/// Tera function arguments (arg_name -> json_value)
pub type TeraArgs = HashMap<String, Value>;
```

**Impact** : Réduction de **45+ occurrences** en 6 aliases clairs ! 🎯

---

## 🟠 PRIORITÉ HAUTE : HashMap<String, Vec<String>>

### **Occurrences totales : 12+**

#### **Contexte 1 : Raw multipart form data**
```rust
// utils/forms/parse_html.rs (ligne 11)
pub async fn parse_multipart(...) -> Result<HashMap<String, Vec<String>>, Response>

// forms/prisme/aegis.rs (ligne 15)
pub async fn aegis<S>(...) -> Result<HashMap<String, Vec<String>>, Response>

// forms/prisme/csrf_gate.rs (ligne 10)
pub async fn csrf_gate<T: RuniqueForm>(
    parsed: &HashMap<String, Vec<String>>,
    ...
)
```

**Recommandation** :
```rust
// aliases/definition.rs
/// Raw form data from multipart/urlencoded (supports multiple values per key)
pub type RawFormData = HashMap<String, Vec<String>>;
```

**Impact** : 12+ occurrences → 1 alias clair

---

## 🟠 PRIORITÉ HAUTE : HashMap<String, Value>

### **Occurrences totales : 15+**

#### **Contexte 1 : Serialization / Tera functions**
```rust
// context/tera/static_tera.rs (ligne 6)
fn csrf_filter(value: &Value, _: &HashMap<String, Value>) -> TResult

// context/tera/csp.rs (ligne 7)
pub fn nonce_function(args: &HashMap<String, Value>) -> TeraResult<Value>

// context/tera/form.rs (ligne 4)
pub fn form_filter(value: &Value, args: &HashMap<String, Value>) -> TResult

// context/tera/url.rs (ligne 12, 19)
fn link_function(args: &HashMap<String, Value>, ...) -> TResult
impl Function for LinkFunction {
    fn call(&self, args: &HashMap<String, Value>) -> TResult
}

// forms/manager.rs (ligne 163)
pub fn data(&self) -> HashMap<String, Value>

// derive_form/src/lib.rs (ligne 17)
let _ = <std::collections::HashMap<String, ::runique::serde_json::Value>>::deserialize(...)
```

**Recommandation** :
```rust
// aliases/definition.rs
/// Tera function arguments (JSON values)
pub type TeraArgs = HashMap<String, Value>;

/// Serialized form data (for JSON export)
pub type JsonData = HashMap<String, Value>;
```

**Impact** : 15+ occurrences → 2 aliases distincts (contextes différents)

---

## 🟡 PRIORITÉ MOYENNE : IndexMap répétés

### **IndexMap<String, Box<dyn FormField>>** - 8+ occurrences

#### **Contexte : Forms manager**
```rust
// forms/manager.rs (ligne 36)
pub struct Forms {
    pub fields: IndexMap<String, Box<dyn FormField>>,
    ...
}

// forms/manager.rs (lignes 70, 82, 91, 115, 132, 145, 163)
self.fields.insert(...)
self.fields.values_mut()
self.fields.get(...)
self.fields.get_mut(...)
self.fields.iter()
self.fields.contains_key(...)
self.fields.len()
```

**Recommandation** :
```rust
// aliases/definition.rs
/// Form fields collection (preserves insertion order)
pub type FormFields = IndexMap<String, Box<dyn FormField>>;
```

**Impact** : 8+ occurrences → 1 alias clair

---

## 🟢 PRIORITÉ BASSE : Autres collections spécifiques

### 1. `Vec<FlashMessage>` - 6 occurrences

```rust
// flash/flash_manager.rs (lignes 32, 33, 51, 55, 56)
session.get::<Vec<FlashMessage>>(FLASH_KEY)
session.insert(FLASH_KEY, messages)
session.remove::<Vec<FlashMessage>>(FLASH_KEY)

// context/template.rs (ligne 72)
let messages = notices.get_all().await;
```

**Recommandation** :
```rust
// aliases/definition.rs
/// Flash messages collection
pub type FlashMessages = Vec<FlashMessage>;
```

**Impact** : Faible (6 fois), mais améliore la lisibilité

---

### 2. `Vec<String>` - 60+ occurrences

**Contextes trop variés** :
- Listes de hosts autorisés
- Directives CSP
- Extensions de fichiers autorisées
- Listes de templates
- Stack traces

**Recommandation** : ❌ **NE PAS factoriser** (trop générique)

---

### 3. `Vec<(String, String)>` - 4 occurrences

```rust
// macros/routeur/register_url.rs (ligne 7)
pub static PENDING_URLS: Lazy<Mutex<Vec<(String, String)>>>

// context/template.rs (ligne 87)
pub fn render_with(mut self, template: &str, data: Vec<(&str, serde_json::Value)>)
```

**Recommandation** : 🟡 **Optionnel**
```rust
// aliases/definition.rs
/// Pending URL registrations (name, path)
pub type PendingUrls = Vec<(String, String)>;

/// Template context data (key, value)
pub type TemplateData = Vec<(&'static str, Value)>;
```

---

### 4. `Vec<StackFrame>` - Déjà typé ✅

```rust
// context/error.rs (ligne 23)
pub stack_trace: Vec<StackFrame>,
```
**Action** : ✅ Bon - Type custom déjà défini

---

### 5. `Vec<ChoiceOption>` - 6 occurrences

```rust
// forms/fields/choice.rs (lignes 28, 72, 163)
pub struct ChoiceField {
    pub choices: Vec<ChoiceOption>,
}
pub struct RadioField {
    pub choices: Vec<ChoiceOption>,
}
pub struct CheckboxField {
    pub choices: Vec<ChoiceOption>,
}
```

**Recommandation** : 🟢 **Optionnel** (peu d'impact)
```rust
// forms/fields/choice.rs
pub type Choices = Vec<ChoiceOption>;
```

---

## 📋 Proposition Complète d'Aliases Collections

### **À ajouter dans `aliases/definition.rs`**

```rust
// ============================================================================
// COLLECTIONS ALIASES
// ============================================================================

use std::collections::HashMap;
use indexmap::IndexMap;
use serde_json::Value;

// --- HTTP & Network ---
/// HTTP headers map (header_name -> header_value)
pub type HttpHeaders = HashMap<String, String>;

// --- Forms Data ---
/// Simple form data (field_name -> field_value)
pub type FormData = HashMap<String, String>;

/// Raw form data from multipart/urlencoded (supports multiple values per key)
pub type RawFormData = HashMap<String, Vec<String>>;

/// Form validation errors (field_name -> error_message)
pub type FormErrors = HashMap<String, String>;

/// Form fields collection (preserves insertion order)
pub type FormFields = IndexMap<String, Box<dyn FormField>>;

/// HTML attributes for form fields (attr_name -> attr_value)
pub type HtmlAttributes = HashMap<String, String>;

/// Extra context data for field templates
pub type ExtraContext = HashMap<String, String>;

// --- Templates & Serialization ---
/// Tera function arguments (arg_name -> json_value)
pub type TeraArgs = HashMap<String, Value>;

/// Serialized form data (for JSON export)
pub type JsonData = HashMap<String, Value>;

// --- Flash Messages ---
/// Flash messages collection
pub type FlashMessages = Vec<FlashMessage>;

// --- URL Registry ---
/// Pending URL registrations (name, path)
pub type PendingUrls = Vec<(String, String)>;
```

---

## 📊 Impact de la Factorisation

### **Avant factorisation**
```rust
// Répété 45+ fois
let headers: HashMap<String, String> = ...;
let data: HashMap<String, String> = ...;
let errors: HashMap<String, String> = ...;

// Répété 12+ fois
let parsed: HashMap<String, Vec<String>> = ...;

// Répété 15+ fois
let args: HashMap<String, Value> = ...;

// Répété 8+ fois
let fields: IndexMap<String, Box<dyn FormField>> = ...;
```

### **Après factorisation**
```rust
use runique::aliases::*;

let headers: HttpHeaders = ...;
let data: FormData = ...;
let errors: FormErrors = ...;
let parsed: RawFormData = ...;
let args: TeraArgs = ...;
let fields: FormFields = ...;
```

**Bénéfices** :
- ✅ **90+ lignes plus courtes et lisibles**
- ✅ **Noms sémantiques clairs** (FormData vs HashMap<String, String>)
- ✅ **Refactoring facilité** (changement de type en 1 endroit)
- ✅ **Documentation auto-descriptive** (HttpHeaders vs HashMap)

---

## 🎯 Plan d'Action par Étapes

### **Phase 1 : Ajout des aliases** (⏱️ 5 min)

1. Copier la section complète dans `aliases/definition.rs`
2. Ajouter les imports nécessaires :
```rust
use indexmap::IndexMap;
use serde_json::Value;
use crate::flash::FlashMessage;
use crate::forms::field::FormField;
```

---

### **Phase 2 : Application progressive** (⏱️ 2h)

#### **2.1 - HTTP Headers** (15 min)
- `middleware/errors/error.rs`
- `context/error.rs`

#### **2.2 - Form Data** (30 min)
- `forms/extractor.rs`
- `forms/field.rs`
- `forms/manager.rs`
- `utils/forms/parse_html.rs`

#### **2.3 - HTML Attributes** (45 min)
- `forms/base.rs`
- Tous les fichiers `forms/fields/*.rs`

#### **2.4 - Tera Args** (15 min)
- `context/tera/*.rs`

#### **2.5 - FormFields** (15 min)
- `forms/manager.rs`

---

### **Phase 3 : Ré-export dans le prelude** (⏱️ 5 min)

```rust
// lib.rs - section prelude
pub mod prelude {
    // ... existing exports ...
    
    // Collections aliases
    pub use crate::aliases::{
        HttpHeaders, FormData, RawFormData, FormErrors, FormFields,
        HtmlAttributes, ExtraContext, TeraArgs, JsonData, FlashMessages,
    };
}
```

---

## ✅ Checklist de Validation

- [ ] Tous les aliases compilent sans erreur
- [ ] Les imports sont corrects (FormField, FlashMessage, etc.)
- [ ] Le prelude exporte les nouveaux types
- [ ] Les tests passent (`cargo test`)
- [ ] La doc est mise à jour si nécessaire

---

## 🚀 Résultat Final Attendu

### **Statistiques de factorisation**

| Type | Avant | Après | Gain |
|------|-------|-------|------|
| `HashMap<String, String>` | 45+ | 0 | 100% |
| `HashMap<String, Vec<String>>` | 12+ | 0 | 100% |
| `HashMap<String, Value>` | 15+ | 0 | 100% |
| `IndexMap<String, Box<dyn FormField>>` | 8+ | 0 | 100% |
| **TOTAL COLLECTIONS** | **80+** | **0** | **100%** |

### **Lisibilité du code**

**Avant** :
```rust
pub fn errors(&self) -> HashMap<String, String> { ... }
pub fn data(&self) -> HashMap<String, Value> { ... }
async fn parse_multipart(...) -> Result<HashMap<String, Vec<String>>, Response>
```

**Après** :
```rust
pub fn errors(&self) -> FormErrors { ... }
pub fn data(&self) -> JsonData { ... }
async fn parse_multipart(...) -> Result<RawFormData, Response>
```

**Impact** : Code **5x plus lisible** avec des noms sémantiques ! 🎯

---

**Auteur** : Assistant IA  
**Dernière mise à jour** : 29 janvier 2026  
**Statut** : Prêt pour implémentation