# Macros déclaratives
> `macro_rules!` : écrire du code qui écrit du code, par filtrage de motifs sur la syntaxe. Tous les fragments, toutes les formes de répétition, les techniques de récursion, et les pièges.

## Objectifs

- Savoir quand une macro est justifiée plutôt qu'une fonction ou un générique
- Écrire une macro avec un ou plusieurs bras
- Connaître tous les types de fragments (`expr`, `ident`, `ty`, `tt`...)
- Utiliser les répétitions `*`, `+`, `?`, imbriquées, avec séparateurs
- Écrire des macros récursives, des règles internes et des « tt munchers »
- Générer des fonctions, des structs, des `impl` et des tests
- Éviter les pièges : évaluation multiple, priorité des opérateurs, hygiène
- Déboguer une macro et produire des erreurs claires

## Table des matières

1. Pourquoi des macros
2. Syntaxe de base
3. Les fragments
4. Répétitions
5. Plusieurs bras
6. Exemples simples
7. Générer du code : fonctions, structs, impl, tests
8. Récursion et règles internes
9. Compter et accumuler
10. Pièges
11. Hygiène
12. Erreurs et débogage
13. Les macros de la bibliothèque standard
14. Exemples dans Runique
15. Macro, fonction ou générique : la même chose trois fois
16. Exercices
17. Aide-mémoire

---

## 1. Pourquoi des macros

Une macro est développée **à la compilation** : elle reçoit du code (des jetons) et produit du code. Elle peut faire ce qu'une fonction ne peut pas :

| Besoin | Fonction | Macro |
|---|---|---|
| nombre variable d'arguments | non | `vec![1, 2, 3]`, `println!("{} {}", a, b)` |
| syntaxe nouvelle | non | `urlpatterns! { "/" => index, }` |
| générer des déclarations (struct, impl, fn) | non | oui |
| vérifier quelque chose à la compilation (chaîne de format) | non | `format!("{x}")` |
| recevoir un nom de variable, un type, un motif | non | oui |
| être lisible, déboguable, autocomplétée par l'éditeur | oui | moins |

Règle : **une fonction d'abord, un générique ensuite, une macro en dernier**. Une macro se justifie quand elle supprime une vraie répétition que ni une fonction ni un trait ne peuvent supprimer.

---

## 2. Syntaxe de base

```rust
macro_rules! bonjour {
    () => {
        println!("Bonjour !");
    };
}

bonjour!();
```

- `macro_rules! nom { ... }` déclare la macro.
- Chaque **bras** a la forme `(motif) => { développement };`.
- On l'appelle avec `nom!`, et au choix des parenthèses, crochets ou accolades : `bonjour!()`, `bonjour![]`, `bonjour!{}` sont équivalents.

Convention : `()` pour une macro qui ressemble à un appel de fonction, `[]` pour une collection (`vec![]`), `{}` pour un bloc de déclarations (`urlpatterns! { }`). Une macro appelée avec `{}` en position d'élément n'a pas besoin de `;`.

### Avec un argument

```rust
macro_rules! carre {
    ($x:expr) => {
        $x * $x
    };
}

let a = carre!(4);         // 16
```

`$x:expr` capture une **expression** et la nomme `$x`. Dans le développement, `$x` est remplacé par le code reçu.

### Ordre de déclaration

Une macro `macro_rules!` doit être déclarée **avant** son utilisation dans le fichier (contrairement aux fonctions). Pour l'utiliser dans d'autres modules ou d'autres crates : voir `macros-export`.

---

## 3. Les fragments

| Fragment | Capture | Exemple |
|---|---|---|
| `expr` | une expression | `2 + 3`, `f(x)`, `vec![]`, `if a { 1 } else { 2 }` |
| `ident` | un identifiant | `x`, `ma_fonction`, `Point` |
| `ty` | un type | `i32`, `Vec<String>`, `&'a str` |
| `path` | un chemin | `std::collections::HashMap`, `crate::a::b` |
| `pat` | un motif (alternatives comprises en édition 2021+) | `Some(x)`, `1..=5`, `_` |
| `pat_param` | un motif sans alternative au premier niveau | idem |
| `stmt` | une instruction (sans `;` final) | `let x = 5` |
| `block` | un bloc entre accolades | `{ println!("a"); 1 }` |
| `item` | une déclaration | `fn f() {}`, `struct S;`, `impl T for S {}` |
| `literal` | un littéral | `42`, `"texte"`, `'c'`, `true` |
| `lifetime` | une durée de vie | `'a`, `'static` |
| `vis` | une visibilité (peut être vide) | `pub`, `pub(crate)`, rien |
| `meta` | le contenu d'un attribut | `derive(Debug)`, `doc = "x"` |
| `tt` | un seul arbre de jetons | `x`, `+`, `(a, b)`, `{ ... }` |

