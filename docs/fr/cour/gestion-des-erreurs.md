# Gestion des erreurs

**`panic!`, `Result`, `Option` et `?`** — distinguer une erreur attendue d'un bug, propager, convertir et définir ses propres erreurs, avec les différentes façons d'écrire chaque cas.

## Objectifs du cours

À la fin de ce cours, tu sauras :

- Choisir entre `panic!` et `Result`
- Traiter un `Result` de toutes les façons : `match`, `if let`, `let ... else`, méthodes, `?`
- Comprendre ce que fait exactement `?`, et le faire fonctionner entre types d'erreurs différents
- Passer d'un `Option` à un `Result` et inversement
- Écrire son propre type d'erreur avec `Display`, `Error` et `From`
- Utiliser `Box<dyn Error>` et faire renvoyer une erreur à `main`
- Savoir quand passer à `thiserror` et `anyhow` (cours suivant)

## Table des matières

1. Deux sortes d'erreurs
2. `panic!`
3. `Result<T, E>`
4. Traiter un `Result` : toutes les écritures
5. `unwrap` et `expect`
6. L'opérateur `?`
7. `Option` et `Result` ensemble
8. Les erreurs de la bibliothèque standard
9. Définir son propre type d'erreur
10. Convertir les erreurs pour `?`
11. `Box<dyn Error>`
12. `main` qui renvoie une erreur
13. Où traiter une erreur
14. La même fonction écrite de plusieurs façons
15. Exercices
16. Aide-mémoire

## 1. Deux sortes d'erreurs

| | Erreur **récupérable** | Erreur **irrécupérable** (bug) |
|---|---|---|
| Exemple | fichier absent, saisie invalide, réseau coupé, utilisateur introuvable | index hors limites, invariant violé, état impossible |
| Prévisible ? | oui : cela arrive en fonctionnement normal | non : le programme est faux |
| Outil | `Result<T, E>` (ou `Option<T>`) | `panic!` |
| Qui décide | l'appelant | personne : le thread s'arrête |

Rust n'a pas d'exceptions : une erreur récupérable est une **valeur** renvoyée par la fonction, visible dans sa signature.

## 2. `panic!`

### 2.1 — Déclencher une panique

```rust
fn main() {
    panic!("état impossible : {}", 42);
}
// thread 'main' panicked at src/main.rs:2:5:
// état impossible : 42
```

D'autres opérations paniquent d'elles-mêmes :

```rust
let v = vec![1, 2, 3];
// v[10];                          // index out of bounds
// let x: u8 = 255; x + 1;         // dépassement (en debug)
// 1 / 0;                          // division par zéro
// None::<i32>.unwrap();           // called `Option::unwrap()` on a `None` value
```

### 2.2 — Ce qui se passe

Une panique **déroule** la pile du thread : chaque valeur est libérée (`Drop` appelé), puis le thread s'arrête. Si c'est le thread principal, le programme s'arrête avec le code 101.

```bash
RUST_BACKTRACE=1 cargo run      # affiche la pile d'appels qui a mené à la panique
```

Dans un serveur web, une panique dans un handler n'arrête que la tâche de la requête : axum renvoie une erreur 500 et continue de servir les autres.

### 2.3 — Quand paniquer

- Dans les **tests** (`assert!`, `unwrap`) : un échec doit arrêter le test.
- Dans un **prototype**, en attendant de traiter les erreurs.
- Quand un **invariant** du programme est violé : continuer donnerait un résultat faux.
- Quand l'erreur est **impossible** et que tu peux expliquer pourquoi (`expect` avec la raison).

```rust
fn moyenne(v: &[f64]) -> f64 {
    assert!(!v.is_empty(), "moyenne appelée sur une liste vide : bug de l'appelant");
    v.iter().sum::<f64>() / v.len() as f64
}
```

Ne panique **jamais** sur une entrée qui vient de l'extérieur (utilisateur, fichier, réseau, base) : c'est une erreur récupérable.

## 3. `Result<T, E>`

```rust
enum Result<T, E> {
    Ok(T),     // la valeur
    Err(E),    // la raison de l'échec
}
```

```rust
use std::fs::File;

fn main() {
    let f = File::open("config.toml");    // Result<File, std::io::Error>
    match f {
        Ok(fichier) => println!("ouvert : {fichier:?}"),
        Err(e) => println!("impossible d'ouvrir : {e}"),
    }
}
```

