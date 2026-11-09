# L'Ownership et le Borrow en Rust
> Qui possède une valeur, quand elle est libérée, comment la prêter — et toutes les façons de résoudre les erreurs du borrow checker.

## Objectifs

- Comprendre la pile, le tas, et pourquoi Rust a besoin de l'ownership
- Appliquer les trois règles de l'ownership
- Savoir quand une valeur est déplacée, copiée ou empruntée
- Prêter une valeur en lecture (`&`) ou en écriture (`&mut`) et connaître les règles d'emprunt
- Lire les erreurs du borrow checker et connaître plusieurs corrections pour chacune
- Utiliser les tranches (`&str`, `&[T]`) pour emprunter une partie d'une valeur
- Choisir la signature d'une fonction : par valeur, `&`, `&mut`

---

## Table des matières

1. Pile, tas et libération de la mémoire
2. Les trois règles de l'ownership
3. Portée et ordre de libération
4. Le déplacement (move)
5. `Copy` et `Clone`
6. Les références `&`
7. Les références `&mut`
8. Les règles du borrow checker
9. Emprunts et structs
10. Les tranches
11. Choisir la signature d'une fonction
12. Les erreurs courantes et leurs corrections
13. Outils pour sortir d'une impasse
14. Exercices
15. Aide-mémoire

---

## 1. Pile, tas et libération de la mémoire

### 1.1 — La pile et le tas

- **La pile** (stack) : mémoire rapide, de taille connue à la compilation. Les entiers, booléens, tableaux de taille fixe, et les « en-têtes » des valeurs y vivent. Elle se libère automatiquement à la sortie de chaque fonction.
- **Le tas** (heap) : mémoire allouée à l'exécution, de taille variable. Le contenu d'une `String`, d'un `Vec`, d'une `Box` y vit.

```rust
let x = 5;                       // entièrement sur la pile
let s = String::from("hello");   // sur la pile : pointeur, longueur, capacité
                                 // sur le tas  : les octets "hello"
```

### 1.2 — Le problème à résoudre

Une mémoire allouée sur le tas doit être libérée **exactement une fois** :

- jamais libérée : fuite mémoire ;
- libérée deux fois : corruption (double free) ;
- utilisée après libération : faille de sécurité (use after free).

| Langage | Solution |
|---|---|
| C | le programmeur appelle `free` à la main |
| Java, Python, Go, JavaScript | un ramasse-miettes cherche périodiquement ce qui n'est plus utilisé |
| Rust | le compilateur sait, pour chaque valeur, **qui** la possède et **quand** la libérer |

L'ownership n'a aucun coût à l'exécution : tout est vérifié à la compilation.

---

## 2. Les trois règles de l'ownership

1. Chaque valeur a **un propriétaire** (une variable, un champ, un élément de collection).
2. Il n'y a **qu'un seul** propriétaire à la fois.
3. Quand le propriétaire sort de portée, la valeur est **libérée** (`drop`).

```rust
fn main() {
    let s = String::from("hello");   // s possède la String
    println!("{s}");
}                                     // s sort de portée : la mémoire est libérée
```

---

## 3. Portée et ordre de libération

### 3.1 — Une portée par bloc

```rust
fn main() {
    let a = String::from("a");
    {
        let b = String::from("b");
        println!("{a} {b}");
    }                              // b est libérée ici
    // println!("{b}");            // ERREUR : b n'existe plus
}                                  // a est libérée ici
```

### 3.2 — Ordre inverse de déclaration

```rust
struct Bruyant(&'static str);

impl Drop for Bruyant {
    fn drop(&mut self) {
        println!("libération de {}", self.0);
    }
}

fn main() {
    let _a = Bruyant("a");
    let _b = Bruyant("b");
    println!("fin de main");
}
// fin de main
// libération de b
// libération de a
```

Les variables sont libérées dans l'**ordre inverse** de leur déclaration : une valeur déclarée après peut emprunter une valeur déclarée avant, celle-ci est donc toujours libérée en dernier. Les champs d'une struct sont libérés dans l'ordre de déclaration.

### 3.3 — Libérer plus tôt

```rust
let gros = vec![0u8; 100_000_000];
traiter(&gros);
drop(gros);               // libère maintenant au lieu d'attendre la fin du bloc
// suite longue qui n'a plus besoin de gros

fn traiter(_: &[u8]) {}
```