### 3.1 Exemples

```rust
macro_rules! declarer {
    ($vis:vis $nom:ident : $t:ty = $valeur:expr) => {
        $vis const $nom: $t = $valeur;
    };
}

declarer!(pub MAX: u32 = 100);
declarer!(MIN: u32 = 0);         // vis vide : privé

macro_rules! est {
    ($valeur:expr, $motif:pat) => {
        matches!($valeur, $motif)
    };
}

let ok = est!(Some(3), Some(1..=5));
```

### 3.2 `tt` : le fragment universel

Un `tt` (*token tree*) est un jeton isolé, ou tout ce qui est entre `()`, `[]` ou `{}`. Avec `$($t:tt)*`, on capture **n'importe quoi** sans l'analyser : c'est la base des macros récursives (section 8).

### 3.3 Ce qui peut suivre un fragment

Pour que la grammaire reste non ambiguë, certains fragments ne peuvent être suivis que de certains jetons :

| Fragment | Peut être suivi de |
|---|---|
| `expr`, `stmt` | `=>`, `,`, `;` |
| `ty`, `path` | `=>`, `,`, `=`, `;`, `:`, `>`, `[`, `{`, `as`, `where`, la barre verticale |
| `pat` | `=>`, `,`, `=`, `if`, `in` |
| `ident`, `tt`, `literal`, `lifetime`, `block` | n'importe quoi |

```rust
// macro_rules! m { ($a:expr $b:expr) => {}; }   // ERREUR : `$a:expr` is followed by `$b:expr`
macro_rules! m { ($a:expr, $b:expr) => {}; }     // OK : une virgule sépare
```

### 3.4 `expr` et l'édition 2024

En édition 2024, `expr` accepte aussi `const { ... }` et `_`. Pour garder l'ancien comportement (édition 2021), il existe `expr_2021`.

---

## 4. Répétitions

### 4.1 Les trois opérateurs

| Opérateur | Signification |
|---|---|
| `$( ... )*` | zéro fois ou plus |
| `$( ... )+` | une fois ou plus |
| `$( ... )?` | zéro ou une fois |

Le séparateur se place avant l'opérateur : `$( ... ),*` = répétitions séparées par des virgules.

### 4.2 Recréer `vec!`

```rust
macro_rules! mon_vec {
    ( $( $x:expr ),* ) => {{
        let mut v = Vec::new();
        $( v.push($x); )*          // une ligne générée par élément capturé
        v
    }};
}

let v = mon_vec![1, 2, 3];
let vide: Vec<i32> = mon_vec![];
```

Les **doubles accolades** `{{ ... }}` : les premières délimitent le développement, les secondes forment un bloc, pour que les variables (`v`) ne fuient pas et que le tout soit une seule expression.

### 4.3 Virgule finale facultative

```rust
macro_rules! liste {
    ( $( $x:expr ),* $(,)? ) => {
        vec![ $( $x ),* ]
    };
}

liste![1, 2, 3];
liste![1, 2, 3,];          // accepté grâce à $(,)?
```

### 4.4 Plusieurs variables dans une répétition

```rust
macro_rules! table {
    ( $( $cle:expr => $valeur:expr ),* $(,)? ) => {{
        let mut m = ::std::collections::HashMap::new();
        $( m.insert($cle, $valeur); )*
        m
    }};
}

let t = table! {
    "un" => 1,
    "deux" => 2,
};
```

Les variables d'une même répétition avancent **ensemble** : `$cle` et `$valeur` doivent avoir été capturés le même nombre de fois.

### 4.5 Répétitions imbriquées

