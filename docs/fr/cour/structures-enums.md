## ■ **Structures et Enums en Rust** 

## Organiser et structurer vos données 

Guide Complet avec Pattern Matching 

## ■ **Objectifs du cours** 

À la fin de ce cours, tu sauras : 

- Créer et utiliser des structures (struct) 

- Définir des méthodes avec impl 

- Créer et utiliser des énumérations (enum) 

- Maîtriser Option<T> et Result<T, E> 

- Utiliser le pattern matching avec match 

## ■ **Table des matières** 

1. Les Structures (struct) 

- 1.1 - Définition de base 

- 1.2 - Instanciation 

- 1.3 - Méthodes et fonctions associées 

- 1.4 - Tuple structs 

- 1.5 - Unit structs 

2. Les Énumérations (enum) 

- 2.1 - Définition de base 

- 2.2 - Enums avec données 

- 2.3 - Option (valeurs optionnelles) 

- 2.4 - Result (gestion d'erreurs) 

3. Pattern Matching 

- 3.1 - L'expression match 

- 3.2 - Patterns avancés 

- 3.3 - if let et while let 

- 3.4 - Déstructuration 

4. Exemples pratiques 

5. Exercices 

6. Aide-mémoire 

## 1. Les Structures (struct)

Les **structures** permettent de regrouper plusieurs données liées ensemble. C'est similaire aux classes dans d'autres langages, mais sans héritage. 

## 1.1 - Définition de base

```
// Définir une structure
struct Utilisateur {
    nom: String,
    email: String,
    age: u32,
    actif: bool,
}
// Structure avec types différents
struct Point {
    x: f64,
    y: f64,
}
struct Rectangle {
    largeur: u32,
    hauteur: u32,
}
```

■ **Convention :** Les noms de struct utilisent **PascalCase** (première lettre de chaque mot en majuscule). 

## 1.2 - Instanciation

```
fn main() {
    // Créer une instance
    let utilisateur1 = Utilisateur {
        nom: String::from("Alice"),
        email: String::from("alice@example.com"),
        age: 25,
        actif: true,
    };
    // Accéder aux champs
    println!("Nom : {}", utilisateur1.nom);
    println!("Email : {}", utilisateur1.email);
    // Instance mutable
    let mut utilisateur2 = Utilisateur {
        nom: String::from("Bob"),
        email: String::from("bob@example.com"),
        age: 30,
        actif: false,
    };
    // Modifier un champ
    utilisateur2.actif = true;
    utilisateur2.age = 31;
}
```

■■ **Important :** Toute l'instance doit être mutable, on ne peut pas rendre seulement certains champs mutables. 

## Raccourcis d'initialisation

```
// Raccourci si variable = nom du champ
fn creer_utilisateur(nom: String, email: String) -> Utilisateur {
    Utilisateur {
        nom,      // Au lieu de nom: nom
        email,    // Au lieu de email: email
        age: 18,
        actif: true,
    }
}
```

```
// Copier depuis une autre instance
fn main() {
    let utilisateur1 = Utilisateur {
        nom: String::from("Alice"),
        email: String::from("alice@example.com"),
        age: 25,
        actif: true,
    };
    // Créer utilisateur2 avec la plupart des champs de utilisateur1
    let utilisateur2 = Utilisateur {
        email: String::from("bob@example.com"),
        ..utilisateur1  // Copie le reste
    };
}
```

## 1.3 - Méthodes et fonctions associées

On utilise `impl` pour définir des méthodes sur une struct. 

```
struct Rectangle {
    largeur: u32,
    hauteur: u32,
}
impl Rectangle {
    // Méthode (prend &self)
    fn aire(&self) -> u32 {
        self.largeur * self.hauteur
    }
    // Méthode avec référence mutable
    fn doubler(&mut self) {
        self.largeur *= 2;
        self.hauteur *= 2;
    }
    // Fonction associée (pas de self)
    fn carre(taille: u32) -> Rectangle {
        Rectangle {
            largeur: taille,
            hauteur: taille,
        }
    }
}
fn main() {
    let rect = Rectangle {
        largeur: 30,
        hauteur: 50,
    };
    println!("Aire : {}", rect.aire());  // 1500
    // Fonction associée (avec ::)
    let carre = Rectangle::carre(20);
    println!("Aire du carré : {}", carre.aire());  // 400
}
```

## ■ **Méthode vs Fonction associée :** 

• **Méthode** : Prend `self` , appelée avec `.` 

• **Fonction associée** : Pas de `self` , appelée avec `::` (comme `String::from` ) 

## 1.4 - Tuple structs

```
// Struct sans noms de champs
struct Couleur(i32, i32, i32);
struct Point(i32, i32, i32);
```

```
fn main() {
    let noir = Couleur(0, 0, 0);
    let origine = Point(0, 0, 0);
    // Accès par index
    println!("Rouge : {}", noir.0);
    println!("X : {}", origine.0);
    // Déstructuration
    let Couleur(r, g, b) = noir;
    println!("RGB : {}, {}, {}", r, g, b);
}
```

## 1.5 - Unit structs

```
// Struct sans champs (pour implémenter des traits)
struct AlwaysEqual;
```

```
fn main() {
    let instance = AlwaysEqual;
}
```

## 2. Les Énumérations (enum)

Les **énumérations** permettent de définir un type avec plusieurs variantes possibles. En Rust, les enums sont très puissants et peuvent contenir des données. 

## 2.1 - Définition de base

```
// Enum simple
enum Mouvement {
    Haut,
    Bas,
    Gauche,
    Droite,
}
fn bouger(direction: Mouvement) {
    // Utilisation avec match
    match direction {
        Mouvement::Haut => println!("On monte"),
        Mouvement::Bas => println!("On descend"),
        Mouvement::Gauche => println!("On va à gauche"),
        Mouvement::Droite => println!("On va à droite"),
    }
}
fn main() {
    let dir = Mouvement::Haut;
    bouger(dir);
}
```

## 2.2 - Enums avec données

C'est  la  vraie  puissance  des  enums  en  Rust  :  chaque  variante  peut  contenir  des  données différentes ! 

```
// Chaque variante peut avoir des données différentes
enum Message {
    Quitter,                        // Pas de données
    Deplacer { x: i32, y: i32 },   // Struct anonyme
    Ecrire(String),                 // String
    ChangerCouleur(i32, i32, i32), // Trois i32
}
impl Message {
    fn appeler(&self) {
        match self {
            Message::Quitter => {
                println!("Quitter l'application");
            }
            Message::Deplacer { x, y } => {
                println!("Déplacer à ({}, {})", x, y);
            }
            Message::Ecrire(texte) => {
                println!("Écrire : {}", texte);
            }
            Message::ChangerCouleur(r, g, b) => {
                println!("Couleur RGB({}, {}, {})", r, g, b);
            }
        }
    }
}
fn main() {
    let msg1 = Message::Ecrire(String::from("Bonjour"));
    let msg2 = Message::Deplacer { x: 10, y: 20 };
    msg1.appeler();  // "Écrire : Bonjour"
    msg2.appeler();  // "Déplacer à (10, 20)"
}
```

## 2.3 - Option<T> (valeurs optionnelles)

`Option<T>` est l'enum le plus important de Rust. Il remplace `null` des autres langages. 

```
// Définition de Option (déjà dans la bibliothèque standard)
enum Option<T> {
    Some(T),  // Contient une valeur
    None,     // Pas de valeur
}
// Utilisation
fn diviser(a: f64, b: f64) -> Option<f64> {
    if b == 0.0 {
        None  // Division par zéro impossible
    } else {
        Some(a / b)
    }
}
fn main() {
    let resultat = diviser(10.0, 2.0);
    match resultat {
        Some(valeur) => println!("Résultat : {}", valeur),
        None => println!("Division par zéro !"),
    }
    // Méthodes pratiques sur Option
    let x: Option<i32> = Some(5);
    println!("{}", x.is_some());     // true
    println!("{}", x.is_none());     // false
    println!("{}", x.unwrap());      // 5 (panic si None !)
    println!("{}", x.unwrap_or(0));  // 5 (ou 0 si None)
    // Map et autres transformations
    let doubled = x.map(|n| n * 2);  // Some(10)
    // Chaînage sûr
    let nombre = Some(5);
    let carre = nombre.map(|n| n * n);  // Some(25)
}
```

## ■■ **unwrap() vs unwrap_or() :** 

• `unwrap()` : Panic si `None` (utiliser seulement si tu es sûr) 

- `unwrap_or(valeur)` : Retourne la valeur par défaut si `None` (plus sûr) 

## 2.4 - Result<T, E> (gestion d'erreurs)

`Result<T, E>` est utilisé pour les opérations qui peuvent échouer. C'est la base de la gestion d'erreurs en Rust. 

```
// Définition de Result
enum Result<T, E> {
    Ok(T),   // Succès avec valeur
    Err(E),  // Erreur
}
// Exemple pratique
use std::fs::File;
use std::io::Error;
fn ouvrir_fichier(nom: &str) -> Result<File, Error> {
    File::open(nom)
}
fn main() {
    match ouvrir_fichier("hello.txt") {
        Ok(fichier) => println!("Fichier ouvert !"),
        Err(erreur) => println!("Erreur : {}", erreur),
    }
    // Raccourci avec ? (propage l'erreur)
    fn lire_fichier() -> Result<String, Error> {
        let mut fichier = File::open("hello.txt")?;  // ? propage l'erreur
        let mut contenu = String::new();
        fichier.read_to_string(&mut contenu)?;
        Ok(contenu)
    }
    // Méthodes pratiques
    let resultat: Result<i32, &str> = Ok(42);
    println!("{}", resultat.is_ok());           // true
    println!("{}", resultat.is_err());          // false
    println!("{}", resultat.unwrap());          // 42
    println!("{}", resultat.unwrap_or(0));      // 42
    println!("{}", resultat.expect("Erreur")); // 42 (message si panic)
}
```

■ **L'opérateur ? :** Propage automatiquement les erreurs. Si `Err` , retourne l'erreur immédiatement. Si `Ok` , extrait la valeur. 

## 3. Pattern Matching

Le **pattern matching** avec `match` est une des fonctionnalités les plus puissantes de Rust. 

## 3.1 - L'expression match

```
// Match simple
fn valeur_en_centimes(piece: Piece) -> u8 {
    match piece {
        Piece::Penny => 1,
        Piece::Nickel => 5,
        Piece::Dime => 10,
        Piece::Quarter => 25,
    }
}
// Match avec enum contenant des données
enum Message {
    Move { x: i32, y: i32 },
    Write(String),
    ChangeColor(i32, i32, i32),
}
fn traiter(msg: Message) {
    match msg {
        Message::Move { x, y } => {
            println!("Déplacer à ({}, {})", x, y);
        }
        Message::Write(texte) => {
            println!("Texte : {}", texte);
        }
        Message::ChangeColor(r, g, b) => {
            println!("RGB({}, {}, {})", r, g, b);
        }
    }
}
// Match avec Option
fn plus_un(x: Option<i32>) -> Option<i32> {
    match x {
        None => None,
        Some(i) => Some(i + 1),
    }
}
// Match exhaustif obligatoire !
fn nombre_pair(x: i32) -> bool {
    match x % 2 {
        0 => true,
        _ => false,  // _ = catch-all (tous les autres cas)
    }
}
```

■■ **Match  exhaustif  :** Tu  dois  couvrir **tous  les  cas  possibles** .  Le  compilateur  vérifie  ! Utilise `_` pour attraper tous les cas restants. 

## 3.2 - Patterns avancés

```
// Match avec plusieurs valeurs
fn decrire_nombre(x: i32) {
    match x {
        1 => println!("Un"),
        2 | 3 | 5 | 7 => println!("Nombre premier petit"),
        10..=20 => println!("Entre 10 et 20"),
        _ => println!("Autre chose"),
    }
}
// Match avec conditions (guards)
fn est_pair_et_positif(x: i32) -> bool {
    match x {
        n if n > 0 && n % 2 == 0 => true,
        _ => false,
    }
}
// Déstructuration dans match
struct Point {
    x: i32,
    y: i32,
}
fn position(point: Point) {
    match point {
        Point { x: 0, y: 0 } => println!("Origine"),
        Point { x: 0, y } => println!("Sur l'axe Y à y={}", y),
        Point { x, y: 0 } => println!("Sur l'axe X à x={}", x),
        Point { x, y } => println!("Point ({}, {})", x, y),
    }
}
// @ binding (capturer et tester)
fn analyser_age(age: i32) {
    match age {
        n @ 0..=12 => println!("Enfant de {} ans", n),
        n @ 13..=19 => println!("Ado de {} ans", n),
        n => println!("Adulte de {} ans", n),
    }
}
```

## 3.3 - if let et while let

Raccourcis quand tu t'intéresses à **un seul cas** d'un enum. 

```
// Avec match (verbeux)
let config_max = Some(3u8);
match config_max {
    Some(max) => println!("Max : {}", max),
    _ => (),  // On ignore None
}
// Avec if let (concis)
if let Some(max) = config_max {
    println!("Max : {}", max);
}
// if let avec else
let nombre = Some(7);
if let Some(n) = nombre {
    println!("Nombre : {}", n);
} else {
    println!("Pas de nombre");
}
// while let (boucle tant que pattern match)
let mut pile = vec![1, 2, 3];
while let Some(sommet) = pile.pop() {
    println!("{}", sommet);  // 3, 2, 1
}
```

■ **Quand utiliser if let ?** 

Quand tu ne t'intéresses qu'à **un seul cas** et que tu veux ignorer les autres. Plus lisible que `match` avec `_` . 

## 3.4 - Déstructuration

```
// Déstructurer un tuple
let (x, y, z) = (1, 2, 3);
println!("{}, {}, {}", x, y, z);
```

```
// Déstructurer une struct
struct Point { x: i32, y: i32 }
let p = Point { x: 0, y: 7 };
let Point { x, y } = p;
println!("x: {}, y: {}", x, y);
```

```
// Renommer pendant la déstructuration
let Point { x: a, y: b } = p;
println!("a: {}, b: {}", a, b);
```

```
// Ignorer des valeurs
let Point { x, .. } = p;  // Ignore y
println!("x: {}", x);
```

```
// Dans les paramètres de fonction
fn afficher_point(&Point { x, y }: &Point) {
    println!("Point ({}, {})", x, y);
}
```

## 4. Exemples pratiques

## Exemple 1 : Système de gestion d'utilisateurs

```
enum Role {
    Admin,
    Moderateur,
    Utilisateur,
}
struct Compte {
    id: u32,
    nom: String,
    email: String,
    role: Role,
}
impl Compte {
    fn nouveau(id: u32, nom: String, email: String) -> Compte {
        Compte {
            id,
            nom,
            email,
            role: Role::Utilisateur,
        }
    }
    fn promouvoir(&mut self, nouveau_role: Role) {
        self.role = nouveau_role;
    }
    fn afficher_permissions(&self) {
        match self.role {
            Role::Admin => println!("{} : Accès total", self.nom),
            Role::Moderateur => println!("{} : Peut modérer", self.nom),
            Role::Utilisateur => println!("{} : Accès limité", self.nom),
        }
    }
}
fn main() {
    let mut compte = Compte::nouveau(
        1,
        String::from("Alice"),
        String::from("alice@example.com")
    );
    compte.afficher_permissions();
    compte.promouvoir(Role::Admin);
    compte.afficher_permissions();
}
```

## Exemple 2 : Calculatrice avec Result

```
enum Operation {
    Addition,
    Soustraction,
    Multiplication,
    Division,
}
fn calculer(a: f64, b: f64, op: Operation) -> Result<f64, String> {
    match op {
        Operation::Addition => Ok(a + b),
        Operation::Soustraction => Ok(a - b),
        Operation::Multiplication => Ok(a * b),
        Operation::Division => {
            if b == 0.0 {
                Err(String::from("Division par zéro"))
            } else {
                Ok(a / b)
            }
        }
    }
}
fn main() {
    let resultat = calculer(10.0, 2.0, Operation::Division);
    match resultat {
        Ok(valeur) => println!("Résultat : {}", valeur),
        Err(erreur) => println!("Erreur : {}", erreur),
    }
    // Avec ?
    fn faire_calculs() -> Result<(), String> {
        let r1 = calculer(10.0, 2.0, Operation::Division)?;
        let r2 = calculer(r1, 3.0, Operation::Addition)?;
        println!("Résultat final : {}", r2);
        Ok(())
    }
}
```

## 5. Exercices pratiques

## ■ **Exercice 1 : Créer une struct Livre** 

```
// Crée une struct Livre avec : titre, auteur, pages
// Ajoute une méthode est_long() qui retourne true si > 300 pages
// Solution :
struct Livre {
    titre: String,
    auteur: String,
    pages: u32,
}
impl Livre {
    fn est_long(&self) -> bool {
        self.pages > 300
    }
}
```

■ **Exercice 2 : Enum avec match** 

```
// Crée un enum Saison avec 4 variantes
// Écris une fonction qui retourne le nombre de jours
// Solution :
enum Saison {
    Printemps,
    Ete,
    Automne,
    Hiver,
}
```

```
fn jours_approximatifs(saison: Saison) -> u32 {
    match saison {
        Saison::Printemps | Saison::Automne => 92,
        Saison::Ete => 93,
        Saison::Hiver => 89,
    }
}
```

## ■ **Exercice 3 : Option et Result** 

```
// Écris une fonction qui trouve un élément dans un Vec
// Retourne Option<usize> (l'index)
```

```
// Solution :
fn trouver<T: PartialEq>(vec: &Vec<T>, element: &T) -> Option<usize> {
    for (index, item) in vec.iter().enumerate() {
        if item == element {
            return Some(index);
        }
    }
    None
}
```

```
fn main() {
    let nombres = vec![1, 2, 3, 4, 5];
```

```
    match trouver(&nombres, &3) {
        Some(index) => println!("Trouvé à l'index {}", index),
        None => println!("Pas trouvé"),
    }
```

```
}
```

## 6. Aide-mémoire

## ■ **Structures** 

|**Syntaxe**|**Exemple**|
|---|---|
|`Définition`|`struct Point { x: i32, y: i32 }`|
|`Instanciation`|`let p = Point { x: 0, y: 0 };`|
|`Accès champ`|`p.x`|
|`Méthode`|`fn aire(&self) -> u32 { ... }`|
|`Fonction associée`|`fn new() -> Self { ... }`|
|`Tuple struct`|`struct Color(i32, i32, i32);`|

## ■ **Énumérations** 

|**Syntaxe**|**Exemple**|
|---|---|
|`Définition simple`|`enum Dir { Haut, Bas }`|
|`Avec données`|`enum Msg { Move { x: i32, y: i32 } }`|
|`Option`|`Some(5) ou None`|
|`Result`|`Ok(value) ou Err(error)`|
|`Match`|`match x { Some(n) => n, None => 0 }`|
|`if let`|`if let Some(n) = x { ... }`|
