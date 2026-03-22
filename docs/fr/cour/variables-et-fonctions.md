## Les Fondamentaux de Rust

## Variables, Mutabilité et Fonctions 

Guide Complet pour Débutants 

## Objectifs du cours

À la fin de ce cours, tu sauras : 

- Déclarer et utiliser des variables en Rust 

- Comprendre la mutabilité et l'immutabilité 

- Créer et appeler des fonctions 

- Utiliser les types de retour et les paramètres 

- Maîtriser ownership et borrowing (bases) 

## Table des matières

1. Les variables en Rust 

- 1.1 - Déclaration de base 

- 1.2 - Immutabilité par défaut 

- 1.3 - Variables mutables 

- 1.4 - Les constantes 

- 1.5 - Le shadowing 

2. Les types de données 

- 2.1 - Types scalaires 

- 2.2 - Types composés 

- 2.3 - Inférence de type 

- 2.4 - Annotation de type 

3. Les fonctions 

- 3.1 - Déclaration de base 

- 3.2 - Paramètres 

- 3.3 - Valeur de retour 

- 3.4 - Expressions vs instructions 

- 3.5 - Fonctions avec références 

4. Ownership et Borrowing (introduction) 

- 4.1 - Le concept d'ownership 

- 4.2 - Les références (&) 

- 4.3 - Les références mutables (&mut;) 

5. Exercices pratiques 

6. Aide-mémoire 

## 1. Les variables en Rust

En Rust, les variables sont déclarées avec le mot-clé `let` . Une des particularités de Rust est que **les variables sont immutables par défaut** . 

## 1.1 - Déclaration de base

```
// Déclaration simple
let x = 5;
println!("La valeur de x est : {}", x);
```

```
// Déclaration avec type explicite
let y: i32 = 10;
println!("La valeur de y est : {}", y);
```

```
// Déclaration de plusieurs variables
let a = 1;
let b = 2;
let c = 3;
```

I **Note :** En Rust, on utilise `println!` (avec un !) pour afficher du texte. Le `{}` est remplacé par la valeur de la variable. 

## 1.2 - Immutabilité par défaut

Par défaut, les variables Rust sont **immutables** : on ne peut pas changer leur valeur après leur déclaration. 

```
let x = 5;
println!("x = {}", x);
```

**`x = 6;  //`** I **`ERREUR DE COMPILATION ! // error[E0384]: cannot assign twice to immutable variable `x``** 

II **Pourquoi ?** L'immutabilité par défaut aide à éviter les bugs. Si tu veux modifier une variable, tu dois le déclarer explicitement avec `mut` . 

## 1.3 - Variables mutables

Pour rendre une variable modifiable, on ajoute `mut` après `let` . 

**`let mut x = 5; println!("x = {}", x);  // Affiche : x = 5 x = 6;  //`** I **`OK ! println!("x = {}", x);  // Affiche : x = 6`** 

```
// Modification multiple
let mut compteur = 0;
compteur = compteur + 1;
compteur = compteur + 1;
println!("compteur = {}", compteur);  // Affiche : compteur = 2
```

I **Conseil :** Utilise `mut` seulement quand nécessaire. Cela rend ton code plus sûr et plus facile à comprendre. 

## 1.4 - Les constantes

Les constantes sont déclarées avec `const` et sont **toujours immutables** . Elles doivent avoir un type explicite et sont évaluées à la compilation. 

```
// Constante (MAJUSCULES par convention)
const MAX_POINTS: u32 = 100_000;
const PI: f64 = 3.14159;
```

```
fn main() {
```

```
    println!("Le maximum est : {}", MAX_POINTS);
```

**`//`** I **`Impossible avec une constante :`** 

- **`// const ne peut pas être mut`** 

```
    // const doit avoir un type explicite
}
```

## Différences entre let et const :

||**let**|**const**|
|---|---|---|
|**Immutable par défaut**|I|I|
|**Peut être mut**|I|I|
|**Type explicite requis**|I|I|
|**Portée**|Block|Globale|
|**Valeur calculée**|Runtime|Compilation|
|**Convention nom**|snake_case|UPPER_CASE|

## 1.5 - Le shadowing

Le **shadowing** permet de redéclarer une variable avec le même nom. C'est différent de la mutabilité ! 

**`let x = 5; println!("x = {}", x);  // 5 let x = x + 1;  //`** I **`Nouvelle variable qui masque la précédente println!("x = {}", x);  // 6 let x = x * 2;  //`** I **`Encore une nouvelle variable println!("x = {}", x);  // 12 // Shadowing permet de changer le type ! let espaces = "   ";  // Type: &str let espaces = espaces.len();  // Type: usize println!("Il y a {} espaces", espaces);  // 3`** 

