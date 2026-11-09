# Pattern Matching
> Tous les motifs de Rust, tous les endroits où ils s'écrivent, et les différentes façons d'exprimer la même correspondance.

## Objectifs

- Connaître chaque sorte de motif : littéral, plage, variable, joker, tuple, struct, enum, tranche, référence
- Savoir où un motif peut apparaître : `match`, `let`, `if let`, `while let`, `let ... else`, `for`, paramètres de fonction et de closure
- Distinguer motif réfutable et irréfutable
- Utiliser `@`, `|`, `..`, les gardes et `matches!`
- Comprendre les modes de liaison : quand un motif déplace, copie ou emprunte
- Choisir entre `match`, `if let`, `let ... else`, `matches!` et les méthodes d'`Option`

---

## Table des matières

1. L'expression `match`
2. Où écrire un motif
3. Réfutable ou irréfutable
4. Littéraux, constantes et plages
5. Variables, `_` et `..`
6. Tuples
7. Structs
8. Enums
9. Tranches
10. Références et modes de liaison
11. `@` : nommer ce qu'on teste
12. `|` : plusieurs motifs
13. Gardes
14. Chaînes de caractères
15. La même correspondance écrite de plusieurs façons
16. Exercices
17. Aide-mémoire

---

## 1. L'expression `match`

`match` compare une valeur à une liste de bras `motif => expression`, de haut en bas, et exécute **le premier** qui correspond.

```rust
let nombre = 7;

match nombre {
    1 => println!("un"),
    2 | 3 => println!("deux ou trois"),
    4..=6 => println!("entre 4 et 6"),
    n => println!("autre : {n}"),     // une variable correspond à tout
}
```

### Exhaustivité

Le compilateur exige que **toutes** les valeurs possibles soient couvertes.

```rust
let x: Option<i32> = None;
// match x {
//     Some(n) => println!("{n}"),
// }                               // ERREUR : pattern `None` not covered
```

### Ordre des bras

```rust
match nombre {
    n => println!("tout"),
    1 => println!("un"),           // avertissement : unreachable pattern
}
```

Un bras placé après un motif plus général n'est jamais atteint : le compilateur le signale. Mets les cas **particuliers d'abord**.

### `match` est une expression

```rust
let parite = match nombre % 2 {
    0 => "pair",
    _ => "impair",
};
```

Tous les bras doivent produire le même type (ou diverger : `return`, `panic!`, `continue`).

---

## 2. Où écrire un motif

Les motifs ne sont pas réservés à `match`.

```rust
// let
let (a, b) = (1, 2);

// if let / else
if let Some(n) = Some(5) { }

// while let
let mut pile = vec![1, 2];
while let Some(x) = pile.pop() { }

// let ... else
let Some(n) = Some(5) else { panic!() };

// for
for (i, c) in "abc".chars().enumerate() { }

// paramètres de fonction
fn afficher(&(x, y): &(i32, i32)) {
    println!("{x} {y}");
}

// paramètres de closure
let somme = |(a, b): (i32, i32)| a + b;

// matches!
let est_petit = matches!(5, 0..=9);
```

---

## 3. Réfutable ou irréfutable

- Un motif **irréfutable** correspond à toutes les valeurs possibles : `x`, `(a, b)`, `Point { x, y }`.
- Un motif **réfutable** peut échouer : `Some(x)`, `1..=5`, `[premier, ..]`.

| Endroit | Motif accepté |
|---|---|
| `let`, `for`, paramètres de fonction et de closure | irréfutable seulement |
| `if let`, `while let`, `let ... else`, bras de `match` | réfutable (un irréfutable y déclenche un avertissement) |

```rust
let opt = Some(3);
// let Some(x) = opt;              // ERREUR : refutable pattern in local binding
let Some(x) = opt else { return };   // OK : le cas None est traité
if let Some(x) = opt { }             // OK
```

---

## 4. Littéraux, constantes et plages

### 4.1 — Littéraux

```rust
let valeur = 0;
let lettre = 'a';
let drapeau = true;

match valeur {
    0 => {}
    -1 => {}
    _ => {}
}

match lettre {
    'a' => {}
    _ => {}
}

match drapeau {
    true => {}
    false => {}
}
```

