# Rust : Send et Sync

**Guide complet pour la concurrence thread-safe** — comprendre les marker traits.

## Table des matières

1. Introduction aux marker traits
2. Le trait `Send`
3. Le trait `Sync`
4. `Send` vs `Sync` : différences clés
5. Types courants et leurs propriétés
6. Cas pratiques : Axum et async
7. Erreurs courantes et solutions
8. Exemple illustratif de trait Rust généralisé
9. Bonnes pratiques
10. Exercices

## 1. Introduction aux marker traits

En Rust, `Send` et `Sync` sont des *marker traits* qui garantissent la sécurité de la concurrence
à la compilation. Ils sont fondamentaux pour écrire du code concurrent sans data races.

### Qu'est-ce qu'un marker trait ?

Un marker trait est un trait sans méthode qui sert uniquement à marquer un type avec une
propriété particulière. `Send` et `Sync` sont implémentés automatiquement par le compilateur
pour la plupart des types.

```rust
// Définitions dans std::marker
pub unsafe auto trait Send {}
pub unsafe auto trait Sync {}

// auto trait = implémenté automatiquement par le compilateur pour tout type
// composé uniquement de champs Send/Sync
// unsafe = si implémenté manuellement, la responsabilité de la sécurité
// retombe sur le développeur
```

## 2. Le trait `Send`

### Définition

`Send` signifie qu'une valeur peut être transférée (*moved*) entre threads en toute sécurité. Si
un type implémente `Send`, on peut le déplacer d'un thread à un autre sans risque.

### Exemple de base

```rust
use std::thread;

fn main() {
    let data = String::from("Hello");

    thread::spawn(move || {
        // OK : String est Send, on peut le déplacer dans un autre thread
        println!("{}", data);
    });
}
```

### Types `Send` courants

| Type | `Send` ? | Raison |
|---|---|---|
| `String` | ✅ Oui | Données possédées, pas de références partagées |
| `Vec<T>` | ✅ Oui (si `T: Send`) | Idem, possède ses données |
| `i32`, `u64`, `bool` | ✅ Oui | Types primitifs copiables |
| `Arc<T>` | ✅ Oui (si `T: Send + Sync`) | Pointeur atomique thread-safe |
| `Rc<T>` | ❌ Non | Compteur de références non atomique |
| `Cell<T>` | ✅ Oui (si `T: Send`) | Possède sa donnée, mais pas thread-safe en partage (voir `Sync`) |

### Exemple avec un type non-`Send`

```rust
use std::rc::Rc;
use std::thread;

fn main() {
    let data = Rc::new(String::from("Hello"));

    // ERREUR : Rc<String> n'est pas Send !
    thread::spawn(move || {
        println!("{}", data);
    });
}
```

```text
error[E0277]: `Rc<String>` cannot be sent between threads safely
   = help: the trait `Send` is not implemented for `Rc<String>`
```

## 3. Le trait `Sync`

### Définition

`Sync` signifie qu'une référence (`&T`) peut être partagée entre threads en toute sécurité. Si
`T` est `Sync`, alors `&T` est `Send`.

### Règle fondamentale

```text
T is Sync  ⇒  &T is Send
```

Si `T` implémente `Sync`, une référence `&T` peut être envoyée entre threads.

### Exemple de base

```rust
use std::thread;
use std::sync::Arc;

fn main() {
    let data = Arc::new(String::from("Hello"));
    let data_ref = Arc::clone(&data);

    thread::spawn(move || {
        // OK : String est Sync, donc &String est Send.
        // Arc permet de partager la référence entre threads.
        println!("{}", data_ref);
    });

    println!("{}", data);
}
```

### Types `Sync` courants