`Result` est marqué `#[must_use]` : ignorer un `Result` provoque un avertissement.

```rust
std::fs::remove_file("tmp.txt");          // avertissement : unused `Result` that must be used
let _ = std::fs::remove_file("tmp.txt");  // ignorer volontairement, et le dire
```

## 4. Traiter un `Result` : toutes les écritures

La même fonction, qui lit un âge dans un texte avec 0 comme valeur de secours.

```rust
fn age_v1(t: &str) -> u32 {
    // 1. match : tous les cas, accès à l'erreur
    match t.trim().parse::<u32>() {
        Ok(n) => n,
        Err(e) => {
            eprintln!("âge invalide ({e}) : 0 par défaut");
            0
        }
    }
}

fn age_v2(t: &str) -> u32 {
    // 2. if let : seul le succès compte
    if let Ok(n) = t.trim().parse::<u32>() {
        n
    } else {
        0
    }
}

fn age_v3(t: &str) -> u32 {
    // 3. let else : le cas d'erreur sort de la fonction
    let Ok(n) = t.trim().parse::<u32>() else {
        return 0;
    };
    n
}

fn age_v4(t: &str) -> u32 {
    // 4. méthode : la plus courte
    t.trim().parse().unwrap_or(0)
}

fn age_v5(t: &str) -> u32 {
    // 5. unwrap_or_else : accès à l'erreur, valeur calculée seulement en cas d'échec
    t.trim().parse().unwrap_or_else(|e| {
        eprintln!("âge invalide ({e})");
        0
    })
}
```

### 4.1 — Les méthodes de `Result`

```rust
let r: Result<i32, String> = Ok(2);

// Extraire
r.clone().unwrap_or(0);
r.clone().unwrap_or_default();
r.clone().unwrap_or_else(|_e| 0);

// Transformer
r.clone().map(|n| n * 10);                       // Ok(20)
r.clone().map_err(|e| format!("erreur : {e}"));  // change le type d'erreur
r.clone().and_then(|n| if n > 0 { Ok(n) } else { Err("négatif".to_string()) });
r.clone().or_else(|_e| Ok::<i32, String>(0));    // récupérer une erreur
r.clone().map_or(0, |n| n * 2);                  // transformer ou valeur de secours

// Tester
r.is_ok();
r.is_err();
r.as_ref().is_ok_and(|&n| n > 1);
r.as_ref().is_err_and(|e| e.is_empty());

// Convertir
r.clone().ok();                                  // Option<i32> : l'erreur est perdue
r.clone().err();                                 // Option<String>

// Inspecter sans modifier
let _ = r.clone().inspect_err(|e| eprintln!("{e}"));
```

## 5. `unwrap` et `expect`

```rust
let n: i32 = "42".parse().unwrap();                    // panique avec l'erreur si échec
let n: i32 = "42".parse().expect("constante valide");  // panique avec TON message + l'erreur
```

| Écriture | Message de panique |
|---|---|
| `.unwrap()` | `called Result::unwrap() on an Err value: ParseIntError { kind: InvalidDigit }` |
| `.expect("le port vient de la config validée")` | `le port vient de la config validée: ParseIntError { kind: InvalidDigit }` |

`expect` documente **pourquoi** l'échec est impossible. Formule le message comme une attente (« le port est un nombre »), pas comme une plainte (« erreur ! »).

### Où `unwrap` est acceptable

```rust
// 1. Tests
#[test]
fn lit() { assert_eq!("1".parse::<i32>().unwrap(), 1); }

// 2. Valeur constante, vérifiable à l'œil
let ip: std::net::IpAddr = "127.0.0.1".parse().unwrap();

// 3. Juste après une vérification
let v = vec![1];
if !v.is_empty() {
    let premier = v.first().unwrap();      // mais `if let Some(p) = v.first()` est plus sûr
}

// 4. Verrou empoisonné : un autre thread a déjà paniqué
let m = std::sync::Mutex::new(0);
let garde = m.lock().unwrap();
```

Partout ailleurs, `unwrap` dans du code de production transforme une erreur récupérable en arrêt brutal.

## 6. L'opérateur `?`

### 6.1 — Ce qu'il fait

```rust
use std::fs;
use std::io;

fn lire_config() -> Result<String, io::Error> {
    let contenu = fs::read_to_string("config.toml")?;
    Ok(contenu.trim().to_string())
}
```

`expression?` est équivalent à :

