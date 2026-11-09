# Borrow avancé — Cow, AsRef, Borrow, ToOwned

> Les traits de conversion entre valeurs possédées et empruntées : écrire des fonctions qui acceptent plusieurs formes d'un même type, et n'allouer que lorsque c'est nécessaire.

## Prérequis

Les cours `borrow-bases`, `lifetimes` et `generics`.

## Objectifs

- Comprendre les paires possédé / emprunté : `String`/`str`, `Vec<T>`/`[T]`, `PathBuf`/`Path`
- Accepter plusieurs types en paramètre avec `AsRef`, `Borrow`, `Into` et savoir lequel choisir
- Utiliser `ToOwned` pour passer de l'emprunté au possédé
- Utiliser `Cow` pour n'allouer que lorsqu'une modification est nécessaire
- Mettre `Cow` dans des structs et des API publiques
- Connaître les techniques courantes pour éviter les copies inutiles

## Table des matières

1. Les paires possédé / emprunté
2. `AsRef<T>` et `AsMut<T>`
3. `Borrow<T>` et `BorrowMut<T>`
4. `ToOwned`
5. `AsRef`, `Borrow`, `Into`, `Deref` : lequel choisir
6. `Cow<'a, B>` : emprunter ou posséder selon le cas
7. Toutes les façons de créer et lire un `Cow`
8. `Cow` en pratique
9. Techniques pour éviter les copies
10. Exemples dans Runique
11. La même fonction écrite de plusieurs façons
12. Exercices
13. Aide-mémoire

---

## 1. Les paires possédé / emprunté

| Possédé (alloue, modifiable) | Emprunté (vue, sans allocation) |
|---|---|
| `String` | `&str` |
| `Vec<T>` | `&[T]` |
| `PathBuf` | `&Path` |
| `OsString` | `&OsStr` |
| `Box<T>` | `&T` |

Chaque paire suit le même principe : le type possédé **contient** les données ; le type emprunté est une **vue** sur des données qui appartiennent à quelqu'un d'autre. On passe de l'un à l'autre :

```rust
let possede: String = String::from("abc");
let vue: &str = &possede;              // possédé -> emprunté : gratuit
let copie: String = vue.to_owned();    // emprunté -> possédé : allocation
```

Les traits de ce cours généralisent ces conversions.

---

## 2. `AsRef<T>` et `AsMut<T>`

### 2.1 Le trait

```rust
pub trait AsRef<T: ?Sized> {
    fn as_ref(&self) -> &T;
}
```

« Je peux te donner une `&T`, sans copie. » `String`, `&str`, `Box<str>`, `Cow<str>` implémentent tous `AsRef<str>`.

### 2.2 Accepter plusieurs types en paramètre

```rust
fn compter_mots<S: AsRef<str>>(texte: S) -> usize {
    texte.as_ref().split_whitespace().count()
}

compter_mots("un deux");                       // &str
compter_mots(String::from("un deux trois"));   // String
compter_mots(&String::from("a"));              // &String
```

La bibliothèque standard l'utilise partout pour les chemins :

```rust
use std::path::Path;

fn existe<P: AsRef<Path>>(chemin: P) -> bool {
    chemin.as_ref().exists()
}

existe("config.toml");                    // &str
existe(String::from("config.toml"));      // String
existe(Path::new("config.toml"));         // &Path
existe(std::path::PathBuf::from("a"));    // PathBuf
```

`std::fs::read_to_string`, `File::open` et `Command::new` acceptent tous `impl AsRef<Path>` ou `impl AsRef<OsStr>`.

### 2.3 Collections de chaînes

```rust
fn joindre<S: AsRef<str>>(parties: &[S]) -> String {
    parties.iter().map(|p| p.as_ref()).collect::<Vec<_>>().join(", ")
}

joindre(&["a", "b"]);
joindre(&[String::from("a"), String::from("b")]);
```

Sans `AsRef`, il faudrait deux fonctions, ou convertir le `Vec<String>` en `Vec<&str>` avant chaque appel.

### 2.4 `AsMut<T>`