```rust
macro_rules! matrice {
    ( $( [ $( $x:expr ),* ] ),* ) => {
        vec![ $( vec![ $( $x ),* ] ),* ]
    };
}

let m = matrice!([1, 2], [3, 4], [5, 6]);   // Vec<Vec<i32>>
```

### 4.6 Partie facultative

```rust
macro_rules! champ {
    ($nom:ident : $t:ty $(= $defaut:expr)?) => {
        // $defaut n'est utilisable que dans une répétition $( ... )?
        let $nom: $t = champ!(@defaut $($defaut)?);
    };
    (@defaut $e:expr) => { $e };
    (@defaut) => { Default::default() };
}
```

---

## 5. Plusieurs bras

Les bras sont essayés **dans l'ordre** ; le premier qui correspond gagne.

```rust
macro_rules! log {
    ($msg:expr) => {
        println!("[INFO] {}", $msg)
    };
    ($niveau:ident, $msg:expr) => {
        println!("[{}] {}", stringify!($niveau), $msg)
    };
    ($niveau:ident, $fmt:literal, $($arg:tt)*) => {
        println!(concat!("[{}] ", $fmt), stringify!($niveau), $($arg)*)   // concat! exige un littéral
    };
}

log!("démarrage");
log!(WARN, "disque presque plein");
log!(ERREUR, "code {}", 500);
```

Place les bras **spécifiques avant les généraux**. Un bras qui commence par un mot-clé littéral (`(admin $path:expr ...)`) doit précéder un bras générique (`($path:expr ...)`).

---

## 6. Exemples simples

### 6.1 Chronométrer un bloc

```rust
macro_rules! chrono {
    ($nom:expr, $bloc:block) => {{
        let debut = ::std::time::Instant::now();
        let r = $bloc;
        println!("{} : {:?}", $nom, debut.elapsed());
        r
    }};
}

let somme = chrono!("somme", { (1..1_000_000u64).sum::<u64>() });
```

### 6.2 Vérifier en début de fonction

```rust
macro_rules! exiger {
    ($cond:expr, $err:expr) => {
        if !$cond {
            return Err($err.into());
        }
    };
}

fn inscrire(age: u32, email: &str) -> Result<(), String> {
    exiger!(age >= 18, "il faut être majeur");
    exiger!(email.contains('@'), "email invalide");
    Ok(())
}
```

C'est le principe de `anyhow::ensure!`. Une macro peut faire `return` **dans la fonction appelante**, ce qu'une fonction ne peut pas.

### 6.3 Afficher une expression et sa valeur

```rust
macro_rules! voir {
    ($($e:expr),+ $(,)?) => {
        $( println!("{} = {:?}", stringify!($e), $e); )+
    };
}

voir!(1 + 2, "abc".len());
// 1 + 2 = 3
// "abc".len() = 3
```

`stringify!` transforme le **code** reçu en texte : c'est ainsi que `assert!` et `dbg!` affichent l'expression qui a échoué.

---

## 7. Générer du code : fonctions, structs, impl, tests

### 7.1 Des getters

```rust
macro_rules! getters {
    ($type:ident { $( $champ:ident : $t:ty ),* $(,)? }) => {
        impl $type {
            $(
                pub fn $champ(&self) -> &$t {
                    &self.$champ
                }
            )*
        }
    };
}

struct Utilisateur { nom: String, age: u32 }
getters!(Utilisateur { nom: String, age: u32 });
```

### 7.2 Un newtype complet

```rust
macro_rules! identifiant {
    ($nom:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub struct $nom(pub i64);

        impl ::std::fmt::Display for $nom {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                write!(f, "{}#{}", stringify!($nom), self.0)
            }
        }

        impl From<i64> for $nom {
            fn from(v: i64) -> Self { $nom(v) }
        }
    };
}

identifiant!(IdUtilisateur);
identifiant!(IdCommande);
```

Deux lignes génèrent deux types distincts avec six traits chacun : c'est le cas typique où une macro supprime une répétition qu'aucun générique ne peut supprimer (chaque type doit être **nommé**).

### 7.3 Un trait pour plusieurs types

