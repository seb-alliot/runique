# Structures de contrôle en Rust

**Conditions, boucles et pattern matching** — toutes les façons d'écrire le flux de contrôle, et quand choisir laquelle.

## Objectifs du cours

À la fin de ce cours, tu sauras :

- Écrire une condition avec `if`, `else if`, `else`, et l'utiliser comme une valeur
- Choisir entre `loop`, `while`, `while let` et `for`
- Itérer de toutes les manières : par valeur, par référence, par référence mutable, avec index, à l'envers, par pas
- Sortir d'une boucle avec une valeur, viser une boucle précise avec un label, sortir d'un bloc
- Utiliser `match`, ses gardes, `matches!`, `if let`, `let ... else` et les let-chains de l'édition 2024
- Remplacer une boucle par un itérateur quand c'est plus clair

## Table des matières

1. Les conditions
2. `if` comme expression et ses alternatives
3. `loop`
4. `while`
5. `for`
6. `break`, `continue` et labels
7. Boucles ou itérateurs : la même fonction écrite de plusieurs façons
8. `match`
9. `if let`, `while let`, `let ... else`
10. Let-chains (édition 2024)
11. `matches!`
12. Exemples pratiques
13. Exercices
14. Aide-mémoire

## 1. Les conditions

Une condition exécute un bloc selon qu'une expression vaut `true` ou `false`. En Rust, la condition est **toujours** un `bool` : il n'y a aucune conversion implicite.

### 1.1 — `if` simple

```rust
fn main() {
    let nombre = 7;
    if nombre < 10 {
        println!("Le nombre est petit");
    }
}
```

Pas de parenthèses autour de la condition (le compilateur les accepte mais `clippy` les signale), accolades **obligatoires** même pour une seule instruction.

### 1.2 — Pas de « valeur vraie » implicite

```rust
let x = 5;
// if x { }            // ERREUR : expected `bool`, found integer
if x != 0 { }          // OK

let nom = String::new();
// if nom { }          // ERREUR aussi
if !nom.is_empty() { } // OK

let valeur: Option<i32> = None;
// if valeur { }       // ERREUR
if valeur.is_some() { } // OK
```

À quoi ça sert : en JavaScript ou en Python, `0`, `""` ou `null` sont « faux », ce qui cache des bugs (un compteur à zéro traité comme absent). En Rust, tu écris toujours le test que tu veux vraiment.

### 1.3 — `else` et `else if`

```rust
fn main() {
    let nombre = 7;

    if nombre % 2 == 0 {
        println!("Pair");
    } else {
        println!("Impair");
    }

    if nombre < 0 {
        println!("Négatif");
    } else if nombre == 0 {
        println!("Zéro");
    } else {
        println!("Positif");
    }
}
```

Les branches sont testées dans l'ordre et **seule la première vraie** s'exécute.

### 1.4 — Opérateurs logiques et court-circuit

```rust
let age = 20;
let inscrit = true;

if age >= 18 && inscrit { }   // ET : les deux doivent être vrais
if age < 12 || age > 65 { }   // OU : l'un des deux suffit
if !inscrit { }               // NON
```

`&&` et `||` **court-circuitent** : la partie droite n'est évaluée que si nécessaire.

```rust
let v: Vec<i32> = vec![];
// Sans court-circuit, v[0] paniquerait sur un vecteur vide
if !v.is_empty() && v[0] > 10 {
    println!("premier élément grand");
}
```

À quoi ça sert : mettre le test « est-ce que je peux » à gauche et le test « est-ce que je dois » à droite. `&` et `|` existent aussi sur les `bool`, mais n'interrompent pas l'évaluation : on ne les utilise pas pour des conditions.

### 1.5 — Comparer trois cas : `if` en chaîne ou `cmp`

```rust
use std::cmp::Ordering;

fn comparer(a: i32, b: i32) -> &'static str {
    // Écriture 1 : if en chaîne
    if a < b {
        "plus petit"
    } else if a > b {
        "plus grand"
    } else {
        "égal"
    }
}

fn comparer_bis(a: i32, b: i32) -> &'static str {
    // Écriture 2 : match sur Ordering, le compilateur vérifie que les 3 cas sont traités
    match a.cmp(&b) {
        Ordering::Less => "plus petit",
        Ordering::Greater => "plus grand",
        Ordering::Equal => "égal",
    }
}
```

La seconde forme est préférable quand les trois cas comptent : impossible d'en oublier un, et `a` et `b` ne sont comparés qu'une fois.

## 2. `if` comme expression et ses alternatives

En Rust, `if` **produit une valeur**. C'est ce qui remplace l'opérateur ternaire `cond ? a : b`, qui n'existe pas.

### 2.1 — Affecter le résultat d'un `if`

```rust
fn main() {
    let age = 20;

    // Écriture 1 : variable mutable puis affectation (style C, à éviter)
    let mut statut = "";
    if age >= 18 {
        statut = "Majeur";
    } else {
        statut = "Mineur";
    }

    // Écriture 2 : if expression, la variable reste immuable
    let statut = if age >= 18 { "Majeur" } else { "Mineur" };
    println!("{statut}");
}
```

La deuxième écriture est la forme idiomatique : la variable n'est jamais dans un état intermédiaire et reste immuable.

### 2.2 — Les branches doivent avoir le même type

