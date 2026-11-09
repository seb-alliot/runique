# Send & Sync

> Les deux traits qui permettent au compilateur de garantir l'absence de course aux données : ce qu'ils signifient, quels types les ont, et comment corriger les erreurs « cannot be sent between threads safely ».

## Objectifs

- Comprendre ce que signifient `Send` et `Sync`, et le lien entre les deux
- Savoir quels types standard les implémentent et pourquoi
- Lire et corriger les erreurs `Send`/`Sync` avec `thread::spawn` et `tokio::spawn`
- Éviter de garder une valeur non `Send` à travers un `.await`
- Déclarer correctement des trait objects et des états partagés pour un serveur
- Connaître les cas où l'on implémente `Send` ou `Sync` à la main, et le danger

## Table des matières

1. Le problème : la course aux données
2. `Send`
3. `Sync`
4. Des traits automatiques
5. Quels types sont `Send` et `Sync`
6. `Send` et `Sync` dans les signatures
7. Threads : corriger les erreurs
8. Async : valeurs gardées à travers un `.await`
9. Trait objects et état partagé d'un serveur
10. Partager une valeur entre threads : toutes les façons
11. Implémenter `Send` ou `Sync` à la main
12. Exercices
13. Aide-mémoire

## 1. Le problème : la course aux données

Une **course aux données** (data race) se produit quand deux threads accèdent à la même mémoire en même temps, qu'au moins l'un écrit, et que rien ne synchronise les accès. Le résultat est imprévisible : valeurs corrompues, plantages aléatoires, failles.

```rust
// En C ou en Java, ceci compile et donne un total faux de façon aléatoire :
// let mut total = 0;
// for _ in 0..10 { spawn(|| total += 1); }
```

En Rust, ce code **ne compile pas**. Le compilateur s'appuie sur deux traits marqueurs, `Send` et `Sync`, pour savoir ce qui peut traverser une frontière entre threads.

## 2. `Send`

> Un type est `Send` si une valeur de ce type peut être **déplacée** vers un autre thread.

```rust
use std::thread;

let v = vec![1, 2, 3];              // Vec<i32> est Send
let h = thread::spawn(move || {
    println!("{v:?}");              // v a été déplacé dans le nouveau thread
});
h.join().unwrap();
```

```rust
use std::rc::Rc;

let r = Rc::new(5);
// thread::spawn(move || println!("{r}"));
// ERREUR E0277 : `Rc<i32>` cannot be sent between threads safely
```

`Rc` n'est pas `Send` : son compteur de références n'est pas protégé. Si deux threads clonaient ou libéraient chacun leur copie du même `Rc` en même temps, le compteur serait faux et la valeur libérée deux fois, ou jamais.

## 3. `Sync`

> Un type `T` est `Sync` si une **référence** `&T` peut être partagée entre plusieurs threads.

Autrement dit : `T: Sync` ⟺ `&T: Send`.

```rust
use std::sync::Arc;
use std::thread;

let compteur = Arc::new(std::sync::atomic::AtomicU32::new(0));   // AtomicU32 est Sync
for _ in 0..4 {
    let c = Arc::clone(&compteur);
    thread::spawn(move || {
        c.fetch_add(1, std::sync::atomic::Ordering::Relaxed);    // plusieurs threads, même valeur
    });
}
```

```rust
use std::cell::RefCell;

let r = RefCell::new(0);
// partager &r entre threads est refusé : RefCell n'est pas Sync
```

`RefCell` compte ses emprunts sans synchronisation : deux threads pourraient obtenir chacun un `borrow_mut()` en même temps.

## 4. Des traits automatiques

On n'implémente presque jamais `Send` ou `Sync` soi-même. Ce sont des **traits automatiques** : un type les a si **tous ses champs** les ont.

```rust
struct Utilisateur {
    nom: String,      // Send + Sync
    age: u32,         // Send + Sync
}
// Utilisateur est automatiquement Send + Sync

struct Session {
    donnees: std::rc::Rc<String>,   // ni Send ni Sync
}
// Session n'est ni Send ni Sync
```

Un seul champ non `Send` suffit à retirer `Send` à toute la struct, et à toute struct qui la contient.

## 5. Quels types sont `Send` et `Sync`