`drop(x)` n'a rien de magique : c'est une fonction qui **prend** la valeur et ne fait rien, donc la valeur est libérée à sa sortie. Elle sert aussi à relâcher un verrou avant la fin d'un bloc.

---

## 4. Le déplacement (move)

### 4.1 — À l'affectation

```rust
let s1 = String::from("hello");
let s2 = s1;                       // la String est DÉPLACÉE de s1 vers s2
// println!("{s1}");               // ERREUR E0382 : borrow of moved value
println!("{s2}");
```

Seul l'en-tête (pointeur, longueur, capacité) est recopié ; les octets sur le tas ne bougent pas. Si `s1` restait valide, deux variables posséderaient le même bloc et le libéreraient deux fois. Rust invalide donc `s1`.

### 4.2 — En passant à une fonction

```rust
fn prendre(s: String) {
    println!("{s}");
}                                   // s est libérée ici

fn main() {
    let texte = String::from("hello");
    prendre(texte);                 // déplacée dans la fonction
    // println!("{texte}");         // ERREUR
}
```

### 4.3 — En renvoyant une valeur

```rust
fn creer() -> String {
    let s = String::from("nouveau");
    s                               // déplacée vers l'appelant : pas libérée ici
}

fn ajouter_point(mut s: String) -> String {
    s.push('.');
    s                               // prise puis rendue
}

fn main() {
    let a = creer();
    let a = ajouter_point(a);
}
```

Prendre puis rendre fonctionne, mais c'est lourd : une référence fait la même chose plus simplement (section 6).

### 4.4 — Dans une collection, un champ, une closure

```rust
let nom = String::from("Ada");
let v = vec![nom];                  // déplacée dans le Vec
// println!("{nom}");               // ERREUR

let titre = String::from("x");
let afficher = move || println!("{titre}");   // move : la closure possède titre
afficher();
// println!("{titre}");             // ERREUR
```

### 4.5 — Dans une boucle

```rust
let s = String::from("a");
for _ in 0..3 {
    // prendre(s);                  // ERREUR : déplacée au premier tour, absente au deuxième
    prendre(s.clone());             // une copie à chaque tour
}

fn prendre(_: String) {}
```

### 4.6 — Déplacement partiel

```rust
struct Personne { nom: String, age: u32 }

let p = Personne { nom: String::from("Ada"), age: 36 };
let nom = p.nom;                    // seul le champ nom est déplacé
println!("{}", p.age);              // OK : age est encore là
// println!("{}", p.nom);           // ERREUR : déplacé
// let q = p;                       // ERREUR : p est incomplète
```

### 4.7 — Déplacement conditionnel

```rust
let s = String::from("a");
let condition = true;
if condition {
    drop(s);
}
// println!("{s}");                 // ERREUR : peut-être déplacée
```

Le compilateur raisonne sur **tous** les chemins possibles : si un seul chemin déplace la valeur, elle est considérée comme indisponible après.

---

## 5. `Copy` et `Clone`

### 5.1 — `Copy` : copie implicite

```rust
let x = 5;
let y = x;            // copie : x reste utilisable
println!("{x} {y}");
```

Un type `Copy` est copié bit à bit au lieu d'être déplacé. Sont `Copy` :

- entiers, flottants, `bool`, `char` ;
- références partagées `&T` ;
- tuples et tableaux dont tous les éléments sont `Copy` ;
- tes types avec `#[derive(Clone, Copy)]`, si tous leurs champs sont `Copy`.

Ne sont **pas** `Copy` : `String`, `Vec`, `Box`, `HashMap`, `&mut T`, et tout type qui implémente `Drop`.

### 5.2 — `Clone` : copie explicite

```rust
let s1 = String::from("hello");
let s2 = s1.clone();              // nouvelle allocation, copie des octets
println!("{s1} {s2}");            // les deux sont valides
```

`.clone()` est visible dans le code : c'est voulu, car une copie de données sur le tas a un coût.

### 5.3 — Comparaison

| | `Copy` | `Clone` |
|---|---|---|
| Déclenchement | automatique, à chaque affectation ou passage | appel explicite `.clone()` |
| Coût | quelques octets copiés | dépend du contenu (peut être gros) |
| Types | petits, sans ressource | tout type qui l'implémente |
| Relation | tout `Copy` est aussi `Clone` | |

### 5.4 — Cloner : quand c'est la bonne solution

Cloner n'est pas une faute. C'est la bonne solution quand :

- la valeur est petite (un identifiant, un nom court) ;
- deux parties du programme ont réellement besoin de leur propre copie ;
- l'alternative rendrait le code beaucoup plus compliqué.