```rust
let condition = true;
let x = if condition {
    5
} else {
    // "six"   // ERREUR : `if` and `else` have incompatible types
    6
};
```

Un `if` **sans `else`** vaut `()` (le type unit). On ne peut donc pas écrire `let x = if c { 5 };` : il manquerait la valeur du cas faux.

### 2.3 — Une branche qui ne revient pas

Une branche qui fait `return`, `panic!`, `break` ou `continue` a le type `!` (« never ») : elle est compatible avec n'importe quel type.

```rust
fn diviser(a: i32, b: i32) -> Result<i32, String> {
    let diviseur = if b != 0 { b } else { return Err("division par zéro".into()) };
    Ok(a / diviseur)
}
```

### 2.4 — Le dernier élément d'un bloc

La valeur d'un bloc est sa dernière expression **sans point-virgule**.

```rust
let x = if true { 5 } else { 6 };    // x = 5
let y = if true { 5; } else { 6; };  // y = () : le ; transforme l'expression en instruction
```

### 2.5 — `bool::then` et `then_some` : obtenir une `Option`

```rust
let age = 20;

// Écriture 1 : if / else
let remise = if age < 26 { Some(10) } else { None };

// Écriture 2 : then_some, la valeur est calculée dans tous les cas
let remise = (age < 26).then_some(10);

// Écriture 3 : then, la closure n'est appelée que si la condition est vraie
let remise = (age < 26).then(|| calcul_couteux());

fn calcul_couteux() -> i32 { 10 }
```

À quoi ça sert : construire une `Option` à partir d'une condition, souvent dans une chaîne d'itérateurs (`filter_map(|x| (x > 0).then_some(x * 2))`). Utilise `then` dès que la valeur coûte quelque chose à calculer.

## 3. `loop`

`loop` répète un bloc **sans condition**. On en sort avec `break`, `return` ou `?`.

### 3.1 — Boucle infinie et sortie

```rust
fn main() {
    let mut compteur = 0;
    loop {
        compteur += 1;
        if compteur == 5 {
            break;
        }
    }
    println!("{compteur}"); // 5
}
```

### 3.2 — `loop` qui renvoie une valeur

`break expr` fait de la boucle une expression. **Seul `loop`** le permet.

```rust
fn main() {
    let mut compteur = 0;
    let resultat = loop {
        compteur += 1;
        if compteur == 10 {
            break compteur * 2;
        }
    };
    println!("{resultat}"); // 20
}
```

Pourquoi pas `for` ou `while` ? Ces boucles peuvent se terminer sans passer par `break` (la condition devient fausse, l'itérateur est vide) : il n'y aurait alors pas de valeur. Le compilateur refuse donc `break 5` dans un `while` (erreur E0571). `loop`, lui, ne se termine **que** par un `break`.

### 3.3 — À quoi sert `loop` : réessayer

```rust
fn connexion() -> Result<&'static str, &'static str> { Err("refus") }

fn main() {
    let mut essais = 0;
    let connexion = loop {
        essais += 1;
        match connexion() {
            Ok(c) => break Ok(c),
            Err(_) if essais < 3 => continue,
            Err(e) => break Err(e),
        }
    };
    println!("{connexion:?}"); // Err("refus")
}
```

### 3.4 — À quoi sert `loop` : serveur ou tâche de fond

```rust
use std::time::Duration;

async fn nettoyage_periodique() {
    let mut intervalle = tokio::time::interval(Duration::from_secs(60));
    loop {
        intervalle.tick().await;
        // purge des sessions expirées, etc.
    }
}
```

Une tâche qui tourne tant que le programme vit s'écrit avec `loop`, jamais avec `while true` (que `clippy` signale d'ailleurs : `while_true`).

### 3.5 — `loop` avec un état : machine à états

```rust
enum Etat { Debut, Lecture(u32), Fin }

fn main() {
    let mut etat = Etat::Debut;
    loop {
        etat = match etat {
            Etat::Debut => Etat::Lecture(0),
            Etat::Lecture(n) if n < 3 => Etat::Lecture(n + 1),
            Etat::Lecture(_) => Etat::Fin,
            Etat::Fin => break,
        };
    }
}
```

## 4. `while`

`while` répète un bloc **tant qu'une condition est vraie**, testée avant chaque tour.

### 4.1 — Forme de base

```rust
fn main() {
    let mut nombre = 3;
    while nombre != 0 {
        println!("{nombre}!");
        nombre -= 1;
    }
    println!("Décollage !");
}
```

### 4.2 — `while` est un `loop` avec un `if`

Ces deux boucles sont équivalentes :

```rust
let mut n = 3;
while n > 0 {
    n -= 1;
}

let mut n = 3;
loop {
    if !(n > 0) {
        break;
    }
    n -= 1;
}
```

`while` vaut toujours `()` : on ne peut pas en sortir avec une valeur.

### 4.3 — Quand utiliser `while`

Quand le nombre de tours **dépend d'un état** qu'on ne connaît pas à l'avance : une saisie, une convergence, un tampon qu'on vide.

```rust
use std::io;

fn main() {
    let mut input = String::new();
    let mut essais = 3;
    while essais > 0 {
        println!("Mot de passe ({essais} essais restants) :");
        input.clear();
        io::stdin().read_line(&mut input).expect("erreur de lecture");
        if input.trim() == "secret" {
            println!("Accès autorisé");
            break;
        }
        essais -= 1;
    }
}
```

