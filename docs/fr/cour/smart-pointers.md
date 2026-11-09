# Smart pointers

> Les pointeurs intelligents possèdent une valeur et ajoutent un comportement : partage (`Rc`, `Arc`), mutabilité interne (`Cell`, `RefCell`), initialisation paresseuse (`OnceCell`, `LazyLock`). Ce cours montre aussi les deux traits qui les rendent possibles : `Deref` et `Drop`.

## Objectifs

- Comprendre ce qui fait d'un type un « smart pointer » : `Deref` et `Drop`
- Écrire son propre `Drop` et maîtriser l'ordre de libération
- Partager une valeur entre plusieurs propriétaires avec `Rc` et `Arc`
- Modifier une valeur derrière une référence partagée avec `Cell` et `RefCell`
- Combiner `Rc<RefCell<T>>` et éviter les fuites avec `Weak`
- Initialiser une valeur une seule fois avec `OnceCell`, `OnceLock`, `LazyLock`
- Choisir le bon outil selon le besoin

## Table des matières

1. Ce qu'est un smart pointer
2. Le trait `Deref`
3. Le trait `Drop`
4. `Rc<T>` : plusieurs propriétaires
5. `Arc<T>` : plusieurs propriétaires entre threads
6. Mutabilité interne : `Cell<T>`
7. Mutabilité interne : `RefCell<T>`
8. `Rc<RefCell<T>>`
9. Cycles et `Weak<T>`
10. Initialisation unique : `OnceCell`, `OnceLock`, `LazyLock`
11. Copie à l'écriture : `Rc::make_mut`
12. Choisir
13. La même structure écrite de plusieurs façons
14. Exercices
15. Aide-mémoire

## 1. Ce qu'est un smart pointer

Une référence `&T` emprunte une valeur. Un **smart pointer** la **possède** et ajoute un comportement. Tu en utilises déjà :

| Type | Ce qu'il ajoute |
|---|---|
| `Box<T>` | valeur sur le tas, possédée seule (cours `box-dynamiques`) |
| `String`, `Vec<T>` | tampon sur le tas qui grandit |
| `Rc<T>`, `Arc<T>` | plusieurs propriétaires, libération au dernier |
| `RefCell<T>`, `Mutex<T>` | modification à travers une référence partagée |
| `MutexGuard<T>`, `Ref<T>` | verrou relâché automatiquement |