### Shadowing vs mut :

• `mut` : Modifie la valeur, **même type** 

- `let` (shadowing) : Crée une nouvelle variable, **peut changer de type** 

## 2. Les types de données

Rust est un langage **statiquement typé** : chaque variable a un type connu à la compilation. Rust peut souvent **inférer** le type, mais tu peux aussi l'annoter explicitement. 

## 2.1 - Types scalaires

Les types scalaires représentent une valeur unique. Rust en a quatre types principaux : 

```
// 1. ENTIERS (integers)
let a: i32 = 42;        // Entier signé 32 bits
let b: u64 = 100;       // Entier non signé 64 bits
let c = 5;              // i32 par défaut
```

```
// Différentes tailles : i8, i16, i32, i64, i128, isize
//                       u8, u16, u32, u64, u128, usize
```

```
// 2. FLOTTANTS (floating-point)
let x: f64 = 2.5;       // 64 bits (défaut)
let y: f32 = 3.14;      // 32 bits
```

```
// 3. BOOLÉENS (boolean)
let vrai: bool = true;
let faux: bool = false;
```

**`// 4. CARACTÈRES (char) let lettre: char = 'A'; let emoji: char = '`** I **`';  // Unicode !`** 

## Types d'entiers en Rust :

|**Type**|**Taille**|**Minimum**|**Maximum**|
|---|---|---|---|
|i8|8 bits|-128|127|
|u8|8 bits|0|255|
|i32|32 bits|-2 milliards|2 milliards|
|u32|32 bits|0|4 milliards|
|i64|64 bits|Très grand négatif|Très grand positif|
|isize|Taille du système|Variable|Variable|

## 2.2 - Types composés

Les types composés regroupent plusieurs valeurs dans un seul type. 

```
// 1. TUPLES - Types différents, taille fixe
let personne: (&str, i32, bool) = ("Alice", 25, true);
```

```
// Accès par déstructuration
let (nom, age, actif) = personne;
println!("{} a {} ans", nom, age);
```

```
// Accès par index
println!("Nom : {}", personne.0);
println!("Age : {}", personne.1);
```

```
// 2. TABLEAUX - Même type, taille fixe
let nombres: [i32; 5] = [1, 2, 3, 4, 5];
```

```
// Accès par index
let premier = nombres[0];  // 1
let dernier = nombres[4];  // 5
```

```
// Tableau avec valeur répétée
let zeros = [0; 10];  // [0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
```

```
// 3. SLICES - Vue sur une partie d'un tableau
let slice = &nombres[1..3];  // [2, 3]
```

## 2.3 - Inférence de type

```
// Rust infère le type automatiquement
let x = 5;           // i32 (défaut pour entiers)
let y = 2.5;         // f64 (défaut pour flottants)
let z = true;        // bool
let nom = "Alice";   // &str
```

```
// Rust peut inférer selon l'utilisation
let mut nombres = Vec::new();  // Type inconnu
nombres.push(1);               // Maintenant Rust sait : Vec<i32>
```

## 2.4 - Annotation de type

```
// Parfois nécessaire pour lever l'ambiguïté
let nombre: u32 = "42".parse().expect("Pas un nombre!");
```

```
// Types complexes
let vecteur: Vec<i32> = vec![1, 2, 3];
let hashmap: std::collections::HashMap<String, i32>
    = std::collections::HashMap::new();
```

## 3. Les fonctions

Les fonctions sont omniprésentes en Rust. Elles sont déclarées avec `fn` et utilisent la convention **snake_case** pour les noms. 

## 3.1 - Déclaration de base

**`// Fonction sans paramètre ni retour fn dire_bonjour() { println!("Bonjour !"); } // Fonction principale fn main() { dire_bonjour();  // Appel de fonction dire_bonjour();  // On peut l'appeler plusieurs fois } // Convention de nommage fn ma_fonction() { }        //`** I **`snake_case fn MaFonction() { }         //`** I **`éviter fn calculer_total() { }     //`** I **`fn calculerTotal() { }      //`** I **`éviter (camelCase)`** 

## 3.2 - Paramètres

Les paramètres doivent **toujours avoir un type explicite** . 

```
// Un paramètre
fn afficher_nombre(x: i32) {
    println!("Le nombre est : {}", x);
}
// Plusieurs paramètres
fn additionner(a: i32, b: i32) {
    let somme = a + b;
    println!("{} + {} = {}", a, b, somme);
}
// Appel
fn main() {
    afficher_nombre(42);
    additionner(5, 7);
}
// Paramètres de types différents
fn presenter(nom: &str, age: i32, actif: bool) {
    println!("{} a {} ans, actif: {}", nom, age, actif);
}
```