```rust
// Convergence : on ne sait pas combien de tours il faudra
fn racine_carree(x: f64) -> f64 {
    let mut estimation = x / 2.0;
    while (estimation * estimation - x).abs() > 1e-10 {
        estimation = (estimation + x / estimation) / 2.0;
    }
    estimation
}
```

### 4.4 — `while` avec index : le piège

```rust
let nombres = [10, 20, 30];

// Écriture 1 : while avec index — fonctionne mais fragile
let mut i = 0;
while i < nombres.len() {
    println!("{}", nombres[i]);
    i += 1; // l'oublier = boucle infinie ; se tromper de borne = panique
}

// Écriture 2 : for — même résultat, aucune borne à gérer
for n in nombres {
    println!("{n}");
}
```

Chaque `nombres[i]` est vérifié à l'exécution (panique si hors limites). `for` sur la collection supprime à la fois le risque et la vérification.

## 5. `for`

`for` parcourt **tout ce qui implémente `IntoIterator`** : plages, tableaux, `Vec`, `HashMap`, `String` (via `.chars()`), itérateurs.

### 5.1 — Ce que fait réellement `for`

```rust
for x in collection {
    // corps
}

// est transformé par le compilateur en :
let mut iter = IntoIterator::into_iter(collection);
loop {
    match iter.next() {
        Some(x) => { /* corps */ }
        None => break,
    }
}
```

Tout le comportement de `for` découle de là : ce que tu donnes après `in` détermine si la collection est consommée, lue ou modifiée.

### 5.2 — Les plages

```rust
for i in 0..5 { }       // 0, 1, 2, 3, 4      (fin exclue)
for i in 0..=5 { }      // 0, 1, 2, 3, 4, 5   (fin incluse)
for i in (0..5).rev() { }        // 4, 3, 2, 1, 0
for i in (0..10).step_by(3) { }  // 0, 3, 6, 9
for i in (0..=10).rev().step_by(5) { } // 10, 5, 0
for c in 'a'..='e' { }  // a, b, c, d, e
for i in 5..0 { }       // aucun tour : plage vide, pas d'erreur
```

Pour compter à rebours, il faut `.rev()` : `5..0` ne compte pas à l'envers, il est vide.

### 5.3 — Les trois façons de parcourir une collection

```rust
fn main() {
    let mut fruits = vec![String::from("pomme"), String::from("poire")];

    // 1. Par référence : on lit, la collection reste utilisable
    for fruit in &fruits {          // équivaut à fruits.iter()
        println!("{fruit}");        // fruit : &String
    }

    // 2. Par référence mutable : on modifie sur place
    for fruit in &mut fruits {      // équivaut à fruits.iter_mut()
        fruit.push('!');            // fruit : &mut String
    }

    // 3. Par valeur : on consomme, la collection n'existe plus après
    for fruit in fruits {           // équivaut à fruits.into_iter()
        println!("{fruit}");        // fruit : String (possédée)
    }
    // println!("{:?}", fruits);    // ERREUR : value moved
}
```

| Écriture | Méthode équivalente | Type de l'élément | La collection après |
|---|---|---|---|
| `for x in &v` | `v.iter()` | `&T` | intacte |
| `for x in &mut v` | `v.iter_mut()` | `&mut T` | modifiée |
| `for x in v` | `v.into_iter()` | `T` | détruite |

Règle pratique : commence par `&v`. Passe à `&mut v` pour modifier, à `v` seulement quand tu veux **déplacer** les éléments ailleurs (les mettre dans une autre collection, les envoyer à un thread).

### 5.4 — Modifier les éléments : trois écritures

```rust
let mut v = vec![1, 2, 3];

// Écriture 1 : référence mutable, déréférencement avec *
for x in &mut v {
    *x *= 2;
}

// Écriture 2 : iter_mut explicite
for x in v.iter_mut() {
    *x += 1;
}

// Écriture 3 : sans boucle, avec un itérateur
v.iter_mut().for_each(|x| *x -= 1);

println!("{v:?}"); // [2, 4, 6]
```

### 5.5 — Avec l'index : `enumerate`

```rust
let noms = ["Ada", "Linus", "Grace"];

for (i, nom) in noms.iter().enumerate() {
    println!("{i} : {nom}");
}

// L'index commence à 0 ; pour une numérotation humaine :
for (i, nom) in noms.iter().enumerate() {
    println!("{}. {nom}", i + 1);
}
```

Évite `for i in 0..noms.len() { noms[i] }` : c'est plus long, chaque accès est vérifié, et `clippy` le signale (`needless_range_loop`). L'index n'est utile que lorsque tu en as besoin **en plus** de l'élément.

### 5.6 — Déstructurer dans le `for`

```rust
use std::collections::HashMap;

let points = vec![(0, 1), (2, 3)];
for (x, y) in &points {
    println!("{x}, {y}");
}

let mut ages = HashMap::new();
ages.insert("Ada", 36);
for (nom, age) in &ages {
    println!("{nom} a {age} ans");
}

// &element pour copier un entier au lieu de garder une référence
let nombres = [1, 2, 3];
for &n in &nombres {
    let double: i32 = n * 2; // n : i32 et non &i32
}
```

### 5.7 — Parcourir plusieurs collections

