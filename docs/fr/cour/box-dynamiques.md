# Box et types dynamiques

> `Box<T>` pour placer une valeur sur le tas, `dyn Trait` pour manipuler des types différents derrière une même interface : quand les utiliser, comment, et ce qu'ils coûtent par rapport aux génériques et aux enums.

## Objectifs

- Savoir quand une valeur doit aller dans une `Box`
- Comprendre ce qu'est un trait object (`dyn Trait`) et un pointeur « large »
- Écrire `&dyn Trait`, `Box<dyn Trait>`, `Arc<dyn Trait>` et choisir entre eux
- Connaître les règles de compatibilité `dyn` d'un trait, et les contourner
- Stocker des trait objects dans des structs et des collections
- Cloner, comparer et convertir des trait objects
- Choisir entre enum, générique et `dyn Trait` pour un même problème

## Table des matières

1. `Box<T>` : une valeur sur le tas
2. Quand utiliser une `Box`
3. Les trait objects
4. Les différentes façons de tenir un trait object
5. Comment fonctionne un appel dynamique
6. Compatibilité `dyn` d'un trait
7. Trait objects dans les structs
8. Collections de trait objects
9. Cloner un trait object
10. Retrouver le type concret : `Any`
11. Remonter vers un supertrait
12. Enum, générique ou `dyn` : la même fonctionnalité trois fois
13. Patterns de conception
14. Exemples réels dans Runique
15. Exercices
16. Points clés à retenir

## 1. `Box<T>` : une valeur sur le tas

```rust
let b = Box::new(5);           // 5 est sur le tas, b (un pointeur) sur la pile
println!("{}", b);             // Box se déréférence automatiquement
let x: i32 = *b;               // * pour récupérer la valeur
```

Une `Box<T>` **possède** sa valeur : quand la `Box` est libérée, la valeur l'est aussi. Elle se comporte comme la valeur elle-même (méthodes, opérateurs) grâce à `Deref`.

```rust
let s = Box::new(String::from("abc"));
println!("{}", s.len());      // méthode de String, appelée à travers la Box
let possede: String = *s;     // sortir la valeur de la Box (la Box est consommée)
```

Sa taille sur la pile est toujours celle d'un pointeur (8 octets sur une machine 64 bits), quelle que soit la taille de `T`.

## 2. Quand utiliser une `Box`

### 2.1 Type récursif

```rust
// enum Liste { Fin, Element(i32, Liste) }       // ERREUR E0072 : recursive type has infinite size

enum Liste {
    Fin,
    Element(i32, Box<Liste>),
}

let l = Liste::Element(1, Box::new(Liste::Element(2, Box::new(Liste::Fin))));
```

Pour calculer la taille de `Liste`, le compilateur aurait besoin de la taille de `Liste`. Avec `Box`, l'élément suivant devient un pointeur de taille fixe.

### 2.2 Grosse valeur qu'on déplace souvent

```rust
struct Image { pixels: [u8; 1_000_000] }

fn traiter(img: Box<Image>) -> Box<Image> { img }    // ne déplace qu'un pointeur
```

Déplacer une valeur copie ses octets ; déplacer une `Box` ne copie que le pointeur.

### 2.3 Réduire la taille d'une enum

```rust
enum Message {
    Ping,
    Gros(Box<[u8; 4096]>),     // sans Box, chaque Ping occuperait 4 Ko
}
```

### 2.4 Trait object possédé

```rust
trait Forme { fn aire(&self) -> f64; }
let f: Box<dyn Forme>;          // voir la suite du cours
```

### 2.5 Tranches et chaînes de taille fixe

```rust
let tranche: Box<[i32]> = vec![1, 2, 3].into_boxed_slice();   // comme un Vec, sans capacité en trop
let texte: Box<str> = String::from("fixe").into_boxed_str();
```

À quoi ça sert : stocker beaucoup de données qui ne grandiront plus, avec un pointeur et une longueur seulement (16 octets au lieu de 24 pour un `Vec` ou une `String`).

### 2.6 Quand ne pas utiliser `Box`