```rust
let contenu = match fs::read_to_string("config.toml") {
    Ok(valeur) => valeur,
    Err(e) => return Err(From::from(e)),   // sort de la fonction avec l'erreur, convertie
};
```

Trois effets :

1. en cas de succès, donne la valeur ;
2. en cas d'erreur, **quitte la fonction** immédiatement ;
3. **convertit** l'erreur avec `From`, vers le type d'erreur de la fonction.

### 6.2 — Enchaîner

```rust
fn lire_port() -> Result<u16, Box<dyn std::error::Error>> {
    let port = std::fs::read_to_string("port.txt")?.trim().parse::<u16>()?;
    Ok(port)
}
```

Chaque `?` peut faire sortir la fonction. La ligne se lit de gauche à droite : lire, puis découper, puis convertir.

### 6.3 — `?` sur une `Option`

```rust
fn initiale_du_nom(texte: &str) -> Option<char> {
    let nom = texte.split_whitespace().nth(1)?;   // None : la fonction renvoie None
    nom.chars().next()
}
```

`?` fonctionne sur `Option` dans une fonction qui renvoie `Option`, sur `Result` dans une fonction qui renvoie `Result`. On ne mélange pas directement les deux (section 7).

### 6.4 — Où `?` ne marche pas

```rust
fn main() {
    // let n: i32 = "x".parse()?;    // ERREUR : main renvoie () (voir section 12)
}

let v = vec!["1", "x"];
// v.iter().map(|s| s.parse::<i32>()?);   // ERREUR : ? sortirait de la closure, qui ne renvoie pas Result
```

Dans une closure, `?` sort de la **closure**, pas de la fonction englobante. Pour une liste : `collect::<Result<Vec<_>, _>>()?`.

```rust
fn tous(v: &[&str]) -> Result<Vec<i32>, std::num::ParseIntError> {
    let nombres = v.iter().map(|s| s.parse::<i32>()).collect::<Result<Vec<_>, _>>()?;
    Ok(nombres)
}
```

## 7. `Option` et `Result` ensemble

### 7.1 — `Option` vers `Result`

```rust
fn trouver(id: u32) -> Option<String> { (id == 1).then(|| "Ada".to_string()) }

fn nom(id: u32) -> Result<String, String> {
    // 1. ok_or : l'erreur est construite dans tous les cas
    let n = trouver(id).ok_or("utilisateur introuvable".to_string())?;

    // 2. ok_or_else : l'erreur n'est construite qu'en cas d'absence
    let n = trouver(id).ok_or_else(|| format!("utilisateur {id} introuvable"))?;

    // 3. let else
    let Some(n) = trouver(id) else {
        return Err(format!("utilisateur {id} introuvable"));
    };

    Ok(n)
}
```

### 7.2 — `Result` vers `Option`

```rust
fn port(t: &str) -> Option<u16> {
    t.parse().ok()            // l'erreur est jetée : à réserver aux cas où sa cause n'intéresse personne
}
```

### 7.3 — Inverser l'imbrication

```rust
let a: Option<Result<i32, String>> = Some(Ok(1));
let b: Result<Option<i32>, String> = a.transpose();   // Ok(Some(1))

// Utile pour un champ optionnel qui doit être valide s'il est présent
fn age_optionnel(t: Option<&str>) -> Result<Option<u32>, std::num::ParseIntError> {
    t.map(|s| s.parse::<u32>()).transpose()
}
```

## 8. Les erreurs de la bibliothèque standard

| Type | Produit par | Contenu utile |
|---|---|---|
| `std::io::Error` | fichiers, réseau, entrées/sorties | `e.kind()` : `NotFound`, `PermissionDenied`... |
| `std::num::ParseIntError` | `"x".parse::<i32>()` | `e.kind()` : `Empty`, `InvalidDigit`, `PosOverflow`... |
| `std::num::ParseFloatError` | `parse::<f64>()` | |
| `std::str::Utf8Error` | `std::str::from_utf8` | position de l'octet invalide |
| `std::string::FromUtf8Error` | `String::from_utf8` | rend les octets : `e.into_bytes()` |
| `std::num::TryFromIntError` | `u8::try_from(300)` | |

### Réagir selon la cause

```rust
use std::io::ErrorKind;

fn lire_ou_creer(chemin: &str) -> std::io::Result<String> {
    match std::fs::read_to_string(chemin) {
        Ok(c) => Ok(c),
        Err(e) if e.kind() == ErrorKind::NotFound => {
            std::fs::write(chemin, "")?;
            Ok(String::new())
        }
        Err(e) => Err(e),
    }
}
```