```rust
let noms = ["Ada", "Linus"];
let ages = [36, 54];

// En parallèle : zip s'arrête à la plus courte
for (nom, age) in noms.iter().zip(ages.iter()) {
    println!("{nom} : {age}");
}

// L'une après l'autre : chain
for n in noms.iter().chain(["Grace"].iter()) {
    println!("{n}");
}

// Imbriquées : toutes les combinaisons
for nom in &noms {
    for age in &ages {
        println!("{nom} / {age}");
    }
}
```

### 5.8 — Parcourir une chaîne

```rust
let texte = "café";

for c in texte.chars() { }          // 'c', 'a', 'f', 'é'  (caractères Unicode)
for b in texte.bytes() { }          // 99, 97, 102, 195, 169 (octets UTF-8)
for (i, c) in texte.char_indices() { } // position en octets + caractère
for mot in "un deux trois".split_whitespace() { }
for ligne in "a\nb".lines() { }
```

`for c in texte` ne compile pas : une `&str` n'est pas itérable directement, il faut choisir entre caractères et octets. `é` occupe 2 octets, d'où la différence.

### 5.9 — Fenêtres et paquets

```rust
let v = [1, 2, 3, 4, 5];

for paire in v.windows(2) { }  // [1,2], [2,3], [3,4], [4,5]  (glissant)
for paquet in v.chunks(2) { }  // [1,2], [3,4], [5]           (découpé)
```

À quoi ça sert : `windows` pour comparer chaque élément au suivant (tri vérifié, écarts), `chunks` pour traiter par lots (insertion en base par paquets de 100).

### 5.10 — Modifier la collection pendant qu'on la parcourt

```rust
let mut v = vec![1, 2, 3];

// for x in &v {
//     v.push(*x);   // ERREUR : v est emprunté par la boucle
// }

// Solution 1 : construire une autre collection
let ajouts: Vec<i32> = v.iter().map(|x| x * 10).collect();
v.extend(ajouts);

// Solution 2 : retirer des éléments selon une condition
v.retain(|&x| x % 2 == 0);
```

Le borrow checker empêche ici l'invalidation d'itérateur, un bug classique en C++ ou en Java (`ConcurrentModificationException`).

## 6. `break`, `continue` et labels

### 6.1 — `break` et `continue`

```rust
for i in 1..10 {
    if i % 2 == 0 {
        continue;      // passe au tour suivant
    }
    if i > 7 {
        break;         // sort de la boucle
    }
    println!("{i}");   // 1, 3, 5, 7
}
```

### 6.2 — `continue` pour éviter l'imbrication

```rust
struct Utilisateur { actif: bool, email: Option<String> }

fn envoyer(utilisateurs: &[Utilisateur]) {
    // Écriture 1 : conditions imbriquées
    for u in utilisateurs {
        if u.actif {
            if let Some(email) = &u.email {
                println!("envoi à {email}");
            }
        }
    }

    // Écriture 2 : on écarte les cas inutiles d'abord
    for u in utilisateurs {
        if !u.actif {
            continue;
        }
        let Some(email) = &u.email else { continue };
        println!("envoi à {email}");
    }
}
```

La seconde forme garde le cas utile au premier niveau d'indentation, quel que soit le nombre de conditions.

### 6.3 — Labels : viser une boucle précise

Sans label, `break` et `continue` agissent sur la boucle **la plus proche**. Un label (`'nom:`) permet d'agir sur une boucle extérieure.

```rust
fn main() {
    'lignes: for ligne in 0..3 {
        for colonne in 0..3 {
            if colonne == 1 {
                continue 'lignes;   // passe à la ligne suivante
            }
            if ligne == 2 {
                break 'lignes;      // sort des deux boucles
            }
            println!("({ligne}, {colonne})");
        }
    }
    // Affiche (0, 0) puis (1, 0)
}
```

### 6.4 — Label et valeur

```rust
let grille = [[1, 2], [3, 4]];

let position = 'recherche: loop {
    for (l, ligne) in grille.iter().enumerate() {
        for (c, &val) in ligne.iter().enumerate() {
            if val == 3 {
                break 'recherche Some((l, c));
            }
        }
    }
    break None;
};
println!("{position:?}"); // Some((1, 0))
```

### 6.5 — Bloc étiqueté (Rust 1.65+)

Un bloc ordinaire peut porter un label et être quitté avec `break 'label valeur`. Ce n'est pas une boucle : `continue` y est interdit.

```rust
fn tarif(age: u32, etudiant: bool) -> u32 {
    let prix = 'calcul: {
        if age < 12 {
            break 'calcul 5;
        }
        if etudiant {
            break 'calcul 8;
        }
        12
    };
    prix
}
```

À quoi ça sert : sortir tôt d'un calcul sans en faire une fonction à part ni utiliser `return` (qui quitterait toute la fonction). Une fonction séparée reste souvent plus lisible ; le bloc étiqueté sert quand le calcul utilise beaucoup de variables locales.

### 6.6 — `return` et `?` dans une boucle

`return` quitte la **fonction** entière, quelle que soit la profondeur de la boucle. `?` fait de même en cas d'erreur.

```rust
fn premier_negatif(v: &[i32]) -> Option<usize> {
    for (i, &x) in v.iter().enumerate() {
        if x < 0 {
            return Some(i);
        }
    }
    None
}

fn somme_textes(textes: &[&str]) -> Result<i32, std::num::ParseIntError> {
    let mut total = 0;
    for t in textes {
        total += t.parse::<i32>()?; // la première erreur arrête tout et remonte
    }
    Ok(total)
}
```