Ce qui les rend possibles : `Deref` (s'utiliser comme la valeur) et `Drop` (faire quelque chose à la libération).

## 2. Le trait `Deref`

### 2.1 Ce qu'il permet

```rust
let b = Box::new(String::from("abc"));
let n = b.len();               // méthode de String, appelée sur une Box<String>
let s: &str = &b;              // &Box<String> devient &String puis &str
```

`Deref` dit « je peux être vu comme une référence vers `Target` ». Le compilateur insère alors les conversions tout seul (*deref coercion*), autant de fois que nécessaire.

### 2.2 L'implémenter

```rust
use std::ops::Deref;

struct Enveloppe<T> {
    valeur: T,
    lectures: std::cell::Cell<u32>,
}

impl<T> Deref for Enveloppe<T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.lectures.set(self.lectures.get() + 1);
        &self.valeur
    }
}

let e = Enveloppe { valeur: vec![1, 2, 3], lectures: std::cell::Cell::new(0) };
println!("{}", e.len());       // méthode de Vec
```

`DerefMut` fait la même chose pour `&mut`.

### 2.3 Quand ne pas l'implémenter

`Deref` est fait pour les **pointeurs**. L'implémenter sur un newtype pour « hériter » des méthodes du type intérieur est déconseillé : toutes les méthodes deviennent accessibles, y compris celles qui contournent l'invariant du newtype.

```rust
struct Email(String);

// À éviter : email.clear() ou email.push_str("x") deviendraient possibles
// impl Deref for Email { type Target = String; ... }

// Préférer exposer ce qui a du sens
impl Email {
    fn as_str(&self) -> &str { &self.0 }
}
```

## 3. Le trait `Drop`

### 3.1 Écrire un `Drop`

```rust
struct Connexion { nom: String }

impl Drop for Connexion {
    fn drop(&mut self) {
        println!("fermeture de {}", self.nom);
    }
}

fn main() {
    let _c = Connexion { nom: "base".into() };
    println!("travail");
}
// travail
// fermeture de base
```

`drop` est appelé **automatiquement** quand la valeur sort de portée, même si le programme panique (déroulement de pile). Il sert à libérer ce que Rust ne connaît pas : fichier, verrou, connexion, fichier temporaire, compteur à décrémenter.

### 3.2 Ordre de libération

```rust
struct Nom(&'static str);
impl Drop for Nom {
    fn drop(&mut self) { println!("drop {}", self.0); }
}

struct Paire { a: Nom, b: Nom }

fn main() {
    let _x = Nom("x");
    let _y = Nom("y");
    let _p = Paire { a: Nom("a"), b: Nom("b") };
}
// drop a, drop b      (champs : ordre de déclaration, après le drop de Paire elle-même)
// drop y, drop x      (variables : ordre inverse de déclaration)
```

| Ce qui est libéré | Ordre |
|---|---|
| variables d'un bloc | inverse de la déclaration |
| champs d'une struct | ordre de déclaration |
| éléments d'un `Vec` ou d'un tableau | du premier au dernier |
| temporaires d'une instruction | à la fin de l'instruction (`;`) |

### 3.3 Libérer plus tôt

```rust
let c = Connexion { nom: "base".into() };
// c.drop();               // ERREUR E0040 : explicit use of destructor method
drop(c);                   // OK : std::mem::drop prend la valeur
```

On ne peut pas appeler la méthode `drop` soi-même (elle serait rappelée à la fin du bloc, soit deux libérations). `drop(c)` déplace `c` dans une fonction vide qui la libère.

### 3.4 Verrou : la libération compte

```rust
use std::sync::Mutex;

let m = Mutex::new(0);
{
    let mut garde = m.lock().unwrap();   // verrou pris
    *garde += 1;
}                                        // garde libérée : verrou relâché

let mut garde = m.lock().unwrap();
*garde += 1;
drop(garde);                             // relâcher avant une longue opération
// longue_operation();
```

Le verrou d'un `Mutex` est un `MutexGuard` dont le `Drop` relâche le verrou. Garder un garde plus longtemps que nécessaire bloque tous les autres threads.

### 3.5 Garde de nettoyage : exécuter du code en sortie, quoi qu'il arrive

```rust
struct ALaSortie<F: FnMut()>(F);

impl<F: FnMut()> Drop for ALaSortie<F> {
    fn drop(&mut self) { (self.0)(); }
}

fn traiter(echec: bool) -> Result<(), String> {
    let _nettoyage = ALaSortie(|| println!("fichier temporaire supprimé"));
    if echec {
        return Err("échec au milieu".into());   // le nettoyage s'exécute quand même
    }
    Ok(())
}
```

### 3.6 Règles

- Un type qui implémente `Drop` ne peut pas être `Copy`.
- On ne peut pas déplacer un champ hors d'une struct qui implémente `Drop` (la struct serait incomplète au moment du `drop`) ; on utilise `std::mem::take` ou `Option::take`.
- `drop` ne doit pas paniquer : une panique pendant un déroulement de pile arrête le programme.
- `std::mem::forget(x)` empêche le `drop` (fuite volontaire) ; `ManuallyDrop<T>` aussi, champ par champ.

## 4. `Rc<T>` : plusieurs propriétaires

### 4.1 Le problème

```rust
let a = String::from("partagée");
let liste1 = vec![a];
// let liste2 = vec![a];      // ERREUR : a a déjà été déplacée
```

Parfois une valeur doit appartenir à **plusieurs** structures à la fois (un nœud référencé par plusieurs parents, une configuration partagée par plusieurs composants).

### 4.2 `Rc` : compteur de références

```rust
use std::rc::Rc;

let a = Rc::new(String::from("partagée"));
let b = Rc::clone(&a);         // même String, compteur à 2
let c = a.clone();             // même chose, écriture plus courte
println!("{}", Rc::strong_count(&a));   // 3
drop(b);
println!("{}", Rc::strong_count(&a));   // 2
// la String est libérée quand le dernier Rc disparaît
```

`Rc::clone` ne copie pas la valeur : il augmente un compteur. `Rc::clone(&a)` est préféré à `a.clone()` dans la plupart des codes, pour qu'on voie au premier coup d'œil qu'il ne s'agit pas d'une copie coûteuse.

### 4.3 `Rc` est en lecture seule

```rust
let r = Rc::new(vec![1, 2]);
// r.push(3);                 // ERREUR : cannot borrow data in an `Rc` as mutable
```

Plusieurs propriétaires qui pourraient tous modifier : c'est exactement ce que les règles d'emprunt interdisent. Pour modifier, il faut une mutabilité interne (section 8).

### 4.4 Limites

- `Rc` n'est **pas** `Send` : il ne peut pas passer à un autre thread (le compteur n'est pas protégé). Pour plusieurs threads : `Arc`.
- Deux `Rc` qui pointent l'un vers l'autre ne sont jamais libérés (section 9).