```rust
trait Nul { fn est_nul(&self) -> bool; }

macro_rules! impl_nul {
    ($($t:ty),*) => {
        $( impl Nul for $t { fn est_nul(&self) -> bool { *self == 0 as $t } } )*
    };
}

impl_nul!(i8, i16, i32, i64, u8, u16, u32, u64, f32, f64);
```

La bibliothèque standard implémente ainsi les traits numériques pour chaque type entier.

### 7.4 Des tests

```rust
fn double(x: i32) -> i32 { x * 2 }

macro_rules! tests_double {
    ($($nom:ident : $entree:expr => $attendu:expr),* $(,)?) => {
        $(
            #[test]
            fn $nom() {
                assert_eq!(double($entree), $attendu);
            }
        )*
    };
}

tests_double! {
    zero: 0 => 0,
    positif: 3 => 6,
    negatif: -2 => -4,
}
```

### 7.5 Créer un identifiant nouveau

`macro_rules!` ne sait pas **fabriquer** un nom (`get_` + `nom`) : il ne peut utiliser que les identifiants qu'il reçoit. Pour concaténer des noms, il faut la crate `paste` ou une macro procédurale (cours suivants).

---

## 8. Récursion et règles internes

### 8.1 Une macro qui s'appelle elle-même

```rust
macro_rules! maximum {
    ($x:expr) => { $x };
    ($x:expr, $($reste:expr),+) => {{
        let a = $x;
        let b = maximum!($($reste),+);
        if a > b { a } else { b }
    }};
}

let m = maximum!(3, 9, 2, 7);    // 9
```

Chaque appel traite le premier élément et rappelle la macro sur le reste, jusqu'au cas d'arrêt (un seul élément).

### 8.2 Règles internes avec `@`

Un bras qui commence par un jeton comme `@etape` est, par convention, **interne** : l'utilisateur ne l'appelle pas, la macro l'utilise pour ses étapes.

```rust
macro_rules! somme_carres {
    ($($x:expr),*) => { somme_carres!(@acc 0; $($x),*) };
    (@acc $acc:expr;) => { $acc };
    (@acc $acc:expr; $x:expr $(, $reste:expr)*) => {
        somme_carres!(@acc $acc + $x * $x; $($reste),*)
    };
}

let s = somme_carres!(1, 2, 3);   // 14
```

### 8.3 Le « tt muncher »

Pour analyser une syntaxe libre, la macro « mange » les jetons un groupe à la fois avec `$($reste:tt)*` et se rappelle sur la suite.

```rust
macro_rules! routes {
    (@parse $v:ident;) => {};
    (@parse $v:ident; get $chemin:literal => $f:ident, $($reste:tt)*) => {
        $v.push(("GET", $chemin, stringify!($f)));
        routes!(@parse $v; $($reste)*);
    };
    (@parse $v:ident; post $chemin:literal => $f:ident, $($reste:tt)*) => {
        $v.push(("POST", $chemin, stringify!($f)));
        routes!(@parse $v; $($reste)*);
    };
    ($($t:tt)*) => {{
        let mut v: Vec<(&str, &str, &str)> = Vec::new();
        routes!(@parse v; $($t)*);
        v
    }};
}

let r = routes! {
    get "/" => index,
    post "/contact" => contact,
};
```

C'est exactement la structure de `urlpatterns!` dans Runique (section 14).

### 8.4 Limite de récursion

Chaque niveau de récursion compte. Au-delà de 128 niveaux par défaut, le compilateur s'arrête (`recursion limit reached`). On l'augmente avec `#![recursion_limit = "256"]` en tête de crate, mais une macro qui en a besoin gagne souvent à être réécrite.

---

## 9. Compter et accumuler

### 9.1 Compter les arguments

```rust
macro_rules! compter {
    () => { 0usize };
    ($tete:tt $($reste:tt)*) => { 1usize + compter!($($reste)*) };
}

const N: usize = compter!(a b c d);   // 4
```

### 9.2 Compter sans récursion

```rust
macro_rules! remplacer { ($_t:tt $sub:expr) => { $sub }; }

macro_rules! compter_vite {
    ($($t:tt)*) => { <[()]>::len(&[$(remplacer!($t ())),*]) };
}

const M: usize = compter_vite!(a b c d e);   // 5
```