## 7. Boucles ou itérateurs : la même fonction écrite de plusieurs façons

Une boucle `for` et une chaîne d'itérateurs produisent le même code machine dans la grande majorité des cas (le compilateur optimise les deux). Le choix est une question de lisibilité.

### 7.1 — Somme de 1 à n

```rust
// Écriture 1 : while
fn somme_while(n: u64) -> u64 {
    let mut i = 1;
    let mut total = 0;
    while i <= n {
        total += i;
        i += 1;
    }
    total
}

// Écriture 2 : for
fn somme_for(n: u64) -> u64 {
    let mut total = 0;
    for i in 1..=n {
        total += i;
    }
    total
}

// Écriture 3 : itérateur
fn somme_iter(n: u64) -> u64 {
    (1..=n).sum()
}

// Écriture 4 : fold, la forme générale de toute accumulation
fn somme_fold(n: u64) -> u64 {
    (1..=n).fold(0, |acc, i| acc + i)
}

// Écriture 5 : formule, sans boucle du tout
fn somme_formule(n: u64) -> u64 {
    n * (n + 1) / 2
}
```

### 7.2 — Chercher un élément

```rust
let v = [3, 8, 12, 5];

// Existe-t-il ?
let mut trouve = false;
for &x in &v {
    if x > 10 { trouve = true; break; }
}
let trouve = v.iter().any(|&x| x > 10);
let contient = v.contains(&12);          // égalité simple

// Tous vérifient-ils ?
let tous_positifs = v.iter().all(|&x| x > 0);

// Lequel ?
let premier_grand: Option<&i32> = v.iter().find(|&&x| x > 10);

// À quelle position ?
let position: Option<usize> = v.iter().position(|&x| x == 12);
```

`any`, `all`, `find` et `position` s'arrêtent au premier résultat, comme un `break`.

### 7.3 — Filtrer et transformer

```rust
let v = vec![1, 2, 3, 4, 5, 6];

// Écriture 1 : boucle et push
let mut carres_pairs = Vec::new();
for x in &v {
    if x % 2 == 0 {
        carres_pairs.push(x * x);
    }
}

// Écriture 2 : filter puis map
let carres_pairs: Vec<i32> = v.iter().filter(|&&x| x % 2 == 0).map(|x| x * x).collect();

// Écriture 3 : filter_map, filtre et transforme en une étape
let carres_pairs: Vec<i32> = v.iter().filter_map(|&x| (x % 2 == 0).then_some(x * x)).collect();
```

### 7.4 — Compter, minimum, maximum

```rust
let v = [4, 9, 1, 7];

let nb_grands = v.iter().filter(|&&x| x > 5).count(); // 2
let min = v.iter().min();                             // Some(1)
let max = v.iter().max();                             // Some(9)

let mots = ["rust", "go", "python"];
let plus_long = mots.iter().max_by_key(|m| m.len());  // Some("python")
```

`min` et `max` renvoient une `Option` : une collection vide n'a pas de minimum.

### 7.5 — `for_each` ou `for` ?

```rust
let v = [1, 2, 3];

for x in &v {
    println!("{x}");
}

v.iter().for_each(|x| println!("{x}"));
```

Préfère `for` dès que le corps fait plus d'une ligne ou utilise `break`, `continue`, `return` ou `?` : ces mots-clés ne traversent pas une closure (`return` dans la closure de `for_each` ne quitte que la closure). `for_each` sert en bout de chaîne d'itérateurs.

### 7.6 — Quelle forme choisir

| Besoin | Forme conseillée |
|---|---|
| Nombre de tours inconnu, arrêt sur un état | `while` ou `loop` |
| Réessayer, tâche permanente, valeur en sortie | `loop` |
| Vider une pile, lire jusqu'à épuisement | `while let` |
| Parcourir une collection avec des effets (afficher, écrire en base) | `for` |
| Calculer une valeur à partir d'une collection | itérateur (`sum`, `filter`, `map`, `collect`) |
| Chercher | `any`, `all`, `find`, `position`, `contains` |

## 8. `match`

`match` compare une valeur à une suite de motifs et exécute la branche du **premier** qui correspond. Le compilateur vérifie que **tous** les cas sont couverts.

### 8.1 — Forme de base et valeur

```rust
let nombre = 2;
let texte = match nombre {
    1 => "un",
    2 => "deux",
    _ => "autre",   // _ : tout le reste
};
```

### 8.2 — Branche sur plusieurs lignes

```rust
match nombre {
    0 => println!("zéro"),
    n => {
        let double = n * 2;
        println!("double : {double}");
    }
}
```

Une branche entre accolades n'a pas besoin de virgule.

### 8.3 — Plusieurs valeurs, plages, liaison

```rust
let n = 42;
match n {
    0 => println!("zéro"),
    1 | 2 | 3 => println!("petit"),           // OU
    4..=9 => println!("chiffre"),             // plage incluse
    x @ 10..=99 => println!("{x} : deux chiffres"), // plage + nom
    _ => println!("grand"),
}
```

### 8.4 — Exhaustivité

```rust
let b = true;
match b {
    true => {}
    false => {}   // les deux cas suffisent, pas besoin de _
}

let octet: u8 = 200;
match octet {
    0..=127 => println!("ASCII"),
    128..=255 => println!("étendu"),   // couvre tout u8 : exhaustif sans _
}
```