### 4.5 Récupérer la valeur

```rust
let r = Rc::new(String::from("x"));
match Rc::try_unwrap(r) {
    Ok(s) => println!("seul propriétaire : {s}"),     // possible si le compteur vaut 1
    Err(r) => println!("encore partagée"),
}
```

## 5. `Arc<T>` : plusieurs propriétaires entre threads

```rust
use std::sync::Arc;
use std::thread;

let config = Arc::new(vec!["a", "b"]);
let mut taches = Vec::new();
for i in 0..3 {
    let c = Arc::clone(&config);
    taches.push(thread::spawn(move || println!("{i} : {:?}", c)));
}
for t in taches {
    t.join().unwrap();
}
```

`Arc` (*atomic reference counted*) a la même interface que `Rc`, avec un compteur atomique, donc partageable entre threads. Le coût : chaque `clone` et chaque libération font une opération atomique, légèrement plus lente.

| | `Rc<T>` | `Arc<T>` |
|---|---|---|
| Threads | un seul | plusieurs (si `T: Send + Sync`) |
| Compteur | simple | atomique |
| Modification | avec `RefCell` | avec `Mutex` ou `RwLock` |

Le détail du partage entre threads est dans `concurrence`.

## 6. Mutabilité interne : `Cell<T>`

Une `Cell<T>` permet de modifier une valeur à travers une référence **partagée** (`&`), en remplaçant la valeur entière.

```rust
use std::cell::Cell;

struct Compteur {
    appels: Cell<u32>,
}

impl Compteur {
    fn compter(&self) {                        // &self, pas &mut self
        self.appels.set(self.appels.get() + 1);
    }
}

let c = Compteur { appels: Cell::new(0) };
c.compter();
c.compter();
println!("{}", c.appels.get());   // 2
```

| Méthode | Effet |
|---|---|
| `get()` | copie de la valeur (si `T: Copy`) |
| `set(v)` | remplace |
| `replace(v)` | remplace et renvoie l'ancienne |
| `take()` | prend la valeur et laisse `Default` |

`Cell` ne donne **jamais** de référence vers son contenu : impossible de créer deux emprunts conflictuels, donc aucune vérification à l'exécution. Idéale pour un compteur, un drapeau, une petite valeur `Copy`.

## 7. Mutabilité interne : `RefCell<T>`

### 7.1 Emprunter à l'exécution

```rust
use std::cell::RefCell;

let v = RefCell::new(vec![1, 2]);
v.borrow_mut().push(3);                  // emprunt mutable temporaire
println!("{:?}", v.borrow());            // emprunt partagé temporaire
```

`RefCell` applique **les mêmes règles** d'emprunt (une écriture ou plusieurs lectures), mais **à l'exécution** au lieu de la compilation.

### 7.2 Violer la règle : panique

```rust
let v = RefCell::new(vec![1]);
let lecture = v.borrow();
// let ecriture = v.borrow_mut();       // PANIQUE : already borrowed: BorrowMutError
drop(lecture);
let ecriture = v.borrow_mut();           // OK
```

### 7.3 Version qui ne panique pas

```rust
match v.try_borrow_mut() {
    Ok(mut e) => e.push(4),
    Err(_) => println!("déjà empruntée ailleurs"),
}
```

### 7.4 Limiter la durée des emprunts

```rust
let donnees = RefCell::new(vec![1, 2, 3]);

// Risqué : l'emprunt vit pendant tout le bloc
{
    let lecture = donnees.borrow();
    for x in lecture.iter() {
        // donnees.borrow_mut().push(*x);   // panique
    }
}

// Sûr : copier ce dont on a besoin, puis relâcher
let copie: Vec<i32> = donnees.borrow().clone();
for x in copie {
    donnees.borrow_mut().push(x);
}
```