## 3.3 - Valeur de retour

Une fonction retourne une valeur avec `->` suivi du type. La dernière expression est retournée automatiquement (pas de `return` nécessaire). 

**`// Fonction avec retour fn additionner(a: i32, b: i32) -> i32 { a + b  //`** II **`PAS de point-virgule ! }`** 

```
fn main() {
    let resultat = additionner(5, 3);
    println!("5 + 3 = {}", resultat);  // 8
}
```

```
// Avec return explicite (utile pour retour anticipé)
fn valeur_absolue(x: i32) -> i32 {
    if x < 0 {
        return -x;  // Retour anticipé
    }
    x  // Retour normal
}
```

```
// Plusieurs retours possibles
fn diviser(a: i32, b: i32) -> Option<i32> {
    if b == 0 {
        return None;  // Division par zéro
    }
    Some(a / b)
}
```

II **Attention !** En Rust, une expression **sans point-virgule** est retournée. Avec point-virgule, c'est une instruction qui ne retourne rien. 

## 3.4 - Expressions vs instructions

**`//`** I **`Expression (retourne une valeur) fn plus_un(x: i32) -> i32 { x + 1  // Pas de ;`** 

```
}
```

**`//`** I **`Instruction (ne retourne rien) fn plus_un_incorrect(x: i32) -> i32 { x + 1;  // Avec ; = ne retourne rien ! // error: mismatched types (expected i32, found ()) }`** 

```
// Les blocs sont des expressions
fn exemple() -> i32 {
    let x = {
        let y = 3;
        y + 1  // Cette expression est la valeur du bloc
    };  // x vaut 4
```

```
    x * 2  // Retourne 8
```

```
}
```

## 3.5 - Fonctions avec références

Pour éviter de copier les données, on peut passer des **références** aux fonctions. 

```
// Sans référence (copie la valeur)
fn afficher_nombre(x: i32) {
    println!("Nombre : {}", x);
}
```

```
// Avec référence (emprunte la valeur)
fn afficher_chaine(s: &String) {
    println!("Chaîne : {}", s);
}
```

```
// Référence mutable (peut modifier)
fn incrementer(x: &mut i32) {
    *x += 1;  // * = déréférencement
}
```

```
fn main() {
    let nombre = 42;
    afficher_nombre(nombre);  // Copie
```

```
    let texte = String::from("Hello");
    afficher_chaine(&texte);  // Emprunte
```

```
    let mut compteur = 0;
    incrementer(&mut compteur);
    println!("Compteur : {}", compteur);  // 1
}
```

## 4. Ownership et Borrowing (introduction)

L' **ownership** est le concept le plus important et unique de Rust. C'est ce qui permet à Rust d'être sûr sans garbage collector. 

## 4.1 - Le concept d'ownership

Règles de base : 

1. Chaque valeur a un **propriétaire** unique 

2. Il ne peut y avoir qu' **un seul propriétaire** à la fois 

3. Quand le propriétaire sort du scope, la valeur est **libérée** 

**`fn main() { let s1 = String::from("hello"); let s2 = s1;  // s1 est "déplacé" vers s2 // println!("{}", s1);  //`** I **`ERREUR ! s1 n'est plus valide println!("{}", s2);     //`** I **`OK } // Exemple avec fonction fn prend_ownership(s: String) { println!("{}", s); }  // s est libéré ici`** 

```
fn main() {
    let texte = String::from("hello");
    prend_ownership(texte);
```

**`// println!("{}", texte);  //`** I **`ERREUR ! // texte a été déplacé dans la fonction }`** 

I **Types Copy :** Les types simples (i32, f64, bool, char) sont `Copy` : ils sont copiés au lieu d'être déplacés. 

## 4.2 - Les références (&)

Les **références** permettent d'emprunter une valeur sans en prendre ownership. 

```
fn calculer_longueur(s: &String) -> usize {
    s.len()
```

```
}  // s sort du scope, mais ne possède pas la String
```

**`fn main() { let texte = String::from("hello"); let longueur = calculer_longueur(&texte); println!("'{}' a {} caractères", texte, longueur); //`** I **`texte est toujours valide ! }`** 

**`// Multiples références immutables OK fn main() { let s = String::from("hello"); let r1 = &s; let r2 = &s; println!("{} et {}", r1, r2);  //`** I **`OK }`** 

