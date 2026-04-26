## Borrow Avancé en Rust
> `Borrow`, `ToOwned`, `Cow`, `AsRef` — abstraire sur owned et emprunté

## Prérequis

- Cours `borrow-bases` — ownership, `&`, `&mut`, règles du borrow checker
- Cours `lifetimes` — annotations `'a`, lifetime elision

## Objectifs

- Comprendre le trait `Borrow<T>` et ses garanties
- Utiliser `ToOwned` pour aller de l'emprunté vers l'owned
- Maîtriser `Cow<'a, B>` pour éviter les allocations inutiles
- Choisir entre `Cow`, `String` et `&str` selon le contexte
- Utiliser `AsRef<T>` et `AsMut<T>` pour des APIs flexibles
- Reconnaître et appliquer les patterns d'optimisation par emprunt

---

## Table des matières

1. [Le trait `Borrow<T>`](#1-le-trait-borrowt)
2. [Le trait `BorrowMut<T>`](#2-le-trait-borrowmutt)
3. [Le trait `ToOwned`](#3-le-trait-toowned)
4. [`Cow<'a, B>` — Clone-on-Write](#4-cow)
5. [Quand utiliser `Cow` vs `String` vs `&str`](#5-choisir)
6. [`Cow` en pratique — éviter les allocations inutiles](#6-cow-en-pratique)
7. [`AsRef<T>` et `AsMut<T>`](#7-asref-et-asmut)
8. [Patterns d'optimisation avec les emprunts](#8-patterns-doptimisation)
9. [Exemples concrets avec Runique](#9-exemples-runique)
10. [Exercices pratiques](#10-exercices-pratiques)
11. [Aide-mémoire](#11-aide-mémoire)

---

## 1. Le trait `Borrow<T>`

`Borrow<T>` est défini dans `std::borrow` :

```rust
pub trait Borrow<Borrowed: ?Sized> {
    fn borrow(&self) -> &Borrowed;
}
```

Il exprime qu'un type peut se comporter **comme une référence** vers `Borrowed`.
La bibliothèque standard l'implémente pour les paires naturelles :

```rust
// String peut se comporter comme &str
impl Borrow<str> for String { ... }

// Vec<T> peut se comporter comme &[T]
impl Borrow<[T]> for Vec<T> { ... }

// PathBuf peut se comporter comme &Path
impl Borrow<Path> for PathBuf { ... }

// Tout type peut s'emprunter lui-même
impl<T> Borrow<T> for T { ... }
impl<T> Borrow<T> for &T { ... }
```

### Pourquoi c'est utile ?

`HashMap` utilise `Borrow` pour permettre de chercher avec `&str` dans une `HashMap<String, V>` :

```rust
use std::collections::HashMap;

fn main() {
    let mut map: HashMap<String, u32> = HashMap::new();
    map.insert("alice".to_string(), 42);
    map.insert("bob".to_string(), 7);

    // On peut chercher avec &str sans allouer un String
    let valeur = map.get("alice"); // &str — fonctionne grâce à Borrow<str>
    println!("{:?}", valeur); // Some(42)

    let aussi = map.get(&"bob".to_string()); // &String — fonctionne aussi
    println!("{:?}", aussi); // Some(7)
}
```

### Garantie sémantique de `Borrow`

`Borrow` impose une contrainte forte : la valeur empruntée doit être **équivalente** à l'originale
pour `Hash`, `Eq` et `Ord`. C'est ce qui permet à `HashMap::get` d'être correct.

```rust
use std::borrow::Borrow;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

fn hash_de<T: Hash>(val: &T) -> u64 {
    let mut h = DefaultHasher::new();
    val.hash(&mut h);
    h.finish()
}

fn main() {
    let owned = String::from("hello");
    let borrowed: &str = "hello";

    // Les deux doivent avoir le même hash — c'est garanti par Borrow
    assert_eq!(hash_de(&owned), hash_de(borrowed));

    let s: &str = owned.borrow();
    println!("{s}"); // "hello"
}
```

---

## 2. Le trait `BorrowMut<T>`

La variante mutable de `Borrow` :

```rust
pub trait BorrowMut<Borrowed: ?Sized>: Borrow<Borrowed> {
    fn borrow_mut(&mut self) -> &mut Borrowed;
}
```

```rust
use std::borrow::BorrowMut;

fn ajouter_element<C>(collection: &mut C, valeur: i32)
where
    C: BorrowMut<Vec<i32>>,
{
    collection.borrow_mut().push(valeur);
}

fn main() {
    let mut v: Vec<i32> = vec![1, 2, 3];

    ajouter_element(&mut v, 4);

    println!("{:?}", v); // [1, 2, 3, 4]
}
```

En pratique, `BorrowMut` est moins fréquent que `Borrow`. On préfère souvent
`AsMut` (voir section 7) pour les APIs de conversion légères.

---

## 3. Le trait `ToOwned`

`ToOwned` est l'inverse de `Borrow` : il permet de créer une version **owned** depuis une référence.

```rust
pub trait ToOwned {
    type Owned: Borrow<Self>;
    fn to_owned(&self) -> Self::Owned;
}
```

Les implémentations standard :

```rust
// &str → String
impl ToOwned for str {
    type Owned = String;
    fn to_owned(&self) -> String { self.to_string() }
}

// &[T] → Vec<T>
impl<T: Clone> ToOwned for [T] {
    type Owned = Vec<T>;
    fn to_owned(&self) -> Vec<T> { self.to_vec() }
}

// &Path → PathBuf
impl ToOwned for Path {
    type Owned = PathBuf;
    ...
}
```

```rust
fn main() {
    let s: &str = "hello";
    let owned: String = s.to_owned(); // alloue un String

    let slice: &[i32] = &[1, 2, 3];
    let vec: Vec<i32> = slice.to_owned(); // alloue un Vec

    println!("{owned}");
    println!("{vec:?}");
}
```

> `.to_owned()` et `.to_string()` sur `&str` font la même chose.
> La convention Rust préfère `.to_owned()` quand on veut explicitement créer une version owned,
> et `.to_string()` quand la valeur représente du texte à afficher.

---

## 4. `Cow<'a, B>` — Clone-on-Write

`Cow` (Clone-on-Write) est une enum qui représente **soit une référence empruntée, soit une valeur possédée**.

```rust
pub enum Cow<'a, B: ?Sized + 'a>
where
    B: ToOwned,
{
    Borrowed(&'a B),
    Owned(<B as ToOwned>::Owned),
}
```

Les variantes concrètes les plus utilisées :

| Type | Variante `Borrowed` | Variante `Owned` |
|---|---|---|
| `Cow<'a, str>` | `&'a str` | `String` |
| `Cow<'a, [T]>` | `&'a [T]` | `Vec<T>` |
| `Cow<'a, Path>` | `&'a Path` | `PathBuf` |

```rust
use std::borrow::Cow;

fn main() {
    // Variante Borrowed — pas d'allocation
    let emprunte: Cow<str> = Cow::Borrowed("hello");

    // Variante Owned — allouée
    let possede: Cow<str> = Cow::Owned(String::from("world"));

    // Les deux s'utilisent comme &str grâce à Deref
    println!("{emprunte} {possede}");

    // is_borrowed / is_owned pour inspecter
    println!("emprunté ? {}", emprunte.is_borrowed()); // true
    println!("possédé ? {}", possede.is_owned());       // true
}
```

### `to_mut()` — le "clone-on-write"

```rust
use std::borrow::Cow;

fn main() {
    let mut cow: Cow<str> = Cow::Borrowed("hello");

    // to_mut() clone seulement si nécessaire
    cow.to_mut().push_str(" world");

    // Maintenant cow est Owned
    println!("{cow}"); // "hello world"
}
```

`to_mut()` :
- Si `Cow::Borrowed` : clone la valeur en `Owned`, puis retourne `&mut Owned`
- Si `Cow::Owned` : retourne `&mut Owned` directement, sans cloner

---

## 5. Quand utiliser `Cow` vs `String` vs `&str`

### `&str` — référence pure

Utiliser quand :
- La fonction ne fait que lire du texte
- Le texte vient toujours de l'extérieur (pas de modification possible)
- On veut être le plus flexible possible en entrée

```rust
fn longueur(s: &str) -> usize {
    s.len()
}
```

### `String` — propriété pleine

Utiliser quand :
- La fonction doit posséder le texte (le stocker dans une struct, le retourner modifié)
- On sait qu'on aura toujours besoin d'allouer

```rust
struct Message {
    contenu: String, // possédé, pas de problème de lifetime
}

fn formater_nom(prenom: &str, nom: &str) -> String {
    format!("{prenom} {nom}") // toujours une nouvelle allocation
}
```

### `Cow<'a, str>` — flexible

Utiliser quand :
- La fonction **peut ou non** avoir besoin d'allouer selon l'entrée
- On veut éviter une allocation dans le cas courant (pas de modification)
- On retourne soit le texte original emprunté, soit un texte modifié

```rust
use std::borrow::Cow;

// Pas d'allocation si l'entrée n'a pas de guillemets
fn echapper_guillemets(texte: &str) -> Cow<str> {
    if texte.contains('"') {
        Cow::Owned(texte.replace('"', "&quot;"))
    } else {
        Cow::Borrowed(texte)
    }
}

fn main() {
    let sans = "bonjour monde";
    let avec = "il a dit \"salut\"";

    let r1 = echapper_guillemets(sans); // Borrowed — zéro allocation
    let r2 = echapper_guillemets(avec); // Owned — allocation nécessaire

    println!("{r1}");
    println!("{r2}");
}
```

---

## 6. `Cow` en pratique — éviter les allocations inutiles

### Normalisation conditionnelle

```rust
use std::borrow::Cow;

fn normaliser_espaces(texte: &str) -> Cow<str> {
    // Vérifier d'abord si une modification est nécessaire
    if !texte.contains("  ") {
        return Cow::Borrowed(texte); // cas courant — aucune allocation
    }

    // Cas rare — allocation seulement si besoin
    let normalise = texte
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");

    Cow::Owned(normalise)
}

fn main() {
    let propre = "bonjour monde";
    let sale = "bonjour   monde   foo";

    println!("{}", normaliser_espaces(propre)); // Borrowed
    println!("{}", normaliser_espaces(sale));   // Owned
}
```

### Dans une struct avec lifetime

```rust
use std::borrow::Cow;

struct Config<'a> {
    nom: Cow<'a, str>,
    valeur: Cow<'a, str>,
}

impl<'a> Config<'a> {
    // Peut prendre &str (Borrowed) ou String (Owned)
    fn new(nom: impl Into<Cow<'a, str>>, valeur: impl Into<Cow<'a, str>>) -> Self {
        Config {
            nom: nom.into(),
            valeur: valeur.into(),
        }
    }
}

fn main() {
    // Pas d'allocation — on emprunte des littéraux
    let c1 = Config::new("host", "localhost");

    // Allocation si le nom est construit dynamiquement
    let cle = format!("option_{}", 42);
    let c2 = Config::new(cle, "valeur");

    println!("{} = {}", c1.nom, c1.valeur);
    println!("{} = {}", c2.nom, c2.valeur);
}
```

### `Cow<'static, str>` — pattern courant pour les messages

`Cow<'static, str>` peut contenir soit un littéral `&'static str` (zéro allocation),
soit un `String` alloué dynamiquement.

```rust
use std::borrow::Cow;

fn message_erreur(code: u32) -> Cow<'static, str> {
    match code {
        404 => Cow::Borrowed("ressource introuvable"),
        500 => Cow::Borrowed("erreur interne du serveur"),
        _   => Cow::Owned(format!("erreur inconnue (code {code})")),
    }
}

fn main() {
    println!("{}", message_erreur(404)); // Borrowed — littéral statique
    println!("{}", message_erreur(500)); // Borrowed — littéral statique
    println!("{}", message_erreur(418)); // Owned — alloué dynamiquement
}
```

---

## 7. `AsRef<T>` et `AsMut<T>`

`AsRef<T>` permet de convertir une référence en une autre référence, de façon légère et sans coût.

```rust
pub trait AsRef<T: ?Sized> {
    fn as_ref(&self) -> &T;
}
```

### Différence avec `Borrow`

| | `Borrow<T>` | `AsRef<T>` |
|---|---|---|
| Garantie sémantique | `Hash`/`Eq`/`Ord` compatibles | Aucune |
| Usage principal | `HashMap`, `BTreeMap` | APIs génériques flexibles |
| Qui l'implémente | Paires naturelles (String/str) | Conversions larges |

### API générique avec `AsRef`

```rust
use std::path::{Path, PathBuf};

// Accepte &str, &String, &Path, &PathBuf, etc.
fn lire_fichier(chemin: impl AsRef<Path>) -> String {
    let chemin = chemin.as_ref();
    std::fs::read_to_string(chemin)
        .unwrap_or_else(|_| format!("fichier {:?} introuvable", chemin))
}

fn main() {
    // Toutes ces formes fonctionnent
    let _ = lire_fichier("config.toml");
    let _ = lire_fichier(String::from("config.toml"));
    let _ = lire_fichier(Path::new("config.toml"));
    let _ = lire_fichier(PathBuf::from("config.toml"));
}
```

### `AsMut<T>` — variante mutable

```rust
fn remplir_zeros(buf: impl AsMut<[u8]>) {
    let mut buf = buf;
    for b in buf.as_mut() {
        *b = 0;
    }
}

fn main() {
    let mut v = vec![1u8, 2, 3, 4];
    remplir_zeros(&mut v);
    println!("{:?}", v); // [0, 0, 0, 0]
}
```

---

## 8. Patterns d'optimisation avec les emprunts

### Pattern 1 — Emprunter dans les hot paths

```rust
use std::collections::HashMap;

struct Cache {
    donnees: HashMap<String, String>,
}

impl Cache {
    // Retourne &str — pas d'allocation
    fn get(&self, cle: &str) -> Option<&str> {
        self.donnees.get(cle).map(String::as_str)
    }

    // Insert prend ownership — pas de clone surprise
    fn insert(&mut self, cle: String, valeur: String) {
        self.donnees.insert(cle, valeur);
    }
}
```

### Pattern 2 — Retourner une référence plutôt qu'une copie

```rust
struct Configuration {
    valeurs: Vec<String>,
    defaut: String,
}

impl Configuration {
    fn get(&self, index: usize) -> &str {
        self.valeurs
            .get(index)
            .map(String::as_str)
            .unwrap_or(&self.defaut) // retourne une ref, pas un clone
    }
}
```

### Pattern 3 — Accumuler dans un `Vec` emprunté

```rust
fn collecter_pairs<'a>(source: &'a [i32], resultat: &mut Vec<&'a i32>) {
    for n in source.iter().filter(|&&n| n % 2 == 0) {
        resultat.push(n); // pousse des références, pas des copies
    }
}

fn main() {
    let nombres = vec![1, 2, 3, 4, 5, 6];
    let mut pairs: Vec<&i32> = Vec::new();

    collecter_pairs(&nombres, &mut pairs);

    println!("{:?}", pairs); // [2, 4, 6]
}
```

### Pattern 4 — Éviter les clones en chaîne

```rust
#[derive(Debug)]
struct Utilisateur {
    nom: String,
    email: String,
}

struct Session<'u> {
    utilisateur: &'u Utilisateur, // référence, pas copie
    token: String,
}

impl<'u> Session<'u> {
    fn nouveau(utilisateur: &'u Utilisateur, token: String) -> Self {
        Session { utilisateur, token }
    }

    fn nom_affiche(&self) -> &str {
        &self.utilisateur.nom // retourne &str depuis la référence
    }
}
```

---

## 9. Exemples concrets avec Runique

Dans Runique, plusieurs fonctions retournent `Cow<'static, str>` pour les messages i18n.
Ce pattern évite d'allouer dans le cas courant (message statique) tout en permettant
de construire des messages dynamiques quand nécessaire.

### Pattern i18n avec `Cow<'static, str>`

```rust
use std::borrow::Cow;

// Signature typique d'une fonction de traduction dans Runique
fn traduire(cle: &str, params: Option<&[(&str, &str)]>) -> Cow<'static, str> {
    match (cle, params) {
        ("erreur.requis", None) => {
            Cow::Borrowed("Ce champ est requis.")
        }

        ("erreur.min_longueur", Some(p)) => {
            let min = p.iter().find(|(k, _)| *k == "min").map(|(_, v)| *v).unwrap_or("?");
            Cow::Owned(format!("Minimum {min} caractères requis."))
        }

        ("erreur.max_longueur", Some(p)) => {
            let max = p.iter().find(|(k, _)| *k == "max").map(|(_, v)| *v).unwrap_or("?");
            Cow::Owned(format!("Maximum {max} caractères autorisés."))
        }

        _ => Cow::Owned(format!("Clé de traduction inconnue : {cle}")),
    }
}

fn main() {
    // Cas courant — zéro allocation
    let msg1 = traduire("erreur.requis", None);
    println!("{msg1}");

    // Cas avec paramètres — allocation nécessaire
    let msg2 = traduire("erreur.min_longueur", Some(&[("min", "8")]));
    println!("{msg2}");
}
```

### `effective_key` dans le LoginGuard

Le `LoginGuard` de Runique utilise `Cow<'_, str>` pour sa clé d'identification :
si un nom d'utilisateur est fourni, on l'emprunte directement ; sinon, on construit
une clé dynamique avec l'IP.

```rust
use std::borrow::Cow;

fn effective_key<'a>(username: &'a str, ip: &str) -> Cow<'a, str> {
    if username.is_empty() {
        // Allocation nécessaire — construit "anonym:{ip}"
        Cow::Owned(format!("anonym:{ip}"))
    } else {
        // Pas d'allocation — emprunte le username directement
        Cow::Borrowed(username)
    }
}

fn main() {
    let cle1 = effective_key("alice", "127.0.0.1");
    println!("{cle1}"); // "alice" — Borrowed

    let cle2 = effective_key("", "192.168.1.10");
    println!("{cle2}"); // "anonym:192.168.1.10" — Owned
}
```

---

## 10. Exercices pratiques

### Exercice 1 — Implémenter `Borrow`

Créez un type `NomNormalise` qui peut se comporter comme `&str` via `Borrow`.

```rust
use std::borrow::Borrow;

struct NomNormalise(String);

impl NomNormalise {
    fn new(s: &str) -> Self {
        NomNormalise(s.trim().to_lowercase())
    }
}

impl Borrow<str> for NomNormalise {
    fn borrow(&self) -> &str {
        &self.0
    }
}

fn main() {
    use std::collections::HashMap;

    let mut map: HashMap<NomNormalise, u32> = HashMap::new();
    // Note : pour utiliser NomNormalise comme clé HashMap,
    // il faut aussi Hash et Eq cohérents avec &str.
    // Exercice simplifié ici pour illustrer Borrow.

    let n = NomNormalise::new("  Alice  ");
    let s: &str = n.borrow();
    println!("{s}"); // "alice"
}
```

### Exercice 2 — Fonction avec `Cow`

Écrivez une fonction qui met en majuscule la première lettre, en retournant `Cow<str>`.

```rust
use std::borrow::Cow;

fn majuscule_initiale(texte: &str) -> Cow<str> {
    let mut chars = texte.chars();

    match chars.next() {
        None => Cow::Borrowed(texte),
        Some(c) if c.is_uppercase() => Cow::Borrowed(texte), // déjà bon
        Some(c) => {
            let majuscule: String = c.to_uppercase().collect::<String>() + chars.as_str();
            Cow::Owned(majuscule)
        }
    }
}

fn main() {
    let deja = "Bonjour";
    let a_modifier = "bonjour";

    println!("{}", majuscule_initiale(deja));     // Borrowed
    println!("{}", majuscule_initiale(a_modifier)); // Owned
}
```

### Exercice 3 — API générique avec `AsRef`

```rust
fn compter_lignes(source: impl AsRef<str>) -> usize {
    source.as_ref().lines().count()
}

fn main() {
    let owned = String::from("ligne 1\nligne 2\nligne 3");
    let borrowed = "a\nb\nc\nd";

    println!("{}", compter_lignes(&owned));   // 3
    println!("{}", compter_lignes(borrowed)); // 4
    println!("{}", compter_lignes(owned));    // 3 — owned aussi accepté
}
```

---

## 11. Aide-mémoire

| Trait | Direction | Usage principal |
|---|---|---|
| `Borrow<T>` | `Owned → &T` | Lookup dans `HashMap`/`BTreeMap` |
| `BorrowMut<T>` | `Owned → &mut T` | Modification via référence |
| `ToOwned` | `&T → Owned` | Créer une version possédée |
| `AsRef<T>` | `&Self → &T` | APIs génériques flexibles |
| `AsMut<T>` | `&mut Self → &mut T` | Modification, APIs flexibles |
| `Cow<'a, B>` | Les deux | Éviter les allocations inutiles |

**Règles de décision :**

| Situation | Type à utiliser |
|---|---|
| Lecture seule, pas de stockage | `&str` / `&[T]` |
| Possession complète nécessaire | `String` / `Vec<T>` |
| Parfois emprunté, parfois alloué | `Cow<'_, str>` / `Cow<'_, [T]>` |
| Messages statiques avec cas dynamiques | `Cow<'static, str>` |
| API qui accepte plusieurs formes | `impl AsRef<str>` / `impl AsRef<Path>` |

**Points clés à retenir :**

- `Borrow` garantit que `Hash`/`Eq`/`Ord` sont cohérents — c'est pour ça que `HashMap::get` accepte `&str` avec une clé `String`
- `ToOwned` est l'inverse de `Borrow` : de `&T` vers `Owned`
- `Cow` est une enum : `Borrowed(&'a B)` ou `Owned(<B as ToOwned>::Owned)`
- `to_mut()` sur un `Cow::Borrowed` clone une seule fois, puis travaille sur l'`Owned`
- `AsRef` n'a pas de garantie sémantique — utiliser pour la flexibilité d'API, pas pour les collections
- `Cow<'static, str>` est idiomatique pour les messages d'erreur et les clés i18n