Les emprunts de `RefCell` (`Ref`, `RefMut`) se libèrent à la fin de leur portée, comme un verrou.

### 7.5 `Cell`, `RefCell` ou `&mut` ?

Commence toujours par `&mut self` : le compilateur vérifie tout. Passe à la mutabilité interne seulement quand la structure l'impose : valeur partagée par `Rc`, cache dans une méthode `&self`, objet simulé dans un test (cours `tests-rust`).

`Cell` et `RefCell` ne sont pas `Sync` : pour plusieurs threads, `Mutex`, `RwLock` ou les types atomiques (`AtomicU32`...).

## 8. `Rc<RefCell<T>>`

Combiner les deux donne une valeur **partagée et modifiable** dans un seul thread.

```rust
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug)]
struct Compte { solde: i64 }

let compte = Rc::new(RefCell::new(Compte { solde: 100 }));
let titulaire = Rc::clone(&compte);
let banque = Rc::clone(&compte);

titulaire.borrow_mut().solde -= 30;
banque.borrow_mut().solde += 5;
println!("{:?}", compte.borrow());      // Compte { solde: 75 }
```

| Rôle | Type |
|---|---|
| plusieurs propriétaires | `Rc` |
| modification | `RefCell` |
| les deux, dans un thread | `Rc<RefCell<T>>` |
| les deux, entre threads | `Arc<Mutex<T>>` ou `Arc<RwLock<T>>` |

Si ton code se remplit de `Rc<RefCell<...>>`, c'est souvent le signe d'une architecture à revoir : un propriétaire unique avec des index (section 13) est généralement plus simple.

## 9. Cycles et `Weak<T>`

### 9.1 La fuite

```rust
use std::cell::RefCell;
use std::rc::Rc;

struct Noeud {
    suivant: RefCell<Option<Rc<Noeud>>>,
}

let a = Rc::new(Noeud { suivant: RefCell::new(None) });
let b = Rc::new(Noeud { suivant: RefCell::new(Some(Rc::clone(&a))) });
*a.suivant.borrow_mut() = Some(Rc::clone(&b));    // a -> b -> a
// à la fin du bloc, chaque compteur reste à 1 : aucun des deux n'est jamais libéré
```

Rust empêche les accès invalides, pas les fuites : un cycle de `Rc` est sûr mais ne libère jamais sa mémoire.

### 9.2 `Weak` : une référence qui ne retient pas

```rust
use std::rc::{Rc, Weak};

let fort = Rc::new(5);
let faible: Weak<i32> = Rc::downgrade(&fort);

println!("{:?}", faible.upgrade());   // Some(5) : la valeur existe encore
drop(fort);
println!("{:?}", faible.upgrade());   // None : libérée
```

Un `Weak` ne compte pas pour la libération. `upgrade()` renvoie un `Rc` si la valeur existe encore, `None` sinon.

### 9.3 Arbre avec lien vers le parent

```rust
use std::cell::RefCell;
use std::rc::{Rc, Weak};

#[derive(Debug)]
struct Noeud {
    valeur: i32,
    parent: RefCell<Weak<Noeud>>,          // faible : l'enfant ne retient pas le parent
    enfants: RefCell<Vec<Rc<Noeud>>>,      // fort : le parent retient ses enfants
}

let feuille = Rc::new(Noeud { valeur: 3, parent: RefCell::new(Weak::new()), enfants: RefCell::new(vec![]) });
let branche = Rc::new(Noeud {
    valeur: 5,
    parent: RefCell::new(Weak::new()),
    enfants: RefCell::new(vec![Rc::clone(&feuille)]),
});
*feuille.parent.borrow_mut() = Rc::downgrade(&branche);

let parent = feuille.parent.borrow().upgrade().map(|p| p.valeur);   // Some(5)
```

Règle : les liens qui vont **vers le bas** (propriétaire → possédé) sont forts, ceux qui **remontent** sont faibles.

## 10. Initialisation unique : `OnceCell`, `OnceLock`, `LazyLock`

### 10.1 La même valeur paresseuse, écrite de trois façons

