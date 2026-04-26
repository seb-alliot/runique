## L'Ownership et le Borrow en Rust
> Le système de propriété — la fondation de la sécurité mémoire sans garbage collector

## Objectifs

- Comprendre ce qu'est l'ownership et pourquoi Rust l'a introduit
- Maîtriser les 3 règles fondamentales de l'ownership
- Comprendre le move semantics
- Distinguer `Clone` et `Copy`
- Utiliser les références `&` et `&mut` correctement
- Appliquer les règles du borrow checker sans se battre contre lui
- Utiliser les slices pour référencer des portions de collections

---

## Table des matières

1. [Qu'est-ce que l'ownership](#1-quest-ce-que-lownership)
2. [Les 3 règles de l'ownership](#2-les-3-règles-de-lownership)
3. [Move semantics — quand une valeur change de propriétaire](#3-move-semantics)
4. [Clone et Copy — dupliquer une valeur](#4-clone-et-copy)
5. [Les références `&` — emprunter sans posséder](#5-les-références)
6. [Les références mutables `&mut`](#6-les-références-mutables)
7. [Les règles du borrow checker](#7-les-règles-du-borrow-checker)
8. [Slices — références sur une partie d'une collection](#8-slices)
9. [Exercices pratiques](#9-exercices-pratiques)
10. [Aide-mémoire](#10-aide-mémoire)

---

## 1. Qu'est-ce que l'ownership

En C/C++, le programmeur gère la mémoire manuellement (`malloc`/`free`).
En Java ou Python, un garbage collector libère la mémoire automatiquement, mais avec un coût à l'exécution.

Rust choisit une troisième voie : l'**ownership** (propriété). Chaque valeur appartient à une variable.
Quand cette variable sort de portée, la mémoire est libérée automatiquement — sans GC, sans fuite.

```rust
fn main() {
    let s = String::from("bonjour"); // s est propriétaire de la chaîne

    // s est utilisable ici
    println!("{s}");

} // s sort de portée → Rust appelle drop() → mémoire libérée automatiquement
```

Ce modèle garantit à la compilation :
- Pas de double `free` (libération de mémoire déjà libérée)
- Pas de use-after-free (utilisation après libération)
- Pas de fuites mémoire (la mémoire est toujours libérée)

---

## 2. Les 3 règles de l'ownership

Ces trois règles sont la base de tout le système. Le compilateur les vérifie à chaque compilation.

> **Règle 1 — Chaque valeur a exactement un propriétaire.**
>
> **Règle 2 — Il ne peut y avoir qu'un seul propriétaire à la fois.**
>
> **Règle 3 — Quand le propriétaire sort de portée, la valeur est détruite.**

```rust
fn main() {
    // Règle 1 : s1 est le seul propriétaire de "hello"
    let s1 = String::from("hello");

    // Règle 2 : si s2 prend la propriété, s1 n'est plus valide
    let s2 = s1; // move — s1 est invalidé

    // println!("{s1}"); // ERREUR : s1 a été déplacé

    println!("{s2}"); // OK

} // Règle 3 : s2 sort de portée → drop() appelé une seule fois
```

Ces règles semblent contraignantes, mais elles permettent au compilateur de garantir la sécurité
mémoire sans aucun runtime.

---

## 3. Move semantics

Quand on assigne une valeur à une autre variable ou qu'on la passe à une fonction,
la **propriété est transférée** (moved). L'ancienne variable devient inutilisable.

```rust
fn afficher(texte: String) {
    println!("{texte}");
} // texte est dropped ici

fn main() {
    let message = String::from("bonjour");

    afficher(message); // move : message passe dans afficher

    // println!("{message}"); // ERREUR : message a été moved
}
```

Ce comportement s'applique à tous les types qui possèdent des ressources (`String`, `Vec`, `Box`, etc.).

```rust
fn main() {
    let v1 = vec![1, 2, 3];
    let v2 = v1; // v1 est moved dans v2

    // v1 n'est plus utilisable
    // v2 est le seul propriétaire du vecteur

    for n in &v2 {
        println!("{n}");
    }
}
```

### Pourquoi ce choix ?

Sans move semantics, deux variables pourraient croire posséder la même donnée sur le tas.
Rust l'interdit pour éviter le double `free`.

---

## 4. Clone et Copy

### Clone — copie explicite profonde

`Clone` permet de créer une copie indépendante d'une valeur. C'est explicite et potentiellement coûteux.

```rust
fn main() {
    let s1 = String::from("hello");
    let s2 = s1.clone(); // copie profonde — allocation sur le tas

    // Les deux sont valides et indépendants
    println!("s1 = {s1}");
    println!("s2 = {s2}");
}
```

### Copy — copie implicite légère

Le trait `Copy` s'applique aux types stockés entièrement sur la pile (entiers, flottants, booléens, char, tuples de `Copy`).
Ces types sont copiés automatiquement à l'assignation — pas de move.

```rust
fn main() {
    let x = 5;
    let y = x; // x est copié, pas moved

    // Les deux sont valides
    println!("x = {x}");
    println!("y = {y}");
}

fn doubler(n: i32) -> i32 {
    n * 2
}

fn main() {
    let a = 10;
    let b = doubler(a); // a est copié, pas moved

    println!("a = {a}"); // toujours valide
    println!("b = {b}");
}
```

### Quelle différence ?

| | `Copy` | `Clone` |
|---|---|---|
| Déclenchement | Implicite (assignation) | Explicite (`.clone()`) |
| Coût | Très faible (pile uniquement) | Variable (peut allouer) |
| Types | `i32`, `f64`, `bool`, `char`… | `String`, `Vec`, structs… |

> `String` ne peut pas être `Copy` car elle possède une allocation sur le tas.
> Deux `String` ne peuvent pas pointer vers les mêmes octets.

---

## 5. Les références `&`

Emprunter une valeur avec `&` permet de l'utiliser sans en prendre la propriété.
La valeur originale reste valide après l'emprunt.

```rust
fn longueur(texte: &String) -> usize {
    texte.len()
} // texte est un emprunt — il n'est pas dropped ici

fn main() {
    let s = String::from("hello world");

    let n = longueur(&s); // on passe une référence, pas la valeur

    // s est toujours valide !
    println!("'{s}' contient {n} caractères");
}
```

Une référence est comme un pointeur, sauf qu'elle est garantie valide par le compilateur :
elle ne peut jamais pointer vers une donnée détruite.

```rust
fn main() {
    let r;
    {
        let x = 5;
        // r = &x; // ERREUR : x ne vivra pas assez longtemps
    }
    // println!("{r}"); // x est déjà détruit ici
}
```

### Références vers des types primitifs

En pratique, on préfère `&str` à `&String` et `&[T]` à `&Vec<T>` pour plus de flexibilité.

```rust
// Accepte &String, &str, littéraux — plus général
fn compter_mots(texte: &str) -> usize {
    texte.split_whitespace().count()
}

fn main() {
    let owned = String::from("bonjour le monde");
    let literal = "foo bar baz";

    println!("{}", compter_mots(&owned));  // coerce &String → &str
    println!("{}", compter_mots(literal)); // &str directement
}
```

---

## 6. Les références mutables `&mut`

Pour modifier une valeur empruntée, il faut une référence mutable.

```rust
fn ajouter_monde(texte: &mut String) {
    texte.push_str(", monde");
}

fn main() {
    let mut s = String::from("bonjour");

    ajouter_monde(&mut s);

    println!("{s}"); // bonjour, monde
}
```

Deux conditions sont nécessaires :
1. La variable doit être déclarée `mut`
2. La référence doit être `&mut`

```rust
fn main() {
    let mut v = vec![1, 2, 3];

    // Emprunt mutable
    let premier = &mut v[0];
    *premier = 100; // déréférencement pour modifier

    println!("{:?}", v); // [100, 2, 3]
}
```

---

## 7. Les règles du borrow checker

Le borrow checker applique deux règles strictes pour éviter les data races et l'accès concurrent.

> **Règle A — À tout moment, on peut avoir SOIT plusieurs `&` SOIT exactement une `&mut`, jamais les deux.**
>
> **Règle B — Les références doivent toujours être valides (pas de dangling references).**

```rust
fn main() {
    let mut s = String::from("hello");

    let r1 = &s;     // OK — première référence immuable
    let r2 = &s;     // OK — deuxième référence immuable
    // let r3 = &mut s; // ERREUR — &mut interdit tant que r1 et r2 existent

    println!("{r1} {r2}"); // r1 et r2 utilisées ici — leur scope se termine

    let r3 = &mut s; // OK maintenant — r1 et r2 ne sont plus actives
    r3.push_str(" world");

    println!("{r3}");
}
```

### Non-Lexical Lifetimes (NLL)

Depuis Rust 2018, le compilateur est plus intelligent : une référence se termine à sa **dernière utilisation**,
pas à la fin de son bloc.

```rust
fn main() {
    let mut s = String::from("hello");

    let r1 = &s;
    println!("{r1}"); // dernière utilisation de r1 — r1 est "libérée" ici

    let r2 = &mut s;  // OK : r1 n'est plus active
    r2.push_str("!");

    println!("{r2}");
}
```

### Erreur classique : référence pendante

```rust
// ERREUR — cette fonction ne peut pas exister
fn creer_reference() -> &String {
    let s = String::from("hello");
    &s // s va être dropped à la fin de cette fonction !
}

// Solution : retourner la valeur elle-même
fn creer_string() -> String {
    String::from("hello")
}
```

---

## 8. Slices

Une **slice** est une référence vers une portion contiguë d'une collection.
Elle ne possède pas les données — elle emprunte une vue sur une partie.

### Slices de chaînes `&str`

```rust
fn premier_mot(texte: &str) -> &str {
    let octets = texte.as_bytes();

    for (i, &octet) in octets.iter().enumerate() {
        if octet == b' ' {
            return &texte[0..i]; // slice des i premiers caractères
        }
    }

    texte // toute la chaîne
}

fn main() {
    let phrase = String::from("bonjour monde");
    let mot = premier_mot(&phrase);

    // phrase.clear(); // ERREUR : phrase est empruntée par mot !

    println!("Premier mot : {mot}");
}
```

### Slices de tableaux `&[T]`

```rust
fn somme(nombres: &[i32]) -> i32 {
    nombres.iter().sum()
}

fn main() {
    let v = vec![1, 2, 3, 4, 5];

    let total = somme(&v);          // slice du Vec entier
    let partiel = somme(&v[1..4]);  // slice des éléments 1, 2, 3

    println!("Total : {total}");    // 15
    println!("Partiel : {partiel}"); // 9
}
```

### Syntaxe des ranges pour les slices

```rust
let s = String::from("bonjour");

let debut = &s[0..3];  // "bon"
let fin   = &s[3..];   // "jour"
let tout  = &s[..];    // "bonjour" (équivalent à &s)
```

---

## 9. Exercices pratiques

### Exercice 1 — Move ou Copy ?

Déterminez ce qui compile et pourquoi.

```rust
fn main() {
    // Cas 1
    let a = 42;
    let b = a;
    println!("{a} {b}"); // compile ? oui — i32 est Copy

    // Cas 2
    let s1 = String::from("hello");
    let s2 = s1;
    // println!("{s1}"); // compile ? non — s1 a été moved
    println!("{s2}");

    // Cas 3
    let v = vec![1, 2, 3];
    let v2 = v.clone();
    println!("{v:?} {v2:?}"); // compile ? oui — clone crée une copie
}
```

### Exercice 2 — Corriger le borrow checker

```rust
// Version cassée — corrigez-la
fn main() {
    let mut texte = String::from("hello");

    let r1 = &texte;
    texte.push_str(" world"); // ERREUR : texte emprunté par r1
    println!("{r1}");
}

// Version corrigée
fn main() {
    let mut texte = String::from("hello");

    {
        let r1 = &texte;
        println!("{r1}"); // r1 utilisée ici, scope terminé
    }

    texte.push_str(" world"); // OK : plus aucun emprunt actif
    println!("{texte}");
}
```

### Exercice 3 — Slice et ownership

```rust
fn derniere_ligne(texte: &str) -> &str {
    texte.lines().last().unwrap_or("")
}

fn main() {
    let contenu = String::from("ligne 1\nligne 2\nligne 3");
    let derniere = derniere_ligne(&contenu);

    // contenu est toujours valide
    println!("Dernière ligne : {derniere}");
}
```

---

## 10. Aide-mémoire

| Concept | Syntaxe | Notes |
|---|---|---|
| Référence immuable | `&T` | Lecture seule, plusieurs autorisées |
| Référence mutable | `&mut T` | Écriture, une seule à la fois |
| Move | `let y = x` | x invalide si T n'est pas Copy |
| Clone | `x.clone()` | Copie profonde explicite |
| Copy | automatique | Types simples sur la pile |
| Slice de chaîne | `&str` | Vue sur une portion de String |
| Slice de tableau | `&[T]` | Vue sur une portion de Vec/tableau |
| Déréférencement | `*ref` | Accéder à la valeur derrière une ref |

**Règles à mémoriser :**

- Un seul propriétaire à la fois
- Plusieurs `&` OU une seule `&mut` — jamais les deux
- Les références ne peuvent pas outlive la valeur qu'elles empruntent
- `Copy` = types simples (pile), copie implicite
- `Clone` = types complexes (tas), copie explicite via `.clone()`
- Préférer `&str` à `&String`, `&[T]` à `&Vec<T>` en paramètre

**Erreurs courantes et solutions :**

| Erreur | Cause | Solution |
|---|---|---|
| "value used after move" | Variable moved dans une fonction | Passer `&` ou `.clone()` |
| "cannot borrow as mutable" | Emprunts immuables actifs | Terminer les emprunts immuables avant |
| "does not live long enough" | Référence vers valeur détruite | Retourner la valeur owned |
| "cannot move out of `*ref`" | Move depuis derrière une référence | Cloner ou utiliser une ref |
