# Smart Pointers
> Box<T>, Rc<T>, Arc<T>, RefCell<T>, Weak<T> — gestion mémoire avancée en Rust

## Objectifs

- Comprendre `Box<T>` pour l'allocation sur le tas
- Partager des données avec `Rc<T>` et `Arc<T>`
- Utiliser `RefCell<T>` pour la mutabilité intérieure
- Éviter les cycles de référence avec `Weak<T>`
- Choisir le bon smart pointer selon le contexte

---

## Table des matières

1. [Box<T> — allocation tas](#1-boxt--allocation-tas)
2. [Rc<T> — référence comptée](#2-rct--référence-comptée)
3. [Arc<T> — référence comptée atomique](#3-arct--référence-comptée-atomique)
4. [RefCell<T> — mutabilité intérieure](#4-refcellt--mutabilité-intérieure)
5. [Weak<T> — référence faible](#5-weakt--référence-faible)
6. [Combinaisons courantes](#6-combinaisons-courantes)
7. [Tableau récapitulatif](#7-tableau-récapitulatif)

---

## 1. Box<T> — allocation tas

`Box<T>` alloue une valeur sur le **tas** (heap) au lieu de la pile (stack).

```rust
// Valeur sur la pile
let x = 5;

// Valeur sur le tas
let b = Box::new(5);
println!("{b}"); // se déréférence automatiquement

// Box se libère automatiquement quand il sort de portée
```

**Quand utiliser `Box<T>` :**

```rust
// 1. Type récursif (taille inconnue à la compilation)
enum Liste {
    Element(i32, Box<Liste>),
    Fin,
}

let liste = Liste::Element(1,
    Box::new(Liste::Element(2,
        Box::new(Liste::Fin)
    ))
);

// 2. Grand objet à déplacer sans copier la pile
let grand = Box::new([0u8; 1_000_000]);

// 3. Trait object (dyn Trait)
trait Animal { fn parler(&self); }
struct Chien;
impl Animal for Chien { fn parler(&self) { println!("Woof"); } }

let animal: Box<dyn Animal> = Box::new(Chien);
animal.parler();
```

---

## 2. Rc<T> — référence comptée

`Rc<T>` (*Reference Counted*) permet **plusieurs propriétaires** d'une même valeur, en thread unique.

```rust
use std::rc::Rc;

let valeur = Rc::new(String::from("bonjour"));

let ref1 = Rc::clone(&valeur);  // incrémente le compteur
let ref2 = Rc::clone(&valeur);  // idem

println!("compteur : {}", Rc::strong_count(&valeur)); // 3
println!("{valeur}");

// La valeur est libérée quand le compteur atteint 0
drop(ref1);
println!("compteur : {}", Rc::strong_count(&valeur)); // 2
```

> **Important :** `Rc<T>` n'est **pas thread-safe**. Pour les threads, utilisez `Arc<T>`.

`Rc<T>` donne un accès **en lecture seule**. Pour modifier, combinez avec `RefCell<T>`.

---

## 3. Arc<T> — référence comptée atomique

`Arc<T>` (*Atomically Reference Counted*) est identique à `Rc<T>` mais **thread-safe**.

```rust
use std::sync::Arc;
use std::thread;

let valeur = Arc::new(vec![1, 2, 3]);

let mut handles = vec![];

for _ in 0..3 {
    let clone = Arc::clone(&valeur);
    let handle = thread::spawn(move || {
        println!("{:?}", clone);
    });
    handles.push(handle);
}

for h in handles { h.join().unwrap(); }
```

> `Arc<T>` a un coût légèrement supérieur à `Rc<T>` (opérations atomiques). N'utilisez `Arc` que si vous en avez besoin.

---

## 4. RefCell<T> — mutabilité intérieure

`RefCell<T>` déplace les vérifications du borrow checker de la **compilation** vers l'**exécution**.

```rust
use std::cell::RefCell;

let donnees = RefCell::new(vec![1, 2, 3]);

// Emprunt immutable
let lecture = donnees.borrow();
println!("{:?}", *lecture);
drop(lecture); // libère l'emprunt

// Emprunt mutable
donnees.borrow_mut().push(4);
println!("{:?}", donnees.borrow()); // [1, 2, 3, 4]
```

> **Attention :** Si les règles du borrow checker sont violées à l'exécution, `RefCell` **panique** (`panic!`).

```rust
// Ceci panique à l'exécution !
let cellule = RefCell::new(5);
let _ref1 = cellule.borrow();
let _ref2 = cellule.borrow_mut(); // PANIC : déjà emprunté immutablement
```

**`try_borrow` et `try_borrow_mut`** pour éviter la panique :

```rust
match donnees.try_borrow_mut() {
    Ok(mut val) => val.push(99),
    Err(_)      => println!("déjà emprunté"),
}
```

---

## 5. Weak<T> — référence faible

`Weak<T>` est une référence qui **ne possède pas** la valeur — ne compte pas dans le `Rc`/`Arc`.

Utilisé pour briser les **cycles de référence** (qui empêcheraient la libération).

```rust
use std::rc::{Rc, Weak};
use std::cell::RefCell;

#[derive(Debug)]
struct Noeud {
    valeur: i32,
    parent: Option<Weak<RefCell<Noeud>>>,   // référence faible vers le parent
    enfants: Vec<Rc<RefCell<Noeud>>>,        // références fortes vers les enfants
}

let parent = Rc::new(RefCell::new(Noeud {
    valeur: 1,
    parent: None,
    enfants: vec![],
}));

let enfant = Rc::new(RefCell::new(Noeud {
    valeur: 2,
    parent: Some(Rc::downgrade(&parent)),  // Weak depuis un Rc
    enfants: vec![],
}));

// Accéder via Weak — retourne Option<Rc<T>>
if let Some(p) = enfant.borrow().parent.as_ref().and_then(|w| w.upgrade()) {
    println!("parent : {}", p.borrow().valeur);
}
```

---

## 6. Combinaisons courantes

```rust
use std::rc::Rc;
use std::cell::RefCell;

// Rc<RefCell<T>> — plusieurs propriétaires + mutation en thread unique
let partagé = Rc::new(RefCell::new(vec![1, 2, 3]));

let clone1 = Rc::clone(&partagé);
let clone2 = Rc::clone(&partagé);

clone1.borrow_mut().push(4);
clone2.borrow_mut().push(5);
println!("{:?}", partagé.borrow()); // [1, 2, 3, 4, 5]

// Arc<Mutex<T>> — plusieurs propriétaires + mutation entre threads
use std::sync::{Arc, Mutex};

let partagé = Arc::new(Mutex::new(vec![]));
let clone   = Arc::clone(&partagé);

std::thread::spawn(move || {
    partagé.lock().unwrap().push(1);
}).join().unwrap();

println!("{:?}", clone.lock().unwrap()); // [1]
```

---

## 7. Tableau récapitulatif

| Type | Propriétaires | Thread-safe | Mutation | Coût |
|---|---|---|---|---|
| `T` | 1 | — | oui (`mut`) | nul |
| `Box<T>` | 1 | — | oui (`mut`) | allocation tas |
| `Rc<T>` | N | ❌ | non (seul) | compteur |
| `Arc<T>` | N | ✅ | non (seul) | compteur atomique |
| `RefCell<T>` | 1 | ❌ | oui (runtime) | vérification runtime |
| `Rc<RefCell<T>>` | N | ❌ | oui (runtime) | compteur + runtime |
| `Arc<Mutex<T>>` | N | ✅ | oui (lock) | atomique + lock |
