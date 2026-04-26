## Gestion d'Erreurs Avancée

Result, ?, Custom Errors et Best Practices 

Maîtriser l'Handling d'Erreurs en Rust 

## Objectifs du cours

À la fin de ce cours, tu sauras : 

- Utiliser l'opérateur ? efficacement 

- Créer tes propres types d'erreurs 

- Choisir entre panic! et Result 

- Utiliser thiserror et anyhow 

- Appliquer les best practices 

## Table des matières

1. L'opérateur ? en profondeur 

- 1.1 - Fonctionnement de base 

- 1.2 - Conversion automatique 

- 1.3 - Chaînage d'opérations 

2. panic! vs Result 

- 2.1 - Quand utiliser panic! 

- 2.2 - Quand utiliser Result 

- 2.3 - unwrap() et expect() 

3. Créer ses propres erreurs 

- 3.1 - Enum d'erreurs basique 

- 3.2 - Implémenter Error trait 

- 3.3 - Utiliser thiserror 

4. anyhow pour les applications 

- 4.1 - Box 

- 4.2 - anyhow::Result 

- 4.3 - Context et with_context 

5. Patterns avancés 

- 5.1 - Conversion d'erreurs 

- 5.2 - Erreurs avec backtrace 

- 5.3 - Early returns 

6. Best practices 

7. Exercices 

## 1. L'opérateur ? en profondeur

L'opérateur `?` est un sucre syntaxique pour propager les erreurs. C'est l'outil le plus utilisé en Rust ! 

## 1.1 - Fonctionnement de base

```
use std::fs::File;
use std::io::{self, Read};
```

```
// Sans ?
fn lire_fichier_verbeux(nom: &str) -> Result<String, io::Error> {
    let fichier = File::open(nom);
    let mut fichier = match fichier {
        Ok(f) => f,
        Err(e) => return Err(e),
    };
    let mut contenu = String::new();
    match fichier.read_to_string(&mut contenu) {
        Ok(_) => Ok(contenu),
        Err(e) => Err(e),
    }
}
// Avec ? (équivalent !)
fn lire_fichier(nom: &str) -> Result<String, io::Error> {
    let mut fichier = File::open(nom)?;
    let mut contenu = String::new();
    fichier.read_to_string(&mut contenu)?;
    Ok(contenu)
}
// Encore plus court
fn lire_fichier_court(nom: &str) -> Result<String, io::Error> {
    let mut contenu = String::new();
    File::open(nom)?.read_to_string(&mut contenu)?;
    Ok(contenu)
}
```

### Comment ? fonctionne :

1. Si `Ok(valeur)` → extrait la valeur 2. Si `Err(e)` → retourne immédiatement `Err(e)` 

## 1.2 - Conversion automatique

```
use std::fs::File;
use std::io;
use std::num::ParseIntError;
```

## `// Deux types d'erreurs différents !`

```
fn lire_et_parser(nom: &str) -> Result<i32, Box<dyn std::error::Error>> {
    let mut contenu = String::new();
    File::open(nom)?.read_to_string(&mut contenu)?;  // io::Error
    let nombre: i32 = contenu.trim().parse()?;       // ParseIntError
    Ok(nombre)
```

```
}
```

```
// ? convertit automatiquement vers le type d'erreur de retour
// grâce au trait From
```

## 1.3 - Chaînage d'opérations

```
use std::fs::File;
use std::io::Read;
```

```
fn obtenir_contenu() -> Result<String, std::io::Error> {
    let mut contenu = String::new();
```

```
    // Chaîner plusieurs opérations
    File::open("config.txt")?
        .read_to_string(&mut contenu)?;
```

```
    Ok(contenu)
}
```

```
// Avec des méthodes qui retournent Result
fn traiter_donnees() -> Result<i32, Box<dyn std::error::Error>> {
    let texte = std::fs::read_to_string("nombre.txt")?;
    let nombre: i32 = texte.trim().parse()?;
    let resultat = nombre.checked_mul(2)
        .ok_or("Overflow")?;
```

```
    Ok(resultat)
```

```
}
```

## 2. panic! vs Result

Choisir entre `panic!` et `Result` est crucial pour un code robuste. 

## 2.1 - Quand utiliser panic!

```
// 1. Bug dans le code (impossible normalement)
fn diviser(a: i32, b: i32) -> i32 {
    if b == 0 {
        panic!("Division par zéro - bug dans le code!");
    }
    a / b
}
```

```
// 2. Tests
#[test]
fn test_division() {
    assert_eq!(diviser(10, 2), 5);
}
```