```rust
use std::collections::HashMap;
use std::sync::{LazyLock, OnceLock};

// 1. LazyLock (Rust 1.80+) : la fonction d'initialisation est donnée à la déclaration
static PAYS: LazyLock<HashMap<&str, &str>> = LazyLock::new(|| {
    HashMap::from([("fr", "France"), ("be", "Belgique")])
});

// 2. OnceLock : la valeur est fixée plus tard, une seule fois
static CONFIG: OnceLock<String> = OnceLock::new();

fn initialiser(url: String) {
    CONFIG.set(url).expect("configuration déjà initialisée");
}

fn config() -> &'static str {
    CONFIG.get().map(String::as_str).unwrap_or("non configuré")
}

// 3. OnceLock avec get_or_init : calcul à la première demande, depuis n'importe où
fn regex_email() -> &'static regex::Regex {
    static R: OnceLock<regex::Regex> = OnceLock::new();
    R.get_or_init(|| regex::Regex::new(r"^[^@\s]+@[^@\s]+$").unwrap())
}

fn main() {
    println!("{}", PAYS["fr"]);
    initialiser("postgres://...".into());
    println!("{}", config());
}
```

| Type | Thread | Valeur fournie |
|---|---|---|
| `std::cell::OnceCell<T>` | un seul | plus tard (`set`, `get_or_init`) |
| `std::cell::LazyCell<T>` | un seul | fonction à la déclaration |
| `std::sync::OnceLock<T>` | plusieurs | plus tard |
| `std::sync::LazyLock<T>` | plusieurs | fonction à la déclaration |

Ces types remplacent les crates `lazy_static` et `once_cell` dans le code récent.

### 10.2 Dans une struct

```rust
use std::cell::OnceCell;

struct Document {
    texte: String,
    nb_mots: OnceCell<usize>,          // calculé à la première demande, puis gardé
}

impl Document {
    fn nb_mots(&self) -> usize {
        *self.nb_mots.get_or_init(|| self.texte.split_whitespace().count())
    }
}
```

## 11. Copie à l'écriture : `Rc::make_mut`

```rust
use std::rc::Rc;

let mut a = Rc::new(vec![1, 2, 3]);
let b = Rc::clone(&a);

Rc::make_mut(&mut a).push(4);      // a est partagé : la valeur est COPIÉE puis modifiée
println!("{:?} {:?}", a, b);       // [1, 2, 3, 4] [1, 2, 3]

Rc::make_mut(&mut a).push(5);      // a est maintenant seul : modifié sur place, sans copie
```

À quoi ça sert : partager une grosse valeur tant que personne ne la modifie, et ne payer la copie qu'au moment d'une modification. `Arc::make_mut` fait la même chose entre threads.

## 12. Choisir

| Besoin | Outil |
|---|---|
| Un seul propriétaire, valeur sur le tas | `Box<T>` |
| Plusieurs propriétaires, lecture seule, un thread | `Rc<T>` |
| Plusieurs propriétaires, lecture seule, plusieurs threads | `Arc<T>` |
| Modifier une petite valeur `Copy` derrière `&` | `Cell<T>` |
| Modifier une valeur quelconque derrière `&`, un thread | `RefCell<T>` |
| Partagée et modifiable, un thread | `Rc<RefCell<T>>` |
| Partagée et modifiable, plusieurs threads | `Arc<Mutex<T>>`, `Arc<RwLock<T>>` |
| Compteur ou drapeau partagé entre threads | `AtomicU64`, `AtomicBool` |
| Lien qui ne doit pas retenir la valeur | `Weak<T>` |
| Valeur calculée une fois | `OnceLock`, `LazyLock` (`OnceCell`, `LazyCell` en un thread) |
| Action garantie à la sortie d'un bloc | type avec `Drop` |

## 13. La même structure écrite de plusieurs façons

Un graphe d'utilisateurs qui se suivent.

```rust
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::{Rc, Weak};

// 1. Rc + RefCell + Weak : des pointeurs entre nœuds
struct Utilisateur1 {
    nom: String,
    suit: RefCell<Vec<Weak<Utilisateur1>>>,
}

// 2. Index dans un Vec : un seul propriétaire, des numéros comme liens
struct Reseau2 {
    noms: Vec<String>,
    suit: Vec<Vec<usize>>,          // suit[i] = index des personnes suivies par i
}

impl Reseau2 {
    fn ajouter(&mut self, nom: &str) -> usize {
        self.noms.push(nom.to_string());
        self.suit.push(Vec::new());
        self.noms.len() - 1
    }
    fn suivre(&mut self, qui: usize, cible: usize) {
        self.suit[qui].push(cible);
    }
}

// 3. Identifiants dans une HashMap : comme en base de données
struct Reseau3 {
    utilisateurs: HashMap<u64, String>,
    suit: HashMap<u64, Vec<u64>>,
}
```