Pour une valeur ordinaire, sans récursion ni trait object : la pile est plus rapide et `Box` n'apporte qu'une allocation. `Box<Vec<T>>` est presque toujours inutile : le `Vec` est déjà sur le tas.

## 3. Les trait objects

Un **trait object** est une valeur dont on ne connaît que le trait, pas le type concret. Il s'écrit `dyn Trait`.

```rust
trait Animal {
    fn nom(&self) -> String;
    fn cri(&self) -> String;
}

struct Chien;
struct Chat;

impl Animal for Chien {
    fn nom(&self) -> String { "chien".into() }
    fn cri(&self) -> String { "wouf".into() }
}

impl Animal for Chat {
    fn nom(&self) -> String { "chat".into() }
    fn cri(&self) -> String { "miaou".into() }
}

fn presenter(a: &dyn Animal) {
    println!("le {} fait {}", a.nom(), a.cri());
}

fn main() {
    presenter(&Chien);
    presenter(&Chat);
    let zoo: Vec<Box<dyn Animal>> = vec![Box::new(Chien), Box::new(Chat)];
    for a in &zoo {
        presenter(a.as_ref());
    }
}
```

`dyn Animal` n'a pas de taille connue (un `Chien` et un `Chat` n'ont pas forcément la même) : on ne le manipule **que derrière un pointeur** (`&`, `Box`, `Arc`, `Rc`).

## 4. Les différentes façons de tenir un trait object

| Écriture | Possède la valeur | Partage | Usage |
|---|---|---|---|
| `&dyn Trait` | non | — | paramètre de fonction, accès temporaire |
| `&mut dyn Trait` | non | — | paramètre qui modifie |
| `Box<dyn Trait>` | oui, seul | non | champ de struct, élément de collection, retour de fonction |
| `Rc<dyn Trait>` | oui, à plusieurs | un thread | graphe, valeur partagée dans un seul thread |
| `Arc<dyn Trait>` | oui, à plusieurs | entre threads | service partagé dans un serveur |

```rust
use std::rc::Rc;
use std::sync::Arc;

let a: &dyn Animal = &Chien;
let b: Box<dyn Animal> = Box::new(Chien);
let c: Rc<dyn Animal> = Rc::new(Chat);
let d: Arc<dyn Animal + Send + Sync> = Arc::new(Chat);
```

### 4.1 Ajouter des contraintes

```rust
Box<dyn Animal + Send>             // peut être envoyé à un autre thread
Box<dyn Animal + Send + Sync>      // peut être partagé entre threads
Box<dyn Animal + 'a>               // peut contenir des références de durée 'a
Box<dyn Animal>                    // = Box<dyn Animal + 'static>
```

Pour un serveur web (tokio, axum), les trait objects partagés sont presque toujours `Send + Sync`.

### 4.2 Paramètre : `&dyn` ou générique

```rust
fn presenter_dyn(a: &dyn Animal) { println!("{}", a.nom()); }
fn presenter_gen(a: &impl Animal) { println!("{}", a.nom()); }
fn presenter_gen2<A: Animal + ?Sized>(a: &A) { println!("{}", a.nom()); }   // accepte aussi &dyn Animal
```

Les trois acceptent `&Chien`. La version générique est copiée pour chaque type (rapide), la version `dyn` existe en un seul exemplaire (plus compacte). `?Sized` permet à la version générique d'accepter aussi un `&dyn Animal`.

## 5. Comment fonctionne un appel dynamique

### 5.1 Le pointeur large

```rust
use std::mem::size_of;

size_of::<&Chien>();           // 8 octets : une adresse
size_of::<&dyn Animal>();      // 16 octets : une adresse + une adresse de vtable
size_of::<Box<dyn Animal>>();  // 16 octets
```

Une référence vers un trait object contient deux pointeurs :

```
&dyn Animal
┌────────────┬────────────┐
│ données ───┼──► Chien   │
│ vtable  ───┼──► [ drop, taille, alignement, nom(), cri() ] pour Chien
└────────────┴────────────┘
```

