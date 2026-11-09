# Les fondamentaux de Rust

**Variables, types et fonctions** — toutes les façons de déclarer, convertir, afficher et organiser du code en fonctions, et ce que chacune apporte.

## Objectifs du cours

À la fin de ce cours, tu sauras :

- Déclarer des variables, des constantes et des statiques, et choisir entre elles
- Utiliser le shadowing à la place de la mutabilité quand c'est plus clair
- Connaître chaque type scalaire, ses limites et le comportement en cas de dépassement
- Convertir entre types de toutes les façons (`as`, `From`, `TryFrom`, `parse`)
- Écrire des fonctions : paramètres, retours, retours multiples, sorties anticipées
- Remplacer ce qui n'existe pas en Rust : paramètres par défaut, surcharge, nombre variable d'arguments
- Afficher des valeurs avec tous les formats de `println!`
- Comprendre les bases de l'ownership et des références

## Table des matières

1. Les variables
2. Constantes et statiques
3. Le shadowing
4. Les types scalaires
5. Dépassement d'entier
6. Les types composés
7. Inférence et annotation de type
8. Conversions entre types
9. Les fonctions
10. Expressions et instructions
11. Ce qui n'existe pas en Rust et comment s'en passer
12. Afficher des valeurs
13. Macros utilitaires
14. Ownership et références (introduction)
15. Exercices
16. Aide-mémoire

## 1. Les variables

### 1.1 — Déclaration

```rust
fn main() {
    let x = 5;                 // type déduit : i32
    let y: f64 = 3.14;         // type annoté
    let nom = "Alice";         // &str
    let actif = true;          // bool
    println!("{x} {y} {nom} {actif}");
}
```

### 1.2 — Immuable par défaut

```rust
let x = 5;
// x = 6;   // ERREUR : cannot assign twice to immutable variable
```

À quoi ça sert : quand une variable ne change pas, tu le sais en lisant sa déclaration, sans parcourir toute la fonction. Le compilateur peut aussi mieux optimiser.

### 1.3 — `mut`

```rust
let mut compteur = 0;
compteur += 1;
compteur = compteur * 2;
```

`mut` signale au lecteur que la valeur va changer. Le compilateur avertit (`unused_mut`) si tu déclares `mut` une variable jamais modifiée.

### 1.4 — Déclarer maintenant, initialiser plus tard

```rust
let resultat: i32;           // déclarée, pas initialisée
let condition = true;
if condition {
    resultat = 1;
} else {
    resultat = 2;
}
println!("{resultat}");      // OK : initialisée dans toutes les branches
```

Pas besoin de `mut` : la variable n'est affectée qu'**une** fois. Le compilateur refuse de la lire avant qu'elle soit initialisée dans tous les chemins. En pratique, `let resultat = if condition { 1 } else { 2 };` est plus direct.

### 1.5 — Déstructurer à la déclaration

```rust
let (a, b) = (1, 2);
let (x, _) = (10, 20);            // _ ignore une valeur
let [premier, .., dernier] = [1, 2, 3, 4];
let (mut c, d) = (0, 0);          // mut sur une seule variable
c += 1;

struct Point { x: i32, y: i32 }
let Point { x, y } = Point { x: 1, y: 2 };
```

### 1.6 — Variables inutilisées

```rust
let _non_utilisee = 5;   // le préfixe _ supprime l'avertissement
let _ = calcul();        // _ seul : la valeur est jetée immédiatement

fn calcul() -> i32 { 1 }
```

Différence subtile : `let _x = verrou()` garde la valeur jusqu'à la fin du bloc, `let _ = verrou()` la libère aussitôt. Avec un verrou ou une connexion, la différence compte.

## 2. Constantes et statiques

### 2.1 — `const`

```rust
const MAX_POINTS: u32 = 100_000;
const DELAI_SECONDES: u64 = 60 * 60 * 24;   // calculé à la compilation

fn main() {
    println!("{MAX_POINTS}");
}
```

- Type **obligatoire**.
- Valeur calculable à la compilation.
- Nom en `MAJUSCULES_AVEC_TIRETS_BAS`.
- Peut être déclarée partout, y compris hors des fonctions.
- La valeur est **recopiée** à chaque utilisation : une constante n'a pas d'adresse fixe.

### 2.2 — `static`

```rust
static NOM_APP: &str = "Runique";
static mut COMPTEUR: u32 = 0;   // à éviter : tout accès demande unsafe

fn main() {
    println!("{NOM_APP}");
}
```

Une `static` a **une seule adresse** en mémoire pour toute la durée du programme. Une `static mut` est presque toujours une erreur : depuis l'édition 2024, même prendre une référence vers elle est refusé. On utilise à la place un type à mutabilité interne sûr.

```rust
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::LazyLock;
use std::collections::HashMap;

static COMPTEUR: AtomicU32 = AtomicU32::new(0);

// Valeur calculée au premier accès (Rust 1.80+)
static CONFIG: LazyLock<HashMap<&str, &str>> = LazyLock::new(|| {
    HashMap::from([("langue", "fr")])
});

fn main() {
    COMPTEUR.fetch_add(1, Ordering::Relaxed);
    println!("{}", CONFIG["langue"]);
}
```