| Type | `Sync` ? | Raison |
|---|---|---|
| `String` | ✅ Oui | Immuable une fois construite, pas de mutabilité intérieure |
| `Vec<T>` | ✅ Oui (si `T: Sync`) | Idem |
| `i32`, `u64`, `bool` | ✅ Oui | Types primitifs |
| `Mutex<T>` | ✅ Oui (si `T: Send`) | Synchronisation explicite intégrée |
| `Arc<T>` | ✅ Oui (si `T: Sync`) | Pointeur atomique |
| `Rc<T>` | ❌ Non | Compteur de références non atomique |
| `Cell<T>` | ❌ Non | Mutabilité intérieure non thread-safe |
| `RefCell<T>` | ❌ Non | Vérifications d'emprunt à l'exécution, non thread-safe |

## 4. `Send` vs `Sync` : différences clés

| Aspect | `Send` | `Sync` |
|---|---|---|
| Signification | La valeur peut être déplacée entre threads | Une référence vers la valeur peut être partagée entre threads |
| Ownership | Transfert de propriété | Partage de référence |
| Exemple d'usage | `move` dans `thread::spawn` | `&T` accessible depuis plusieurs threads |
| Pattern typique | `thread::spawn(move \|\| data)` | `Arc<T>` partagé entre threads |
| Vérification | À la compilation | À la compilation |

### Diagramme mental

| Situation | Trait requis |
|---|---|
| Je déplace une valeur dans un autre thread | `Send` |
| Je partage une référence entre threads | `Sync` |
| J'utilise un `Arc<T>` partagé | `T: Send + Sync` |
| J'utilise un `Mutex<T>` partagé | `T: Send` |

### Types courants et leurs propriétés (récapitulatif)

| Type | `Send` | `Sync` | Notes |
|---|---|---|---|
| `String` | ✅ | ✅ | Sûr pour la concurrence |
| `Vec<T>` | ✅* | ✅* | *si `T: Send`/`Sync` |
| `HashMap<K,V>` | ✅* | ✅* | *si `K`,`V`: `Send`/`Sync` |
| `i32`, `u64`, `bool` | ✅ | ✅ | Types primitifs |
| `Arc<T>` | ✅* | ✅* | *si `T: Send + Sync` |
| `Rc<T>` | ❌ | ❌ | Compteur non atomique |
| `Cell<T>` | ✅* | ❌ | *si `T: Send` — jamais `Sync` |
| `RefCell<T>` | ✅* | ❌ | *si `T: Send` — vérif. d'emprunt runtime non thread-safe |
| `Mutex<T>` | ✅* | ✅* | *si `T: Send` (pas besoin de `T: Sync`) |
| `RwLock<T>` | ✅* | ✅* | *si `T: Send + Sync` |

## 5. Cas pratiques : Axum et async

Dans Axum et Tokio, les traits `Send` et `Sync` sont cruciaux car les futures peuvent être
déplacées entre threads par l'exécuteur (le runtime multi-thread de Tokio peut migrer une tâche
d'un thread worker à un autre entre deux points d'`.await`).

### Pourquoi `Sync` est nécessaire sur un trait de formulaire

```rust
// Trait générique illustratif pour un formulaire
pub trait FormulaireTrait: Send + Sync {   // ← Sync important !
    fn new() -> Self;
    fn validate(&mut self, raw_data: &HashMap<String, String>) -> bool;
}

// Sans Sync, une erreur de ce type peut survenir dans un extracteur Axum :
#[async_trait]
impl<S, T> FromRequest<S> for AxumForm<T>
where
    T: FormulaireTrait + 'static,   // ← doit être Send + Sync
{
    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        // Cette future peut être déplacée entre threads par Tokio.
        // Si T n'est pas Sync et qu'une référence &T existe à travers un .await,
        // le compilateur rejette le code.
    }
}
```

### Exemple concret : un type qui casse la compilation

