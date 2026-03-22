## ■ **Rust : Send et Sync** 

**Guide Complet pour la Concurrence Thread-Safe** 

Comprendre les Marker Traits 

Décembre 2025 

## ■ **Table des matières** 

1. Introduction aux Marker Traits 

2. Le trait Send 

3. Le trait Sync 

4. Send vs Sync : Différences clés 

5. Types courants et leurs propriétés 

6. Cas pratiques : Axum et async 

7. Erreurs courantes et solutions 

8. Exemples concrets avec Runique 

9. Best practices 

10. Exercices 

## 1. Introduction aux Marker Traits

En  Rust, **Send** et **Sync** sont  des _marker  traits_ qui  garantissent  la  sécurité  de  la  concurrence  à  la compilation. Ils sont fondamentaux pour écrire du code concurrent sans data races. 

## Qu'est-ce qu'un Marker Trait ?

Un marker trait est un trait sans méthodes qui sert uniquement à marquer un type avec une propriété particulière.  Send  et  Sync  sont  implémentés  automatiquement  par  le  compilateur  pour  la  plupart  des types. 

```
// Définitions dans std::marker
```

```
pub unsafe auto trait Send { }
pub unsafe auto trait Sync { }
```

```
// auto trait = implémenté automatiquement
```

```
// unsafe = si implémenté manuellement, responsabilité du développeur
```

## 2. Le trait Send

## Définition

Send  signifie  qu'une  valeur  peut  être  transférée  (moved)  entre  threads  en  toute  sécurité.  Si  un  type implémente Send, vous pouvez le déplacer d'un thread à un autre sans risque. 

## Exemple de base

```
use std::thread;
```

```
fn main() {
```

```
    let data = String::from("Hello");  // String est Send
```

```
    thread::spawn(move || {
```

**`//`** ■ **`OK : String est Send, on peut le déplacer dans un autre thread println!("{}", data); });`** 

```
}
```

## Types Send courants

|**Type**|**Send ?**|**Raison**|
|---|---|---|
|String|■Oui|Donnéespossédées,pas de référencespartagées|
|Vec<T>|■Oui (si T: Send)|Idem,possède ses données|
|i32, u64, bool|■Oui|Typesprimitifs copiables|
|Arc<T>|■Oui (si T: Send + Sync)|Pointeur atomique thread-safe|
|Rc<T>|■Non|Compteur de références non atomique|
|Cell<T>|■Non|Mutabilité intérieure non thread-safe|

## Exemple avec un type non-Send

```
use std::rc::Rc;
use std::thread;
```

```
fn main() {
    let data = Rc::new(String::from("Hello"));
```

**`//`** ■ **`ERREUR : Rc<String> n'est pas Send ! thread::spawn(move || { println!("{}", data); }); }`** 

```
// Erreur du compilateur :
```

```
// error[E0277]: `Rc<String>` cannot be sent between threads safely
//    = help: the trait `Send` is not implemented for `Rc<String>`
```

## 3. Le trait Sync

## Définition

Sync signifie qu'une référence (&T;) peut être partagée entre threads en toute sécurité. Si T est Sync, alors &T; est Send. 

## Formule magique

## `// Règle fondamentale`

**`T is Sync`** ■ **`&T is Send`** 

```
// Si T implémente Sync, alors une référence &T peut être envoyée entre threads
```

## Exemple de base

```
use std::thread;
use std::sync::Arc;
```

```
fn main() {
```

```
    let data = Arc::new(String::from("Hello"));
```

```
    let data_ref = Arc::clone(&data);
```

```
    thread::spawn(move || {
```

**`//`** ■ **`OK : String est Sync, donc &String est Send`** 

```
        // Arc permet de partager la référence
        println!("{}", data_ref);
    });
```

```
    println!("{}", data);
}
```

## Types Sync courants