C'est la mauvaise quand on clone pour faire taire le compilateur dans une boucle chaude, ou un gros `Vec` à chaque appel alors qu'une référence suffirait.

---

## 6. Les références `&`

### 6.1 — Emprunter au lieu de prendre

```rust
fn longueur(s: &String) -> usize {
    s.len()
}                                 // s est une référence : rien n'est libéré

fn main() {
    let texte = String::from("hello");
    let n = longueur(&texte);     // on prête texte
    println!("{texte} : {n}");    // texte est toujours là
}
```

Une référence `&T` est un pointeur vers une valeur **possédée par quelqu'un d'autre**. Elle ne la libère jamais.

### 6.2 — Plusieurs références partagées

```rust
let s = String::from("hello");
let r1 = &s;
let r2 = &s;
let r3 = &s;
println!("{r1} {r2} {r3}");       // autant de lecteurs que l'on veut
```

### 6.3 — Une référence ne permet pas de modifier

```rust
fn modifier(s: &String) {
    // s.push('!');               // ERREUR : cannot borrow `*s` as mutable
}
```

### 6.4 — Déréférencer

```rust
let x = 5;
let r = &x;
assert_eq!(*r, 5);                // * pour lire la valeur pointée
assert_eq!(r.pow(2), 25);         // les méthodes déréférencent automatiquement
let somme = r + 1;                // les opérateurs aussi, pour les nombres
```

---

## 7. Les références `&mut`

### 7.1 — Prêter en écriture

```rust
fn ajouter_monde(s: &mut String) {
    s.push_str(", world");
}

fn main() {
    let mut texte = String::from("hello");   // la variable doit être mut
    ajouter_monde(&mut texte);               // on prête en écriture
    println!("{texte}");                     // hello, world
}
```

Il faut `mut` à **deux** endroits : sur la variable (`let mut`) et sur l'emprunt (`&mut`).

### 7.2 — Modifier à travers la référence

```rust
fn doubler(n: &mut i32) {
    *n *= 2;                    // * obligatoire pour affecter la valeur pointée
}

let mut x = 5;
doubler(&mut x);
assert_eq!(x, 10);
```

### 7.3 — Une seule à la fois

```rust
let mut s = String::from("a");
let r1 = &mut s;
// let r2 = &mut s;             // ERREUR E0499 : two mutable borrows
r1.push('b');
```

À quoi ça sert : si deux références pouvaient modifier la même valeur, l'une pourrait réallouer le contenu pendant que l'autre le lit. C'est la cause des « data races » entre threads ; Rust les rend impossibles à la compilation.

---

## 8. Les règles du borrow checker

### 8.1 — La règle

À tout instant, pour une valeur donnée :

- soit **une** référence `&mut` ;
- soit **autant** de références `&` que l'on veut ;
- **jamais les deux** en même temps.

Et une référence ne vit jamais plus longtemps que la valeur qu'elle désigne.

### 8.2 — Un emprunt dure jusqu'à sa dernière utilisation

```rust
let mut s = String::from("hello");

let r1 = &s;
let r2 = &s;
println!("{r1} {r2}");           // dernière utilisation de r1 et r2

let r3 = &mut s;                 // OK : r1 et r2 ne sont plus utilisées
r3.push('!');
```

Un emprunt ne dure pas jusqu'à la fin du bloc mais jusqu'à sa **dernière utilisation** (règle appelée NLL, Non-Lexical Lifetimes). Réordonner les lignes suffit souvent à corriger une erreur.

### 8.3 — Pendant qu'une valeur est prêtée, son propriétaire est limité

```rust
let mut v = vec![1, 2, 3];
let premier = &v[0];             // v est prêtée en lecture
// v.push(4);                    // ERREUR E0502 : v ne peut pas être modifiée
println!("{premier}");
v.push(4);                       // OK : premier ne sert plus
```

`push` peut réallouer le tableau : `premier` pointerait alors vers une mémoire libérée.

### 8.4 — Référence pendante

```rust
// fn creer() -> &String {
//     let s = String::from("hello");
//     &s                        // ERREUR : s est libérée à la fin de la fonction
// }

fn creer() -> String {            // correction : renvoyer la valeur elle-même
    String::from("hello")
}
```

### 8.5 — Réemprunt

```rust
fn ajouter(v: &mut Vec<i32>) {
    v.push(1);
}

let mut v = vec![];
let r = &mut v;
ajouter(r);                       // réemprunt temporaire de *r
ajouter(r);                       // r est toujours utilisable
r.push(2);
```