`std::io::Result<T>` est un alias de `Result<T, std::io::Error>`. Beaucoup de crates définissent le leur (`fmt::Result`, `sea_orm::DbErr`, `AppResult` dans Runique).

## 9. Définir son propre type d'erreur

### 9.1 — Une enum par sous-système

```rust
#[derive(Debug)]
pub enum ErreurInscription {
    EmailInvalide(String),
    MotDePasseTropCourt { longueur: usize, minimum: usize },
    DejaInscrit,
}
```

Une enum laisse l'appelant **réagir** à chaque cas (afficher un message sous le bon champ, renvoyer un 409...), ce qu'une simple `String` ne permet pas.

### 9.2 — `Display` : le message

```rust
use std::fmt;

impl fmt::Display for ErreurInscription {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmailInvalide(e) => write!(f, "adresse email invalide : {e}"),
            Self::MotDePasseTropCourt { longueur, minimum } => {
                write!(f, "mot de passe trop court ({longueur} caractères, minimum {minimum})")
            }
            Self::DejaInscrit => write!(f, "cette adresse est déjà inscrite"),
        }
    }
}
```

### 9.3 — `Error` : le trait commun

```rust
impl std::error::Error for ErreurInscription {}
```

`Error` demande `Debug` et `Display`. Il rend ton erreur compatible avec `Box<dyn Error>`, `anyhow` et tous les outils qui affichent une chaîne de causes.

### 9.4 — Garder la cause : `source`

```rust
#[derive(Debug)]
pub enum ErreurConfig {
    Lecture(std::io::Error),
    Format(std::num::ParseIntError),
}

impl fmt::Display for ErreurConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Lecture(_) => write!(f, "impossible de lire la configuration"),
            Self::Format(_) => write!(f, "configuration mal formée"),
        }
    }
}

impl std::error::Error for ErreurConfig {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Lecture(e) => Some(e),
            Self::Format(e) => Some(e),
        }
    }
}
```

`source` relie l'erreur à celle qui l'a causée. Un outil d'affichage peut alors écrire « impossible de lire la configuration : fichier introuvable ». Règle : le `Display` décrit **ton** niveau, la cause est dans `source` ; ne répète pas le message de la cause dans les deux.

### 9.5 — Afficher toute la chaîne

```rust
fn afficher_chaine(e: &dyn std::error::Error) {
    eprintln!("erreur : {e}");
    let mut cause = e.source();
    while let Some(c) = cause {
        eprintln!("  cause : {c}");
        cause = c.source();
    }
}
```

### 9.6 — Ne pas fuiter d'information

Le message d'une erreur finit souvent devant un utilisateur ou dans une réponse HTTP. Ne mets pas dans le `Display` destiné au public : chemin de fichier du serveur, requête SQL, mot de passe, jeton, trace interne. Garde ces détails dans les logs (`Debug`, `source`), et montre au visiteur un message générique.

## 10. Convertir les erreurs pour `?`

### 10.1 — Le problème

```rust
fn lire_port() -> Result<u16, ErreurConfig> {
    let t = std::fs::read_to_string("port")?;      // io::Error, pas ErreurConfig
    let p = t.trim().parse::<u16>()?;               // ParseIntError
    Ok(p)
}
// ERREUR : `?` couldn't convert the error to `ErreurConfig`
```

### 10.2 — Trois solutions

```rust
// 1. map_err à chaque appel
fn lire_port_v1() -> Result<u16, ErreurConfig> {
    let t = std::fs::read_to_string("port").map_err(ErreurConfig::Lecture)?;
    let p = t.trim().parse::<u16>().map_err(ErreurConfig::Format)?;
    Ok(p)
}

// 2. Implémenter From une fois : ? convertit tout seul
impl From<std::io::Error> for ErreurConfig {
    fn from(e: std::io::Error) -> Self { Self::Lecture(e) }
}
impl From<std::num::ParseIntError> for ErreurConfig {
    fn from(e: std::num::ParseIntError) -> Self { Self::Format(e) }
}

fn lire_port_v2() -> Result<u16, ErreurConfig> {
    let t = std::fs::read_to_string("port")?;
    let p = t.trim().parse::<u16>()?;
    Ok(p)
}

// 3. Box<dyn Error> : tout se convertit, mais l'appelant ne peut plus distinguer les cas
fn lire_port_v3() -> Result<u16, Box<dyn std::error::Error>> {
    let t = std::fs::read_to_string("port")?;
    Ok(t.trim().parse::<u16>()?)
}
```