### 5.2 Le coût

| | Appel statique (générique, type concret) | Appel dynamique (`dyn`) |
|---|---|---|
| Choix de la fonction | à la compilation | à l'exécution, via la vtable |
| Intégration (inlining) | possible | impossible en général |
| Taille du code | une copie par type | une seule copie |
| Coût d'un appel | nul | une lecture mémoire + un appel indirect |

Ce coût est négligeable devant une requête réseau ou une lecture en base. Il compte dans une boucle très serrée (millions d'appels par seconde).

## 6. Compatibilité `dyn` d'un trait

Tous les traits ne peuvent pas devenir des trait objects. Un trait est « dyn-compatible » (anciennement *object safe*) si ses méthodes peuvent être appelées sans connaître le type concret.

### 6.1 Ce qui empêche `dyn`

```rust
trait Copiable {
    fn copier(&self) -> Self;                  // renvoie Self : taille inconnue derrière dyn
}

trait Convertisseur {
    fn convertir<T>(&self, x: T) -> String;    // méthode générique : infinité de versions
}

trait Constructible {
    fn nouveau() -> Self;                      // pas de self : rien à appeler sur un objet
}

// let x: Box<dyn Copiable>;                   // ERREUR E0038 : the trait cannot be made into an object
```

| Règle | Exemple refusé |
|---|---|
| pas de méthode qui renvoie `Self` | `fn clone(&self) -> Self` |
| pas de méthode générique | `fn f<T>(&self, x: T)` |
| pas de fonction associée sans `self` | `fn new() -> Self` |
| pas de constante associée | `const N: usize;` |
| le trait ne doit pas exiger `Sized` | `trait T: Sized` |

C'est pourquoi `Clone` n'est pas dyn-compatible : `Box<dyn Clone>` n'existe pas.

### 6.2 Exclure une méthode avec `where Self: Sized`

```rust
trait Forme {
    fn aire(&self) -> f64;

    fn nouveau_carre(cote: f64) -> Self        // pas appelable sur un dyn Forme...
    where
        Self: Sized;                           // ...mais n'empêche plus dyn Forme

    fn dupliquer(&self) -> Self where Self: Sized;
}
```

Les méthodes marquées `where Self: Sized` n'existent que sur les types concrets. Le reste du trait reste utilisable en `dyn`.

### 6.3 Remplacer une méthode générique

```rust
// trait Journal { fn ecrire<T: std::fmt::Display>(&self, x: T); }    // pas dyn

trait Journal {
    fn ecrire(&self, x: &dyn std::fmt::Display);                         // dyn
}
```

## 7. Trait objects dans les structs

### 7.1 Un comportement injectable

```rust
trait Notificateur {
    fn envoyer(&self, a: &str, msg: &str) -> Result<(), String>;
}

struct Email;
impl Notificateur for Email {
    fn envoyer(&self, a: &str, msg: &str) -> Result<(), String> {
        println!("email à {a} : {msg}");
        Ok(())
    }
}

struct Sms;
impl Notificateur for Sms {
    fn envoyer(&self, a: &str, msg: &str) -> Result<(), String> {
        println!("sms à {a} : {msg}");
        Ok(())
    }
}

struct Service {
    notificateur: Box<dyn Notificateur>,
}

impl Service {
    fn new(notificateur: Box<dyn Notificateur>) -> Self {
        Self { notificateur }
    }
    fn inscrire(&self, contact: &str) -> Result<(), String> {
        self.notificateur.envoyer(contact, "Bienvenue")
    }
    fn changer(&mut self, n: Box<dyn Notificateur>) {
        self.notificateur = n;               // le comportement change à l'exécution
    }
}

fn main() {
    let mut s = Service::new(Box::new(Email));
    s.inscrire("ada@ex.com").unwrap();
    s.changer(Box::new(Sms));
}
```

### 7.2 La même struct en générique

```rust
struct ServiceGen<N: Notificateur> {
    notificateur: N,
}

impl<N: Notificateur> ServiceGen<N> {
    fn inscrire(&self, contact: &str) -> Result<(), String> {
        self.notificateur.envoyer(contact, "Bienvenue")
    }
}

let s = ServiceGen { notificateur: Email };
// impossible de remplacer Email par Sms dans la même variable : ServiceGen<Email> et ServiceGen<Sms> sont deux types
```

| | `Box<dyn Notificateur>` | `ServiceGen<N>` |
|---|---|---|
| Changer d'implémentation à l'exécution | oui | non |
| Choisir d'après la configuration (`.env`) | oui, facilement | possible avec un `match` qui crée des types différents |
| Paramètre de type à propager | non | oui, partout où `ServiceGen` apparaît |
| Performance | appel indirect | appel direct |

### 7.3 Option et valeur par défaut

```rust
struct Config {
    journal: Option<Box<dyn Fn(&str)>>,      // comportement facultatif
}

impl Config {
    fn log(&self, msg: &str) {
        if let Some(f) = &self.journal {
            f(msg);
        }
    }
}
```

## 8. Collections de trait objects

### 8.1 Des types différents dans un même `Vec`

```rust
trait Forme {
    fn aire(&self) -> f64;
    fn nom(&self) -> &str;
}

struct Carre(f64);
struct Cercle(f64);

impl Forme for Carre {
    fn aire(&self) -> f64 { self.0 * self.0 }
    fn nom(&self) -> &str { "carré" }
}
impl Forme for Cercle {
    fn aire(&self) -> f64 { std::f64::consts::PI * self.0 * self.0 }
    fn nom(&self) -> &str { "cercle" }
}

fn main() {
    let formes: Vec<Box<dyn Forme>> = vec![Box::new(Carre(2.0)), Box::new(Cercle(1.0))];

    let total: f64 = formes.iter().map(|f| f.aire()).sum();
    let plus_grande = formes.iter().max_by(|a, b| a.aire().total_cmp(&b.aire()));
    for f in &formes {
        println!("{} : {:.2}", f.nom(), f.aire());
    }
}
```

### 8.2 Une table de traitements

```rust
use std::collections::HashMap;

type Commande = Box<dyn Fn(&[&str]) -> String>;

fn commandes() -> HashMap<&'static str, Commande> {
    let mut m: HashMap<&'static str, Commande> = HashMap::new();
    m.insert("echo", Box::new(|args| args.join(" ")));
    m.insert("compter", Box::new(|args| args.len().to_string()));
    m
}

fn main() {
    let c = commandes();
    if let Some(f) = c.get("echo") {
        println!("{}", f(&["a", "b"]));
    }
}
```

### 8.3 Ordre d'insertion

`IndexMap` (crate `indexmap`) garde l'ordre d'insertion, ce qu'une `HashMap` ne fait pas. Utile pour une liste de champs d'un formulaire, affichés dans l'ordre déclaré.

## 9. Cloner un trait object

`Clone` n'est pas dyn-compatible (section 6) : `Box<dyn Forme>` ne peut pas être cloné directement.

### 9.1 Écrire la méthode soi-même

```rust
trait Forme {
    fn aire(&self) -> f64;
    fn cloner(&self) -> Box<dyn Forme>;
}

#[derive(Clone)]
struct Carre(f64);

impl Forme for Carre {
    fn aire(&self) -> f64 { self.0 * self.0 }
    fn cloner(&self) -> Box<dyn Forme> { Box::new(self.clone()) }
}

impl Clone for Box<dyn Forme> {
    fn clone(&self) -> Self { self.cloner() }
}
```

### 9.2 Avec la crate `dyn-clone`

```rust
use dyn_clone::DynClone;

trait Forme: DynClone {
    fn aire(&self) -> f64;
}
dyn_clone::clone_trait_object!(Forme);

#[derive(Clone)]
struct Cercle(f64);
impl Forme for Cercle {
    fn aire(&self) -> f64 { 3.14 * self.0 * self.0 }
}

let a: Box<dyn Forme> = Box::new(Cercle(1.0));
let b = a.clone();          // fonctionne : la macro a écrit l'impl Clone
```

La crate génère exactement le code de la section 9.1. Runique l'utilise pour ses champs de formulaire (section 14).

### 9.3 Partager au lieu de cloner

```rust
use std::sync::Arc;

let a: Arc<dyn Forme> = Arc::new(Cercle(1.0));
let b = Arc::clone(&a);     // même objet, compteur de références augmenté
```

Si les copies n'ont pas à être modifiées séparément, `Arc` évite toute copie.

## 10. Retrouver le type concret : `Any`

```rust
use std::any::Any;

trait Composant: Any {
    fn nom(&self) -> &str;
    fn comme_any(&self) -> &dyn Any;
}

struct Bouton { libelle: String }

impl Composant for Bouton {
    fn nom(&self) -> &str { "bouton" }
    fn comme_any(&self) -> &dyn Any { self }
}

fn libelle(c: &dyn Composant) -> Option<&str> {
    c.comme_any().downcast_ref::<Bouton>().map(|b| b.libelle.as_str())
}
```

`downcast_ref` renvoie `Some` si le type concret est le bon, `None` sinon. Si tu en as souvent besoin, c'est le signe qu'une méthode manque dans le trait, ou qu'une enum conviendrait mieux.

## 11. Remonter vers un supertrait

Depuis Rust 1.86, un trait object se convertit vers le trait object d'un **supertrait** :

```rust
trait Nomme {
    fn nom(&self) -> String;
}

trait Animal: Nomme {
    fn cri(&self) -> String;
}

struct Chien;
impl Nomme for Chien { fn nom(&self) -> String { "chien".into() } }
impl Animal for Chien { fn cri(&self) -> String { "wouf".into() } }

fn afficher_nom(n: &dyn Nomme) {
    println!("{}", n.nom());
}

fn main() {
    let a: Box<dyn Animal> = Box::new(Chien);
    afficher_nom(&*a);                         // &dyn Animal -> &dyn Nomme
    let n: Box<dyn Nomme> = a;                 // Box<dyn Animal> -> Box<dyn Nomme>
}
```

Avant 1.86, il fallait ajouter dans le trait une méthode `fn comme_nomme(&self) -> &dyn Nomme`.

## 12. Enum, générique ou `dyn` : la même fonctionnalité trois fois

Un système de paiement avec plusieurs moyens.

```rust
// 1. Enum : ensemble FERMÉ, connu à l'avance
enum Moyen {
    Carte { numero: String },
    Virement { iban: String },
}

impl Moyen {
    fn payer(&self, montant: u32) -> String {
        match self {
            Moyen::Carte { numero } => format!("carte {} : {montant}", &numero[numero.len() - 4..]),
            Moyen::Virement { iban } => format!("virement {iban} : {montant}"),
        }
    }
}

// 2. Trait + générique : ensemble ouvert, un seul type par utilisation
trait Paiement {
    fn payer(&self, montant: u32) -> String;
}

struct Carte { numero: String }
impl Paiement for Carte {
    fn payer(&self, montant: u32) -> String { format!("carte : {montant}") }
}

fn encaisser<P: Paiement>(p: &P, montant: u32) -> String { p.payer(montant) }

// 3. Trait object : ensemble ouvert, types mélangés à l'exécution
fn encaisser_tous(moyens: &[Box<dyn Paiement>], montant: u32) -> Vec<String> {
    moyens.iter().map(|m| m.payer(montant)).collect()
}
```

| Critère | Enum | Générique | `dyn Trait` |
|---|---|---|---|
| Ajouter un cas sans toucher au code existant | non (modifier l'enum et les `match`) | oui (nouvel `impl`) | oui (nouvel `impl`) |
| Ajouter une opération à tous les cas | facile (un `match` de plus) | modifier le trait et chaque `impl` | idem |
| Le compilateur vérifie que tous les cas sont traités | oui | — | — |
| Types mélangés dans une collection | oui | non | oui |
| Un utilisateur de la bibliothèque peut ajouter un cas | non | oui | oui |
| Performance | `match` direct | appel direct | appel indirect |
| Pas d'allocation | oui | oui | non (`Box`) sauf `&dyn` |

Règle : **enum** si les cas sont connus et peu nombreux (états, commandes, types de message) ; **générique** pour une abstraction choisie à la compilation ; **`dyn`** pour des plugins, des comportements choisis à l'exécution ou une collection hétérogène.

## 13. Patterns de conception

### 13.1 Stratégie : changer un algorithme à l'exécution

C'est le `Service` de la section 7.1 : l'algorithme (email, SMS) est un trait object remplaçable.

### 13.2 État : chaque état décide du suivant

```rust
trait Etat {
    fn publier(self: Box<Self>) -> Box<dyn Etat>;
    fn nom(&self) -> &'static str;
}

struct Brouillon;
struct EnRelecture;
struct Publie;

impl Etat for Brouillon {
    fn publier(self: Box<Self>) -> Box<dyn Etat> { Box::new(EnRelecture) }
    fn nom(&self) -> &'static str { "brouillon" }
}
impl Etat for EnRelecture {
    fn publier(self: Box<Self>) -> Box<dyn Etat> { Box::new(Publie) }
    fn nom(&self) -> &'static str { "en relecture" }
}
impl Etat for Publie {
    fn publier(self: Box<Self>) -> Box<dyn Etat> { self }
    fn nom(&self) -> &'static str { "publié" }
}

struct Article { etat: Option<Box<dyn Etat>> }

impl Article {
    fn publier(&mut self) {
        if let Some(e) = self.etat.take() {
            self.etat = Some(e.publier());
        }
    }
}
```

`self: Box<Self>` prend possession de l'état pour le remplacer. Le même automate s'écrit plus simplement avec une enum (`enum Etat { Brouillon, EnRelecture, Publie }`) et un `match` : la version `dyn` n'est préférable que si de nouveaux états doivent pouvoir être ajoutés de l'extérieur.

### 13.3 Commande : des actions stockées et annulables

```rust
trait Action {
    fn executer(&mut self, doc: &mut String);
    fn annuler(&mut self, doc: &mut String);
}

struct Ajouter(String);

impl Action for Ajouter {
    fn executer(&mut self, doc: &mut String) { doc.push_str(&self.0); }
    fn annuler(&mut self, doc: &mut String) {
        let n = doc.len() - self.0.len();
        doc.truncate(n);
    }
}

struct Editeur { doc: String, historique: Vec<Box<dyn Action>> }

impl Editeur {
    fn faire(&mut self, mut a: Box<dyn Action>) {
        a.executer(&mut self.doc);
        self.historique.push(a);
    }
    fn defaire(&mut self) {
        if let Some(mut a) = self.historique.pop() {
            a.annuler(&mut self.doc);
        }
    }
}
```

### 13.4 Observateur : prévenir plusieurs abonnés

```rust
struct Evenements {
    abonnes: Vec<Box<dyn Fn(&str) + Send + Sync>>,
}

impl Evenements {
    fn abonner(&mut self, f: impl Fn(&str) + Send + Sync + 'static) {
        self.abonnes.push(Box::new(f));
    }
    fn emettre(&self, e: &str) {
        for a in &self.abonnes {
            a(e);
        }
    }
}
```

## 14. Exemples réels dans Runique

### 14.1 Les champs de formulaire

```rust
// runique/src/forms/base.rs
pub trait FormField: CommonFieldConfig + DynClone + std::fmt::Debug + Send + Sync { /* ... */ }

// runique/src/forms/field.rs
dyn_clone::clone_trait_object!(FormField);

// runique/src/utils/aliases/definition.rs
pub type FieldsMap = IndexMap<String, Box<dyn FormField>>;
```

Un formulaire contient des champs de types différents (`TextField`, `MediaField`, `SelectField`...) dans une seule table ordonnée : c'est exactement le cas d'usage d'une collection de trait objects (section 8). `IndexMap` garde l'ordre de déclaration pour le rendu HTML ; `DynClone` permet de cloner le formulaire entier ; `Send + Sync` permet de le partager entre les tâches du serveur.

### 14.2 Les algorithmes de hachage de mot de passe

```rust
// runique/src/utils/password/mod.rs
pub trait PasswordHandler: Send + Sync + DynClone { /* ... */ }
dyn_clone::clone_trait_object!(PasswordHandler);
```

L'algorithme (Argon2, bcrypt, scrypt) est choisi par la configuration : un trait object permet de le décider au démarrage sans paramètre générique propagé dans tout le framework (le pattern stratégie de la section 13.1).

## 15. Exercices

### Exercice 1 : système de fichiers virtuel

Définis un trait `Noeud` avec `nom(&self) -> &str` et `taille(&self) -> u64`. Implémente `Fichier { nom, taille }` et `Dossier { nom, enfants: Vec<Box<dyn Noeud>> }` dont la taille est la somme des enfants.

```rust
// Solution :
trait Noeud {
    fn nom(&self) -> &str;
    fn taille(&self) -> u64;
}

struct Fichier { nom: String, taille: u64 }
struct Dossier { nom: String, enfants: Vec<Box<dyn Noeud>> }

impl Noeud for Fichier {
    fn nom(&self) -> &str { &self.nom }
    fn taille(&self) -> u64 { self.taille }
}

impl Noeud for Dossier {
    fn nom(&self) -> &str { &self.nom }
    fn taille(&self) -> u64 { self.enfants.iter().map(|e| e.taille()).sum() }
}
```

### Exercice 2 : calculatrice extensible

Crée un trait `Operation { fn symbole(&self) -> char; fn calculer(&self, a: f64, b: f64) -> Option<f64>; }` et une calculatrice qui stocke ses opérations dans une `HashMap<char, Box<dyn Operation>>`. Ajoute l'addition et la division (qui renvoie `None` sur zéro).

```rust
// Solution :
use std::collections::HashMap;

trait Operation {
    fn symbole(&self) -> char;
    fn calculer(&self, a: f64, b: f64) -> Option<f64>;
}

struct Addition;
struct Division;

impl Operation for Addition {
    fn symbole(&self) -> char { '+' }
    fn calculer(&self, a: f64, b: f64) -> Option<f64> { Some(a + b) }
}
impl Operation for Division {
    fn symbole(&self) -> char { '/' }
    fn calculer(&self, a: f64, b: f64) -> Option<f64> { (b != 0.0).then(|| a / b) }
}

struct Calculatrice { ops: HashMap<char, Box<dyn Operation>> }

impl Calculatrice {
    fn new() -> Self { Self { ops: HashMap::new() } }
    fn ajouter(&mut self, op: Box<dyn Operation>) { self.ops.insert(op.symbole(), op); }
    fn calculer(&self, a: f64, s: char, b: f64) -> Option<f64> {
        self.ops.get(&s)?.calculer(a, b)
    }
}
```

### Exercice 3 : même problème, enum

Réécris l'exercice 2 avec une enum `Operation { Addition, Division }`. Quelle version permet à un utilisateur de ta bibliothèque d'ajouter la multiplication sans modifier ton code ?

Réponse : la version `dyn`. Avec l'enum, ajouter une opération demande de modifier l'enum et son `match`.

## 16. Points clés à retenir

- `Box<T>` place une valeur sur le tas et la possède ; elle sert aux types récursifs, aux grosses valeurs, aux trait objects.
- `dyn Trait` se manipule toujours derrière un pointeur : `&dyn`, `Box<dyn>`, `Arc<dyn>`.
- Un pointeur vers un trait object fait deux mots : les données et la vtable.
- Un trait n'est utilisable en `dyn` que sans `Self` en retour, sans méthode générique, sans fonction sans `self` (sauf `where Self: Sized`).
- `Clone` sur un trait object : méthode `cloner` ou crate `dyn-clone`.
- Enum pour un ensemble fermé, générique pour un choix à la compilation, `dyn` pour un ensemble ouvert ou mélangé à l'exécution.
- Ajoute `Send + Sync` aux trait objects partagés dans un serveur.