| Type | `Send` | `Sync` | Pourquoi |
|---|---|---|---|
| `i32`, `bool`, `String`, `Vec<T>`, `Box<T>` | oui | oui | (si `T` l'est) données possédées, sans état caché |
| `&T` | si `T: Sync` | si `T: Sync` | |
| `&mut T` | si `T: Send` | si `T: Sync` | |
| `Rc<T>` | **non** | **non** | compteur non atomique |
| `Arc<T>` | si `T: Send + Sync` | si `T: Send + Sync` | compteur atomique |
| `Cell<T>`, `RefCell<T>` | si `T: Send` | **non** | modification sans synchronisation |
| `Mutex<T>` | si `T: Send` | si `T: Send` | le verrou sérialise les accès |
| `RwLock<T>` | si `T: Send` | si `T: Send + Sync` | plusieurs lecteurs simultanés |
| `MutexGuard<'_, T>` (std) | **non** | si `T: Sync` | doit être relâché par le thread qui l'a pris |
| `AtomicU32`, `AtomicBool`... | oui | oui | opérations atomiques |
| `*const T`, `*mut T` | **non** | **non** | le compilateur ne sait rien de ce qu'ils pointent |
| `tokio::sync::Mutex<T>` et son garde | oui (si `T: Send`) | oui | conçu pour l'async |

Deux combinaisons à retenir :

- `Arc<Mutex<T>>` : partager **et** modifier entre threads. `Mutex` rend `T` `Sync`, `Arc` le partage.
- `Arc<RefCell<T>>` : refusé par `spawn`, car `RefCell` n'est pas `Sync`.

## 6. `Send` et `Sync` dans les signatures

### 6.1 `thread::spawn`

```rust
pub fn spawn<F, T>(f: F) -> JoinHandle<T>
where
    F: FnOnce() -> T + Send + 'static,
    T: Send + 'static,
```

- la closure est **envoyée** au nouveau thread : `Send` ;
- elle peut lui survivre : `'static` (aucun emprunt temporaire) ;
- le résultat revient au thread qui fait `join` : `T: Send`.

### 6.2 `tokio::spawn`

```rust
pub fn spawn<F>(future: F) -> JoinHandle<F::Output>
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
```

Le runtime multithread de tokio peut déplacer une tâche d'un thread à l'autre à chaque `.await` : tout ce que le futur garde entre deux `.await` doit être `Send`.

### 6.3 Exiger `Send` dans tes propres fonctions

```rust
fn en_arriere_plan<T: Send + 'static>(valeur: T, traitement: fn(T)) {
    std::thread::spawn(move || traitement(valeur));
}
```

## 7. Threads : corriger les erreurs

### 7.1 « cannot be sent between threads safely »

```rust
use std::rc::Rc;
use std::sync::Arc;

// Refusé : Rc
// let r = Rc::new(vec![1]);
// std::thread::spawn(move || println!("{r:?}"));

// Correction 1 : Arc à la place de Rc
let a = Arc::new(vec![1]);
let a2 = Arc::clone(&a);
std::thread::spawn(move || println!("{a2:?}"));

// Correction 2 : déplacer une valeur possédée
let v = vec![1];
std::thread::spawn(move || println!("{v:?}"));
```

### 7.2 « cannot be shared between threads safely »

```rust
use std::sync::{Arc, Mutex};

// Refusé : Arc<RefCell<i32>> (RefCell n'est pas Sync)
// let c = Arc::new(std::cell::RefCell::new(0));

let c = Arc::new(Mutex::new(0));                 // Mutex à la place de RefCell
let mut handles = vec![];
for _ in 0..4 {
    let c = Arc::clone(&c);
    handles.push(std::thread::spawn(move || {
        *c.lock().unwrap() += 1;
    }));
}
for h in handles {
    h.join().unwrap();
}
assert_eq!(*c.lock().unwrap(), 4);
```

### 7.3 « borrowed value does not live long enough » / `'static`

```rust
let donnees = vec![1, 2, 3];

// Refusé : la closure emprunte donnees, qui pourrait disparaître avant le thread
// std::thread::spawn(|| println!("{donnees:?}"));

// 1. move : le thread possède la valeur
// 2. Arc : chaque thread possède un pointeur partagé
// 3. thread::scope : les threads ne peuvent pas survivre au bloc, l'emprunt devient permis
std::thread::scope(|s| {
    s.spawn(|| println!("{donnees:?}"));
    s.spawn(|| println!("{}", donnees.len()));
});
println!("{donnees:?}");          // toujours disponible
```

`thread::scope` attend la fin de tous ses threads avant de rendre la main : ils peuvent donc emprunter les variables locales, sans `Arc` ni `'static`.

### 7.4 Lire le message d'erreur

```text
error[E0277]: `Rc<String>` cannot be sent between threads safely
   |
   = help: within `Session`, the trait `Send` is not implemented for `Rc<String>`
note: required because it appears within the type `Session`
note: required by a bound in `spawn`
```

Le compilateur remonte la chaîne : il nomme le **champ** fautif (`Rc<String>`), la struct qui le contient (`Session`) et la fonction qui exige `Send` (`spawn`). On corrige le champ, pas l'appel.

## 8. Async : valeurs gardées à travers un `.await`

### 8.1 Le verrou std gardé pendant un `.await`

```rust
use std::sync::{Arc, Mutex};

async fn mettre_a_jour(etat: Arc<Mutex<Vec<String>>>) {
    let mut v = etat.lock().unwrap();        // MutexGuard (std) : pas Send
    v.push("a".into());
    // envoyer_notification().await;         // ERREUR avec tokio::spawn : future is not `Send`
}
```

Le garde est gardé **à travers** l'`.await` : il ferait partie de l'état du futur, et le futur ne serait plus `Send`. De plus, garder un verrou bloquant pendant une attente bloque les autres tâches.

### 8.2 Trois corrections

```rust
use std::sync::{Arc, Mutex};

async fn envoyer_notification() {}

// 1. Relâcher le verrou avant l'.await, dans un bloc
async fn version1(etat: Arc<Mutex<Vec<String>>>) {
    {
        let mut v = etat.lock().unwrap();
        v.push("a".into());
    }                                         // garde libéré ici
    envoyer_notification().await;
}

// 2. Copier ce dont on a besoin, puis relâcher
async fn version2(etat: Arc<Mutex<Vec<String>>>) {
    let nombre = etat.lock().unwrap().len();  // le garde est libéré à la fin de l'instruction
    if nombre > 10 {
        envoyer_notification().await;
    }
}

// 3. Verrou asynchrone quand il DOIT être gardé pendant l'attente
async fn version3(etat: Arc<tokio::sync::Mutex<Vec<String>>>) {
    let mut v = etat.lock().await;            // garde Send
    v.push("a".into());
    envoyer_notification().await;             // permis, mais les autres tâches attendent
}
```

Préfère les versions 1 et 2 : un `std::sync::Mutex` est plus rapide qu'un verrou async tant qu'il n'est jamais gardé pendant un `.await`.

### 8.3 `Rc` dans une fonction async

```rust
async fn compter() {
    let r = std::rc::Rc::new(5);
    // quelque_chose().await;                 // r vit encore : le futur n'est pas Send
    println!("{r}");
}

async fn compter_ok() {
    {
        let r = std::rc::Rc::new(5);
        println!("{r}");
    }                                          // r libérée avant l'attente
    // quelque_chose().await;                 // OK
}
```

Une variable non `Send` ne pose problème que si elle est **encore vivante** au moment d'un `.await`.

### 8.4 Exécuter une tâche non `Send`

```rust
// Une tâche liée à un seul thread : LocalSet + spawn_local
async fn local() {
    let local = tokio::task::LocalSet::new();
    local
        .run_until(async {
            tokio::task::spawn_local(async {
                let r = std::rc::Rc::new(5);
                tokio::task::yield_now().await;      // permis : la tâche ne change jamais de thread
                println!("{r}");
            })
            .await
            .unwrap();
        })
        .await;
}
```

## 9. Trait objects et état partagé d'un serveur

### 9.1 Ajouter `Send + Sync` aux trait objects

```rust
trait Notificateur {
    fn envoyer(&self, msg: &str);
}

struct Etat {
    // notificateur: Box<dyn Notificateur>,             // ni Send ni Sync : refusé par axum et tokio
    notificateur: Box<dyn Notificateur + Send + Sync>,   // partageable entre les tâches
}
```

Un `dyn Trait` ne dit rien de son type concret : le compilateur ne peut pas supposer qu'il est `Send`. Il faut le demander explicitement. Autre écriture : faire de `Send + Sync` des supertraits.

```rust
trait Notificateur2: Send + Sync {
    fn envoyer(&self, msg: &str);
}
struct Etat2 { notificateur: Box<dyn Notificateur2> }   // Send + Sync déjà inclus
```

C'est ce que fait Runique : `FormField` et `PasswordHandler` ont `Send + Sync` dans leurs supertraits.

### 9.2 L'état d'une application axum

```rust
use std::sync::Arc;

#[derive(Clone)]
struct AppState {
    config: Arc<Config>,               // lecture seule, partagée
    compteur: Arc<std::sync::atomic::AtomicU64>,
    cache: Arc<tokio::sync::RwLock<std::collections::HashMap<String, String>>>,
}

struct Config { nom: String }
```

axum exige pour l'état `Clone + Send + Sync + 'static` : chaque requête reçoit un clone, traité sur n'importe quel thread. Avec des `Arc`, cloner l'état ne coûte que quelques compteurs. Runique suit le même principe avec `AEngine = Arc<RuniqueEngine>`.

## 10. Partager une valeur entre threads : toutes les façons

Compter des événements depuis plusieurs threads.

```rust
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;

// 1. Arc<Mutex<T>> : général, pour n'importe quelle donnée
fn v1() -> u64 {
    let total = Arc::new(Mutex::new(0u64));
    let hs: Vec<_> = (0..4).map(|_| {
        let t = Arc::clone(&total);
        thread::spawn(move || *t.lock().unwrap() += 1)
    }).collect();
    hs.into_iter().for_each(|h| h.join().unwrap());
    let r = *total.lock().unwrap();
    r
}

// 2. Atomique : pour un simple nombre ou un drapeau, sans verrou
fn v2() -> u64 {
    let total = Arc::new(AtomicU64::new(0));
    let hs: Vec<_> = (0..4).map(|_| {
        let t = Arc::clone(&total);
        thread::spawn(move || { t.fetch_add(1, Ordering::Relaxed); })
    }).collect();
    hs.into_iter().for_each(|h| h.join().unwrap());
    total.load(Ordering::Relaxed)
}

// 3. Canal : les threads envoient, un seul propriétaire additionne (pas de partage du tout)
fn v3() -> u64 {
    let (tx, rx) = mpsc::channel();
    for _ in 0..4 {
        let tx = tx.clone();
        thread::spawn(move || tx.send(1u64).unwrap());
    }
    drop(tx);                              // ferme le canal quand tous les émetteurs ont fini
    rx.iter().sum()
}

// 4. Threads de portée : chaque thread calcule sa part, on additionne les résultats
fn v4() -> u64 {
    thread::scope(|s| {
        let hs: Vec<_> = (0..4).map(|_| s.spawn(|| 1u64)).collect();
        hs.into_iter().map(|h| h.join().unwrap()).sum()
    })
}
```

| Version | Quand |
|---|---|
| 1 `Arc<Mutex<T>>` | état partagé complexe, modifié par plusieurs |
| 2 atomique | compteur, drapeau, identifiant à incrémenter |
| 3 canal | producteurs et consommateur, pas d'état commun |
| 4 scope + retour | découper un calcul, réunir les résultats |

« Ne communiquez pas en partageant la mémoire ; partagez la mémoire en communiquant » : les versions 3 et 4 évitent tout verrou.

## 11. Implémenter `Send` ou `Sync` à la main

### 11.1 Retirer `Send` volontairement

```rust
use std::marker::PhantomData;

struct LieAuThread {
    _pas_send: PhantomData<*const ()>,    // un pointeur brut n'est ni Send ni Sync
}
```

À quoi ça sert : un type qui représente une ressource liée au thread qui l'a créée (contexte graphique, handle d'une bibliothèque C non thread-safe).

### 11.2 Ajouter `Send` : `unsafe impl`

```rust
struct Tampon {
    ptr: *mut u8,          // pointeur brut : rend le type ni Send ni Sync
    len: usize,
}

// SAFETY : Tampon possède seul la mémoire pointée par ptr, aucun autre alias n'existe.
unsafe impl Send for Tampon {}
```

`unsafe impl` signifie : « je garantis ce que le compilateur ne peut pas vérifier ». Une erreur ici crée exactement les courses aux données que Rust empêche ailleurs. On ne l'écrit que pour envelopper une ressource bas niveau dont on maîtrise **tous** les accès, avec un commentaire `SAFETY` qui explique pourquoi c'est sûr.

Dans du code applicatif ordinaire, il n'y a **jamais** besoin d'`unsafe impl Send` : si le compilateur refuse, la correction est de changer de type (`Rc` → `Arc`, `RefCell` → `Mutex`).

## 12. Exercices

### Exercice 1 — corriger

Ce code ne compile pas. Corrige-le de deux façons.

```rust
use std::cell::RefCell;
use std::rc::Rc;

fn main() {
    let liste = Rc::new(RefCell::new(Vec::new()));
    let l = Rc::clone(&liste);
    std::thread::spawn(move || l.borrow_mut().push(1)).join().unwrap();
}
```

```rust
// Solution 1 : Arc + Mutex
use std::sync::{Arc, Mutex};
fn main() {
    let liste = Arc::new(Mutex::new(Vec::new()));
    let l = Arc::clone(&liste);
    std::thread::spawn(move || l.lock().unwrap().push(1)).join().unwrap();
}

// Solution 2 : le thread renvoie sa valeur, pas de partage
fn main2() {
    let mut liste = Vec::new();
    let x = std::thread::spawn(|| 1).join().unwrap();
    liste.push(x);
}
```

### Exercice 2 — futur non `Send`

Pourquoi cette fonction ne peut-elle pas être passée à `tokio::spawn` ? Corrige-la.

```rust
async fn ajouter(etat: std::sync::Arc<std::sync::Mutex<Vec<u32>>>) {
    let mut v = etat.lock().unwrap();
    v.push(1);
    tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    v.push(2);
}
```

```rust
// Solution : le MutexGuard std (non Send) est gardé à travers l'.await.
async fn ajouter(etat: std::sync::Arc<std::sync::Mutex<Vec<u32>>>) {
    etat.lock().unwrap().push(1);
    tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    etat.lock().unwrap().push(2);
}
```

Attention : entre les deux `push`, une autre tâche peut modifier le `Vec`. Si les deux ajouts doivent être atomiques ensemble, il faut un `tokio::sync::Mutex` gardé pendant l'attente.

### Exercice 3 — trait object partagé

Déclare un état d'application qui contient un `Box<dyn Stockage>` utilisable depuis plusieurs tâches tokio.

```rust
// Solution :
trait Stockage: Send + Sync {
    fn lire(&self, cle: &str) -> Option<String>;
}

#[derive(Clone)]
struct Etat {
    stockage: std::sync::Arc<dyn Stockage>,
}
```

`Arc` plutôt que `Box` : l'état doit être `Clone`, et un `Box<dyn Stockage>` ne l'est pas.

## 13. Aide-mémoire

```text
T: Send   → une valeur T peut être déplacée vers un autre thread
T: Sync   → &T peut être partagée entre threads   (T: Sync ⟺ &T: Send)

Rc        → ni Send ni Sync           → Arc
RefCell   → pas Sync                  → Mutex / RwLock
Cell<u32> → pas Sync                  → AtomicU32
MutexGuard (std) à travers .await     → relâcher avant, ou tokio::sync::Mutex
Box<dyn Trait>                        → Box<dyn Trait + Send + Sync>
emprunt dans thread::spawn            → move, Arc, ou thread::scope
```

### Points clés

- `Send` : déplaçable vers un autre thread. `Sync` : partageable par référence.
- Ce sont des traits automatiques : un type les a si tous ses champs les ont.
- `Rc`, `RefCell`, `Cell` et les pointeurs bruts bloquent ; `Arc`, `Mutex`, `RwLock`, atomiques débloquent.
- Un futur doit être `Send` pour `tokio::spawn` : rien de non `Send` ne doit vivre au moment d'un `.await`.
- Les trait objects partagés doivent dire `+ Send + Sync`.
- Le message d'erreur nomme le champ fautif : c'est lui qu'on change.
- `unsafe impl Send` est réservé aux ressources bas niveau dont on maîtrise tous les accès.