`map_err` sert quand la même erreur source doit donner des variantes différentes selon l'endroit (deux fichiers différents, deux champs différents). `From` sert quand une erreur source correspond toujours à la même variante.

`thiserror` génère `Display`, `Error`, `source` et `From` à partir d'attributs : voir `gestion-erreurs-avancee`.

## 11. `Box<dyn Error>`

```rust
use std::error::Error;

fn charger() -> Result<u32, Box<dyn Error>> {
    let t = std::fs::read_to_string("n.txt")?;       // io::Error convertie
    let n: u32 = t.trim().parse()?;                   // ParseIntError convertie
    if n == 0 {
        return Err("zéro interdit".into());           // une &str devient aussi une erreur
    }
    Ok(n)
}
```

- Accepte n'importe quelle erreur qui implémente `Error`.
- `"texte".into()` et `format!(..).into()` créent une erreur à partir d'un message.
- L'appelant ne peut plus faire de `match` sur les cas, seulement afficher ou essayer `downcast_ref`.

```rust
fn appel() {
    if let Err(e) = charger() {
        if let Some(io) = e.downcast_ref::<std::io::Error>() {
            eprintln!("problème de fichier : {}", io.kind());
        } else {
            eprintln!("autre erreur : {e}");
        }
    }
}
```

Pour des threads ou des tâches asynchrones : `Box<dyn Error + Send + Sync>`.

## 12. `main` qui renvoie une erreur

```rust
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let config = std::fs::read_to_string("config.toml")?;
    println!("{config}");
    Ok(())
}
```

Si `main` renvoie `Err`, le programme affiche l'erreur (avec `Debug`) et s'arrête avec le code 1. Pour un message plus soigné et un code de sortie choisi :

```rust
use std::process::ExitCode;

fn main() -> ExitCode {
    match lancer() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("erreur : {e}");
            ExitCode::from(2)
        }
    }
}

fn lancer() -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}
```

`std::process::exit(code)` arrête aussi le programme, mais **sans** appeler les `Drop` en cours (fichiers non vidés, connexions non fermées) : préfère renvoyer depuis `main`.

## 13. Où traiter une erreur

Une erreur se **propage** (`?`) tant que la fonction courante ne sait pas quoi en faire, et se **traite** là où une décision est possible.

```
lire_fichier()        → ne sait pas si l'absence est grave       → ?
charger_config()      → sait qu'une config absente = valeurs par défaut → traite
demarrer_serveur()    → une config illisible empêche de démarrer → ? jusqu'à main
main()                → affiche l'erreur et sort avec un code     → traite
```

Dans un handler web, l'erreur se traite en **réponse HTTP** : 404 pour une ressource absente, 400 pour une saisie invalide, 500 pour une panne interne (avec le détail dans les logs, pas dans la page).

### Ne pas avaler les erreurs

```rust
fn sauver(contenu: &str) {
    // Mauvais : l'échec disparaît sans trace
    let _ = std::fs::write("data.txt", contenu);

    // Mieux : au minimum, le journaliser
    if let Err(e) = std::fs::write("data.txt", contenu) {
        eprintln!("sauvegarde échouée : {e}");
    }
}
```

## 14. La même fonction écrite de plusieurs façons

Lire un fichier de nombres (un par ligne) et renvoyer leur somme.

