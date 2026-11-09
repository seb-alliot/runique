# Traits — Les bases
> Un trait décrit un comportement que plusieurs types peuvent partager. Ce cours montre comment en définir, en implémenter, les utiliser en paramètre, et implémenter les traits de la bibliothèque standard.

## Objectifs

- Définir un trait avec des méthodes, des fonctions associées et des constantes
- L'implémenter pour ses types, et pour des types d'autres crates
- Écrire des méthodes par défaut qui s'appuient sur les autres
- Utiliser un trait en paramètre et en retour, de toutes les façons
- Implémenter à la main les traits standard les plus utiles (`Display`, `From`, `FromStr`, `Default`, `PartialEq`, `Ord`)
- Comprendre la règle de cohérence (orphelin) et les traits d'extension
- Lever une ambiguïté quand deux traits ont une méthode du même nom

## Table des matières

1. Qu'est-ce qu'un trait
2. Définir un trait
3. Implémenter un trait
4. Méthodes par défaut
5. Utiliser un trait : paramètres et retours
6. Le trait doit être importé
7. Dériver ou implémenter à la main
8. Les traits standard à connaître
9. Implémenter `Display`
10. Conversions : `From`, `Into`, `TryFrom`, `FromStr`
11. Comparaisons : `PartialEq`, `Eq`, `PartialOrd`, `Ord`, `Hash`
12. Plusieurs traits sur un même type
13. La règle de cohérence et les traits d'extension
14. La même fonctionnalité écrite de plusieurs façons
15. Exercices pratiques
16. Aide-mémoire

---

## 1. Qu'est-ce qu'un trait

Un **trait** est un ensemble de méthodes qu'un type promet de fournir. C'est l'équivalent des interfaces en Java ou en TypeScript, avec deux différences :

- on peut implémenter un trait pour un type **après coup**, sans modifier sa définition (même `i32` ou `String`) ;
- un trait peut fournir du code par défaut.

```rust
trait Decrire {
    fn decrire(&self) -> String;
}

struct Chien { nom: String }
struct Voiture { modele: String }

impl Decrire for Chien {
    fn decrire(&self) -> String { format!("le chien {}", self.nom) }
}

impl Decrire for Voiture {
    fn decrire(&self) -> String { format!("une {}", self.modele) }
}

fn presenter(x: &impl Decrire) {
    println!("Voici {}", x.decrire());
}
```

`Chien` et `Voiture` n'ont rien en commun, sauf ce comportement. `presenter` accepte n'importe quel type qui l'a.

---

## 2. Définir un trait

```rust
trait Forme {
    // Méthode à implémenter : seulement la signature
    fn aire(&self) -> f64;

    // Méthode qui modifie
    fn agrandir(&mut self, facteur: f64);

    // Fonction associée (pas de self) : un constructeur commun
    fn unite() -> Self where Self: Sized;

    // Constante associée
    const COTES: u32;

    // Méthode par défaut (section 4)
    fn description(&self) -> String {
        format!("forme de {} côtés, aire {:.2}", Self::COTES, self.aire())
    }
}
```

| Élément | Syntaxe | Appel |
|---|---|---|
| méthode | `fn f(&self)` | `valeur.f()` |
| méthode qui modifie | `fn f(&mut self)` | `valeur.f()` |
| méthode qui consomme | `fn f(self)` | `valeur.f()` |
| fonction associée | `fn f() -> Self` | `Type::f()` ou `T::f()` |
| constante | `const N: u32;` | `Type::N` ou `T::N` |

---

## 3. Implémenter un trait

```rust
struct Carre { cote: f64 }

impl Forme for Carre {
    fn aire(&self) -> f64 { self.cote * self.cote }
    fn agrandir(&mut self, facteur: f64) { self.cote *= facteur; }
    fn unite() -> Self { Carre { cote: 1.0 } }
    const COTES: u32 = 4;
    // description n'est pas écrite : la version par défaut s'applique
}

fn main() {
    let mut c = Carre::unite();
    c.agrandir(3.0);
    println!("{}", c.description());   // forme de 4 côtés, aire 9.00
}
```

Toutes les méthodes **sans** corps doivent être écrites, avec exactement la même signature. Le compilateur liste celles qui manquent (`missing ... in implementation`).