À quoi ça sert : si tu ajoutes une variante à un enum, chaque `match` qui ne la traite pas devient une erreur de compilation. C'est la raison d'éviter `_` sur tes propres enums : il ferait taire cette alerte.

### 8.5 — Gardes

Une garde ajoute une condition après le motif.

```rust
let temperature = Some(-3);
match temperature {
    Some(t) if t < 0 => println!("gel"),
    Some(t) if t < 25 => println!("normal"),
    Some(_) => println!("chaud"),
    None => println!("pas de mesure"),
}
```

Le compilateur ne tient pas compte des gardes pour l'exhaustivité : la branche `Some(_)` sans garde reste nécessaire.

### 8.6 — Tuples : combiner plusieurs conditions

```rust
fn fizzbuzz(i: u32) -> String {
    match (i % 3, i % 5) {
        (0, 0) => "FizzBuzz".into(),
        (0, _) => "Fizz".into(),
        (_, 0) => "Buzz".into(),
        _ => i.to_string(),
    }
}
```

### 8.7 — `if` en chaîne ou `match` : la même fonction

```rust
fn categorie(age: u32) -> &'static str {
    if age <= 12 {
        "enfant"
    } else if age <= 17 {
        "adolescent"
    } else if age <= 64 {
        "adulte"
    } else {
        "senior"
    }
}

fn categorie_match(age: u32) -> &'static str {
    match age {
        0..=12 => "enfant",
        13..=17 => "adolescent",
        18..=64 => "adulte",
        65.. => "senior",   // plage ouverte : 65 et au-delà
    }
}
```

Avec `match`, le compilateur signale une plage qui se recoupe avec une autre ou un trou (par exemple si tu oublies 13..=17 sans `_`).

## 9. `if let`, `while let`, `let ... else`

Ces formes sont des raccourcis de `match` quand **un seul motif** t'intéresse.

### 9.1 — `if let`

```rust
let valeur: Option<i32> = Some(5);

// Écriture 1 : match
match valeur {
    Some(n) => println!("{n}"),
    None => {}
}

// Écriture 2 : if let
if let Some(n) = valeur {
    println!("{n}");
}

// Avec else
if let Some(n) = valeur {
    println!("{n}");
} else {
    println!("rien");
}
```

### 9.2 — `else if let` en chaîne

```rust
enum Message { Texte(String), Nombre(i32), Vide }

fn traiter(m: Message) {
    if let Message::Texte(t) = m {
        println!("texte : {t}");
    } else if let Message::Nombre(n) = m {
        println!("nombre : {n}");
    } else {
        println!("autre");
    }
}
```

Dès qu'on enchaîne plusieurs `else if let` sur la même valeur, un `match` est plus clair et vérifie l'exhaustivité.

### 9.3 — `while let`

```rust
let mut pile = vec![1, 2, 3];

// Écriture 1 : loop + match
loop {
    match pile.pop() {
        Some(x) => println!("{x}"),
        None => break,
    }
}

// Écriture 2 : while let
let mut pile = vec![1, 2, 3];
while let Some(x) = pile.pop() {
    println!("{x}"); // 3, 2, 1
}
```

À quoi ça sert : vider une pile ou une file en ajoutant des éléments pendant le parcours (impossible avec `for`), lire un flux jusqu'à sa fin.

```rust
// Parcours en largeur : on ajoute à la file pendant qu'on la vide
use std::collections::VecDeque;

let mut file = VecDeque::from([1]);
while let Some(n) = file.pop_front() {
    if n < 8 {
        file.push_back(n * 2);
    }
    println!("{n}"); // 1, 2, 4, 8
}
```

```rust
// Recevoir des messages jusqu'à la fermeture du canal (tokio)
async fn recevoir(mut rx: tokio::sync::mpsc::Receiver<String>) {
    while let Some(msg) = rx.recv().await {
        println!("{msg}");
    }
}
```

### 9.4 — `let ... else` (Rust 1.65+)

`let ... else` extrait une valeur ou **sort** : le bloc `else` doit obligatoirement diverger (`return`, `break`, `continue`, `panic!`).

```rust
fn lire_id(texte: &str) -> Option<u32> {
    // Écriture 1 : match
    let id = match texte.parse::<u32>() {
        Ok(id) => id,
        Err(_) => return None,
    };

    // Écriture 2 : let else, même effet
    let Ok(id) = texte.parse::<u32>() else {
        return None;
    };

    // Écriture 3 : ok() puis ?, le plus court quand la sortie est None
    let id = texte.parse::<u32>().ok()?;

    Some(id)
}
```

À quoi ça sert : enchaîner les vérifications en début de fonction sans empiler les niveaux d'indentation. La variable extraite reste disponible dans toute la suite de la fonction, contrairement à `if let` où elle n'existe que dans le bloc.

## 10. Let-chains (édition 2024)

Depuis Rust 1.88 en **édition 2024**, on peut enchaîner `let` et conditions avec `&&` dans un `if` ou un `while`.

### 10.1 — La même condition écrite trois fois