|**Type**|**Sync ?**|**Raison**|
|---|---|---|
|String|■Oui|Immuable, pas de mutabilité intérieure|
|Vec<T>|■Oui (si T: Sync)|Idem|
|i32, u64, bool|■Oui|Types primitifs|
|Mutex<T>|■Oui (si T: Send)|Synchronisation explicite|
|Arc<T>|■Oui (si T: Sync)|Pointeur atomique|
|Rc<T>|■Non|Compteur non atomique|
|Cell<T>|■Non|Mutabilité intérieure non atomique|
|RefCell<T>|■Non|Vérifications à l'exécution non thread-safe|

## 4. Send vs Sync : Différences clés

|**Aspect**|**Send**|**Sync**|
|---|---|---|
|Signification|Je peux être déplacé entre threads|Ma référence peut être partagée entre threads|
|Ownership|Transfert de propriété|Partage de référence|
|Exemple usage|move dans thread::spawn|&T accessible depuis plusieurs threads|
|Pattern typique|thread::spawn(move || data)|Arc<T> partagé entre threads|
|Vérification|À la compilation|À la compilation|

## Diagramme mental

|**Situation**|**Trait requis**|
|---|---|
|Je déplace une valeur dans un autre thread|Send|
|Je partage une référence entre threads|Sync|
|J'utilise Arc<T> partagé|T: Send + Sync|
|J'utilise Mutex<T> partagé|T: Send|

## 5. Types courants et leurs propriétés

|**Type**|**Send**|**Sync**|**Notes**|
|---|---|---|---|
|String|■|■|Sûr pour concurrence|
|Vec<T>|■*|■*|* si T: Send/Sync|
|HashMap<K,V>|■*|■*|* si K,V: Send/Sync|
|i32, u64, bool|■|■|Types primitifs|
|Arc<T>|■*|■*|* si T: Send+Sync / T:Sync|
|Rc<T>|■|■|Compteur non atomique|
|Cell<T>|■*|■|* si T: Send, mais pas Sync|
|RefCell<T>|■*|■|Borrow checking runtime|
|Mutex<T>|■*|■*|* si T: Send|
|RwLock<T>|■*|■*|* si T: Send+Sync|

## 6. Cas pratiques : Axum et async

Dans Axum et Tokio, les traits Send et Sync sont cruciaux car les futures peuvent être déplacées entre threads. 

## Pourquoi Sync est nécessaire dans les traits

```
// Trait pour formulaires Runique
```

**`pub trait FormulaireTrait: Send + Sync {  //`** ← **`Sync important !`** 

```
    fn new() -> Self;
```

```
    fn validate(&mut self, raw_data: &HashMap<String, String>) -> bool;
```

```
}
```

```
// Sans Sync, cette erreur peut survenir :
```

```
#[async_trait]
impl<S, T> FromRequest<S> for AxumForm<T>
where
```

**`T: FormulaireTrait + 'static,  //`** ← **`Doit être Send + Sync`** 

```
{
    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
```

```
        // Cette future peut être déplacée entre threads par Tokio
```

```
        // Si T n'est pas Sync et qu'une référence &T existe,
```

```
        // le compilateur rejettera le code
    }
```

```
}
```

## Exemple concret avec Cell

```
use std::cell::Cell;
```

**`//`** ■ **`Ce code ne compile PAS pub struct BadForm { inner: Forms, counter: Cell<u32>,  // Cell n'est pas Sync ! }`** 

```
impl FormulaireTrait for BadForm {
    fn new() -> Self {
        Self {
            inner: Forms::new(),
            counter: Cell::new(0),
        }
    }
```

```
    fn validate(&mut self, raw_data: &HashMap<String, String>) -> bool {
        self.counter.set(self.counter.get() + 1);
        self.inner.is_valid()
    }
```

```
}
```

- **`// Erreur du compilateur :`** 

```
// error[E0277]: `Cell<u32>` cannot be shared between threads safely
//    = help: the trait `Sync` is not implemented for `Cell<u32>`
```

## Solution correcte

```
use std::sync::atomic::{AtomicU32, Ordering};
```