```
// 3. Situation vraiment irrécupérable
fn initialiser_systeme() {
    let config = charger_config()
        .expect("Impossible de charger la config - arrêt");
    // Le programme ne peut pas continuer sans config
}
```

```
// 4. Prototypes et exemples
fn main() {
    let fichier = File::open("data.txt")
        .unwrap();  // OK pour un proto rapide
}
```

II **panic! termine le thread !** Dans une application web, ça peut tuer tout le serveur. Utilise `Result` pour les erreurs récupérables. 

## 2.2 - Quand utiliser Result

```
// 1. Erreurs attendues et récupérables
fn ouvrir_fichier(nom: &str) -> Result<File, io::Error> {
    File::open(nom)  // Le fichier peut ne pas exister
}
```

```
// 2. Opérations réseau
async fn fetch_data(url: &str) -> Result<String, reqwest::Error> {
    reqwest::get(url).await?.text().await
```

```
}
```

```
// 3. Parsing et validation
fn parser_age(texte: &str) -> Result<u8, String> {
    let age: u8 = texte.parse()
        .map_err(|_| "Pas un nombre valide".to_string())?;
    if age > 150 {
        return Err("Âge irréaliste".to_string());
    }
```

```
    Ok(age)
}
```

```
// 4. Logique métier
fn retirer_argent(compte: &mut Compte, montant: f64)
    -> Result<(), String>
```

```
{
    if montant > compte.solde {
        return Err("Solde insuffisant".to_string());
    }
    compte.solde -= montant;
    Ok(())
}
```

## 2.3 - unwrap() et expect()

```
// unwrap() - panic si Err
let x: Result<i32, &str> = Ok(5);
let valeur = x.unwrap();  // 5
```

**`let y: Result<i32, &str> = Err("erreur"); // let valeur = y.unwrap();  //`** I **`PANIC!`** 

```
// expect() - panic avec message personnalisé
let config = charger_config()
```

```
    .expect("Config manquante - vérifier config.toml");
```

```
// unwrap_or() - valeur par défaut si Err
let x: Result<i32, &str> = Err("erreur");
let valeur = x.unwrap_or(0);  // 0
```

```
// unwrap_or_else() - calculer la valeur par défaut
let valeur = x.unwrap_or_else(|err| {
    eprintln!("Erreur: {}", err);
    0
```

```
});
```

```
// unwrap_or_default() - valeur par défaut du type
let valeur: i32 = x.unwrap_or_default();  // 0
```

## 3. Créer ses propres erreurs

Pour un code professionnel, crée tes propres types d'erreurs spécifiques à ton domaine. 

## 3.1 - Enum d'erreurs basique

```
#[derive(Debug)]
enum ConfigError {
    FichierManquant,
    FormatInvalide,
    CleManquante(String),
}
fn charger_config(chemin: &str) -> Result<Config, ConfigError> {
    let contenu = std::fs::read_to_string(chemin)
        .map_err(|_| ConfigError::FichierManquant)?;
    // Parser le contenu...
    if !contenu.contains("port") {
        return Err(ConfigError::CleManquante("port".to_string()));
    }
    Ok(Config { /* ... */ })
}
```

```
// Utilisation
fn main() {
    match charger_config("config.toml") {
        Ok(config) => println!("Config chargée"),
        Err(ConfigError::FichierManquant) => {
            eprintln!("Créer config.toml");
        }
        Err(ConfigError::FormatInvalide) => {
            eprintln!("Format TOML invalide");
        }
        Err(ConfigError::CleManquante(cle)) => {
            eprintln!("Clé manquante: {}", cle);
        }
    }
```

```
}
```

## 3.2 - Implémenter Error trait

```
use std::fmt;
use std::error::Error;
#[derive(Debug)]
enum MonErreur {
    Io(std::io::Error),
    Parse(std::num::ParseIntError),
    Custom(String),
}
// Implémenter Display (requis)
impl fmt::Display for MonErreur {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MonErreur::Io(e) => write!(f, "Erreur IO: {}", e),
            MonErreur::Parse(e) => write!(f, "Erreur parse: {}", e),
            MonErreur::Custom(msg) => write!(f, "Erreur: {}", msg),
        }
    }
}
// Implémenter Error
impl Error for MonErreur {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            MonErreur::Io(e) => Some(e),
            MonErreur::Parse(e) => Some(e),
            MonErreur::Custom(_) => None,
        }
    }
}
// Conversions automatiques avec From
impl From<std::io::Error> for MonErreur {
    fn from(err: std::io::Error) -> Self {
        MonErreur::Io(err)
    }
}
impl From<std::num::ParseIntError> for MonErreur {
    fn from(err: std::num::ParseIntError) -> Self {
        MonErreur::Parse(err)
    }
}
```