```rust
use std::cell::Cell;

// Ce code ne compile PAS :
pub struct BadForm {
    inner: Forms,
    counter: Cell<u32>,   // Cell n'est pas Sync !
}

impl FormulaireTrait for BadForm {
    fn new() -> Self {
        Self { inner: Forms::new(), counter: Cell::new(0) }
    }
    fn validate(&mut self, raw_data: &HashMap<String, String>) -> bool {
        self.counter.set(self.counter.get() + 1);
        self.inner.is_valid()
    }
}
```

```text
error[E0277]: `Cell<u32>` cannot be shared between threads safely
   = help: the trait `Sync` is not implemented for `Cell<u32>`
```

### Solution correcte

```rust
use std::sync::atomic::{AtomicU32, Ordering};

// Ce code compile :
pub struct GoodForm {
    inner: Forms,
    counter: AtomicU32,   // AtomicU32 est Send + Sync
}

impl FormulaireTrait for GoodForm {
    fn new() -> Self {
        Self { inner: Forms::new(), counter: AtomicU32::new(0) }
    }
    fn validate(&mut self, raw_data: &HashMap<String, String>) -> bool {
        self.counter.fetch_add(1, Ordering::Relaxed);
        self.inner.is_valid()
    }
}
```

## 6. Erreurs courantes et solutions

### Erreur 1 — `Rc` dans un contexte async

```rust
// Erreur
use std::rc::Rc;

async fn handler(data: Rc<String>) {
    // Erreur : Rc n'est pas Send
}
```

```rust
// Solution
use std::sync::Arc;

async fn handler(data: Arc<String>) {
    // OK : Arc est Send + Sync
}
```

### Erreur 2 — `Cell`/`RefCell` dans un type censé être `Sync`

```rust
// Erreur
use std::cell::Cell;

struct MyStruct {
    value: Cell<i32>,   // Cell n'est pas Sync
}
```

```rust
// Solution : utiliser un type atomique
use std::sync::atomic::{AtomicI32, Ordering};

struct MyStruct {
    value: AtomicI32,   // AtomicI32 est Send + Sync
}
```

### Erreur 3 — oublier `Sync` sur un trait public

```rust
// Risque futur : Send seul suffit aujourd'hui mais bloquera dès qu'un usage
// exigera Sync
pub trait MyTrait: Send {
    // ...
}

// Meilleure pratique : poser les deux bounds dès le départ si le trait
// est destiné à un contexte partagé/async
pub trait MyTrait: Send + Sync {
    // ...
}
```

## 7. Exemple illustratif — état partagé dans Axum

```rust
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone)]
struct AppState {
    counter: Arc<Mutex<i32>>,   // Mutex<T> est Sync si T: Send
    config: Arc<Settings>,      // Arc<T> est Sync si T: Sync
}

async fn handler(State(state): State<AppState>) -> Response {
    // OK : AppState est Send + Sync
    let mut counter = state.counter.lock().await;
    *counter += 1;
    // ...
}
```

## 8. Bonnes pratiques

1. **Ajouter `Send + Sync` aux traits publics destinés à un contexte async/partagé** — garantit
   la compatibilité avec Tokio dès la conception, plutôt que de le découvrir à l'usage.
2. **Préférer `Arc` à `Rc` dans tout code touché par async** — `Arc` est thread-safe, `Rc` ne
   l'est jamais.
3. **Utiliser `AtomicXxx` plutôt que `Cell`/`RefCell`** dès qu'un partage entre threads est
   possible — mutabilité intérieure thread-safe.
4. **Documenter les contraintes `Send`/`Sync`** explicitement dans les commentaires de trait —
   facilite la compréhension pour les futurs développeurs.
5. **Lire attentivement les erreurs du compilateur** — les messages `Send`/`Sync` nomment
   précisément le type fautif et pourquoi.

## 9. Exercices

### Exercice 1 — identifier `Send` et `Sync`

Pour chaque type, déterminer s'il est `Send` et/ou `Sync` :

| Type | `Send` ? | `Sync` ? |
|---|---|---|
| `String` | ? | ? |
| `Vec<Rc<i32>>` | ? | ? |
| `Arc<Mutex<String>>` | ? | ? |
| `Cell<String>` | ? | ? |
| `&str` | ? | ? |