Chaque élément devient un `()` dans un tableau, dont on prend la longueur : aucune limite de récursion.

### 9.3 Construire un tableau de taille connue

```rust
macro_rules! tableau_noms {
    ($($nom:ident),* $(,)?) => {
        [$(stringify!($nom)),*]
    };
}

const CHAMPS: [&str; 3] = tableau_noms!(nom, email, age);
```

---

## 10. Pièges

### 10.1 Évaluation multiple

```rust
macro_rules! carre_naif {
    ($x:expr) => { $x * $x };
}

let mut compteur = 0;
let mut suivant = || { compteur += 1; compteur };
let c = carre_naif!(suivant());   // suivant() est appelé DEUX fois : 1 * 2 = 2, pas 1
```

Le code reçu est **recopié** à chaque `$x`. Une expression avec effet de bord (ou coûteuse) est exécutée autant de fois. Correction : l'évaluer une fois dans une variable.

```rust
macro_rules! carre {
    ($x:expr) => {{
        let v = $x;
        v * v
    }};
}
```

### 10.2 Priorité des opérateurs

Un fragment `expr` est inséré comme **un seul bloc** (entre parenthèses implicites), ce qui évite le piège du préprocesseur C :

```rust
macro_rules! double { ($x:expr) => { $x * 2 }; }
let r = double!(1 + 2);      // 6 : (1 + 2) * 2, pas 1 + 2 * 2
```

Mais avec `tt`, les jetons sont recopiés tels quels :

```rust
macro_rules! double_tt { ($($x:tt)*) => { $($x)* * 2 }; }
let r = double_tt!(1 + 2);   // 5 : 1 + 2 * 2
```

Règle : capture une expression avec `expr`, sauf si tu as besoin d'analyser ses jetons.

### 10.3 Chemins absolus

```rust
macro_rules! nouvelle_map {
    () => { HashMap::new() };               // échoue si l'utilisateur n'a pas importé HashMap
}

macro_rules! nouvelle_map_ok {
    () => { ::std::collections::HashMap::new() };   // fonctionne partout
}
```

Dans le développement d'une macro, écris des chemins complets : `::std::...` pour la bibliothèque standard, `$crate::...` pour ta propre crate (voir `macros-export`).

### 10.4 Une macro mal placée

Une macro ne peut produire que ce qui est valide **à l'endroit de l'appel** : une expression là où une expression est attendue, des éléments (`fn`, `struct`) là où des déclarations sont attendues. Une macro qui produit plusieurs instructions en position d'expression doit les envelopper dans un bloc `{{ ... }}`.

---

## 11. Hygiène

### 11.1 Les variables locales sont protégées

```rust
macro_rules! avec_temp {
    ($e:expr) => {{
        let temp = 10;
        $e + temp
    }};
}

let temp = 1;
let r = avec_temp!(temp);    // 11 : le `temp` de l'appelant n'est pas celui de la macro
```

Une variable déclarée **dans** une macro ne peut pas entrer en collision avec une variable de l'appelant, même de même nom : `macro_rules!` est **hygiénique** pour les variables locales.

### 11.2 Ce qui n'est pas protégé

```rust
macro_rules! declarer_fonction {
    () => { fn aide() -> u32 { 42 } };
}

declarer_fonction!();
let x = aide();              // la fonction est visible : les éléments (fn, struct, impl) ne sont pas hygiéniques
```

Les noms de fonctions, de types, de modules et de traits générés par une macro sont visibles normalement. Les chemins dans le développement sont résolus **à l'endroit de l'appel**, d'où la règle des chemins absolus (section 10.3).

### 11.3 Donner accès à une variable

Pour que le code de l'appelant puisse utiliser une variable créée par la macro, son nom doit **venir de l'appelant** :

```rust
macro_rules! pour_chaque {
    ($var:ident in $liste:expr => $corps:block) => {
        for $var in $liste $corps
    };
}

pour_chaque!(x in vec![1, 2] => { println!("{x}"); });
```

---

## 12. Erreurs et débogage

### 12.1 Un message d'erreur clair