## 3.3 - Utiliser thiserror

La crate `thiserror` génère automatiquement l'implémentation de `Error` . 

```
use thiserror::Error;
```

```
#[derive(Error, Debug)]
enum AppError {
    #[error("Fichier non trouvé: {0}")]
    FileNotFound(String),
    #[error("Erreur IO")]
    Io(#[from] std::io::Error),
    #[error("Erreur de parsing")]
    Parse(#[from] std::num::ParseIntError),
    #[error("Valeur invalide: {valeur}, attendu: {attendu}")]
    InvalidValue { valeur: i32, attendu: i32 },
```

```
    #[error("Erreur réseau: {0}")]
    Network(#[from] reqwest::Error),
}
// Utilisation (beaucoup plus simple !)
fn traiter() -> Result<(), AppError> {
    let contenu = std::fs::read_to_string("data.txt")?;  // Converti auto
    let nombre: i32 = contenu.trim().parse()?;            // Converti auto
    if nombre < 0 {
        return Err(AppError::InvalidValue {
            valeur: nombre,
            attendu: 0,
        });
    }
    Ok(())
}
```

I **thiserror** est parfait pour les **bibliothèques** où tu veux des types d'erreurs précis et bien définis. 

## 4. anyhow pour les applications

Pour les **applications** (pas les bibliothèques), `anyhow` simplifie énormément la gestion d'erreurs. 

## 4.1 - Box<dyn Error>

## `use std::error::Error;`

## `// Type d'erreur générique`

```
fn faire_tout() -> Result<String, Box<dyn Error>> {
    let contenu = std::fs::read_to_string("file.txt")?;
    let nombre: i32 = contenu.trim().parse()?;
    let resultat = fetch_data(&nombre.to_string()).await?;
    Ok(resultat)
```

## `}`

## `// Avantages:`

**`//`** I **`Accepte n'importe quel type d'erreur`** 

**`//`** I **`Simple pour débuter`** 

## `// Inconvénients:`

**`//`** I **`Allocation heap`** 

**`//`** I **`Perd le type exact de l'erreur`** 

**`//`** I **`Pas de downcast facile`** 

## 4.2 - anyhow::Result

```
use anyhow::{Result, Context};
```

```
// Type alias: Result<T> = Result<T, anyhow::Error>
fn charger_config() -> Result<Config> {
    let contenu = std::fs::read_to_string("config.toml")
        .context("Impossible de lire config.toml")?;
```

```
    let config: Config = toml::from_str(&contenu)
        .context("Format TOML invalide")?;
```

```
    Ok(config)
}
```

```
// Avantages d'anyhow:
```

**`//`** I **`Messages d'erreur détaillés automatiques`** 

**`//`** I **`Backtrace si RUST_BACKTRACE=1`** 

**`//`** I **`Plus simple que Box<dyn Error>`** 

- **`//`** I **`Conversion automatique de tous les types d'erreur`** 

## 4.3 - Context et with_context

```
use anyhow::{Context, Result};
```

```
fn traiter_fichier(chemin: &str) -> Result<()> {
    let contenu = std::fs::read_to_string(chemin)
        .context(format!("Lecture de {}", chemin))?;
    let lignes: Vec<&str> = contenu.lines().collect();
    for (i, ligne) in lignes.iter().enumerate() {
        traiter_ligne(ligne)
            .with_context(|| format!("Erreur ligne {}", i + 1))?;
    }
```

```
    Ok(())
}
```

```
// Chaîne de contexte complète en cas d'erreur:
// Error: Erreur ligne 5
// Caused by:
//     0: Valeur invalide
//     1: Lecture de data.txt
```

### Quand utiliser quoi ?