### Le bloc `impl Trait for Type` est séparé du bloc `impl Type`

```rust
impl Carre {
    fn diagonale(&self) -> f64 { self.cote * 2f64.sqrt() }   // méthode propre à Carre
}

impl Forme for Carre {
    // seulement les éléments déclarés par le trait : aire, agrandir, unite, COTES
    // (et éventuellement description, pour remplacer la version par défaut)
}
```

On ne peut pas ajouter de méthode supplémentaire dans un `impl Trait for Type` : elle irait dans `impl Type`.

---

## 4. Méthodes par défaut

### 4.1 Une méthode par défaut peut appeler les autres

```rust
trait Resume {
    fn auteur(&self) -> String;

    fn resume(&self) -> String {
        format!("(Lire la suite de {}…)", self.auteur())
    }
}

struct Tweet { utilisateur: String, texte: String }

impl Resume for Tweet {
    fn auteur(&self) -> String { format!("@{}", self.utilisateur) }
    // resume est fourni
}
```

Le trait demande le **minimum** (`auteur`) et construit le reste. C'est le principe de `Iterator` : on écrit `next`, on obtient `map`, `filter`, `sum` et 70 autres méthodes.

### 4.2 Remplacer une méthode par défaut

```rust
struct Article { titre: String, auteur: String }

impl Resume for Article {
    fn auteur(&self) -> String { self.auteur.clone() }
    fn resume(&self) -> String {                      // remplace la version par défaut
        format!("{}, par {}", self.titre, self.auteur)
    }
}
```