```rust
fn mettre_a_zero<B: AsMut<[u8]>>(mut tampon: B) {
    tampon.as_mut().fill(0);
}

let mut v = vec![1u8, 2, 3];
mettre_a_zero(&mut v);
let mut t = [9u8; 4];
mettre_a_zero(&mut t);
```

### 2.5 Implémenter `AsRef` pour son type

```rust
struct Email(String);

impl AsRef<str> for Email {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

compter_mots(Email("ada@ex.com".into()));
```

---

## 3. `Borrow<T>` et `BorrowMut<T>`

### 3.1 Le trait

```rust
pub trait Borrow<Borrowed: ?Sized> {
    fn borrow(&self) -> &Borrowed;
}
```

La signature est la même que `AsRef`. La différence est une **promesse** : `Borrow` garantit que la forme empruntée se compare, se hache et s'ordonne **exactement comme** la forme possédée.

### 3.2 Pourquoi c'est utile : les recherches dans une map

```rust
use std::collections::HashMap;

let mut m: HashMap<String, u32> = HashMap::new();
m.insert("ada".to_string(), 36);

let age = m.get("ada");          // on cherche avec une &str, sans créer de String
```

La signature de `get` :

```rust
pub fn get<Q: ?Sized>(&self, k: &Q) -> Option<&V>
where
    K: Borrow<Q>,
    Q: Hash + Eq,
```

`String: Borrow<str>` permet de chercher avec une `&str`. La promesse de `Borrow` garantit que `"ada"` haché comme une `&str` donne le même résultat que `String::from("ada")` : sinon la recherche échouerait en silence.

### 3.3 Le respect de la promesse

```rust
// Une clé insensible à la casse
#[derive(PartialEq, Eq, Hash)]
struct CleMinuscule(String);

// NE PAS FAIRE : impl Borrow<str> for CleMinuscule
// "ADA" et "ada" sont égales en tant que CleMinuscule mais différentes en tant que &str :
// une recherche par &str donnerait des résultats incohérents.
```

Si l'égalité ou le hachage de ton type diffèrent de ceux de la forme empruntée, implémente `AsRef`, pas `Borrow`.

### 3.4 Recherche avec un type composé

```rust
use std::collections::HashSet;

let mut s: HashSet<Vec<u8>> = HashSet::new();
s.insert(vec![1, 2]);
s.contains(&[1u8, 2][..]);      // Vec<u8>: Borrow<[u8]> : on cherche avec une tranche
```

### 3.5 `BorrowMut<T>`

L'équivalent mutable, rarement utilisé directement.

---

## 4. `ToOwned`

### 4.1 Le trait

```rust
pub trait ToOwned {
    type Owned: Borrow<Self>;
    fn to_owned(&self) -> Self::Owned;
}
```

`Clone` ne marche que de `&T` vers `T`. `ToOwned` généralise : d'une forme empruntée vers **sa** forme possédée, même si les types diffèrent.

```rust
let s: &str = "abc";
let a: String = s.to_owned();          // str -> String
let t: &[i32] = &[1, 2];
let b: Vec<i32> = t.to_owned();        // [i32] -> Vec<i32>
let p = std::path::Path::new("a");
let c: std::path::PathBuf = p.to_owned();
```

### 4.2 `to_owned`, `to_string`, `String::from`, `into`

```rust
let s = "texte";
let a = s.to_owned();        // ToOwned : générique, le plus direct
let b = s.to_string();       // ToString (via Display) : marche pour tout type affichable
let c = String::from(s);     // From
let d: String = s.into();    // Into
```

Les quatre donnent le même résultat pour une `&str`, avec le même coût. `to_string` est le seul qui marche aussi pour `42.to_string()`.

---

## 5. `AsRef`, `Borrow`, `Into`, `Deref` : lequel choisir

| Trait | Promet | Coût | Utilisation en paramètre |
|---|---|---|---|
| `AsRef<T>` | « je peux donner une `&T` » | aucun | la fonction **lit** : `fn f(p: impl AsRef<Path>)` |
| `Borrow<T>` | idem + même égalité et même hachage | aucun | clés de collections (`get`, `contains`) |
| `Into<T>` | « je peux devenir un `T` » | peut allouer | la fonction **garde** la valeur : `fn new(nom: impl Into<String>)` |
| `Deref` | « je suis un pointeur vers `T` » | aucun | ne s'utilise pas en contrainte : conversion automatique |