### 4.2 — Plages

```rust
let n = 42;
match n {
    0 => "zéro",
    1..=9 => "un chiffre",          // incluse
    10..100 => "deux chiffres",     // exclusive (Rust 1.80+) : 10 à 99
    100.. => "trois ou plus",       // ouverte vers le haut
    ..=-1 => "négatif",             // ouverte vers le bas, incluse
};

let c = 'k';
match c {
    'a'..='z' => "minuscule",
    'A'..='Z' => "majuscule",
    '0'..='9' => "chiffre",
    _ => "autre",
};
```

Les plages fonctionnent sur les entiers et les `char`. Sur les flottants, elles sont acceptées mais déconseillées (problèmes d'arrondi).

### 4.3 — Constantes

```rust
const ADMIN: u32 = 1;
const ANONYME: u32 = 0;

fn role(id: u32) -> &'static str {
    match id {
        ADMIN => "admin",
        ANONYME => "anonyme",
        _ => "membre",
    }
}
```

Une constante en `MAJUSCULES` est comparée à la valeur. Piège : un nom en minuscules est une **nouvelle variable** qui attrape tout.

```rust
let admin = 1;
match 5 {
    admin => println!("toujours ici !"),   // admin est une nouvelle variable, vaut 5
}
```

Le compilateur avertit (`unreachable pattern` sur les bras suivants, variable inutilisée), mais seule une constante (`const`) ou une garde (`x if x == admin`) compare réellement.

---

## 5. Variables, `_` et `..`

### 5.1 — Lier une variable

```rust
match Some(5) {
    Some(n) => println!("{n}"),     // n reçoit la valeur
    None => {}
}
```

### 5.2 — `_` : ignorer une valeur

```rust
let (x, _) = (1, 2);           // le second élément est ignoré
match Some(5) {
    Some(_) => println!("il y a quelque chose"),
    None => {}
}
```

`_` **ne lie rien** : la valeur n'est ni déplacée ni copiée.

```rust
let s = Some(String::from("a"));
if let Some(_) = s { }          // s n'est pas déplacée
println!("{s:?}");              // OK

if let Some(_t) = s { }         // _t est une vraie variable : s est déplacée
// println!("{s:?}");           // ERREUR
```

`_t` supprime seulement l'avertissement de variable inutilisée ; `_` ne lie pas du tout.

### 5.3 — `..` : ignorer le reste

```rust
struct Point3 { x: i32, y: i32, z: i32 }
let p = Point3 { x: 1, y: 2, z: 3 };

let Point3 { x, .. } = p;                // y et z ignorés

let t = (1, 2, 3, 4, 5);
let (premier, .., dernier) = t;          // 1 et 5
let (a, b, ..) = t;                      // 1 et 2
// let (.., m, ..) = t;                  // ERREUR : .. ne peut apparaître qu'une fois
```

---

## 6. Tuples

```rust
let point = (0, 5);

match point {
    (0, 0) => println!("origine"),
    (0, y) => println!("sur l'axe Y à {y}"),
    (x, 0) => println!("sur l'axe X à {x}"),
    (x, y) => println!("({x}, {y})"),
}
```

### Combiner plusieurs valeurs dans un tuple

Matcher sur un tuple créé pour l'occasion permet de tester plusieurs valeurs ensemble.

```rust
fn acces(connecte: bool, admin: bool) -> &'static str {
    match (connecte, admin) {
        (false, _) => "connexion requise",
        (true, false) => "accès membre",
        (true, true) => "accès admin",
    }
}
```

Le compilateur vérifie que les quatre combinaisons sont couvertes.

---

## 7. Structs

### 7.1 — Toutes les écritures

```rust
struct Point { x: i32, y: i32 }
let p = Point { x: 3, y: 7 };

let Point { x, y } = p;                   // raccourci : variables x et y
let Point { x: abs, y: ord } = p;         // renommer
let Point { x, .. } = p;                  // une partie

match p {
    Point { x: 0, y } => println!("sur l'axe Y à {y}"),   // champ fixé + champ lié
    Point { x, y: 0 } => println!("sur l'axe X à {x}"),
    Point { x: 1..=5, y: 1..=5 } => println!("dans le carré"),
    Point { x, y } => println!("({x}, {y})"),
}
```

### 7.2 — Tuple struct et imbrication

```rust
struct Couleur(u8, u8, u8);
struct Pixel { pos: Point, couleur: Couleur }

let px = Pixel { pos: Point { x: 0, y: 0 }, couleur: Couleur(255, 0, 0) };

match px {
    Pixel { couleur: Couleur(255, 0, 0), .. } => println!("rouge"),
    Pixel { pos: Point { x: 0, y: 0 }, couleur: Couleur(r, g, b) } => {
        println!("origine en {r},{g},{b}")
    }
    _ => {}
}
```

Les motifs s'imbriquent à n'importe quelle profondeur.

### 7.3 — Dans les paramètres

```rust
fn norme(&Point { x, y }: &Point) -> f64 {
    ((x * x + y * y) as f64).sqrt()
}
```

---

## 8. Enums

```rust
enum Message {
    Quitter,
    Deplacer { x: i32, y: i32 },
    Ecrire(String),
    Couleur(u8, u8, u8),
}

fn traiter(msg: Message) {
    match msg {
        Message::Quitter => println!("quitter"),
        Message::Deplacer { x, y } => println!("vers {x},{y}"),
        Message::Ecrire(texte) => println!("écrire : {texte}"),
        Message::Couleur(r, _, _) if r > 200 => println!("plutôt rouge"),
        Message::Couleur(r, g, b) => println!("{r},{g},{b}"),
    }
}
```

### Enums imbriquées

```rust
let r: Result<Option<i32>, String> = Ok(Some(3));

match r {
    Ok(Some(n)) if n > 0 => println!("positif {n}"),
    Ok(Some(_)) => println!("négatif ou nul"),
    Ok(None) => println!("absent"),
    Err(e) => println!("erreur {e}"),
}
```

Un seul `match` remplace deux niveaux de `match` ou de `if let`.

---

## 9. Tranches

```rust
fn decrire(v: &[i32]) -> String {
    match v {
        [] => "vide".into(),
        [x] => format!("un seul : {x}"),
        [a, b] => format!("deux : {a}, {b}"),
        [premier, .., dernier] => format!("de {premier} à {dernier}"),
    }
}
```

### Lier le reste avec `@ ..`

```rust
fn somme(v: &[i32]) -> i32 {
    match v {
        [] => 0,
        [tete, reste @ ..] => tete + somme(reste),   // reste : &[i32]
    }
}

fn commande(args: &[&str]) {
    match args {
        ["aide"] => println!("aide"),
        ["ajouter", nom] => println!("ajout de {nom}"),
        ["supprimer", noms @ ..] if !noms.is_empty() => println!("suppression de {noms:?}"),
        _ => println!("commande inconnue"),
    }
}
```

### Sur un `Vec` ou un tableau

```rust
let v = vec![1, 2, 3];
match v.as_slice() {               // un Vec se matche via sa tranche
    [1, ..] => println!("commence par 1"),
    _ => {}
}

let tableau = [1, 2, 3];
let [a, b, c] = tableau;           // taille connue : motif irréfutable
```

---

## 10. Références et modes de liaison

### 10.1 — Le problème

```rust
let opt = Some(String::from("bonjour"));

match opt {
    Some(s) => println!("{s}"),    // s : String — la valeur est DÉPLACÉE dans s
    None => {}
}
// println!("{opt:?}");            // ERREUR : opt a été partiellement déplacé
```

### 10.2 — Trois façons de ne pas déplacer

```rust
let opt = Some(String::from("bonjour"));

// 1. Matcher sur une référence (forme moderne)
match &opt {
    Some(s) => println!("{s}"),    // s : &String
    None => {}
}

// 2. as_ref : Option<String> -> Option<&String>
match opt.as_ref() {
    Some(s) => println!("{s}"),
    None => {}
}

// 3. ref dans le motif (forme ancienne)
match opt {
    Some(ref s) => println!("{s}"),
    None => {}
}

println!("{opt:?}");               // OK dans les trois cas
```

### 10.3 — Modifier à travers le motif

```rust
let mut opt = Some(5);

match &mut opt {
    Some(n) => *n += 1,            // n : &mut i32
    None => {}
}

if let Some(n) = opt.as_mut() {
    *n *= 2;
}

if let Some(ref mut n) = opt {
    *n -= 1;
}
```

### 10.4 — Les règles des modes de liaison

Quand on matche une **référence** avec un motif qui n'en est pas une (`Some(s)` contre `&Option<String>`), Rust passe automatiquement en mode « emprunt » : chaque variable liée devient une référence.

| On matche | Motif | Type de la variable |
|---|---|---|
| `Option<String>` | `Some(s)` | `String` (déplacée) |
| `&Option<String>` | `Some(s)` | `&String` |
| `&mut Option<String>` | `Some(s)` | `&mut String` |
| `Option<String>` | `Some(ref s)` | `&String` |
| `&Option<i32>` | `&Some(n)` | `i32` (copiée) |

### 10.5 — Le motif `&` : déréférencer

```rust
let nombres = vec![1, 2, 3];

for n in &nombres {               // n : &i32
    let _: &i32 = n;
}
for &n in &nombres {              // n : i32 (copié)
    let _: i32 = n;
}

let pairs: Vec<&i32> = nombres.iter().filter(|&&x| x % 2 == 0).collect();
//                                         ^^^ filter reçoit &&i32 : deux & à retirer
```

`&n` ne fonctionne que si la valeur pointée est `Copy` : on ne peut pas sortir une `String` d'une référence.

### 10.6 — Édition 2024 : règles plus strictes

En édition 2024, on ne peut plus écrire `ref`, `ref mut`, `mut` ni `&` dans une partie de motif où Rust est déjà passé en mode emprunt automatique.

```rust
let opt = &Some(5);
// if let Some(ref n) = opt { }    // refusé en 2024 : n est déjà une référence
if let Some(n) = opt { }           // n : &i32
if let &Some(n) = opt { }          // n : i32, le & est au niveau extérieur : accepté
```

`cargo fix --edition` réécrit automatiquement les motifs concernés.

---

## 11. `@` : nommer ce qu'on teste

`nom @ motif` teste le motif **et** lie la valeur entière.

```rust
let age = 15;
match age {
    n @ 0..=12 => println!("enfant de {n} ans"),
    n @ 13..=17 => println!("ado de {n} ans"),
    n => println!("adulte de {n} ans"),
}

// Avec plusieurs alternatives : parenthèses
match age {
    n @ (1 | 5 | 10) => println!("âge rond : {n}"),
    _ => {}
}

// Dans une enum
enum Message { Id { id: i32 } }
let msg = Message::Id { id: 5 };
match msg {
    Message::Id { id: id_variable @ 3..=7 } => println!("id dans la plage : {id_variable}"),
    Message::Id { id } => println!("autre id : {id}"),
}
```

Sans `@`, il faudrait choisir entre tester la plage (`3..=7`) et récupérer la valeur (`id`). La garde est l'autre solution : `Message::Id { id } if (3..=7).contains(&id)`.

---

## 12. `|` : plusieurs motifs

```rust
let n = 1;
let opt = Some(1);

match n {
    1 | 2 | 3 => "petit",
    _ => "autre",
};

// Imbriqué (Rust 1.53+)
match opt {
    Some(1 | 2) => "un ou deux",
    Some(_) => "autre nombre",
    None => "rien",
};

// Avec liaison : chaque alternative doit lier les mêmes variables, du même type
enum Forme { Carre(f64), Cercle(f64), Rectangle(f64, f64) }
fn dimension(f: &Forme) -> f64 {
    match f {
        Forme::Carre(c) | Forme::Cercle(c) => *c,
        Forme::Rectangle(l, _) => *l,
    }
}
```

---

## 13. Gardes

Une garde `if condition` ajoute un test après le motif. Elle peut utiliser les variables liées et toutes les variables visibles.

```rust
let paire = (2, -2);

match paire {
    (x, y) if x == y => println!("égaux"),
    (x, y) if x + y == 0 => println!("opposés"),
    (x, _) if x % 2 == 0 => println!("x pair"),
    _ => println!("autre"),
}
```

### Comparer à une variable

```rust
let cible = 5;
match Some(5) {
    Some(n) if n == cible => println!("trouvé"),
    Some(_) => println!("autre"),
    None => {}
}
```

### Garde et `|`

```rust
let x = 4;
let y = false;
match x {
    4 | 5 | 6 if y => println!("oui"),     // la garde s'applique aux TROIS valeurs
    _ => println!("non"),
}
```

`4 | 5 | 6 if y` signifie `(4 | 5 | 6) if y`, pas `4 | 5 | (6 if y)`.

### Gardes et exhaustivité

Le compilateur ne tient **pas** compte des gardes pour vérifier l'exhaustivité :

```rust
match x {
    n if n > 0 => {}
    n if n <= 0 => {}
    _ => {}          // nécessaire : le compilateur ne sait pas que les deux gardes couvrent tout
}
```

Quand c'est possible, préfère des plages (`1..` et `..=0`) : elles sont vérifiées.

---

## 14. Chaînes de caractères

On ne peut pas matcher une `String` directement contre des littéraux : il faut une `&str`.

```rust
let commande = String::from("stop");

match commande.as_str() {
    "start" => println!("démarrage"),
    "stop" => println!("arrêt"),
    autre => println!("inconnu : {autre}"),
}

// Insensible à la casse
match commande.to_lowercase().as_str() {
    "stop" | "halt" => {}
    _ => {}
}

// Selon un préfixe : garde ou strip_prefix
match commande.as_str() {
    c if c.starts_with("get ") => println!("lecture"),
    _ => {}
}
if let Some(nom) = commande.strip_prefix("get ") {
    println!("lecture de {nom}");
}
```

Sur une `Option<String>`, `as_deref` donne une `Option<&str>` matchable :

```rust
let langue: Option<String> = Some("fr".into());
match langue.as_deref() {
    Some("fr") => "Bonjour",
    Some("en") => "Hello",
    Some(_) | None => "Hello",
};
```

---

## 15. La même correspondance écrite de plusieurs façons

### 15.1 — Extraire une valeur ou sortir

```rust
// 1. match
fn longueur_v1(nom: Option<&str>) -> usize {
    let n = match nom {
        Some(n) => n,
        None => return 0,
    };
    n.len()
}

// 2. let else : même effet, la variable reste au premier niveau
fn longueur_v2(nom: Option<&str>) -> usize {
    let Some(n) = nom else { return 0 };
    n.len()
}

// 3. if let : la suite vit dans le bloc
fn longueur_v3(nom: Option<&str>) -> usize {
    if let Some(n) = nom {
        n.len()
    } else {
        0
    }
}

// 4. méthode d'Option
fn longueur_v4(nom: Option<&str>) -> usize {
    nom.map_or(0, str::len)
}
```

### 15.2 — Tester une forme

```rust
enum Etat { Actif { depuis: u32 }, Suspendu, Supprime }

fn est_actif(e: &Etat) -> bool {
    // 1. match
    match e {
        Etat::Actif { .. } => true,
        _ => false,
    }
}

fn est_actif_bis(e: &Etat) -> bool {
    // 2. matches!
    matches!(e, Etat::Actif { .. })
}

fn est_actif_ancien(e: &Etat) -> bool {
    // 3. let-chain (édition 2024) quand une condition s'ajoute
    if let Etat::Actif { depuis } = e && *depuis > 30 {
        true
    } else {
        false
    }
}

fn est_actif_ancien_bis(e: &Etat) -> bool {
    // 4. matches! avec garde : même chose en une ligne
    matches!(e, Etat::Actif { depuis } if *depuis > 30)
}
```

### 15.3 — Plusieurs niveaux

```rust
struct Requete { utilisateur: Option<Utilisateur> }
struct Utilisateur { email: Option<String> }

fn email(r: &Requete) -> Option<&str> {
    // 1. if let imbriqués
    if let Some(u) = &r.utilisateur {
        if let Some(e) = &u.email {
            return Some(e.as_str());
        }
    }
    None
}

fn email_motif(r: &Requete) -> Option<&str> {
    // 2. un seul motif imbriqué
    match r {
        Requete { utilisateur: Some(Utilisateur { email: Some(e) }) } => Some(e.as_str()),
        _ => None,
    }
}

fn email_chaine(r: &Requete) -> Option<&str> {
    // 3. méthodes
    r.utilisateur.as_ref()?.email.as_deref()
}
```

### 15.4 — Choisir

| Besoin | Forme |
|---|---|
| Plusieurs cas, chacun avec son traitement | `match` |
| Un seul cas déclenche une action | `if let` |
| Le cas manquant arrête la fonction | `let ... else` |
| Obtenir un `bool` | `matches!` |
| Transformer une `Option` / un `Result` | `map`, `and_then`, `?` |
| Un motif plus des conditions | let-chain ou garde |

---

## 16. Exercices

### Exercice 1 — classer un point

Avec un seul `match` sur `(x, y)`, renvoie « origine », « axe X », « axe Y », « quadrant 1 » à « quadrant 4 ».

```rust
// Solution :
fn quadrant(p: (i32, i32)) -> &'static str {
    match p {
        (0, 0) => "origine",
        (_, 0) => "axe X",
        (0, _) => "axe Y",
        (1.., 1..) => "quadrant 1",
        (..=-1, 1..) => "quadrant 2",
        (..=-1, ..=-1) => "quadrant 3",
        (1.., ..=-1) => "quadrant 4",
    }
}
```

Les plages ouvertes rendent le `match` exhaustif sans `_` : le compilateur vérifie qu'aucun point n'est oublié.

### Exercice 2 — analyseur de commande

Matche une tranche `&[&str]` : `["aller", direction]`, `["prendre", objets @ ..]` avec au moins un objet, `["quitter"]`, sinon « inconnu ».

```rust
// Solution :
fn analyser(mots: &[&str]) -> String {
    match mots {
        ["aller", dir] => format!("aller vers {dir}"),
        ["prendre", objets @ ..] if !objets.is_empty() => format!("prendre {}", objets.join(", ")),
        ["quitter"] => "au revoir".into(),
        _ => "inconnu".into(),
    }
}
```

### Exercice 3 — sans déplacer

Affiche le contenu d'un `Option<String>` dans un `match`, puis réutilise la variable. Écris-le de deux façons.

```rust
// Solution :
fn main() {
    let nom = Some(String::from("Ada"));
    match &nom {
        Some(n) => println!("{n}"),
        None => {}
    }
    if let Some(n) = nom.as_deref() {
        println!("{n}");
    }
    println!("{nom:?}");
}
```

## 17. Aide-mémoire

```rust
match v {
    0 => {}                       // littéral
    1 | 2 => {}                   // alternatives
    3..=9 => {}                   // plage incluse
    10..20 => {}                  // plage exclusive
    100.. => {}                   // plage ouverte
    CONSTANTE => {}               // comparaison à une const
    n @ 20..=29 => {}             // test + liaison
    n if n < 0 => {}              // garde
    _ => {}                       // tout le reste, sans lier
}

let (a, _, ..) = tuple;           // ignorer
let Point { x, y: ord, .. } = p;  // struct, renommage
match slice { [] => {}, [x] => {}, [premier, reste @ ..] => {} }
match &opt { Some(s) => {}, None => {} }        // emprunte au lieu de déplacer
match s.as_str() { "a" => {}, _ => {} }         // String contre littéraux
if let Some(x) = opt {}
let Some(x) = opt else { return };
matches!(v, Some(1..=5))
```

### Points clés

- Le premier bras qui correspond gagne : les cas particuliers d'abord.
- Un nom en minuscules dans un motif est une nouvelle variable, jamais une comparaison.
- `_` ne lie rien ; `_x` lie (et peut déplacer).
- Matcher sur `&valeur` évite de déplacer le contenu.
- `@` combine test et liaison ; `|` s'imbrique ; une garde s'applique à toutes les alternatives du bras.
- Les gardes ne comptent pas pour l'exhaustivité : préfère des plages quand c'est possible.