Une implémentation qui remplace la méthode par défaut **ne peut pas** appeler la version par défaut (il n'y a pas de `super`). Pour la réutiliser, on la place dans une fonction libre que les deux appellent.

### 4.3 Concevoir un trait

Demande le moins de méthodes obligatoires possible, et fournis par défaut tout ce qui peut s'en déduire. Ajouter plus tard une méthode **avec** défaut ne casse personne ; ajouter une méthode **obligatoire** casse toutes les implémentations existantes.

---

## 5. Utiliser un trait : paramètres et retours

### 5.1 En paramètre : la même fonction de quatre façons

```rust
// 1. impl Trait
fn notifier_v1(x: &impl Resume) {
    println!("{}", x.resume());
}

// 2. Générique avec contrainte
fn notifier_v2<T: Resume>(x: &T) {
    println!("{}", x.resume());
}

// 3. where
fn notifier_v3<T>(x: &T)
where
    T: Resume,
{
    println!("{}", x.resume());
}

// 4. Trait object : un seul exemplaire compilé, choix à l'exécution
fn notifier_v4(x: &dyn Resume) {
    println!("{}", x.resume());
}
```

Les trois premières sont équivalentes (génériques, voir `generics`). La quatrième est dynamique (voir `box-dynamiques`).

### 5.2 Plusieurs contraintes

```rust
use std::fmt::Display;

fn afficher_resume(x: &(impl Resume + Display)) {
    println!("{x} : {}", x.resume());
}

fn afficher_resume2<T: Resume + Display>(x: &T) {
    println!("{x} : {}", x.resume());
}
```

### 5.3 En retour

```rust
fn creer() -> impl Resume {
    Tweet { utilisateur: "rust".into(), texte: "1.0".into() }
}
```

`impl Resume` en retour cache le type concret, mais il doit être **unique** :

```rust
// fn creer(court: bool) -> impl Resume {
//     if court { Tweet { .. } } else { Article { .. } }    // ERREUR : deux types différents
// }

fn creer_dyn(court: bool) -> Box<dyn Resume> {
    if court {
        Box::new(Tweet { utilisateur: "a".into(), texte: "b".into() })
    } else {
        Box::new(Article { titre: "t".into(), auteur: "a".into() })
    }
}
```

### 5.4 Appeler une fonction associée sur un générique

```rust
trait Defaut {
    fn defaut() -> Self;
}

impl Defaut for u32 { fn defaut() -> Self { 0 } }

fn remplir<T: Defaut>(n: usize) -> Vec<T> {
    (0..n).map(|_| T::defaut()).collect()        // T::f() : fonction associée du type générique
}

let v: Vec<u32> = remplir(3);
```

---

## 6. Le trait doit être importé

```rust
mod formes {
    pub trait Aire { fn aire(&self) -> f64; }
    impl Aire for f64 { fn aire(&self) -> f64 { self * self } }
}

fn main() {
    let x = 2.0_f64;
    // x.aire();                 // ERREUR : no method named `aire` found
    use formes::Aire;            // le trait doit être en portée
    x.aire();
}
```

Les méthodes d'un trait ne sont visibles **que si le trait est importé**. C'est voulu : deux crates peuvent ajouter une méthode `aire` à `f64` sans conflit, tant qu'un même fichier n'importe pas les deux. Les préludes (`use runique::prelude::*`, `use std::io::prelude::*`) importent les traits courants d'un coup.

---

## 7. Dériver ou implémenter à la main

```rust
#[derive(Debug, Clone, PartialEq, Default)]
struct Point { x: i32, y: i32 }
```

`#[derive(...)]` génère l'implémentation la plus évidente, champ par champ. On l'écrit à la main quand le comportement voulu est différent :

| Trait | Dérivé | À la main quand... |
|---|---|---|
| `Debug` | tous les champs | un champ est secret (mot de passe, jeton) |
| `Clone` | clone chaque champ | rarement |
| `PartialEq` | compare tous les champs | l'égalité porte sur un identifiant seulement |
| `Default` | `Default` de chaque champ | zéro n'a pas de sens (port, limite, délai) |
| `PartialOrd`/`Ord` | ordre des champs | l'ordre voulu n'est pas celui des champs |
| `Hash` | tous les champs | doit rester cohérent avec un `PartialEq` écrit à la main |

### Exemple : `Debug` sans le secret

```rust
use std::fmt;

struct Identifiants { utilisateur: String, mot_de_passe: String }

impl fmt::Debug for Identifiants {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Identifiants")
            .field("utilisateur", &self.utilisateur)
            .field("mot_de_passe", &"***")
            .finish()
    }
}
```

Un `#[derive(Debug)]` afficherait le mot de passe dans les logs au premier `{:?}` ou au premier `dbg!`.

---

## 8. Les traits standard à connaître

| Trait | Sert à | Se dérive |
|---|---|---|
| `Debug` | `{:?}`, `dbg!`, `assert_eq!` | oui |
| `Display` | `{}`, `.to_string()` | non |
| `Clone`, `Copy` | copier | oui |
| `Default` | valeur par défaut | oui |
| `PartialEq`, `Eq` | `==` | oui |
| `PartialOrd`, `Ord` | `<`, tri | oui |
| `Hash` | clé de `HashMap` | oui |
| `From`, `Into` | conversion sans échec | non |
| `TryFrom`, `TryInto` | conversion qui peut échouer | non |
| `FromStr` | `"..".parse::<T>()` | non |
| `AsRef`, `Borrow` | emprunter sous une autre forme | non |
| `Iterator`, `IntoIterator` | parcourir, `for` | non |
| `Deref`, `Drop` | smart pointers | non |
| `Add`, `Sub`, `Mul`, `Index`... | opérateurs (voir `traits-avances`) | non |
| `Error` | type d'erreur | non (`thiserror`) |
| `Send`, `Sync` | threads (automatiques) | — |
| `Serialize`, `Deserialize` (serde) | sérialisation | oui, avec serde |

---

## 9. Implémenter `Display`

```rust
use std::fmt;

struct Prix { centimes: i64 }

impl fmt::Display for Prix {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{},{:02} €", self.centimes / 100, self.centimes % 100)
    }
}

let p = Prix { centimes: 1250 };
println!("{p}");                 // 12,50 €
let texte = p.to_string();       // offert par Display
```

### Respecter les options de formatage

```rust
struct Code(&'static str);

impl fmt::Display for Code {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.pad(self.0)               // pad applique largeur et alignement demandés
    }
}

println!("[{:>8}]", Code("abc"));  // [     abc]
```

`write!(f, ...)` ignore `{:>8}` ; `f.pad(...)` le respecte.

---

## 10. Conversions : `From`, `Into`, `TryFrom`, `FromStr`

### 10.1 `From` : on écrit l'un, on obtient l'autre

```rust
struct Celsius(f64);
struct Fahrenheit(f64);

impl From<Celsius> for Fahrenheit {
    fn from(c: Celsius) -> Self { Fahrenheit(c.0 * 9.0 / 5.0 + 32.0) }
}

let f = Fahrenheit::from(Celsius(100.0));
let f: Fahrenheit = Celsius(100.0).into();     // Into vient gratuitement
```

On implémente **toujours `From`**, jamais `Into` : la bibliothèque standard fournit `Into` automatiquement pour tout `From`.

### 10.2 `TryFrom` : conversion qui peut échouer

```rust
struct Age(u8);

impl TryFrom<i32> for Age {
    type Error = String;
    fn try_from(v: i32) -> Result<Self, Self::Error> {
        if (0..=150).contains(&v) {
            Ok(Age(v as u8))
        } else {
            Err(format!("âge impossible : {v}"))
        }
    }
}

let a = Age::try_from(30);
let b: Result<Age, _> = 200_i32.try_into();   // le suffixe dit quelle conversion utiliser
```

### 10.3 `FromStr` : pour `parse`

```rust
use std::str::FromStr;

#[derive(Debug)]
enum Couleur { Rouge, Vert, Bleu }

impl FromStr for Couleur {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "rouge" => Ok(Couleur::Rouge),
            "vert" => Ok(Couleur::Vert),
            "bleu" => Ok(Couleur::Bleu),
            autre => Err(format!("couleur inconnue : {autre}")),
        }
    }
}

let c: Couleur = "Rouge".parse().unwrap();
let d = "violet".parse::<Couleur>();          // Err
```

`FromStr` (texte → valeur) et `Display` (valeur → texte) forment une paire : un type qui a les deux s'écrit et se relit.

---

## 11. Comparaisons : `PartialEq`, `Eq`, `PartialOrd`, `Ord`, `Hash`

### 11.1 Égalité sur un identifiant

```rust
use std::hash::{Hash, Hasher};

struct Utilisateur { id: u32, nom: String }

impl PartialEq for Utilisateur {
    fn eq(&self, autre: &Self) -> bool { self.id == autre.id }
}
impl Eq for Utilisateur {}

impl Hash for Utilisateur {
    fn hash<H: Hasher>(&self, h: &mut H) { self.id.hash(h); }   // cohérent avec eq
}
```

Règle impérative : si `a == b`, alors `hash(a) == hash(b)`. Un `Hash` qui inclurait `nom` alors que `eq` ne le compare pas casserait les `HashMap` en silence.

### 11.2 `PartialEq` ou `Eq`, `PartialOrd` ou `Ord`

- `PartialEq` : `==` existe, mais certaines valeurs peuvent ne pas être égales à elles-mêmes (`f64::NAN != f64::NAN`).
- `Eq` : l'égalité est totale (`a == a` toujours). Aucune méthode à écrire : c'est une promesse.
- `PartialOrd` : certaines paires ne sont pas comparables (`NaN`).
- `Ord` : tout est comparable ; requis par `sort()`, `BTreeMap`, `max()`.

### 11.3 Un ordre choisi

```rust
use std::cmp::Ordering;

#[derive(PartialEq, Eq)]
struct Tache { priorite: u8, titre: String }

impl Ord for Tache {
    fn cmp(&self, autre: &Self) -> Ordering {
        autre.priorite.cmp(&self.priorite)              // priorité décroissante
            .then_with(|| self.titre.cmp(&autre.titre))  // puis titre croissant
    }
}

impl PartialOrd for Tache {
    fn partial_cmp(&self, autre: &Self) -> Option<Ordering> { Some(self.cmp(autre)) }
}
```

`PartialOrd` délègue à `Ord` pour rester cohérent : c'est la forme standard.

Souvent, un tri ponctuel suffit sans implémenter `Ord` : `v.sort_by_key(|t| std::cmp::Reverse(t.priorite))`.

---

## 12. Plusieurs traits sur un même type

### 12.1 Un type, plusieurs comportements

```rust
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
struct Temperature(f64);

impl fmt::Display for Temperature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{:.1} °C", self.0) }
}

impl From<f64> for Temperature {
    fn from(v: f64) -> Self { Temperature(v) }
}

impl Default for Temperature {
    fn default() -> Self { Temperature(20.0) }
}
```

### 12.2 Deux traits avec une méthode du même nom

```rust
trait Pilote { fn voler(&self) -> &'static str; }
trait Sorcier { fn voler(&self) -> &'static str; }

struct Humain;

impl Pilote for Humain { fn voler(&self) -> &'static str { "décollage" } }
impl Sorcier for Humain { fn voler(&self) -> &'static str { "balai" } }
impl Humain { fn voler(&self) -> &'static str { "bras qui battent" } }

fn main() {
    let h = Humain;
    h.voler();                       // la méthode propre au type gagne : "bras qui battent"
    Pilote::voler(&h);               // "décollage"
    Sorcier::voler(&h);              // "balai"
    <Humain as Pilote>::voler(&h);   // forme complète
}
```

Pour une fonction associée sans `self`, seule la forme complète marche : `<Humain as Pilote>::nom()`.

---

## 13. La règle de cohérence et les traits d'extension

### 13.1 La règle de l'orphelin

On peut écrire `impl Trait for Type` seulement si **le trait ou le type** est défini dans ta crate.

```rust
use std::fmt;

struct MonType;
trait MonTrait {}

impl fmt::Display for MonType {                          // OK : le type est local
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "mon type")
    }
}

impl MonTrait for Vec<i32> {}                            // OK : le trait est local

// impl fmt::Display for Vec<i32> { .. }                 // ERREUR E0117 : ni l'un ni l'autre
```

Pourquoi : si deux crates pouvaient chacune implémenter `Display` pour `Vec<i32>`, le compilateur ne saurait pas laquelle choisir dans un programme qui utilise les deux.

### 13.2 Contourner : le newtype

```rust
use std::fmt;

struct Liste(Vec<i32>);

impl fmt::Display for Liste {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let textes: Vec<String> = self.0.iter().map(|x| x.to_string()).collect();
        write!(f, "[{}]", textes.join(" | "))
    }
}
```

### 13.3 Ajouter des méthodes à un type étranger : le trait d'extension

```rust
trait TexteExt {
    fn est_vide_ou_espaces(&self) -> bool;
    fn tronquer(&self, n: usize) -> String;
}

impl TexteExt for str {
    fn est_vide_ou_espaces(&self) -> bool {
        self.trim().is_empty()
    }
    fn tronquer(&self, n: usize) -> String {
        self.chars().take(n).collect()
    }
}

fn main() {
    "   ".est_vide_ou_espaces();          // true
    "bonjour".tronquer(3);                // "bon"
    String::from("abc").tronquer(1);      // marche aussi sur String (Deref vers str)
}
```

Le trait est local : la règle de l'orphelin est respectée. Convention de nommage : `NomDuTypeExt`. `itertools::Itertools` ajoute ainsi des dizaines de méthodes à tous les itérateurs.

---

## 14. La même fonctionnalité écrite de plusieurs façons

Calculer le prix TTC de plusieurs sortes d'articles.

```rust
// 1. Une fonction par type : duplication
fn ttc_livre(prix: f64) -> f64 { prix * 1.055 }
fn ttc_jeu(prix: f64) -> f64 { prix * 1.20 }

// 2. Une enum : ensemble fermé, un match
enum Article { Livre(f64), Jeu(f64) }
impl Article {
    fn ttc(&self) -> f64 {
        match self {
            Article::Livre(p) => p * 1.055,
            Article::Jeu(p) => p * 1.20,
        }
    }
}

// 3. Un trait avec constante associée et méthode par défaut : ensemble ouvert
trait Taxable {
    const TAUX: f64;
    fn prix_ht(&self) -> f64;
    fn ttc(&self) -> f64 { self.prix_ht() * (1.0 + Self::TAUX) }
}

struct Livre(f64);
struct Jeu(f64);
impl Taxable for Livre { const TAUX: f64 = 0.055; fn prix_ht(&self) -> f64 { self.0 } }
impl Taxable for Jeu { const TAUX: f64 = 0.20; fn prix_ht(&self) -> f64 { self.0 } }

fn total<T: Taxable>(articles: &[T]) -> f64 {
    articles.iter().map(|a| a.ttc()).sum()
}
```

| Version | Ajouter un type d'article | Ajouter une opération |
|---|---|---|
| fonctions | une fonction de plus par opération | une fonction par type |
| enum | modifier l'enum et chaque `match` | un `match` de plus |
| trait | un `impl` de plus, ailleurs | modifier le trait (avec un défaut : sans casser) |

Note : un trait avec une **constante associée** n'est pas utilisable en `dyn Taxable` (voir `box-dynamiques`) ; pour des articles mélangés dans un `Vec`, on remplacerait `TAUX` par une méthode `fn taux(&self) -> f64`.

---

## 15. Exercices pratiques

### Exercice 1 — trait `Convertible`

Définis `Convertible` avec `fn en_texte(&self) -> String` et une méthode par défaut `fn en_texte_majuscules(&self) -> String`. Implémente-le pour `i32` et pour une struct `Personne { nom: String }`.

```rust
// Solution :
trait Convertible {
    fn en_texte(&self) -> String;
    fn en_texte_majuscules(&self) -> String {
        self.en_texte().to_uppercase()
    }
}

impl Convertible for i32 {
    fn en_texte(&self) -> String { self.to_string() }
}

struct Personne { nom: String }

impl Convertible for Personne {
    fn en_texte(&self) -> String { format!("Personne({})", self.nom) }
}
```

### Exercice 2 — `FromStr` et `Display`

Écris `struct Version { majeur: u32, mineur: u32 }` qui se lit depuis `"4.0"` et s'affiche `"4.0"`.

```rust
// Solution :
use std::fmt;
use std::str::FromStr;

struct Version { majeur: u32, mineur: u32 }

impl FromStr for Version {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (a, b) = s.split_once('.').ok_or("format attendu : X.Y")?;
        Ok(Version {
            majeur: a.parse().map_err(|_| "majeur invalide")?,
            mineur: b.parse().map_err(|_| "mineur invalide")?,
        })
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}", self.majeur, self.mineur)
    }
}

let v: Version = "4.0".parse().unwrap();
assert_eq!(v.to_string(), "4.0");
```

`ok_or("...")?` et `map_err(|_| "...")?` fonctionnent ici parce que `String: From<&str>` : `?` convertit le `&str` en `String`.

### Exercice 3 — tri par un ordre choisi

Trie un `Vec<Version>` de la plus récente à la plus ancienne, sans implémenter `Ord`, puis en l'implémentant.

```rust
// Solution 1 : sort_by
fn trier(v: &mut [Version]) {
    v.sort_by(|a, b| (b.majeur, b.mineur).cmp(&(a.majeur, a.mineur)));
}

// Solution 2 : Ord (ordre croissant naturel), puis reverse
// #[derive(PartialEq, Eq, PartialOrd, Ord)] struct Version { majeur: u32, mineur: u32 }
// v.sort(); v.reverse();
```

Avec `derive`, l'ordre compare `majeur` puis `mineur` : l'ordre des champs fait l'ordre de tri.

---

## 16. Aide-mémoire

```rust
trait T {
    fn obligatoire(&self) -> u32;
    fn par_defaut(&self) -> u32 { self.obligatoire() * 2 }
    fn associee() -> Self where Self: Sized;
    const N: u32;
}
impl T for MonType { .. }

fn f(x: &impl T)              fn f<X: T>(x: &X)          fn f<X>(x: &X) where X: T
fn f(x: &dyn T)               fn f() -> impl T           fn f() -> Box<dyn T>
X::associee()                 <MonType as T>::associee()  T::methode(&x)

impl fmt::Display for X       impl From<A> for B         impl TryFrom<A> for B
impl FromStr for X            impl PartialEq for X       impl Ord + PartialOrd
trait StrExt { .. } impl StrExt for str { .. }            // trait d'extension
```

### Points clés

- Un trait déclare un comportement ; un type l'implémente, même après coup.
- Demande peu de méthodes obligatoires, fournis le reste par défaut.
- Les méthodes d'un trait ne sont visibles que si le trait est importé.
- Dérive quand le comportement évident convient ; écris à la main pour un secret, une égalité sur identifiant, un défaut non nul.
- Implémente `From`, jamais `Into`. `FromStr` + `Display` pour un type qui s'écrit et se relit.
- `Hash` doit rester cohérent avec `PartialEq`.
- Règle de l'orphelin : le trait ou le type doit être local ; sinon newtype ou trait d'extension.