```rust
use std::fs;

// 1. Prototype : panique à la moindre erreur
fn somme_v1(chemin: &str) -> i64 {
    fs::read_to_string(chemin).unwrap()
        .lines()
        .map(|l| l.trim().parse::<i64>().unwrap())
        .sum()
}

// 2. Box<dyn Error> : toutes les erreurs remontent, sans distinction
fn somme_v2(chemin: &str) -> Result<i64, Box<dyn std::error::Error>> {
    let mut total = 0;
    for ligne in fs::read_to_string(chemin)?.lines() {
        total += ligne.trim().parse::<i64>()?;
    }
    Ok(total)
}

// 3. Itérateur + collect dans un Result
fn somme_v3(chemin: &str) -> Result<i64, Box<dyn std::error::Error>> {
    let texte = fs::read_to_string(chemin)?;
    let nombres = texte
        .lines()
        .map(|l| l.trim().parse::<i64>())
        .collect::<Result<Vec<i64>, _>>()?;
    Ok(nombres.iter().sum())
}

// 4. Erreur typée : l'appelant sait quelle ligne est fausse
#[derive(Debug)]
enum ErreurSomme {
    Lecture(std::io::Error),
    Ligne { numero: usize, texte: String },
}

fn somme_v4(chemin: &str) -> Result<i64, ErreurSomme> {
    let texte = fs::read_to_string(chemin).map_err(ErreurSomme::Lecture)?;
    let mut total = 0;
    for (i, ligne) in texte.lines().enumerate() {
        let n: i64 = ligne.trim().parse().map_err(|_| ErreurSomme::Ligne {
            numero: i + 1,
            texte: ligne.to_string(),
        })?;
        total += n;
    }
    Ok(total)
}

// 5. Tolérante : ignore les lignes invalides
fn somme_v5(chemin: &str) -> std::io::Result<i64> {
    Ok(fs::read_to_string(chemin)?
        .lines()
        .filter_map(|l| l.trim().parse::<i64>().ok())
        .sum())
}
```

| Version | Pour qui |
|---|---|
| 1 | script jetable |
| 2, 3 | application : l'erreur est affichée, pas analysée |
| 4 | bibliothèque, ou interface qui doit indiquer précisément ce qui ne va pas |
| 5 | données réputées sales, où une ligne fausse n'est pas grave |

## 15. Exercices

### Exercice 1 — division sûre

Écris `diviser(a: i32, b: i32) -> Result<i32, String>` qui refuse la division par zéro **et** `i32::MIN / -1`.

```rust
// Solution :
fn diviser(a: i32, b: i32) -> Result<i32, String> {
    if b == 0 {
        return Err("division par zéro".into());
    }
    a.checked_div(b).ok_or_else(|| format!("{a} / {b} dépasse la capacité d'un i32"))
}
```

### Exercice 2 — erreur d'inscription

Valide un email (contient `@`) et un mot de passe (8 caractères minimum) avec l'enum `ErreurInscription` de la section 9, puis affiche le message.

```rust
// Solution :
fn valider(email: &str, mdp: &str) -> Result<(), ErreurInscription> {
    if !email.contains('@') {
        return Err(ErreurInscription::EmailInvalide(email.to_string()));
    }
    let longueur = mdp.chars().count();
    if longueur < 8 {
        return Err(ErreurInscription::MotDePasseTropCourt { longueur, minimum: 8 });
    }
    Ok(())
}

fn main() {
    if let Err(e) = valider("ada", "x") {
        println!("{e}");     // adresse email invalide : ada
    }
}
```

### Exercice 3 — conversion avec `From`

Fais compiler `lire_port_v2` (section 10) en n'écrivant que les deux `impl From`, puis vérifie qu'un fichier absent donne `ErreurConfig::Lecture`.

```rust
// Solution :
#[test]
fn fichier_absent() {
    assert!(matches!(lire_port_v2(), Err(ErreurConfig::Lecture(_))));
}
```

## 16. Aide-mémoire

```rust
panic!("msg")                         // bug : arrête le thread
Result<T, E>   Ok(v)   Err(e)         // erreur attendue
r?                                    // propage, convertit avec From
r.unwrap()   r.expect("pourquoi c'est impossible")
r.unwrap_or(v)   r.unwrap_or_else(|e| ..)   r.unwrap_or_default()
r.map(f)   r.map_err(f)   r.and_then(f)   r.ok()   r.is_ok_and(p)
opt.ok_or(e)   opt.ok_or_else(|| e)   opt?   opt_res.transpose()
iter.collect::<Result<Vec<_>, _>>()?
impl fmt::Display for MonErreur { .. }
impl std::error::Error for MonErreur { fn source(&self) -> .. }
impl From<io::Error> for MonErreur { .. }
fn main() -> Result<(), Box<dyn Error>> { .. }
```

### Points clés

- `panic!` pour les bugs, `Result` pour ce qui peut arriver normalement.
- Ne panique jamais sur une entrée extérieure.
- `?` = renvoyer l'erreur convertie par `From`, ou continuer avec la valeur.
- Dans une closure, `?` ne sort que de la closure.
- `expect` documente pourquoi un échec est impossible.
- Un type d'erreur : une enum, `Display`, `Error`, `source`, `From`.
- `Box<dyn Error>` pour une application, une enum typée pour une bibliothèque.
- Un message d'erreur public ne contient ni chemin, ni requête, ni secret.