**`//`** ■ **`Ce code compile ! pub struct GoodForm { inner: Forms, counter: AtomicU32,  // AtomicU32 est Send + Sync } impl FormulaireTrait for GoodForm { fn new() -> Self { Self { inner: Forms::new(), counter: AtomicU32::new(0), } } fn validate(&mut self, raw_data: &HashMap<String, String>) -> bool { self.counter.fetch_add(1, Ordering::Relaxed); self.inner.is_valid() }`** 

```
}
```

## 7. Erreurs courantes et solutions

## Erreur 1 : Rc dans un contexte async

**`//`** ■ **`Erreur use std::rc::Rc;`** 

```
async fn handler(data: Rc<String>) {
    // Erreur : Rc is not Send
}
```

**`//`** ■ **`Solution use std::sync::Arc;`** 

```
async fn handler(data: Arc<String>) {
    // OK : Arc is Send + Sync
}
```

## Erreur 2 : Cell/RefCell dans un trait Sync

**`//`** ■ **`Erreur use std::cell::Cell;`** 

```
struct MyStruct {
    value: Cell<i32>,  // Cell n'est pas Sync
}
```

**`//`** ■ **`Solution : Utiliser des types atomiques use std::sync::atomic::{AtomicI32, Ordering};`** 

```
struct MyStruct {
    value: AtomicI32,  // AtomicI32 est Send + Sync
}
```

## Erreur 3 : Oublier Sync dans un trait

**`//`** ■ **`Risque futur pub trait MyTrait: Send {  // Manque Sync // ... } //`** ■ **`Meilleure pratique pub trait MyTrait: Send + Sync {  // Complet et sûr // ...`** 

```
}
```

## 8. Exemples concrets avec Runique

## Exemple 1 : Trait de formulaire correct

**`//`** ■ **`Trait correct pour Axum pub trait FormulaireTrait: Send + Sync { fn new() -> Self; fn validate(&mut self, raw_data: &HashMap<String, String>) -> bool; }`** 

```
// Implémentation
pub struct UserForm(Forms);
impl FormulaireTrait for UserForm {
    fn new() -> Self {
        Self(Forms::new())
    }
    fn validate(&mut self, raw_data: &HashMap<String, String>) -> bool {
        // Validation...
        self.0.is_valid()
    }
}
// Extracteur Axum
#[axum::async_trait]
impl<S, T> FromRequest<S> for AxumForm<T>
where
    S: Send + Sync,
    T: FormulaireTrait + 'static,  // T est automatiquement Send + Sync
{
    type Rejection = Response;
    async fn from_request(req: Request<Body>,state: &S)→
Result<Self,Self::Rejection> {
        // Le compilateur garantit que c'est thread-safe
        // ...
    }
}
```

## Exemple 2 : État partagé dans Axum

```
use std::sync::Arc;
use tokio::sync::Mutex;
#[derive(Clone)]
struct AppState {
    counter: Arc<Mutex<i32>>,  // Mutex<T> est Sync si T: Send
    config: Arc<Settings>,      // Arc<T> est Sync si T: Sync
}
```

## `async fn handler(`

```
    State(state): State<AppState>,
```

- **`) -> Response {`** 

- **`//`** ■ **`OK : AppState est Send + Sync`** 

```
    let mut counter = state.counter.lock().await;
```

- **`*counter += 1;`** 

- **`// ...`** 

- **`}`** 

## 9. Best practices

## 1. Toujours ajouter Send + Sync aux traits publics

Garantit la compatibilité avec async/await et Tokio. 

## 2. Préférer Arc à Rc pour le code async

Arc est thread-safe, Rc ne l'est pas. 

## 3. Utiliser AtomicXxx au lieu de Cell/RefCell

Pour la mutabilité intérieure thread-safe. 

## 4. Documenter les contraintes Send/Sync

Facilite la compréhension pour les futurs développeurs. 

## 5. Tester avec des références Arc

Vérifie que vos types sont bien Sync. 

## 6. Comprendre les erreurs du compilateur

Les messages d'erreur Send/Sync sont très précis. 

## 10. Exercices