Passer une `&mut` à une fonction ne la déplace pas : Rust crée un réemprunt (`&mut *r`) qui dure le temps de l'appel.

### 8.6 — Une exception commode

```rust
let mut v: Vec<usize> = vec![1, 2, 3];
v.push(v.len());                  // OK : v.len() est évalué avant l'emprunt mutable de push
```

---

## 9. Emprunts et structs

### 9.1 — Champs distincts : accepté directement

```rust
struct Jeu { joueurs: Vec<String>, scores: Vec<u32> }

let mut j = Jeu { joueurs: vec!["a".into()], scores: vec![0] };
let joueurs = &j.joueurs;         // emprunt de j.joueurs seulement
j.scores.push(10);                // OK : champ différent
println!("{joueurs:?}");
```

Le compilateur suit les emprunts **champ par champ**.

### 9.2 — À travers une méthode : refusé

```rust
impl Jeu {
    fn joueurs(&self) -> &Vec<String> { &self.joueurs }
    fn ajouter_score(&mut self, s: u32) { self.scores.push(s); }
}

let joueurs = j.joueurs();        // emprunte TOUT j
// j.ajouter_score(10);           // ERREUR : j est déjà empruntée
println!("{joueurs:?}");
```

Une méthode `&self` emprunte toute la struct : le compilateur ne regarde pas ce que la méthode lit vraiment. Trois corrections :

```rust
// 1. Accéder aux champs directement (dans le module qui les voit)
let joueurs = &j.joueurs;
j.scores.push(10);

// 2. Copier ce dont on a besoin avant
let nb = j.joueurs().len();
j.ajouter_score(10);

// 3. Une méthode qui rend les deux parties en même temps
impl Jeu {
    fn parties(&mut self) -> (&Vec<String>, &mut Vec<u32>) {
        (&self.joueurs, &mut self.scores)
    }
}
let (joueurs, scores) = j.parties();
scores.push(10);
```

---

## 10. Les tranches

Une tranche est une référence vers une **partie contiguë** d'une collection.

### 10.1 — `&str`

```rust
let s = String::from("hello world");
let hello: &str = &s[0..5];
let world: &str = &s[6..];
let tout: &str = &s[..];
let litteral: &str = "texte";     // un littéral est déjà une &str
```

Les index sont en **octets** : couper au milieu d'un caractère accentué fait paniquer.

```rust
let mot = "café";
// let x = &mot[0..4];            // panique : 'é' occupe les octets 3 et 4
let x = mot.get(0..4);            // None, sans panique
let premiers: String = mot.chars().take(3).collect();   // "caf", en caractères
```

### 10.2 — `&[T]`

```rust
let v = vec![1, 2, 3, 4, 5];
let milieu: &[i32] = &v[1..4];    // [2, 3, 4]
let t = [10, 20, 30];
let debut = &t[..2];

let (a, b) = v.split_at(2);       // deux tranches : [1, 2] et [3, 4, 5]
```

### 10.3 — La tranche protège de l'invalidation

```rust
fn premier_mot(s: &str) -> &str {
    s.split(' ').next().unwrap_or("")
}

let mut s = String::from("hello world");
let mot = premier_mot(&s);
// s.clear();                     // ERREUR : mot emprunte encore s
println!("{mot}");
```

Si la fonction renvoyait un index (`usize`), rien n'empêcherait de vider `s` puis d'utiliser l'index devenu faux. La tranche lie le résultat à la donnée.

### 10.4 — Syntaxe des plages

| Écriture | Éléments |
|---|---|
| `&v[a..b]` | de `a` inclus à `b` exclu |
| `&v[a..=b]` | de `a` à `b` inclus |
| `&v[..b]` | du début à `b` exclu |
| `&v[a..]` | de `a` à la fin |
| `&v[..]` | tout |

---

## 11. Choisir la signature d'une fonction

### 11.1 — La même fonction, quatre signatures

```rust
// 1. Prend la String : l'appelant la perd
fn majuscule_v1(s: String) -> String { s.to_uppercase() }

// 2. Emprunte une &String : n'accepte que des String
fn majuscule_v2(s: &String) -> String { s.to_uppercase() }

// 3. Emprunte une &str : accepte String, &str, littéraux, tranches
fn majuscule_v3(s: &str) -> String { s.to_uppercase() }

// 4. Modifie sur place : pas de nouvelle allocation
fn majuscule_v4(s: &mut String) { s.make_ascii_uppercase(); }

fn main() {
    let mut a = String::from("abc");
    majuscule_v3(&a);
    majuscule_v3("littéral");
    majuscule_v4(&mut a);
    let b = majuscule_v1(a);       // a n'est plus utilisable
}
```