```rust
struct Session { utilisateur: Option<u32>, active: bool }

fn verifier(session: &Session) {
    // Écriture 1 : imbrication (seule possibilité avant 2024)
    if let Some(id) = session.utilisateur {
        if session.active {
            if id > 0 {
                println!("utilisateur {id}");
            }
        }
    }

    // Écriture 2 : let-chain
    if let Some(id) = session.utilisateur && session.active && id > 0 {
        println!("utilisateur {id}");
    }

    // Écriture 3 : match avec garde
    match session.utilisateur {
        Some(id) if session.active && id > 0 => println!("utilisateur {id}"),
        _ => {}
    }
}
```

### 10.2 — Plusieurs `let` dans la même chaîne

```rust
fn ville(json: &std::collections::HashMap<String, String>) {
    if let Some(adresse) = json.get("adresse")
        && let Some(ville) = adresse.split(',').nth(1)
        && !ville.trim().is_empty()
    {
        println!("ville : {}", ville.trim());
    }
}
```

Chaque `let` peut utiliser les variables liées par les précédents. La chaîne s'arrête au premier échec.

### 10.3 — Limites

- `||` n'est pas autorisé avec un `let` dans la chaîne : seul `&&` l'est.
- Le crate doit être en `edition = "2024"` dans `Cargo.toml` (voir le cours Cargo).
- Les gardes de `match` n'acceptent pas encore de `let` (`Some(x) if let ...` est instable) : on y écrit seulement des conditions booléennes.

### 10.4 — Portée des temporaires en édition 2024

En édition 2024, les valeurs temporaires créées dans la condition d'un `if let` sont libérées **avant** le bloc `else`, et non plus à la fin de tout le `if`.

```rust
use std::sync::Mutex;

let m = Mutex::new(Some(1));
if let Some(x) = *m.lock().unwrap() {
    println!("{x}");
} else {
    // En édition 2021, le verrou était encore tenu ici : relocker = interblocage.
    // En édition 2024, il est déjà relâché.
    let _ = m.lock().unwrap();
}
```

## 11. `matches!`

`matches!(valeur, motif)` renvoie un `bool` : vrai si la valeur correspond au motif.

### 11.1 — La même fonction écrite trois fois

```rust
enum Role { Admin, Moderateur, Membre }

fn peut_moderer(r: &Role) -> bool {
    // Écriture 1 : match
    match r {
        Role::Admin | Role::Moderateur => true,
        _ => false,
    }
}

fn peut_moderer_bis(r: &Role) -> bool {
    // Écriture 2 : matches!
    matches!(r, Role::Admin | Role::Moderateur)
}
```

`==` ne fonctionne que si le type implémente `PartialEq`, et ne sait pas tester une forme (`Some(_)`) ou une plage. `matches!` sait tout ce que sait un motif.

### 11.2 — Avec plage et garde

```rust
let c = 'x';
let est_minuscule = matches!(c, 'a'..='z');

let note: Option<u32> = Some(15);
let mention = matches!(note, Some(n) if n >= 14);
```

### 11.3 — Dans un filtre

```rust
let roles = [Role::Admin, Role::Membre, Role::Moderateur];
let nb_staff = roles.iter().filter(|r| matches!(r, Role::Admin | Role::Moderateur)).count();
```

## 12. Exemples pratiques

### Exemple 1 — calculatrice

```rust
fn calculer(operateur: char, a: i32, b: i32) -> Option<i32> {
    match operateur {
        '+' => a.checked_add(b),
        '-' => a.checked_sub(b),
        '*' => a.checked_mul(b),
        '/' => a.checked_div(b),
        _ => None,
    }
}

fn calculer_garde(operateur: char, a: i32, b: i32) -> Option<i32> {
    match operateur {
        '+' => Some(a + b),
        '/' if b != 0 => Some(a / b),   // garde à la place d'un if dans la branche
        _ => None,
    }
}

fn main() {
    match calculer('/', 10, 0) {
        Some(n) => println!("Résultat : {n}"),
        None => println!("Opération invalide"),
    }
}
```

`checked_*` renvoie `None` au lieu de paniquer : `checked_div` couvre à la fois la division par zéro et `i32::MIN / -1`, qui dépasse la capacité d'un `i32`. La version avec garde montre la syntaxe, mais elle oublie ce second cas.

### Exemple 2 — nombre premier, trois écritures

```rust
// Écriture 1 : for avec sortie anticipée
fn est_premier(n: u32) -> bool {
    if n < 2 {
        return false;
    }
    for i in 2..n {
        if n % i == 0 {
            return false;
        }
    }
    true
}

// Écriture 2 : on s'arrête à la racine carrée (bien plus rapide)
fn est_premier_rapide(n: u32) -> bool {
    if n < 2 {
        return false;
    }
    let mut i = 2;
    while i * i <= n {
        if n % i == 0 {
            return false;
        }
        i += 1;
    }
    true
}

// Écriture 3 : itérateur
fn est_premier_iter(n: u32) -> bool {
    n >= 2 && (2..).take_while(|i| i * i <= n).all(|i| n % i != 0)
}
```

### Exemple 3 — recherche dans un tableau, quatre écritures

```rust
fn trouver_index(tableau: &[i32], valeur: i32) -> Option<usize> {
    for (index, &element) in tableau.iter().enumerate() {
        if element == valeur {
            return Some(index);
        }
    }
    None
}

fn trouver_index_iter(tableau: &[i32], valeur: i32) -> Option<usize> {
    tableau.iter().position(|&x| x == valeur)
}

// Sur un tableau TRIÉ : recherche dichotomique, O(log n) au lieu de O(n)
fn trouver_index_trie(tableau: &[i32], valeur: i32) -> Option<usize> {
    tableau.binary_search(&valeur).ok()
}

fn main() {
    let nombres = [10, 20, 30, 40, 50];
    if let Some(index) = trouver_index_iter(&nombres, 30) {
        println!("Trouvé à l'index {index}");
    }
}
```