```rust
macro_rules! couleur {
    (rouge) => { (255, 0, 0) };
    (vert) => { (0, 255, 0) };
    ($autre:tt) => {
        compile_error!(concat!("couleur inconnue : ", stringify!($autre)))
    };
}

let c = couleur!(rouge);
// let d = couleur!(bleu);   // ERREUR : couleur inconnue : bleu
```

Sans le dernier bras, l'erreur serait « no rules expected the token `bleu` », moins parlante.

### 12.2 Voir le code généré

```bash
cargo install cargo-expand
cargo expand                  # toute la crate développée
cargo expand module::chemin   # un seul module
```

C'est l'outil indispensable pour comprendre une erreur dans une macro : on lit le code produit, et on compile la partie fautive à la main.

### 12.3 Lire une erreur

Une erreur dans le code **généré** est signalée à la ligne de l'appel de la macro, avec une note `in this macro invocation`. `cargo expand` montre la ligne réellement fautive.

---

## 13. Les macros de la bibliothèque standard

| Macro | Rôle |
|---|---|
| `println!`, `print!`, `eprintln!`, `format!`, `write!`, `writeln!` | formatage (chaîne vérifiée à la compilation) |
| `vec!` | construire un `Vec` |
| `assert!`, `assert_eq!`, `assert_ne!`, `debug_assert!` | vérifications |
| `panic!`, `todo!`, `unimplemented!`, `unreachable!` | arrêts |
| `matches!` | tester un motif |
| `dbg!` | afficher une expression et sa valeur |
| `concat!`, `stringify!` | assembler des littéraux, transformer du code en texte |
| `include_str!`, `include_bytes!` | inclure un fichier à la compilation |
| `env!`, `option_env!` | variable d'environnement **à la compilation** |
| `cfg!` | tester une configuration (feature, OS) |
| `file!`, `line!`, `column!`, `module_path!` | position dans le code source |
| `compile_error!` | arrêter la compilation avec un message |

```rust
const VERSION: &str = env!("CARGO_PKG_VERSION");          // version du Cargo.toml, figée à la compilation
const README: &str = include_str!("../README.md");        // le fichier est intégré au binaire
```

---

## 14. Exemples dans Runique

### 14.1 `urlpatterns!` : un tt muncher

```rust
// runique/src/macros/routeur/router.rs (début)
macro_rules! urlpatterns {
    (@parse $routes:ident;) => {};

    (@parse $routes:ident;
        admin $path:expr => $handler:expr,
        name = $name:expr,
        for_model = $model:ty,
        overrides = $view:expr,
        $($rest:tt)*
    ) => {
        $routes.add_admin_route::<$model>($path, $name, $handler, $view);
        $crate::urlpatterns!(@parse $routes; $($rest)*);
    };
    // ... d'autres bras pour les autres formes de route ...
}
```

On y retrouve toutes les techniques de ce cours : des règles internes `@parse`, un cas d'arrêt (`(@parse $routes:ident;) => {}`), la capture de la suite avec `$($rest:tt)*`, l'appel récursif, des bras spécifiques (qui commencent par le mot `admin`) placés avant les bras généraux, et `$crate::` pour que l'appel récursif fonctionne depuis la crate de l'utilisateur.

### 14.2 `impl_from_error!` : générer des `impl`

```rust
// runique/src/macros/context/impl_error.rs
#[macro_export]
macro_rules! impl_from_error {
    ($($err:ty => $method:ident),*) => {
        $(
            impl From<$err> for AppError {
                fn from(err: $err) -> Self { Self { context: ErrorContext::$method(&err) } }
            }
            impl From<$err> for Box<AppError> {
                fn from(err: $err) -> Self { Box::new(AppError::from(err)) }
            }
        )*
    };
}

impl_from_error!(
    anyhow::Error => from_anyhow,
    DbErr => database,
    tower_sessions::session::Error => database
);
```

Une répétition avec deux variables (`$err` et `$method`), qui génère deux `impl` par entrée : trois lignes d'appel remplacent six blocs `impl`.

### 14.3 Les réglages uniques du builder

Le builder de Runique génère ses traits de typestate avec une macro `once_only!` qui reçoit un attribut de documentation (`$(#[$doc:meta])*`), un nom (`$name:ident`) et des messages (`$message:literal`), puis produit un trait scellé avec `#[diagnostic::on_unimplemented]` (voir `traits-avances`, sections 9 et 10).