### Exercice 2 — corriger le code

```rust
// Ce code ne compile pas. Pourquoi ? Comment le corriger ?
use std::rc::Rc;
use std::thread;

fn main() {
    let data = Rc::new(vec![1, 2, 3]);
    thread::spawn(move || {
        println!("{:?}", data);
    });
}
```

### Exercice 3 — implémenter un trait thread-safe

Créer un trait `CacheTrait` qui :
- soit utilisable dans du code async ;
- permette de stocker et récupérer des valeurs ;
- soit thread-safe.

---

## Solutions des exercices

### Solution Exercice 1

| Type | `Send` | `Sync` | Explication |
|---|---|---|---|
| `String` | ✅ | ✅ | Type standard thread-safe |
| `Vec<Rc<i32>>` | ❌ | ❌ | `Rc` n'est ni `Send` ni `Sync`, donc le `Vec` qui le contient non plus |
| `Arc<Mutex<String>>` | ✅ | ✅ | `Arc` + `Mutex` = combinaison thread-safe standard |
| `Cell<String>` | ✅ | ❌ | `Send` car `String: Send`, mais jamais `Sync` (mutabilité intérieure) |
| `&str` | ✅ | ✅ | Référence immuable vers des données `Sync` |

### Solution Exercice 2

```rust
// Problème : Rc n'est pas Send, impossible de le déplacer dans un thread.
// Solution : utiliser Arc à la place.
use std::sync::Arc;   // ← changement ici
use std::thread;

fn main() {
    let data = Arc::new(vec![1, 2, 3]);   // ← Arc au lieu de Rc
    thread::spawn(move || {
        println!("{:?}", data);
    });
}
// Compile et fonctionne.
```

### Solution Exercice 3

```rust
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

// Trait thread-safe pour cache
pub trait CacheTrait: Send + Sync {
    type Key: Send + Sync;
    type Value: Send + Sync;

    fn get(&self, key: &Self::Key) -> Option<Self::Value>;
    fn set(&self, key: Self::Key, value: Self::Value);
}

// Implémentation avec Mutex
pub struct Cache<K, V> {
    data: Arc<Mutex<HashMap<K, V>>>,
}

impl<K, V> Cache<K, V>
where
    K: Send + Sync + Eq + std::hash::Hash + Clone,
    V: Send + Sync + Clone,
{
    pub fn new() -> Self {
        Self { data: Arc::new(Mutex::new(HashMap::new())) }
    }
}

impl<K, V> CacheTrait for Cache<K, V>
where
    K: Send + Sync + Eq + std::hash::Hash + Clone,
    V: Send + Sync + Clone,
{
    type Key = K;
    type Value = V;

    fn get(&self, key: &Self::Key) -> Option<Self::Value> {
        self.data.lock().unwrap().get(key).cloned()
    }
    fn set(&self, key: Self::Key, value: Self::Value) {
        self.data.lock().unwrap().insert(key, value);
    }
}
// Ce cache est Send + Sync et utilisable dans du code async.
```

## Conclusion

- `Send` = la valeur peut être déplacée entre threads.
- `Sync` = une référence vers la valeur peut être partagée entre threads.
- Vérification entièrement faite à la compilation : élimine les data races par construction.
- Auto-implémenté par le compilateur pour la quasi-totalité des types.
- Essentiel dès qu'on touche Axum, Tokio, ou async/await en général.

## Ressources

- *The Rust Book* — chapitre 16 (Concurrency)
- *Rust Nomicon* — Send and Sync
- Documentation Tokio
- Documentation Axum

> Pour voir `Send`/`Sync` appliqués à du vrai code Runique (pas un exemple générique), voir
> `~/Bureau/revision/18_send_sync.md` — extraits réels de `runique/src/forms/base.rs` et
> `field.rs`.