### Exemple 4 — lire une configuration ligne par ligne

```rust
fn lire_config(texte: &str) -> Vec<(String, String)> {
    let mut paires = Vec::new();
    for ligne in texte.lines() {
        let ligne = ligne.trim();
        if ligne.is_empty() || ligne.starts_with('#') {
            continue;
        }
        let Some((cle, valeur)) = ligne.split_once('=') else {
            continue;
        };
        paires.push((cle.trim().to_string(), valeur.trim().to_string()));
    }
    paires
}
```

Les cas à ignorer sont écartés d'abord (`continue`), puis `let ... else` extrait la paire.

## 13. Exercices

### Exercice 1 — classification d'âge

Classe un âge : 0-12 enfant, 13-17 adolescent, 18-64 adulte, 65 et plus senior. Écris-le avec `if`, puis avec `match`.

```rust
// Solution :
fn classifier_age(age: u32) -> &'static str {
    match age {
        0..=12 => "Enfant",
        13..=17 => "Adolescent",
        18..=64 => "Adulte",
        65.. => "Senior",
    }
}
```

### Exercice 2 — somme des multiples

Calcule la somme des nombres de 1 à n divisibles par 3 ou par 5, d'abord avec `for`, puis avec un itérateur.

```rust
// Solution :
fn somme_for(n: u32) -> u32 {
    let mut total = 0;
    for i in 1..=n {
        if i % 3 == 0 || i % 5 == 0 {
            total += i;
        }
    }
    total
}

fn somme_iter(n: u32) -> u32 {
    (1..=n).filter(|i| i % 3 == 0 || i % 5 == 0).sum()
}
```

### Exercice 3 — première paire

Dans une grille, renvoie la position du premier nombre pair, en sortant des deux boucles dès qu'il est trouvé.

```rust
// Solution :
fn premier_pair(grille: &[Vec<i32>]) -> Option<(usize, usize)> {
    let mut resultat = None;
    'grille: for (l, ligne) in grille.iter().enumerate() {
        for (c, &v) in ligne.iter().enumerate() {
            if v % 2 == 0 {
                resultat = Some((l, c));
                break 'grille;
            }
        }
    }
    resultat
}
```

### Exercice 4 — compte à rebours

Affiche 10, 8, 6, 4, 2, 0 sans `while`.

```rust
// Solution :
fn main() {
    for i in (0..=10).rev().step_by(2) {
        println!("{i}");
    }
}
```

## 14. Aide-mémoire

### Conditions

```rust
if a { } else if b { } else { }
let x = if a { 1 } else { 2 };          // pas d'opérateur ternaire
let o = cond.then_some(valeur);          // Option depuis un bool
let o = cond.then(|| calcul());          // idem, calcul paresseux
```

### Boucles

```rust
loop { break; }                          // sans condition
let v = loop { break 42; };              // loop seul renvoie une valeur
while cond { }                           // tant que
while let Some(x) = pile.pop() { }       // tant que le motif correspond
for i in 0..10 { }                       // 0 à 9
for i in 0..=10 { }                      // 0 à 10
for i in (0..10).rev() { }               // 9 à 0
for i in (0..10).step_by(2) { }          // 0, 2, 4, 6, 8
for x in &v { }                          // lecture
for x in &mut v { *x += 1; }             // modification
for x in v { }                           // consommation
for (i, x) in v.iter().enumerate() { }   // avec index
for (a, b) in x.iter().zip(&y) { }       // en parallèle
'ext: for ... { break 'ext; }            // sortir d'une boucle extérieure
'ext: for ... { continue 'ext; }         // tour suivant de la boucle extérieure
let r = 'b: { break 'b 1; };             // bloc étiqueté
```

### Pattern matching

```rust
match v {
    1 => {}
    2 | 3 => {}
    4..=10 => {}
    x @ 11..=20 => {}
    n if n > 100 => {}
    _ => {}
}
if let Some(x) = opt { }
let Some(x) = opt else { return; };
if let Some(x) = opt && x > 0 { }        // let-chain, édition 2024
matches!(v, Some(1..=5))
```

### Points clés

- Les conditions sont des `bool`, sans conversion implicite.
- `if`, `match`, `loop` et les blocs sont des expressions.
- Seul `loop` peut sortir avec une valeur ; `while` et `for` valent `()`.
- `for x in v` consomme `v` : utilise `&v` ou `&mut v` pour la garder.
- Pour compter à l'envers : `.rev()`, jamais `5..0`.
- `match` est exhaustif : évite `_` sur tes propres enums.
- `let ... else` et `continue` évitent les imbrications.
- Un calcul sur une collection se lit souvent mieux en itérateur ; une boucle avec effets et sorties anticipées reste un `for`.

## Pour aller plus loin

- `pattern-matching.md` : tous les motifs (structs, slices, `ref`, `@`).
- `closures-iterateurs.md` : écrire et combiner des itérateurs.
- `collections.md` : parcourir `Vec`, `HashMap` et `HashSet`.
- `cargo-dependances.md` : choisir l'édition 2024 pour les let-chains.