### 2.3 — Choisir

| Besoin | Choix |
|---|---|
| Valeur fixe connue à la compilation (taille, limite, texte) | `const` |
| Valeur globale unique avec une adresse (tableau volumineux, référence partagée) | `static` |
| Valeur globale calculée au démarrage (regex, configuration) | `static` + `LazyLock` |
| Compteur global modifiable | `static` + atomique ou `Mutex` |
| Valeur qui dépend de l'exécution dans une fonction | `let` |

### 2.4 — `const fn`

```rust
const fn kilo_octets(n: usize) -> usize {
    n * 1024
}

const TAILLE_MAX: usize = kilo_octets(512);   // évalué à la compilation

fn main() {
    let a = kilo_octets(3);                    // utilisable aussi à l'exécution
}
```

Une `const fn` peut être appelée dans une constante. Son corps est limité (pas d'allocation, pas de trait non constant), mais `if`, `match`, `loop` et `while` y sont permis.

## 3. Le shadowing

Redéclarer une variable avec `let` crée une **nouvelle** variable qui masque l'ancienne.

### 3.1 — Transformer une valeur par étapes

```rust
let entree = "  42  ";
let entree = entree.trim();              // &str
let entree: u32 = entree.parse().unwrap(); // u32 : le type change
println!("{entree}");
```

### 3.2 — `mut` ou shadowing : la même chose écrite deux fois

```rust
// Écriture 1 : mut — le type ne peut pas changer
let mut prix = 100;
prix = prix * 2;
prix = prix + 5;

// Écriture 2 : shadowing — chaque étape est immuable
let prix = 100;
let prix = prix * 2;
let prix = prix + 5;
```

| | `mut` | shadowing |
|---|---|---|
| Changer le type | non | oui |
| La variable reste modifiable ensuite | oui | non (sauf `let mut`) |
| Adapté à une boucle | oui | non |
| Adapté à une suite de transformations | moins | oui |

### 3.3 — Shadowing dans un bloc

```rust
let x = 5;
{
    let x = x * 2;
    println!("dans le bloc : {x}");   // 10
}
println!("après : {x}");              // 5 : l'ancienne variable réapparaît
```

### 3.4 — Rendre immuable après préparation

```rust
let mut liste = vec![3, 1, 2];
liste.sort();
let liste = liste;   // à partir d'ici, plus de modification possible
```

## 4. Les types scalaires

### 4.1 — Entiers

| Signé | Non signé | Taille | Plage (signé) |
|---|---|---|---|
| `i8` | `u8` | 8 bits | -128 à 127 / 0 à 255 |
| `i16` | `u16` | 16 bits | -32 768 à 32 767 |
| `i32` | `u32` | 32 bits | environ ±2,1 milliards |
| `i64` | `u64` | 64 bits | environ ±9,2 × 10¹⁸ |
| `i128` | `u128` | 128 bits | très grand |
| `isize` | `usize` | taille d'un pointeur (64 bits sur PC) | dépend de la machine |

- `i32` : type par défaut d'un littéral entier.
- `usize` : type des **index** et des **longueurs** (`v.len()`, `v[i]`).
- `u8` : octets.
- `u64`/`i64` : identifiants, horodatages, tailles de fichiers.

```rust
println!("{} {}", i32::MIN, i32::MAX);
println!("{}", u8::MAX);   // 255
```

### 4.2 — Écrire un littéral

```rust
let million = 1_000_000;      // _ pour la lisibilité
let hexa = 0xff;              // 255
let octal = 0o77;             // 63
let binaire = 0b1111_0000;    // 240
let octet = b'A';             // u8 : 65
let avec_suffixe = 42u8;      // suffixe de type
let grand = 5_000_000_000i64;
let flottant = 1e6;           // 1 000 000.0
```

### 4.3 — Flottants

```rust
let a = 2.0;        // f64 par défaut
let b: f32 = 3.0;   // moins précis, moitié moins de mémoire

println!("{}", 0.1 + 0.2);            // 0.30000000000000004
println!("{}", 0.1 + 0.2 == 0.3);     // false

// Comparer avec une tolérance
let egal = (0.1_f64 + 0.2 - 0.3).abs() < f64::EPSILON;

println!("{}", 1.0 / 0.0);            // inf
println!("{}", (0.0_f64 / 0.0).is_nan()); // true
```

Un flottant n'est **pas** exact : n'utilise jamais `f64` pour de l'argent. Les montants se stockent en centimes (`i64`) ou en décimal (`rust_decimal::Decimal`, ce que fait le champ `decimal` de Runique).

`f64` n'implémente pas `Eq` ni `Ord` (à cause de `NaN`) : on ne peut pas le mettre comme clé de `HashMap` ni trier un `Vec<f64>` avec `.sort()`. On utilise `v.sort_by(|a, b| a.total_cmp(b))`.

### 4.4 — Opérations

```rust
let somme = 5 + 10;
let difference = 95.5 - 4.3;
let produit = 4 * 30;
let quotient = 7 / 2;        // 3 : division entière, tronquée vers zéro
let quotient_f = 7.0 / 2.0;  // 3.5
let reste = 7 % 2;           // 1
let negatif = -7 / 2;        // -3
let modulo = (-7_i32).rem_euclid(2); // 1 : toujours positif

let puissance = 2_i32.pow(10);   // 1024
let racine = 16.0_f64.sqrt();    // 4.0
let absolu = (-5_i32).abs();
let max = 3.max(7);
let borne = 15.clamp(0, 10);     // 10
```

Il n'y a pas d'opérateur `++` ni `--` : on écrit `x += 1`.

Les deux opérandes doivent avoir **le même type** : `5_i32 + 2_i64` ne compile pas, il faut convertir (section 8).

### 4.5 — Booléen et caractère

```rust
let vrai: bool = true;
let c: char = 'é';        // 4 octets : un caractère Unicode
let emoji = '🦀';
let chiffre = '7'.to_digit(10);   // Some(7)
let est_lettre = 'a'.is_alphabetic();
let majuscule = 'a'.to_ascii_uppercase();
```

`char` (guillemets simples) est un caractère Unicode ; `&str` (guillemets doubles) est une chaîne UTF-8. `'a'` et `"a"` sont deux types différents.

## 5. Dépassement d'entier

### 5.1 — Le comportement par défaut

```rust
fn ajouter(a: u8, b: u8) -> u8 {
    a + b
}

fn main() {
    let r = ajouter(250, 10);
    // cargo run           : panique "attempt to add with overflow"
    // cargo run --release : r vaut 4 (260 - 256), sans erreur
}
```

En **debug**, un dépassement fait paniquer. En **release**, il **s'enroule** silencieusement. Ne compte jamais sur l'un ou l'autre : choisis explicitement.

### 5.2 — Les quatre façons de gérer un dépassement

```rust
let a: u8 = 250;

// 1. checked : None en cas de dépassement
let r1 = a.checked_add(10);        // None
let r2 = a.checked_add(5);         // Some(255)

// 2. saturating : bloque à la limite
let r3 = a.saturating_add(10);     // 255
let r4 = 5_u8.saturating_sub(10);  // 0

// 3. wrapping : enroulement voulu
let r5 = a.wrapping_add(10);       // 4

// 4. overflowing : résultat enroulé + indicateur
let (r6, deborde) = a.overflowing_add(10); // (4, true)
```

| Méthode | À quoi elle sert |
|---|---|
| `checked_*` | entrées utilisateur, calculs de tailles : refuser proprement |
| `saturating_*` | compteurs, jauges, temps restant : rester dans les bornes |
| `wrapping_*` | hachage, nombres aléatoires, numéros de séquence : l'enroulement est voulu |
| `overflowing_*` | arithmétique sur plusieurs mots (grands nombres) |

### 5.3 — Le piège de la soustraction non signée

```rust
let longueur: usize = 3;
let index: usize = 5;
// let ecart = longueur - index;          // panique en debug, énorme nombre en release
let ecart = longueur.saturating_sub(index); // 0
let ecart = longueur.checked_sub(index);    // None
let ecart = longueur.abs_diff(index);       // 2
```

## 6. Les types composés

### 6.1 — Tuples

```rust
let point: (i32, f64, &str) = (10, 3.5, "a");

let premier = point.0;              // accès par position
let (x, y, nom) = point;            // déstructuration

let vide = ();                      // unit : le « rien » de Rust
let seul = (5,);                    // tuple d'un élément : la virgule est obligatoire
let pas_un_tuple = (5);             // simplement 5 entre parenthèses
```

### 6.2 — Tableaux

```rust
let nombres: [i32; 5] = [1, 2, 3, 4, 5];  // taille fixe, connue à la compilation
let zeros = [0; 100];                      // 100 fois 0
let premier = nombres[0];
let longueur = nombres.len();

// let x = nombres[10];          // erreur de compilation : index constant hors limites
let i = 10;
let x = nombres.get(i);          // None, sans panique
```

Un tableau ne peut pas grandir. Pour une taille variable : `Vec<T>` (cours `collections`).

### 6.3 — Tranches (slices)

```rust
let nombres = [1, 2, 3, 4, 5];
let milieu: &[i32] = &nombres[1..4];   // [2, 3, 4]
let debut = &nombres[..2];             // [1, 2]
let fin = &nombres[3..];               // [4, 5]
let tout = &nombres[..];

let texte = String::from("bonjour");
let mot: &str = &texte[0..3];          // "bon"
```

Une tranche est une **vue** sur une partie d'un tableau, d'un `Vec` ou d'une chaîne, sans copie. Une fonction qui prend `&[T]` accepte un tableau, un `Vec` ou une partie de l'un ou de l'autre.

Attention : `&texte[0..3]` coupe en **octets**. Couper au milieu d'un caractère accentué fait paniquer.

### 6.4 — `String` et `&str`

```rust
let fixe: &str = "littéral";          // texte figé dans le binaire, non modifiable
let mut possede = String::from("a");  // texte alloué, modifiable
possede.push_str("bc");

let vue: &str = &possede;             // une String se prête en &str
let copie: String = fixe.to_string(); // un &str se copie en String
let copie2: String = fixe.to_owned();
let copie3 = String::from(fixe);
let copie4: String = fixe.into();
```

Les quatre dernières lignes font la même chose. Règle pour les paramètres : accepte `&str`, renvoie `String`.

## 7. Inférence et annotation de type

### 7.1 — Où l'annotation est nécessaire

```rust
let x = 5;                  // déduit : i32
let v = Vec::new();         // ERREUR si rien ne permet de deviner le type des éléments...
let mut v2 = Vec::new();
v2.push(1u8);               // ...ici c'est possible : Vec<u8>

let n = "42".parse().unwrap();      // ERREUR : parse vers quel type ?
```

### 7.2 — Trois façons de préciser le type

```rust
// 1. Annoter la variable
let n: u32 = "42".parse().unwrap();

// 2. Turbofish ::<> sur la fonction
let n = "42".parse::<u32>().unwrap();

// 3. Suffixe sur un littéral
let n = 42u32;

// Avec collect
let v: Vec<i32> = (1..4).collect();
let v = (1..4).collect::<Vec<i32>>();
let v = (1..4).collect::<Vec<_>>();     // _ : laisse deviner le type des éléments
```

Le turbofish sert quand il n'y a pas de variable à annoter : au milieu d'une chaîne, ou dans un argument de fonction.

## 8. Conversions entre types

Rust ne convertit **jamais** implicitement entre types numériques.

### 8.1 — `as` : la conversion brute

```rust
let a: i32 = 300;
let b = a as u8;          // 44 : tronqué (300 - 256), sans erreur
let c = -1_i32 as u32;    // 4294967295
let d = 3.99_f64 as i32;  // 3 : tronqué vers zéro
let e = 1e20_f64 as i32;  // 2147483647 : saturé à la limite
let f = 'A' as u8;        // 65
let g = 65_u8 as char;    // 'A'
let h = true as i32;      // 1
```

`as` ne peut pas échouer : il **perd silencieusement** de l'information. À réserver aux cas où la perte est voulue ou impossible.

### 8.2 — `From` / `Into` : la conversion sans perte

```rust
let petit: u8 = 200;
let grand: u32 = u32::from(petit);   // toujours possible
let grand: u32 = petit.into();       // même chose, type déduit de la variable
let f: f64 = f64::from(3_i32);

// u32 -> u8 n'existe pas avec From : la perte serait possible
// let x: u8 = u8::from(300_u32);   // ERREUR de compilation
```

### 8.3 — `TryFrom` / `TryInto` : la conversion qui peut échouer

```rust
use std::convert::TryFrom;

let grand: i64 = 300;
let r = u8::try_from(grand);          // Err(TryFromIntError)
let ok: Result<u8, _> = 200_i64.try_into(); // Ok(200)

fn index_valide(i: i64) -> Option<usize> {
    usize::try_from(i).ok()           // négatif -> None
}
```

### 8.4 — `parse` et `to_string` : depuis et vers du texte

```rust
let n: i32 = "42".parse().unwrap();
let r = "abc".parse::<i32>();         // Err(ParseIntError)
let f: f64 = "3.14".parse().unwrap();
let b: bool = "true".parse().unwrap();

let t = 42.to_string();
let t = format!("{:.2}", 3.14159);    // "3.14"
```

### 8.5 — Choisir

| Situation | Outil |
|---|---|
| Conversion toujours possible (`u8` vers `u32`, `i32` vers `f64`) | `From` / `into()` |
| Conversion qui peut échouer (`i64` vers `u8`, négatif vers `usize`) | `TryFrom` / `try_into()` |
| Texte vers nombre | `parse` |
| Nombre vers texte | `to_string`, `format!` |
| Perte voulue (troncature, octet) | `as` |

## 9. Les fonctions

### 9.1 — Déclaration

```rust
fn saluer() {
    println!("Bonjour !");
}

fn main() {
    saluer();
}
```

Noms en `snake_case`. L'ordre de déclaration n'a pas d'importance : une fonction peut être appelée avant d'être écrite dans le fichier.

### 9.2 — Paramètres

```rust
fn presenter(nom: &str, age: u32) {
    println!("{nom} a {age} ans");
}

// Paramètre déstructuré
fn distance((x, y): (f64, f64)) -> f64 {
    (x * x + y * y).sqrt()
}

// Paramètre modifiable localement (la copie, pas l'original)
fn decompte(mut n: u32) {
    while n > 0 {
        n -= 1;
    }
}

// Paramètre ignoré
fn rappel(_evenement: &str) {}
```

Le type de chaque paramètre est **obligatoire** : il n'y a pas d'inférence sur la signature.

### 9.3 — Valeur de retour : trois écritures

```rust
// 1. Dernière expression, sans point-virgule (forme idiomatique)
fn carre(x: i32) -> i32 {
    x * x
}

// 2. return explicite
fn carre_bis(x: i32) -> i32 {
    return x * x;
}

// 3. return anticipé puis expression finale
fn valeur_absolue(x: i32) -> i32 {
    if x >= 0 {
        return x;
    }
    -x
}
```

On réserve `return` aux sorties **anticipées**. Un point-virgule en trop transforme l'expression finale en instruction :

```rust
fn carre_casse(x: i32) -> i32 {
    x * x;   // ERREUR : mismatched types, expected i32, found ()
}
```

### 9.4 — Renvoyer plusieurs valeurs

```rust
// 1. Tuple
fn min_max(v: &[i32]) -> (i32, i32) {
    let mut min = v[0];
    let mut max = v[0];
    for &x in v {
        min = min.min(x);
        max = max.max(x);
    }
    (min, max)
}

// 2. Struct nommée : plus lisible au-delà de deux valeurs
struct Stats { min: i32, max: i32, moyenne: f64 }

fn stats(v: &[i32]) -> Stats {
    let (min, max) = min_max(v);
    let moyenne = v.iter().sum::<i32>() as f64 / v.len() as f64;
    Stats { min, max, moyenne }
}

fn main() {
    let (min, max) = min_max(&[3, 1, 4]);
    let s = stats(&[3, 1, 4]);
    println!("{} {}", s.min, s.moyenne);
}
```

### 9.5 — Renvoyer « rien » ou « peut-être »

```rust
fn log(msg: &str) {}                       // renvoie () implicitement
fn log_explicite(msg: &str) -> () {}       // identique

fn premier_pair(v: &[i32]) -> Option<i32> {   // une valeur ou rien
    v.iter().copied().find(|x| x % 2 == 0)
}

fn diviser(a: i32, b: i32) -> Result<i32, String> {  // une valeur ou une erreur
    if b == 0 {
        return Err("division par zéro".to_string());
    }
    Ok(a / b)
}
```

`premier_pair` avec un `v` vide ne panique pas, contrairement à `min_max` ci-dessus qui lit `v[0]`. Renvoyer une `Option` est la façon de rendre un cas impossible visible dans la signature.

### 9.6 — Une fonction qui ne revient jamais

```rust
fn arreter(message: &str) -> ! {
    panic!("{message}");
}

fn boucle_serveur() -> ! {
    loop {
        // accepter des connexions
    }
}
```

Le type `!` signale que la fonction ne rend jamais la main.

### 9.7 — Paramètres par valeur ou par référence

```rust
fn longueur(s: &String) -> usize { s.len() }       // emprunte : l'appelant garde la String
fn longueur_mieux(s: &str) -> usize { s.len() }    // accepte &String et "littéral"
fn ajouter(s: &mut String) { s.push('!'); }        // emprunte en modification
fn consommer(s: String) -> String { s + "!" }      // prend la String, en rend une nouvelle

fn main() {
    let mut t = String::from("a");
    longueur(&t);
    longueur_mieux(&t);
    longueur_mieux("littéral");
    ajouter(&mut t);
    let t = consommer(t);
}
```

| Signature | Quand l'utiliser |
|---|---|
| `x: T` (type `Copy` : nombres, `bool`, `char`) | toujours par valeur, la copie est gratuite |
| `x: &T`, `&str`, `&[T]` | lire sans prendre |
| `x: &mut T` | modifier la valeur de l'appelant |
| `x: T` (type possédé : `String`, `Vec`) | la fonction garde ou transforme la valeur |

### 9.8 — Fonction imbriquée

```rust
fn traiter(v: &[i32]) -> i32 {
    fn double(x: i32) -> i32 {
        x * 2
    }
    v.iter().map(|&x| double(x)).sum()
}
```

Une fonction imbriquée n'est visible que dans la fonction qui la contient, mais **ne capture pas** ses variables. Pour capturer, on utilise une closure (`let double = |x| x * facteur;`).

### 9.9 — Méthodes et fonctions associées

```rust
struct Rectangle { largeur: u32, hauteur: u32 }

impl Rectangle {
    fn new(largeur: u32, hauteur: u32) -> Self {     // fonction associée (pas de self)
        Self { largeur, hauteur }
    }
    fn aire(&self) -> u32 {                          // méthode qui lit
        self.largeur * self.hauteur
    }
    fn agrandir(&mut self, n: u32) {                 // méthode qui modifie
        self.largeur += n;
    }
    fn en_carre(self) -> Rectangle {                 // méthode qui consomme
        Rectangle { hauteur: self.largeur, ..self }
    }
}

fn main() {
    let mut r = Rectangle::new(2, 3);   // appel avec ::
    r.agrandir(1);                      // appel avec .
    let a = Rectangle::aire(&r);        // même chose que r.aire()
}
```

Détails dans le cours `structures-enums`.

### 9.10 — Fonctions comme valeurs

```rust
fn double(x: i32) -> i32 { x * 2 }

fn appliquer(f: fn(i32) -> i32, x: i32) -> i32 {
    f(x)
}

fn main() {
    let f = double;                  // une fonction est une valeur
    println!("{}", appliquer(f, 5)); // 10
    let v: Vec<i32> = [1, 2].iter().map(|&x| double(x)).collect();
    let v: Vec<i32> = [1, 2].into_iter().map(double).collect(); // passée directement
}
```

### 9.11 — Documenter et marquer une fonction

```rust
/// Calcule l'aire d'un cercle.
///
/// # Exemple
///
/// ```
/// assert_eq!(aire(1.0).round(), 3.0);
/// ```
#[must_use]
pub fn aire(rayon: f64) -> f64 {
    std::f64::consts::PI * rayon * rayon
}

fn main() {
    aire(2.0);   // avertissement : la valeur renvoyée n'est pas utilisée
}
```

`///` génère la documentation (`cargo doc`) et ses exemples sont exécutés comme tests. `#[must_use]` avertit l'appelant qui ignore le résultat : utile sur toute fonction sans effet de bord.

## 10. Expressions et instructions

### 10.1 — La différence

- Une **expression** produit une valeur : `5`, `x + 1`, `if`, `match`, un bloc `{ }`, un appel de fonction.
- Une **instruction** n'en produit pas : `let x = 5;`, une expression suivie de `;`.

```rust
let y = {
    let x = 3;
    x + 1           // dernière expression du bloc : sa valeur
};                  // y = 4

let z = {
    let x = 3;
    x + 1;          // ; : le bloc vaut ()
};                  // z = ()
```

### 10.2 — Ce qui en découle

```rust
// let x = (let y = 6);     // ERREUR : let n'est pas une expression
// let a = b = 5;           // ERREUR : l'affectation vaut (), pas 5

let valeur = match 3 { 1 => "un", _ => "autre" };
let statut = if true { "ok" } else { "ko" };
let n = loop { break 7; };
```

Presque tout est une expression en Rust : c'est ce qui permet d'écrire les variables immuables sans valeur provisoire.

## 11. Ce qui n'existe pas en Rust et comment s'en passer

### 11.1 — Paramètres par défaut

Il n'y a pas de `fn f(x: i32 = 5)`. Quatre façons d'obtenir le même effet :

```rust
// 1. Option : None veut dire « valeur par défaut »
fn connecter(port: Option<u16>) -> u16 {
    port.unwrap_or(8080)
}

// 2. Deux fonctions
fn connecter_defaut() -> u16 { connecter_sur(8080) }
fn connecter_sur(port: u16) -> u16 { port }

// 3. Struct de configuration avec Default
#[derive(Default)]
struct Options { port: u16, verbeux: bool }

fn demarrer(opts: Options) {}

// 4. Builder : chaque option est une méthode
struct Serveur { port: u16, verbeux: bool }
impl Serveur {
    fn new() -> Self { Self { port: 8080, verbeux: false } }
    fn port(mut self, p: u16) -> Self { self.port = p; self }
    fn verbeux(mut self) -> Self { self.verbeux = true; self }
}

fn main() {
    connecter(None);
    connecter(Some(3000));
    demarrer(Options { port: 3000, ..Default::default() });
    let s = Serveur::new().port(3000).verbeux();
}
```

Le builder est la forme qu'utilise Runique (`RuniqueAppBuilder::new(config).with_database()...`) : il reste lisible avec beaucoup d'options et en ajouter une ne casse aucun appel existant.

### 11.2 — Surcharge (même nom, paramètres différents)

```rust
// Interdit : deux fonctions ne peuvent pas porter le même nom
// fn afficher(x: i32) {}
// fn afficher(x: &str) {}

// 1. Des noms différents
fn afficher_nombre(x: i32) {}
fn afficher_texte(x: &str) {}

// 2. Un générique avec un trait
fn afficher<T: std::fmt::Display>(x: T) {
    println!("{x}");
}

// 3. Into pour accepter plusieurs types
fn titre(t: impl Into<String>) -> String {
    t.into()
}

fn main() {
    afficher(5);
    afficher("a");
    titre("littéral");
    titre(String::from("possédée"));
}
```

### 11.3 — Nombre variable d'arguments

```rust
// 1. Une tranche
fn somme(nombres: &[i32]) -> i32 {
    nombres.iter().sum()
}

// 2. Un itérateur : accepte un Vec, un tableau, une plage
fn somme_iter(nombres: impl IntoIterator<Item = i32>) -> i32 {
    nombres.into_iter().sum()
}

fn main() {
    somme(&[1, 2, 3]);
    somme_iter(vec![1, 2]);
    somme_iter(1..=10);
}
```

Seules les macros (`println!`, `vec!`) acceptent réellement un nombre variable d'arguments.

### 11.4 — Valeur nulle

Il n'y a pas de `null`. Une valeur absente s'écrit `Option<T>` : `None` ou `Some(valeur)`. Le compilateur oblige à traiter le cas `None` avant d'utiliser la valeur.

### 11.5 — Exceptions

Il n'y a pas de `try`/`catch`. Une erreur attendue est une valeur `Result<T, E>`, propagée avec `?`. Une erreur de programmation fait `panic!`. Voir le cours `gestion-des-erreurs`.

## 12. Afficher des valeurs

### 12.1 — Les macros d'affichage

```rust
println!("avec retour à la ligne");
print!("sans retour à la ligne");
eprintln!("vers la sortie d'erreur");
let s: String = format!("dans une String : {}", 5);
```

### 12.2 — Placer les valeurs : quatre écritures

```rust
let nom = "Ada";
let age = 36;

println!("{} a {} ans", nom, age);       // 1. par position
println!("{0} a {1} ans, {0} !", nom, age); // 2. par index, réutilisable
println!("{n} a {a} ans", n = nom, a = age); // 3. par nom
println!("{nom} a {age} ans");          // 4. variable capturée (Rust 1.58+)
```

La forme 4 ne marche qu'avec un **nom de variable**, pas une expression : `{personne.nom}` ou `{x + 1}` sont refusés, il faut alors la forme 1.

### 12.3 — `Display` et `Debug`

```rust
let v = vec![1, 2, 3];
println!("{v:?}");    // [1, 2, 3]        Debug : pour le développeur
println!("{v:#?}");   // une valeur par ligne, indentée

#[derive(Debug)]
struct Point { x: i32, y: i32 }
println!("{:?}", Point { x: 1, y: 2 });   // Point { x: 1, y: 2 }
```

`{}` demande le trait `Display` (affichage pour l'utilisateur, à écrire à la main). `{:?}` demande `Debug` (souvent dérivé).

### 12.4 — Largeur, alignement, précision

```rust
let x = 3.14159;
println!("[{x:.2}]");       // [3.14]          2 décimales
println!("[{x:8.2}]");      // [    3.14]      largeur 8
println!("[{x:<8.2}]");     // [3.14    ]      aligné à gauche
println!("[{x:^8.2}]");     // [  3.14  ]      centré
println!("[{x:08.2}]");     // [00003.14]      complété par des zéros
println!("[{:*>6}]", "ab"); // [****ab]        caractère de remplissage
println!("[{:+}]", 5);      // [+5]            signe toujours affiché

let largeur = 10;
println!("[{:>largeur$}]", "a");   // largeur dans une variable
println!("[{:.*}]", 3, x);         // précision passée en argument : 3.142
```

### 12.5 — Bases et notations

```rust
let n = 255;
println!("{n:x}");    // ff
println!("{n:X}");    // FF
println!("{n:#x}");   // 0xff
println!("{n:b}");    // 11111111
println!("{n:#010b}");// 0b11111111
println!("{n:o}");    // 377
println!("{:e}", 1234.5);  // 1.2345e3
println!("{{}}");     // {} : accolades littérales
```

## 13. Macros utilitaires

### 13.1 — `dbg!`

```rust
let a = 2;
let b = dbg!(a * 3) + 1;   // affiche [src/main.rs:2:9] a * 3 = 6 sur stderr, renvoie 6
dbg!(&b);                  // passer une référence pour ne pas déplacer la valeur
```

Affiche le fichier, la ligne, l'expression et sa valeur, puis **renvoie** la valeur : on peut l'insérer au milieu d'un calcul. À retirer avant de livrer (`clippy::dbg_macro` le signale).

### 13.2 — `todo!`, `unimplemented!`, `unreachable!`

```rust
fn exporter_pdf() -> Vec<u8> {
    todo!()                          // pas encore écrit : compile, panique à l'appel
}

fn exporter_xml() -> Vec<u8> {
    unimplemented!("pas de XML sur cette plateforme")   // ne sera pas fait ici
}

fn signe(x: i32) -> &'static str {
    match x.signum() {
        1 => "positif",
        -1 => "négatif",
        0 => "zéro",
        _ => unreachable!("signum ne renvoie que -1, 0 ou 1"),   // impossible
    }
}
```

Les trois paniquent ; leur nom dit **pourquoi** au lecteur. Ils ont le type `!` et satisfont donc n'importe quel type de retour.

### 13.3 — `assert!` et `debug_assert!`

```rust
fn moyenne(v: &[f64]) -> f64 {
    assert!(!v.is_empty(), "moyenne d'une liste vide");      // vérifié en debug ET release
    debug_assert!(v.iter().all(|x| x.is_finite()));          // vérifié en debug seulement
    v.iter().sum::<f64>() / v.len() as f64
}
```

## 14. Ownership et références (introduction)

L'**ownership** permet à Rust de libérer la mémoire sans ramasse-miettes, en vérifiant tout à la compilation. Ce chapitre donne les bases ; le cours `borrow-bases` les détaille.

### 14.1 — Les trois règles

1. Chaque valeur a un **propriétaire**.
2. Il n'y a qu'**un** propriétaire à la fois.
3. Quand le propriétaire sort de portée, la valeur est **libérée**.

```rust
fn main() {
    let s1 = String::from("hello");
    let s2 = s1;              // s1 est déplacé vers s2
    // println!("{s1}");      // ERREUR : value borrowed here after move
    println!("{s2}");
}
```

### 14.2 — Déplacer, copier, cloner

```rust
let a = 5;
let b = a;                     // copie : i32 est Copy, a reste utilisable

let s = String::from("x");
let t = s.clone();             // copie explicite du contenu : s reste utilisable
let u = s;                     // déplacement : s n'est plus utilisable
```

Sont `Copy` : entiers, flottants, `bool`, `char`, références partagées `&T`, et tuples ou tableaux de types `Copy`. Ne sont pas `Copy` : `String`, `Vec`, `Box`, et tout ce qui possède de la mémoire.

### 14.3 — Passer à une fonction : trois façons

```rust
fn prendre(s: String) {}            // déplace : l'appelant perd la valeur
fn lire(s: &String) {}              // emprunte en lecture
fn modifier(s: &mut String) { s.push('!'); } // emprunte en écriture

fn main() {
    let mut texte = String::from("a");
    lire(&texte);
    modifier(&mut texte);
    prendre(texte);
    // lire(&texte);   // ERREUR : texte a été déplacé
}
```

### 14.4 — Les règles d'emprunt

À un instant donné, soit **une** référence mutable, soit **autant** de références partagées que l'on veut, jamais les deux.

```rust
let mut s = String::from("a");
let r1 = &s;
let r2 = &s;
println!("{r1} {r2}");     // dernière utilisation de r1 et r2
let r3 = &mut s;           // OK : r1 et r2 ne servent plus
r3.push('b');
```

Un emprunt dure jusqu'à sa **dernière utilisation**, pas jusqu'à la fin du bloc.

## 15. Exercices

### Exercice 1 — conversion de température

Écris `celsius_vers_fahrenheit(c: f64) -> f64` (`F = C × 9/5 + 32`), puis affiche le résultat avec une décimale.

```rust
// Solution :
fn celsius_vers_fahrenheit(c: f64) -> f64 {
    c * 9.0 / 5.0 + 32.0
}

fn main() {
    println!("{:.1}", celsius_vers_fahrenheit(37.0)); // 98.6
}
```

### Exercice 2 — âge en jours, sans dépassement

Calcule un nombre de jours à partir d'un âge en `u32`, en renvoyant `None` en cas de dépassement.

```rust
// Solution :
fn jours(age: u32) -> Option<u32> {
    age.checked_mul(365)
}
```

### Exercice 3 — lire un nombre

Lis un texte, enlève les espaces, convertis-le en `u8` et renvoie `None` si ce n'est pas possible. Écris-le avec `match`, puis en une ligne.

```rust
// Solution :
fn lire_u8(texte: &str) -> Option<u8> {
    match texte.trim().parse::<u8>() {
        Ok(n) => Some(n),
        Err(_) => None,
    }
}

fn lire_u8_court(texte: &str) -> Option<u8> {
    texte.trim().parse().ok()
}
```

### Exercice 4 — paramètre par défaut

Écris `saluer(nom: Option<&str>) -> String` qui renvoie `"Bonjour, invité"` quand le nom est absent.

```rust
// Solution :
fn saluer(nom: Option<&str>) -> String {
    format!("Bonjour, {}", nom.unwrap_or("invité"))
}
```

## 16. Aide-mémoire

### Variables

```rust
let x = 5;                       // immuable
let mut y = 5;                   // modifiable
let z: u64 = 5;                  // annotée
let (a, b) = (1, 2);             // déstructurée
let x = x + 1;                   // shadowing
const MAX: u32 = 100;            // constante de compilation
static NOM: &str = "app";        // globale, adresse unique
```

### Conversions

```rust
let a = x as u8;                 // brute, peut perdre
let b: u32 = petit.into();       // sans perte
let c = u8::try_from(grand)?;    // peut échouer
let d: i32 = "42".parse()?;      // depuis du texte
let e = 42.to_string();          // vers du texte
```

### Arithmétique sûre

```rust
a.checked_add(b)     // Option
a.saturating_sub(b)  // bloque aux bornes
a.wrapping_mul(b)    // enroulement voulu
a.abs_diff(b)        // écart sans signe
```

### Fonctions

```rust
fn f(x: i32) -> i32 { x * 2 }              // retour par dernière expression
fn g(x: i32) -> Option<i32> { None }       // peut-être
fn h(x: i32) -> Result<i32, String> { Ok(x) } // ou une erreur
fn i() -> ! { panic!() }                   // ne revient jamais
fn j(t: impl Into<String>) {}              // accepte plusieurs types
fn k(v: &[i32]) {}                         // tableau, Vec ou tranche
```

### Formats

```rust
"{x}"  "{:?}"  "{:#?}"  "{:.2}"  "{:>8}"  "{:<8}"  "{:^8}"  "{:08.2}"  "{:x}"  "{:b}"  "{:e}"
```

### Points clés

- Immuable par défaut : `mut` signale un changement à venir.
- `const` pour les valeurs de compilation, `static` + `LazyLock` pour les globales calculées.
- Le shadowing permet de changer le type d'une valeur en la transformant.
- Un dépassement panique en debug et s'enroule en release : utilise `checked_`, `saturating_` ou `wrapping_`.
- `as` perd silencieusement ; `From` ne perd jamais ; `TryFrom` signale l'échec.
- Pas de paramètres par défaut, pas de surcharge : `Option`, builder, génériques, `impl Into`.
- La dernière expression d'un bloc est sa valeur ; un `;` en trop la transforme en `()`.