| Version | Avantages | Inconvénients |
|---|---|---|
| 1 | proche d'un modèle objet | `borrow()` partout, risque de cycle, paniques possibles à l'exécution |
| 2 | simple, rapide, aucun emprunt à gérer | un index peut devenir invalide après une suppression |
| 3 | suppressions simples, modèle d'une base | une recherche par lien |

Dans la plupart des programmes Rust, les versions 2 et 3 sont préférées à la version 1.

## 14. Exercices

### Exercice 1 — journal de fermeture

Écris un type `Chrono` qui affiche, quand il est libéré, le temps écoulé depuis sa création.

```rust
// Solution :
use std::time::Instant;

struct Chrono { nom: &'static str, debut: Instant }

impl Chrono {
    fn new(nom: &'static str) -> Self { Self { nom, debut: Instant::now() } }
}

impl Drop for Chrono {
    fn drop(&mut self) {
        println!("{} : {:?}", self.nom, self.debut.elapsed());
    }
}

fn main() {
    let _c = Chrono::new("calcul");
    let _s: u64 = (0..1_000_000).sum();
}   // affiche la durée, même si le calcul panique
```

### Exercice 2 — liste partagée

Deux structs `Equipe` partagent la même liste de membres et doivent toutes deux pouvoir en ajouter. Choisis le type et écris-le.

```rust
// Solution :
use std::cell::RefCell;
use std::rc::Rc;

struct Equipe { nom: String, membres: Rc<RefCell<Vec<String>>> }

fn main() {
    let commun = Rc::new(RefCell::new(Vec::new()));
    let a = Equipe { nom: "A".into(), membres: Rc::clone(&commun) };
    let b = Equipe { nom: "B".into(), membres: Rc::clone(&commun) };
    a.membres.borrow_mut().push("Ada".into());
    b.membres.borrow_mut().push("Linus".into());
    assert_eq!(commun.borrow().len(), 2);
}
```

### Exercice 3 — cache dans une méthode `&self`

Ajoute à `struct Article { texte: String }` une méthode `resume(&self) -> &str` qui calcule les 50 premiers caractères une seule fois.

```rust
// Solution :
use std::cell::OnceCell;

struct Article { texte: String, resume: OnceCell<String> }

impl Article {
    fn resume(&self) -> &str {
        self.resume.get_or_init(|| self.texte.chars().take(50).collect())
    }
}
```

## 15. Aide-mémoire

```rust
impl Drop for T { fn drop(&mut self) { .. } }     drop(x)
impl Deref for T { type Target = U; fn deref(&self) -> &U { .. } }

Rc::new(v)   Rc::clone(&r)   Rc::strong_count(&r)   Rc::downgrade(&r)   Rc::try_unwrap(r)   Rc::make_mut(&mut r)
Arc::new(v)  Arc::clone(&a)
weak.upgrade()                                     // Option<Rc<T>>
Cell::new(v)   c.get()   c.set(v)   c.replace(v)   c.take()
RefCell::new(v)   r.borrow()   r.borrow_mut()   r.try_borrow_mut()
OnceLock::new()   o.set(v)   o.get()   o.get_or_init(|| v)
static X: LazyLock<T> = LazyLock::new(|| v);
```

### Points clés

- `Deref` rend un pointeur transparent ; à réserver aux pointeurs, pas aux newtypes.
- `Drop` s'exécute toujours, même en cas de panique : idéal pour libérer une ressource.
- Variables libérées dans l'ordre inverse, champs dans l'ordre de déclaration.
- `Rc` partage dans un thread, `Arc` entre threads ; `clone` ne copie que le pointeur.
- `Cell` pour les petites valeurs `Copy`, `RefCell` pour le reste ; `RefCell` panique si on viole les règles.
- Un cycle de `Rc` fuit : utilise `Weak` pour les liens qui remontent.
- `LazyLock` et `OnceLock` remplacent `lazy_static` et `once_cell`.
- Avant `Rc<RefCell<T>>`, envisage un propriétaire unique et des index.