### 5.1 La même fonction avec chaque forme

```rust
struct Utilisateur { nom: String }

// &str : simple, accepte &String par conversion automatique (Deref), pas String
fn saluer_v1(nom: &str) -> String { format!("Bonjour {nom}") }

// AsRef : accepte aussi une String par valeur
fn saluer_v2(nom: impl AsRef<str>) -> String { format!("Bonjour {}", nom.as_ref()) }

// Into<String> : la fonction a besoin d'une String possédée
fn creer(nom: impl Into<String>) -> Utilisateur { Utilisateur { nom: nom.into() } }

fn main() {
    let s = String::from("Ada");
    saluer_v1(&s);
    saluer_v1("Linus");
    saluer_v2(s.clone());
    creer("Grace");          // alloue une String
    creer(s);                // aucune copie : la String est déplacée
}
```

Règle simple :
- la fonction **lit** une chaîne : `&str` (ou `impl AsRef<str>` si l'API doit accepter beaucoup de formes) ;
- la fonction **stocke** une chaîne : `impl Into<String>` (ou `String`) ;
- la fonction fait une **recherche** dans une collection : une contrainte `Borrow`.

---

## 6. `Cow<'a, B>` : emprunter ou posséder selon le cas

### 6.1 Définition

```rust
pub enum Cow<'a, B: ?Sized + ToOwned + 'a> {
    Borrowed(&'a B),
    Owned(<B as ToOwned>::Owned),
}
```

`Cow` (*clone on write*) contient **soit** une référence, **soit** une valeur possédée. `Cow<'a, str>` est soit une `&'a str`, soit une `String`.

### 6.2 Le problème résolu

```rust
// Toujours allouer : une String est créée même quand rien ne change
fn sans_espaces_v1(s: &str) -> String {
    s.replace(' ', "_")
}

// N'allouer que si nécessaire
use std::borrow::Cow;

fn sans_espaces_v2(s: &str) -> Cow<'_, str> {
    if s.contains(' ') {
        Cow::Owned(s.replace(' ', "_"))   // une modification est nécessaire : on alloue
    } else {
        Cow::Borrowed(s)                   // rien à changer : on renvoie l'entrée telle quelle
    }
}
```

Si la plupart des entrées n'ont pas d'espace, la seconde version n'alloue presque jamais.

### 6.3 Un `Cow` s'utilise comme la valeur

```rust
let r = sans_espaces_v2("bonjour le monde");
println!("{r}");                 // Display
println!("{}", r.len());         // Deref vers str : toutes les méthodes de str
let s: &str = &r;
```

---

## 7. Toutes les façons de créer et lire un `Cow`

### 7.1 Créer

```rust
use std::borrow::Cow;

let a: Cow<str> = Cow::Borrowed("texte");
let b: Cow<str> = Cow::Owned(String::from("texte"));
let c: Cow<str> = "texte".into();                 // From<&str> : Borrowed
let d: Cow<str> = String::from("texte").into();   // From<String> : Owned
let e: Cow<[i32]> = Cow::from(&[1, 2][..]);
let f: Cow<[i32]> = Cow::from(vec![1, 2]);
```

### 7.2 Lire et convertir

```rust
let mut c: Cow<str> = Cow::Borrowed("abc");

let longueur = c.len();                   // via Deref
let vue: &str = c.as_ref();
let est_emprunte = matches!(c, Cow::Borrowed(_));

c.to_mut().push('!');                     // si Borrowed : copie d'abord, puis modifie ; c devient Owned
let possede: String = c.into_owned();     // String : sans copie si déjà Owned
```

| Méthode | Effet |
|---|---|
| `Deref` (`*c`, `c.len()`) | lire, dans tous les cas, sans copie |
| `to_mut()` | `&mut` vers la valeur possédée ; copie la première fois si `Borrowed` |
| `into_owned()` | la valeur possédée ; copie seulement si `Borrowed` |
| `clone()` | si `Borrowed`, copie la référence ; si `Owned`, copie la valeur |

### 7.3 Modifier seulement si nécessaire

```rust
use std::borrow::Cow;

fn normaliser(entree: &str) -> Cow<'_, str> {
    let mut r = Cow::Borrowed(entree);
    if r.starts_with(' ') || r.ends_with(' ') {
        r = Cow::Owned(r.trim().to_string());
    }
    if r.contains("  ") {
        let simplifie = r.split_whitespace().collect::<Vec<_>>().join(" ");
        r = Cow::Owned(simplifie);
    }
    r
}
```

---

## 8. `Cow` en pratique

### 8.1 Échappement HTML

```rust
use std::borrow::Cow;

fn echapper(s: &str) -> Cow<'_, str> {
    if !s.contains(['<', '>', '&', '"', '\'']) {
        return Cow::Borrowed(s);               // le cas courant : aucun caractère spécial
    }
    let mut r = String::with_capacity(s.len() + 8);
    for c in s.chars() {
        match c {
            '<' => r.push_str("&lt;"),
            '>' => r.push_str("&gt;"),
            '&' => r.push_str("&amp;"),
            '"' => r.push_str("&quot;"),
            '\'' => r.push_str("&#x27;"),
            _ => r.push(c),
        }
    }
    Cow::Owned(r)
}
```

C'est le principe des moteurs de templates (Tera échappe toutes les variables affichées avec `{{ }}`) : la plupart des textes n'ont rien à échapper.

### 8.2 Messages : `Cow<'static, str>`

```rust
use std::borrow::Cow;

fn message(code: u16, detail: Option<&str>) -> Cow<'static, str> {
    match (code, detail) {
        (404, None) => Cow::Borrowed("Page introuvable"),
        (404, Some(d)) => Cow::Owned(format!("Page introuvable : {d}")),
        (_, _) => Cow::Borrowed("Erreur"),
    }
}
```

`Cow<'static, str>` est soit un littéral (aucune allocation, vit tout le programme), soit une `String` construite. C'est un type idéal pour des messages, des noms de champs, des clés de configuration.

### 8.3 Dans une struct

```rust
use std::borrow::Cow;

struct Requete<'a> {
    chemin: Cow<'a, str>,
}

impl<'a> Requete<'a> {
    fn depuis(chemin: &'a str) -> Self {
        Self { chemin: Cow::Borrowed(chemin) }
    }

    fn normaliser(&mut self) {
        if self.chemin.ends_with('/') && self.chemin.len() > 1 {
            let sans = self.chemin.trim_end_matches('/').to_string();
            self.chemin = Cow::Owned(sans);
        }
    }

    fn into_static(self) -> Requete<'static> {
        Requete { chemin: Cow::Owned(self.chemin.into_owned()) }   // détacher de la source
    }
}
```

### 8.4 `Cow` et serde

```rust
#[derive(serde::Deserialize)]
struct Article<'a> {
    #[serde(borrow)]
    titre: Cow<'a, str>,     // emprunte dans le JSON source quand aucun échappement n'est à décoder
}
```

Avec `#[serde(borrow)]`, un champ `Cow<str>` emprunte directement le texte JSON s'il ne contient pas de séquence d'échappement (`\n`, `\"`), et alloue sinon.

### 8.5 Quand ne pas utiliser `Cow`

`Cow` ajoute une durée de vie et un `match` interne. Pour une fonction qui modifie **toujours** son entrée, renvoie directement une `String`. Pour une fonction qui ne modifie **jamais**, renvoie une `&str`. `Cow` n'a d'intérêt que si les deux cas existent et que le cas sans modification est fréquent.

---

## 9. Techniques pour éviter les copies

### 9.1 Emprunter dans les boucles

```rust
let utilisateurs = vec![String::from("ada"), String::from("linus")];

// Copie chaque nom
let noms: Vec<String> = utilisateurs.iter().cloned().collect();

// Emprunte : aucune copie, valable tant que utilisateurs existe
let vues: Vec<&str> = utilisateurs.iter().map(String::as_str).collect();
```

### 9.2 Renvoyer une référence plutôt qu'une copie

```rust
struct Profil { nom: String }

impl Profil {
    fn nom(&self) -> &str { &self.nom }              // pas de clone
    fn nom_copie(&self) -> String { self.nom.clone() }   // seulement si l'appelant doit posséder
}
```

### 9.3 Écrire dans un tampon fourni

```rust
use std::fmt::Write;

fn ecrire_ligne(sortie: &mut String, nom: &str, age: u32) {
    let _ = writeln!(sortie, "{nom};{age}");     // ajoute au tampon existant
}

let mut csv = String::with_capacity(1024);
for (n, a) in [("ada", 36), ("linus", 54)] {
    ecrire_ligne(&mut csv, n, a);                 // une seule allocation pour tout le fichier
}
```

### 9.4 Déplacer au lieu de cloner

```rust
fn enregistrer(nom: String) {}

let nom = String::from("ada");
// enregistrer(nom.clone());       // inutile si nom ne sert plus
enregistrer(nom);                   // déplacement : aucune copie
```

`clippy::redundant_clone` signale les clones dont la source n'est plus utilisée ensuite.

### 9.5 Prendre au lieu de cloner

```rust
struct Formulaire { erreurs: Vec<String> }

impl Formulaire {
    fn vider_erreurs(&mut self) -> Vec<String> {
        std::mem::take(&mut self.erreurs)        // récupère le Vec, laisse un Vec vide
    }
}
```

---

## 10. Exemples dans Runique

### 10.1 Traductions : `Cow<'static, str>`

```rust
// runique/src/utils/trad/switch_lang.rs
pub fn t(key: &str) -> Cow<'static, str> { /* ... */ }
pub fn tf<T: Display>(key: &str, args: &[T]) -> String { /* ... */ }
```

`t` renvoie un `Cow<'static, str>` : la traduction d'une clé sans paramètre est un texte fixe, renvoyé sans allocation. `tf` remplit des paramètres (`{}`) : le résultat est forcément nouveau, d'où une `String`. Les deux signatures suivent exactement la règle de la section 8.5.

### 10.2 `effective_key` du `LoginGuard`

```rust
// runique/src/auth/guard.rs
pub fn effective_key<'a>(username: &'a str, ip: &str) -> std::borrow::Cow<'a, str> {
    let username = username.trim();
    if username.is_empty() {
        std::borrow::Cow::Owned(format!("anonym:{ip}"))
    } else {
        std::borrow::Cow::Owned(username.to_lowercase())
    }
}
```

La clé du compteur de tentatives de connexion est normalisée (`trim` puis `to_lowercase`) : `admin`, `Admin` et `admin ` partagent un seul compteur. Sans cela, changer la casse contournerait le blocage. La normalisation alloue, donc les deux branches renvoient `Owned` ; la signature en `Cow` laisse la possibilité de renvoyer `Borrowed` (par exemple pour une entrée déjà en minuscules) sans changer les appelants.

### 10.3 Paramètres souples

Les constructeurs de Runique acceptent souvent `impl Into<String>` pour les noms et les libellés : l'utilisateur passe un littéral ou une `String` sans conversion explicite, et la valeur est stockée possédée (section 5).

---

## 11. La même fonction écrite de plusieurs façons

Mettre en majuscule la première lettre d'un mot.

```rust
use std::borrow::Cow;

// 1. Toujours une nouvelle String
fn capitaliser_v1(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        Some(p) => p.to_uppercase().collect::<String>() + c.as_str(),
        None => String::new(),
    }
}

// 2. Cow : aucune allocation si la première lettre est déjà une majuscule
fn capitaliser_v2(s: &str) -> Cow<'_, str> {
    match s.chars().next() {
        Some(p) if p.is_uppercase() || !p.is_alphabetic() => Cow::Borrowed(s),
        Some(_) => Cow::Owned(capitaliser_v1(s)),
        None => Cow::Borrowed(s),
    }
}

// 3. Sur place, dans la String de l'appelant (ASCII seulement : même longueur en octets)
fn capitaliser_v3(s: &mut String) {
    if let Some(premier) = s.get_mut(0..1) {
        premier.make_ascii_uppercase();
    }
}

// 4. Générique en entrée, possédé en sortie
fn capitaliser_v4(s: impl AsRef<str>) -> String {
    capitaliser_v1(s.as_ref())
}
```

| Version | Allocation | Quand |
|---|---|---|
| 1 | toujours | simple, quand le résultat doit être stocké |
| 2 | seulement si nécessaire | appelée très souvent sur des données déjà correctes |
| 3 | jamais | l'appelant possède déjà la `String` et n'a plus besoin de l'original |
| 4 | toujours | API qui accepte `&str`, `String`, `Cow`... |

---

## 12. Exercices

### Exercice 1 — `AsRef`

Écris `extension(chemin: impl AsRef<Path>) -> Option<String>` qui renvoie l'extension en minuscules, et appelle-la avec une `&str`, une `String` et un `PathBuf`.

```rust
// Solution :
use std::path::{Path, PathBuf};

fn extension(chemin: impl AsRef<Path>) -> Option<String> {
    chemin.as_ref().extension()?.to_str().map(str::to_lowercase)
}

fn main() {
    extension("photo.JPG");
    extension(String::from("doc.pdf"));
    extension(PathBuf::from("archive.tar.gz"));
}
```

### Exercice 2 — `Cow`

Écris `retirer_accents(s: &str) -> Cow<str>` qui remplace `é`, `è`, `ê` par `e` et `à` par `a`, sans allouer quand il n'y a rien à remplacer.

```rust
// Solution :
use std::borrow::Cow;

fn retirer_accents(s: &str) -> Cow<'_, str> {
    if !s.contains(['é', 'è', 'ê', 'à']) {
        return Cow::Borrowed(s);
    }
    Cow::Owned(
        s.chars()
            .map(|c| match c {
                'é' | 'è' | 'ê' => 'e',
                'à' => 'a',
                autre => autre,
            })
            .collect(),
    )
}

assert!(matches!(retirer_accents("rust"), Cow::Borrowed(_)));
assert_eq!(retirer_accents("été"), "ete");
```

### Exercice 3 — clé de recherche

Une `HashMap<String, u32>` est interrogée des milliers de fois avec des `&str`. Écris la recherche sans créer de `String`.

```rust
// Solution :
use std::collections::HashMap;

fn age(m: &HashMap<String, u32>, nom: &str) -> Option<u32> {
    m.get(nom).copied()       // String: Borrow<str>
}
```

## 13. Aide-mémoire

```rust
fn f(x: impl AsRef<str>)        // lire : accepte &str, String, &String, Cow<str>
fn f(x: impl AsRef<Path>)       // chemins
fn f(x: impl Into<String>)      // stocker : accepte &str (alloue) et String (déplace)
map.get(&cle_empruntee)         // grâce à K: Borrow<Q>
x.to_owned()                    // emprunté -> possédé (&str -> String, &[T] -> Vec<T>)

Cow::Borrowed(&x)   Cow::Owned(v)   "x".into()   String::from("x").into()
c.to_mut()          c.into_owned()  *c / c.len()
fn f(s: &str) -> Cow<'_, str>   // alloue seulement si nécessaire
Cow<'static, str>               // littéral ou String construite
```

### Points clés

- Chaque type possédé a sa vue empruntée : `String`/`str`, `Vec<T>`/`[T]`, `PathBuf`/`Path`.
- `AsRef` pour lire plusieurs formes, `Into` pour stocker, `Borrow` pour les recherches dans les collections.
- `Borrow` promet la même égalité et le même hachage : ne l'implémente pas si ce n'est pas vrai.
- `Cow` n'alloue que lorsqu'une modification est nécessaire ; utile si le cas sans modification est fréquent.
- `Cow<'static, str>` : texte fixe ou construit, sans durée de vie à propager.
- Déplacer, emprunter et prendre (`mem::take`) évitent la plupart des `clone`.