---

## 15. Macro, fonction ou générique : la même chose trois fois

Créer une `HashMap` à partir de paires.

```rust
use std::collections::HashMap;
use std::hash::Hash;

// 1. Fonction générique : lisible, autocomplétée, déboguable
fn map_de<K: Hash + Eq, V, const N: usize>(paires: [(K, V); N]) -> HashMap<K, V> {
    HashMap::from(paires)
}

// 2. Bibliothèque standard : rien à écrire
let a = HashMap::from([("un", 1), ("deux", 2)]);

// 3. Macro : syntaxe plus légère, rien de plus
macro_rules! map {
    ($($k:expr => $v:expr),* $(,)?) => {
        ::std::collections::HashMap::from([$(($k, $v)),*])
    };
}
let b = map! { "un" => 1, "deux" => 2 };
let c = map_de([("un", 1)]);
```

Ici, la macro n'apporte qu'un peu de syntaxe : `HashMap::from` suffit. Elle se justifie quand elle génère des **déclarations** (section 7) ou une **syntaxe** qu'aucune fonction ne peut accepter (section 8.3).

---

## 16. Exercices

### Exercice 1 — `min!`

Écris `min!` qui accepte un ou plusieurs arguments et n'évalue chacun qu'une fois.

```rust
// Solution :
macro_rules! min {
    ($x:expr) => { $x };
    ($x:expr, $($reste:expr),+ $(,)?) => {{
        let a = $x;
        let b = min!($($reste),+);
        if a < b { a } else { b }
    }};
}

assert_eq!(min!(5, 2, 8), 2);
```

### Exercice 2 — énumération avec liste

Écris `enum_liste!` qui génère une enum et une constante `TOUTES` contenant toutes ses variantes.

```rust
// Solution :
macro_rules! enum_liste {
    ($vis:vis enum $nom:ident { $($variante:ident),* $(,)? }) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        $vis enum $nom { $($variante),* }

        impl $nom {
            pub const TOUTES: &'static [$nom] = &[$($nom::$variante),*];
        }
    };
}

enum_liste!(pub enum Couleur { Rouge, Vert, Bleu });
assert_eq!(Couleur::TOUTES.len(), 3);
```

### Exercice 3 — erreur claire

Écris `taille!` qui accepte `petit`, `moyen`, `grand` et renvoie 1, 2, 3, avec une erreur de compilation lisible pour tout autre mot.

```rust
// Solution :
macro_rules! taille {
    (petit) => { 1 };
    (moyen) => { 2 };
    (grand) => { 3 };
    ($x:tt) => { compile_error!(concat!("taille inconnue : ", stringify!($x), " (petit, moyen ou grand)")) };
}
```

## 17. Aide-mémoire

```rust
macro_rules! nom {
    () => { .. };                                   // sans argument
    ($x:expr) => { .. };                            // une expression
    ($a:ident, $t:ty) => { .. };                    // identifiant, type
    ($($x:expr),* $(,)?) => { $( f($x); )* };       // répétition, virgule finale facultative
    ($($k:expr => $v:expr),*) => { .. };            // paires
    (@interne $acc:expr; $($reste:tt)*) => { .. };  // règle interne + tt muncher
    ($x:tt) => { compile_error!("..") };            // dernier recours : erreur claire
}
```

Fragments : `expr` `ident` `ty` `path` `pat` `stmt` `block` `item` `literal` `lifetime` `vis` `meta` `tt`.
Répétitions : `*` (0+), `+` (1+), `?` (0-1).

### Points clés

- Fonction d'abord, générique ensuite, macro en dernier.
- Les bras sont essayés dans l'ordre : spécifiques avant généraux.
- `{{ ... }}` pour qu'un développement soit une seule expression.
- Évalue une fois dans une variable tout argument utilisé plusieurs fois.
- Chemins absolus dans le développement : `::std::...`, `$crate::...`.
- Les variables locales sont hygiéniques, les éléments générés ne le sont pas.
- `compile_error!` pour une erreur lisible, `cargo expand` pour voir le code produit.