### 11.2 — Règles de choix

| La fonction... | Paramètre |
|---|---|
| lit seulement | `&T`, `&str`, `&[T]` |
| modifie la valeur de l'appelant | `&mut T` |
| garde la valeur (la stocke dans une struct, l'envoie à un thread) | `T` par valeur |
| reçoit un petit type `Copy` | `T` par valeur |

Prendre par valeur quand on stocke évite un `clone` caché dans la fonction : l'appelant choisit de cloner ou de céder sa valeur.

```rust
struct Utilisateur { nom: String }

// Mauvais : clone obligatoire dans la fonction, même si l'appelant n'avait plus besoin du nom
fn creer_v1(nom: &str) -> Utilisateur { Utilisateur { nom: nom.to_string() } }

// Bon : l'appelant cède sa String (aucune copie) ou passe un littéral converti
fn creer_v2(nom: String) -> Utilisateur { Utilisateur { nom } }

// Souple : accepte &str et String
fn creer_v3(nom: impl Into<String>) -> Utilisateur { Utilisateur { nom: nom.into() } }
```

---

## 12. Les erreurs courantes et leurs corrections

### 12.1 — E0382 : utilisation après déplacement

```rust
let s = String::from("a");
let t = s;
// println!("{s}");
```

Corrections :

```rust
let t = s.clone();      // 1. copier
let t = &s;             // 2. emprunter
// 3. utiliser s avant de la déplacer
println!("{s}");
let t = s;
```

### 12.2 — E0499 : deux emprunts mutables

```rust
let mut v = vec![1, 2];
let a = &mut v[0];
// let b = &mut v[1];
// *a += *b;
```

Corrections :

```rust
// 1. split_at_mut : deux tranches mutables qui ne se recouvrent pas
let (gauche, droite) = v.split_at_mut(1);
gauche[0] += droite[0];

// 2. Passer par les index
let b = v[1];
v[0] += b;

// 3. swap pour échanger
v.swap(0, 1);
```

### 12.3 — E0502 : lecture et écriture en même temps

```rust
let mut v = vec![1, 2, 3];
// for x in &v {
//     if *x == 2 { v.push(4); }
// }
```

Corrections :

```rust
// 1. Décider d'abord, modifier ensuite
let contient = v.contains(&2);
if contient { v.push(4); }

// 2. Collecter les ajouts puis les appliquer
let ajouts: Vec<i32> = v.iter().filter(|&&x| x == 2).map(|_| 4).collect();
v.extend(ajouts);

// 3. Boucler sur les index quand la modification dépend de l'élément courant
for i in 0..v.len() {
    if v[i] == 2 { v[i] = 20; }
}
```

### 12.4 — E0505 : déplacer une valeur empruntée

```rust
let s = String::from("a");
let r = &s;
// drop(s);
println!("{r}");
```

Correction : terminer l'utilisation de l'emprunt avant de déplacer (`println!` avant `drop`), ou cloner.

### 12.5 — E0507 : sortir une valeur d'une référence

```rust
struct Boite { contenu: String }

fn vider(b: &mut Boite) -> String {
    // b.contenu                  // ERREUR : cannot move out of `b.contenu` which is behind a mutable reference
    std::mem::take(&mut b.contenu)    // prend la valeur, laisse String::new()
}

fn lire(b: &Boite) -> String {
    b.contenu.clone()             // ou renvoyer &str : &b.contenu
}
```

Avec une `Option` : `opt.take()` prend la valeur et laisse `None`.

### 12.6 — E0597 : la valeur ne vit pas assez longtemps

```rust
let r;
{
    let x = 5;
    r = &x;
}                                 // x libérée
// println!("{r}");               // ERREUR : `x` does not live long enough
```

Correction : déclarer `x` dans la même portée que `r` (ou avant), ou stocker la valeur elle-même plutôt qu'une référence.

### 12.7 — E0106 : durée de vie manquante

```rust
// fn plus_long(a: &str, b: &str) -> &str { if a.len() > b.len() { a } else { b } }
fn plus_long<'a>(a: &'a str, b: &'a str) -> &'a str {
    if a.len() > b.len() { a } else { b }
}
```

Quand une fonction renvoie une référence qui peut venir de plusieurs paramètres, il faut dire de quel paramètre elle dépend. Détails dans le cours `lifetimes`.

---

## 13. Outils pour sortir d'une impasse

| Situation | Outil |
|---|---|
| Prendre une valeur derrière `&mut` | `std::mem::take`, `std::mem::replace`, `Option::take` |
| Échanger deux valeurs | `std::mem::swap(&mut a, &mut b)`, `v.swap(i, j)` |
| Deux parties mutables d'une tranche | `split_at_mut`, `chunks_mut`, `iter_mut` |
| Lire ou insérer dans une map en une fois | `map.entry(k)` |
| Plusieurs propriétaires d'une même valeur | `Rc<T>` (un thread), `Arc<T>` (plusieurs threads) |
| Modifier une valeur partagée | `RefCell<T>`, `Mutex<T>`, `RwLock<T>` |
| Éviter une copie quand on ne sait pas si on modifiera | `Cow<str>` |
| Structure avec des liens entre éléments (graphe) | index dans un `Vec` au lieu de références |

```rust
use std::mem;

let mut a = String::from("ancien");
let ancien = mem::replace(&mut a, String::from("nouveau"));
let vide = mem::take(&mut a);          // a devient ""
let mut x = 1;
let mut y = 2;
mem::swap(&mut x, &mut y);
```

`Rc`, `RefCell`, `Arc` et `Mutex` sont dans `smart-pointers` et `concurrence`, `Cow` dans `borrow-avance`.

---

## 14. Exercices

### Exercice 1 — move ou copy ?

Pour chaque ligne, dis si la variable reste utilisable.

```rust
let a = 5;            let b = a;            // a ?
let s = String::new(); let t = s;           // s ?
let c = 'x';          let d = c;            // c ?
let v = vec![1];      let w = &v;           // v ?
let p = (1, 2.0);     let q = p;            // p ?
let r = (1, String::new()); let u = r;      // r ?
```

Réponses : `a` oui (`i32` Copy), `s` non (déplacée), `c` oui, `v` oui (emprunt), `p` oui (tuple de Copy), `r` non (contient une `String`).

### Exercice 2 — corriger sans cloner

```rust
fn main() {
    let mut liste = vec![1, 2, 3];
    let premier = &liste[0];
    liste.push(4);
    println!("{premier}");
}
```

```rust
// Solution 1 : copier l'entier (Copy, gratuit)
let premier = liste[0];
liste.push(4);
println!("{premier}");

// Solution 2 : réordonner
let premier = &liste[0];
println!("{premier}");
liste.push(4);
```

### Exercice 3 — vider une option dans une struct

```rust
struct Session { jeton: Option<String> }

// Écris `fn deconnecter(s: &mut Session) -> Option<String>` qui renvoie le jeton et le retire.
```

```rust
// Solution :
fn deconnecter(s: &mut Session) -> Option<String> {
    s.jeton.take()
}
```

### Exercice 4 — signature

Écris `compter_voyelles` qui accepte une `String`, une `&String` et un littéral sans rien copier.

```rust
// Solution :
fn compter_voyelles(s: &str) -> usize {
    s.chars().filter(|c| "aeiouyAEIOUY".contains(*c)).count()
}
```

---

## 15. Aide-mémoire

```rust
let t = s;              // move (type non Copy)
let t = s.clone();      // copie explicite
let r = &s;             // emprunt en lecture (plusieurs possibles)
let r = &mut s;         // emprunt en écriture (un seul, rien d'autre en même temps)
*r = valeur;            // écrire à travers &mut
drop(s);                // libérer maintenant
std::mem::take(&mut x)  // prendre en laissant Default
opt.take()              // prendre en laissant None
v.split_at_mut(i)       // deux &mut disjoints
```

### Points clés

- Un seul propriétaire ; la valeur est libérée quand il sort de portée.
- Affecter, passer à une fonction ou mettre dans une collection **déplace** un type non `Copy`.
- `Copy` pour les petits types sans ressource, `Clone` explicite pour le reste.
- Une `&mut` ou plusieurs `&`, jamais les deux.
- Un emprunt dure jusqu'à sa dernière utilisation : réordonner corrige souvent l'erreur.
- Une méthode `&self` emprunte toute la struct ; l'accès direct aux champs emprunte champ par champ.
- En paramètre : `&str` et `&[T]` pour lire, `&mut T` pour modifier, `T` pour garder.