• **Bibliothèques** → `thiserror` (types d'erreurs précis) 

• **Applications** → `anyhow` (simplicité et contexte) 

## 5. Patterns avancés

## 5.1 - Conversion d'erreurs

```
// map_err pour convertir les erreurs
fn lire_nombre(chemin: &str) -> Result<i32, String> {
    let contenu = std::fs::read_to_string(chemin)
        .map_err(|e| format!("Lecture échouée: {}", e))?;
    contenu.trim()
        .parse()
        .map_err(|e| format!("Parse échoué: {}", e))
}
```

```
// ok_or et ok_or_else pour Option -> Result
fn trouver_user(id: u32) -> Result<User, String> {
    DATABASE.get(&id)
        .ok_or_else(|| format!("User {} introuvable", id))
}
```

```
// and_then pour chaîner des Results
fn traiter() -> Result<i32, String> {
    lire_fichier("data.txt")
        .and_then(|contenu| parser(&contenu))
        .and_then(|nombre| calculer(nombre))
}
```

## 5.2 - Erreurs avec backtrace

```
use std::backtrace::Backtrace;
```

```
#[derive(Debug)]
struct MonErreur {
    message: String,
    backtrace: Backtrace,
}
impl MonErreur {
    fn new(message: String) -> Self {
        Self {
            message,
            backtrace: Backtrace::capture(),
        }
    }
}
// Avec anyhow, le backtrace est automatique !
// RUST_BACKTRACE=1 cargo run
```

## 5.3 - Early returns

```
// Pattern: vérifier les conditions tôt
fn traiter_commande(cmd: &Commande) -> Result<(), String> {
    // Vérifications en premier
    if cmd.montant <= 0.0 {
        return Err("Montant invalide".to_string());
    }
```

```
    if cmd.items.is_empty() {
        return Err("Commande vide".to_string());
    }
```

```
    if !cmd.client_existe() {
        return Err("Client inconnu".to_string());
    }
```

```
    // Logique principale ensuite
    valider_stock(cmd)?;
    calculer_prix(cmd)?;
    enregistrer(cmd)?;
```

```
    Ok(())
```

```
}
```

## 6. Best practices

## • **1. Ne pas abuser de unwrap()** 

Utilise `?` , `unwrap_or()` ou `expect()` avec un message clair. 

## • **2. Propager les erreurs** 

Laisse l'appelant décider comment gérer l'erreur. N'utilise `panic!` que pour les bugs. 

## • **3. Messages d'erreur clairs** 

Inclus le contexte : quel fichier, quelle ligne, quelle valeur était attendue. 

## • **4. Types d'erreurs spécifiques** 

Dans les bibliothèques, crée des enums d'erreurs précis avec `thiserror` . 

## • **5. anyhow pour les apps** 

Pour les applications, utilise `anyhow` avec `.context()` . 

## • **6. Documenter les erreurs** 

Dans la doc, indique quelles erreurs peuvent survenir et pourquoi. 

## • **7. Tests d'erreurs** 

Teste les cas d'erreur autant que les cas de succès ! 

## 7. Exercices pratiques

### Exercice 1 : Créer un type d'erreur

```
// Crée un enum ValidationError pour valider un email
// Doit gérer: vide, pas de @, domaine invalide
```

```
// Solution avec thiserror:
use thiserror::Error;
```

```
#[derive(Error, Debug)]
enum ValidationError {
    #[error("Email vide")]
    Empty,
```

```
    #[error("@ manquant")]
    NoAtSign,
```

```
    #[error("Domaine invalide: {0}")]
    InvalidDomain(String),
}
```

```
fn valider_email(email: &str) -> Result<(), ValidationError> {
    if email.is_empty() {
```

```
        return Err(ValidationError::Empty);
    }
```

```
    if !email.contains('@') {
        return Err(ValidationError::NoAtSign);
    }
```

```
    let parts: Vec<&str> = email.split('@').collect();
    if parts.len() != 2 || !parts[1].contains('.') {
        return Err(ValidationError::InvalidDomain(
            parts.get(1).unwrap_or(&"").to_string()
        ));
    }
```

```
    Ok(())
```

```
}
```

## Exercice 2 : Utiliser anyhow

```
// Réécris cette fonction avec anyhow et context
```

```
use anyhow::{Context, Result};
```

```
fn charger_et_traiter(chemin: &str) -> Result<i32> {
    let contenu = std::fs::read_to_string(chemin)
        .context(format!("Impossible de lire {}", chemin))?;
```

```
    let nombre: i32 = contenu.trim()
        .parse()
        .context("Le fichier ne contient pas un nombre valide")?;
```

```
    if nombre < 0 {
        anyhow::bail!("Le nombre doit être positif");
```

- **`}`** 

```
    Ok(nombre * 2)
}
```

### Excellent !

Tu maîtrises maintenant la gestion d'erreurs en Rust ! 

Points clés à retenir : • Utilise `?` pour propager • `thiserror` pour les libs • `anyhow` pour les apps • Toujours ajouter du contexte ! 

I **Ton code sera robuste !** I