## 4.3 - Les références mutables (&mut;)

```
fn ajouter_monde(s: &mut String) {
    s.push_str(", world!");
}
```

```
fn main() {
    let mut texte = String::from("hello");
    ajouter_monde(&mut texte);
    println!("{}", texte);  // "hello, world!"
}
```

**`//`** II **`UNE SEULE référence mutable à la fois ! fn main() { let mut s = String::from("hello"); let r1 = &mut s; // let r2 = &mut s;  //`** I **`ERREUR ! // On ne peut pas avoir deux références mutables println!("{}", r1); } //`** II **`Pas de mélange immutable + mutable fn main() { let mut s = String::from("hello"); let r1 = &s;      // OK let r2 = &s;      // OK // let r3 = &mut s;  //`** I **`ERREUR ! println!("{} et {}", r1, r2); }`** 

II **Règles du borrowing :** 1. Soit **une** référence mutable 2. Soit **plusieurs** références immutables 3. Mais **jamais les deux en même temps** ! 

## 5. Exercices pratiques

Voici quelques exercices pour pratiquer : 

### Exercice 1 : Variables

Complète ce code pour qu'il compile : 

```
fn main() {
    let x = 5;
    // Ajoute le code nécessaire pour que x devienne 10
    println!("x = {}", x);  // Doit afficher "x = 10"
}
// Solution :
fn main() {
    let mut x = 5;  // Ajouter mut
    x = 10;
    println!("x = {}", x);
}
```

### Exercice 2 : Fonction simple

Crée une fonction qui multiplie deux nombres : 

```
// À compléter
fn multiplier(/* paramètres */) /* -> type */ {
    // ton code
}
fn main() {
    let resultat = multiplier(6, 7);
    println!("6 x 7 = {}", resultat);  // Doit afficher 42
}
// Solution :
fn multiplier(a: i32, b: i32) -> i32 {
    a * b
}
```

## Exercice 3 : Références

Crée une fonction qui double un nombre sans le déplacer : 

```
fn doubler(/* référence mutable */) {
    // ton code
}
fn main() {
    let mut nombre = 5;
    doubler(/* passer référence */);
    println!("Nombre : {}", nombre);  // Doit afficher 10
}
// Solution :
fn doubler(x: &mut i32) {
    *x *= 2;
}
fn main() {
    let mut nombre = 5;
    doubler(&mut nombre);
    println!("Nombre : {}", nombre);
}
```

## 6. Aide-mémoire

## Variables

|**Syntaxe**|**Description**|**Exemple**|
|---|---|---|
|let x = 5;|Variable immutable|let nom = "Alice";|
|let mut x = 5;|Variable mutable|let mut compteur = 0;|
|const MAX: i32 = 100;|Constante|const PI: f64 = 3.14;|
|let x = 5; let x = 10;|Shadowing|let x = x + 1;|

## Fonctions

|**Syntaxe**|**Description**|**Exemple**|
|---|---|---|
|fn nom() { }|Fonction simple|fn dire_bonjour() { }|
|fn nom(x: i32) { }|Avec paramètres|fn afficher(n: i32) { }|
|fn nom() -> i32 { }|Avec retour|fn double(x: i32) -> i32 { x * 2 }|
|fn nom(s: &String) { }|Référence|fn longueur(s: &String) -> usize { }|
|fn nom(x: &mut i32) { }|Réf. mutable|fn incrementer(x: &mut i32) { }|

## Types courants

|**Type**|**Description**|**Exemple**|
|---|---|---|
|i32, u32, i64...|Entiers|let x: i32 = 42;|
|f32, f64|Flottants|let pi: f64 = 3.14;|
|bool|Booléen|let actif: bool = true;|
|char|Caractère|let lettre: char = 'A';|
|&str|Chaîne immutable|let texte = "hello";|
|String|Chaîne mutable|let s = String::from("hi");|

## Bravo !

Tu connais maintenant les bases de Rust ! 

## Les prochaines étapes : 

• Pratiquer avec de petits programmes 

• Apprendre les structures (struct) et enums 

• Maîtriser le pattern matching 

• Explorer les collections (Vec, HashMap) 

I **Continue à coder et n'aie pas peur des erreurs du compilateur : elles sont là pour t'aider !** I 

## Ressources recommandées :

I The Rust Book (en français) : https://jimskapt.github.io/rust-book-fr/ 

I Rustlings (exercices) : https://github.com/rust-lang/rustlings 

I Rust by Example : https://doc.rust-lang.org/rust-by-example/ 

I Forum Rust : https://users.rust-lang.org/