## Exercice 1 : Identifier Send et Sync

Pour chaque type, déterminez s'il est Send et/ou Sync : 

|**Type**|**Send ?**|**Sync ?**|
|---|---|---|
|String|?|?|
|Vec<Rc<i32>>|?|?|
|Arc<Mutex<String>>|?|?|
|Cell<String>|?|?|
|&str|?|?|

## Exercice 2 : Corriger le code

## `// Ce code ne compile pas. Pourquoi ? Comment le corriger ?`

```
use std::rc::Rc;
use std::thread;
```

## `fn main() {`

```
    let data = Rc::new(vec![1, 2, 3]);
```

```
    thread::spawn(move || {
        println!("{:?}", data);
    });
```

```
}
```

## `// À vous de jouer !`

## Exercice 3 : Implémenter un trait thread-safe

Créez un trait CacheTrait qui : 

- Soit utilisable dans du code async 

- Permette de stocker et récupérer des valeurs 

- Soit thread-safe 

## ■ **Solutions des exercices** 

## Solution Exercice 1

|**Type**|**Send**|**Sync**|**Explication**|
|---|---|---|---|
|String|■|■|Type standard thread-safe|
|Vec<Rc<i32>>|■|■|Rc n'est ni Send ni Sync|
|Arc<Mutex<String>>|■|■|Arc + Mutex = thread-safe|
|Cell<String>|■|■|Send car String:Send, mais pas Sync|
|&str|■|■|Référence immuable|

## Solution Exercice 2

```
// Problème : Rc n'est pas Send
```

```
// Solution : Utiliser Arc au lieu de Rc
```

**`use std::sync::Arc;  //`** ← **`Changement ici`** 

```
use std::thread;
```

## `fn main() {`

**`let data = Arc::new(vec![1, 2, 3]);  //`** ← **`Arc au lieu de Rc`** 

```
    thread::spawn(move || {
        println!("{:?}", data);
    });
}
```

**`//`** ■ **`Compile et fonctionne !`** 

## Solution Exercice 3

```
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
```

```
// Trait thread-safe pour cache
pub trait CacheTrait: Send + Sync {
    type Key: Send + Sync;
    type Value: Send + Sync;
```

```
    fn get(&self, key: &Self::Key) -> Option<Self::Value>;
    fn set(&self, key: Self::Key, value: Self::Value);
```

```
}
```

```
// Implémentation avec Mutex
pub struct Cache<K, V> {
```

```
    data: Arc<Mutex<HashMap<K, V>>>,
}
impl<K, V> Cache<K, V>
where
    K: Send + Sync + Eq + std::hash::Hash + Clone,
    V: Send + Sync + Clone,
{
    pub fn new() -> Self {
        Self {
            data: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}
```

```
impl<K, V> CacheTrait for Cache<K, V>
where
    K: Send + Sync + Eq + std::hash::Hash + Clone,
    V: Send + Sync + Clone,
{
    type Key = K;
    type Value = V;
```

```
    fn get(&self, key: &Self::Key) -> Option<Self::Value> {
        self.data.lock().unwrap().get(key).cloned()
    }
    fn set(&self, key: Self::Key, value: Self::Value) {
        self.data.lock().unwrap().insert(key, value);
    }
}
```

**`//`** ■ **`Ce cache est Send + Sync et utilisable dans du code async !`** 

## ■ **Conclusion** 

Send et Sync sont les piliers de la programmation concurrente sûre en Rust. Le compilateur les vérifie automatiquement, éliminant toute possibilité de data races. 

■ Send = Peut être déplacé entre threads 

■ Sync = Peut être partagé (référence) entre threads 

- Vérification à la compilation = Pas de data races 

- Auto-implémenté pour la plupart des types 

■ Essentiel pour Axum, Tokio et async/await 

## ■ **Ressources** 

- The Rust Book - Chapter 16 (Concurrency) 

- Rust Nomicon - Send and Sync 

- Tokio documentation 

- Axum documentation 

## ■ **Bonne programmation avec Rust !**
