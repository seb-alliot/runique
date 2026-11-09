# Lifetimes

> Les durées de vie disent au compilateur combien de temps une référence reste valide. Ce cours montre quand il faut les écrire, quand il les devine, et toutes les façons de s'en passer.

## Objectifs

- Comprendre ce qu'est une durée de vie et pourquoi le compilateur en a besoin
- Lire et écrire `'a` dans une fonction, une struct, un bloc `impl`, un trait
- Connaître les trois règles d'élision qui évitent de les écrire la plupart du temps
- Distinguer `&'static T` et la contrainte `T: 'static`
- Utiliser plusieurs durées de vie et les relations `'a: 'b`
- Corriger les erreurs courantes (E0106, E0515, E0597, E0716)
- Savoir quand renvoyer une valeur possédée plutôt qu'une référence

## Table des matières

1. Le problème
2. Ce qu'est une durée de vie
3. Syntaxe
4. Durées de vie dans les fonctions
5. Les règles d'élision
6. Durées de vie dans les structs
7. Durées de vie dans les blocs `impl`
8. `'static`
9. Plusieurs durées de vie et relations
10. Durées de vie, traits et closures
11. Erreurs courantes et corrections
12. La même fonction écrite de plusieurs façons
13. Exercices
14. Aide-mémoire

---

## 1. Le problème

```rust
fn main() {
    let r;
    {
        let x = 5;
        r = &x;
    }                       // x est libérée ici
    // println!("{r}");     // ERREUR E0597 : `x` does not live long enough
}
```

`r` pointerait vers une mémoire libérée. Le compilateur compare la **durée de vie** de `x` (le bloc intérieur) à celle de `r` (toute la fonction) et refuse.

---

## 2. Ce qu'est une durée de vie

Une durée de vie est la **portion du programme** pendant laquelle une référence est valide. Elle n'existe qu'à la compilation : aucune trace dans le binaire, aucun coût.

```rust
fn main() {
    let x = 5;              // ----------+-- 'b (durée de vie de x)
    let r = &x;             // --+-- 'a  |
    println!("{r}");        //   |       |
                            // --+       |
}                           // ----------+
```

La référence `r` (durée `'a`) vit moins longtemps que `x` (durée `'b`) : c'est valide.

Dans l'immense majorité des cas, le compilateur **déduit** les durées de vie. On ne les écrit que quand il ne peut pas deviner une relation entre plusieurs références.

---

## 3. Syntaxe

```rust
&i32            // une référence
&'a i32         // une référence de durée de vie 'a
&'a mut i32     // une référence mutable de durée de vie 'a
```

- Le nom commence par une apostrophe, en minuscules, souvent court : `'a`, `'b`, ou parlant : `'req`, `'conn`.
- Il se **déclare** entre chevrons, comme un paramètre générique : `fn f<'a>(...)`, `struct S<'a>`, `impl<'a>`.
- `'_` est une durée de vie anonyme : « déduis-la ».
- `'static` est la seule durée de vie nommée prédéfinie.

Une annotation **ne change pas** la durée de vie réelle d'une valeur. Elle décrit une relation que le compilateur vérifie.

---

## 4. Durées de vie dans les fonctions

### 4.1 — Le cas qui demande une annotation

```rust
// fn plus_long(a: &str, b: &str) -> &str {
//     if a.len() > b.len() { a } else { b }
// }
// ERREUR E0106 : missing lifetime specifier
```

Le résultat vient de `a` **ou** de `b` : le compilateur ne sait pas combien de temps il reste valide.

```rust
fn plus_long<'a>(a: &'a str, b: &'a str) -> &'a str {
    if a.len() > b.len() { a } else { b }
}
```

Lecture : « pour une durée `'a`, si `a` et `b` vivent au moins `'a`, le résultat vit aussi `'a` ». Concrètement, le résultat est valide tant que **les deux** arguments le sont.

### 4.2 — Ce que l'annotation permet et interdit

```rust
fn main() {
    let a = String::from("longue chaîne");
    let resultat;
    {
        let b = String::from("court");
        resultat = plus_long(&a, &b);
        println!("{resultat}");      // OK : a et b vivent encore
    }
    // println!("{resultat}");       // ERREUR : b est libérée, resultat pourrait venir de b
}
```

Le compilateur ne regarde pas **lequel** des deux est renvoyé à l'exécution : il applique la signature.

### 4.3 — Quand le résultat ne dépend que d'un argument

```rust
fn premier_mot<'a>(texte: &'a str, _separateur: &str) -> &'a str {
    texte.split(' ').next().unwrap_or("")
}
```

`_separateur` n'a pas besoin de `'a` : le résultat ne vient jamais de lui. Ne lier que ce qui est réellement lié rend la fonction plus facile à appeler.

### 4.4 — On ne peut pas renvoyer une référence vers une variable locale

```rust
// fn creer<'a>() -> &'a str {
//     let s = String::from("bonjour");
//     &s                       // ERREUR E0515 : returns a reference to data owned by the current function
// }

fn creer() -> String {          // renvoyer la valeur elle-même
    String::from("bonjour")
}

fn creer_constant() -> &'static str {   // ou un littéral, qui vit tout le programme
    "bonjour"
}
```

Aucune annotation ne peut prolonger la vie d'une valeur locale : elle est libérée à la fin de la fonction.

---

## 5. Les règles d'élision

Le compilateur applique trois règles pour déduire les durées de vie d'une signature. Si elles suffisent, on n'écrit rien.

1. Chaque paramètre référence reçoit **sa propre** durée de vie.
2. S'il y a **exactement un** paramètre référence, sa durée de vie est donnée à toutes les références du résultat.
3. S'il y a `&self` ou `&mut self`, la durée de vie de `self` est donnée à toutes les références du résultat.

### Exemples

```rust
fn longueur(s: &str) -> usize                 // aucun retour référence : rien à déduire
fn premier(s: &str) -> &str                   // règle 2 : fn premier<'a>(s: &'a str) -> &'a str
fn nom(&self) -> &str                         // règle 3 : lié à self
fn choisir(a: &str, b: &str) -> &str          // deux paramètres, pas de self : ANNOTATION OBLIGATOIRE
fn get(&self, cle: &str) -> &str              // règle 3 : lié à self, pas à cle
```

La dernière ligne est un piège : si la méthode renvoie en réalité une partie de `cle`, la règle 3 est fausse et il faut l'écrire :

```rust
struct Dico;
impl Dico {
    fn ou_cle<'a>(&self, cle: &'a str) -> &'a str {
        cle
    }
}
```

---

## 6. Durées de vie dans les structs

### 6.1 — Une struct qui emprunte

```rust
struct Extrait<'a> {
    texte: &'a str,
}

fn main() {
    let roman = String::from("Il était une fois. La suite.");
    let premiere = roman.split('.').next().unwrap();
    let e = Extrait { texte: premiere };
    println!("{}", e.texte);
}   // e doit disparaître avant roman : garanti par 'a
```

`Extrait<'a>` signifie : « un `Extrait` ne peut pas vivre plus longtemps que le texte qu'il emprunte ».

### 6.2 — Possédé ou emprunté : la même struct deux fois

```rust
// 1. Emprunte : aucune copie, mais liée à la durée de vie de la source
struct LigneRef<'a> {
    nom: &'a str,
    valeur: &'a str,
}

// 2. Possède : indépendante, peut être stockée, renvoyée, envoyée à un thread
struct Ligne {
    nom: String,
    valeur: String,
}

fn analyser<'a>(texte: &'a str) -> Vec<LigneRef<'a>> {
    texte
        .lines()
        .filter_map(|l| l.split_once('='))
        .map(|(nom, valeur)| LigneRef { nom, valeur })
        .collect()
}
```

| | Struct qui emprunte | Struct qui possède |
|---|---|---|
| Copie des données | aucune | une allocation par champ |
| Peut survivre à la source | non | oui |
| Utilisable dans un thread, une tâche async, un cache | difficilement | oui |
| Complexité | durées de vie à propager partout | aucune |

Règle pratique : commence par des champs **possédés**. Passe aux références quand une mesure montre que les copies coûtent, ou pour des vues temporaires (analyseur, itérateur).

### 6.3 — Propagation

Une struct qui contient une struct à durée de vie en a une aussi :

```rust
struct Document<'a> {
    lignes: Vec<LigneRef<'a>>,
}
```

---

## 7. Durées de vie dans les blocs `impl`

### 7.1 — Déclarer puis utiliser

```rust
impl<'a> Extrait<'a> {
    fn texte(&self) -> &str {           // élision règle 3 : lié à self
        self.texte
    }

    fn texte_source(&self) -> &'a str {  // lié à la source, pas à self
        self.texte
    }
}
```

La différence compte : le résultat de `texte_source` reste valide après la disparition de l'`Extrait`, tant que la source existe.

```rust
fn extraire(roman: &str) -> &str {
    let e = Extrait { texte: roman };
    e.texte_source()                     // OK : lié à roman
    // e.texte()                         // ERREUR : lié à e, qui disparaît ici
}
```

### 7.2 — `'_` dans un `impl`

```rust
impl Extrait<'_> {
    fn longueur(&self) -> usize {
        self.texte.len()
    }
}
```

`'_` évite de déclarer `'a` quand le bloc ne s'en sert pas.

---

## 8. `'static`

`'static` a deux sens selon l'endroit.

### 8.1 — `&'static T` : une référence valide jusqu'à la fin du programme

```rust
let s: &'static str = "littéral";      // les littéraux sont dans le binaire
static NOM: &str = "Runique";           // une static est 'static
```

On peut aussi en fabriquer une en « fuyant » volontairement de la mémoire :

```rust
let config: &'static str = Box::leak(String::from("lue au démarrage").into_boxed_str());
```

À réserver aux valeurs créées une fois au démarrage et utilisées jusqu'à la fin.

### 8.2 — `T: 'static` : un type qui ne contient pas de référence temporaire

```rust
fn lancer<T: Send + 'static>(valeur: T) {
    std::thread::spawn(move || {
        let _v = valeur;
    });
}

lancer(String::from("ok"));          // String : possède ses données, donc 'static
lancer(42);                          // OK
let local = String::from("x");
// lancer(&local);                   // ERREUR : &local ne vit pas 'static
```

`T: 'static` ne veut **pas** dire que la valeur vit toute la durée du programme. Cela veut dire qu'elle **pourrait** : elle ne dépend d'aucun emprunt. Une `String`, un `Vec<u8>`, un `Arc<T>` sont `'static`. C'est la contrainte de `thread::spawn` et `tokio::spawn`, parce que la tâche peut survivre à la fonction qui la lance.

### 8.3 — Corriger « borrowed value does not live long enough » avec `spawn`

```rust
use std::sync::Arc;

let donnees = vec![1, 2, 3];

// 1. Déplacer la valeur dans la tâche
std::thread::spawn(move || println!("{donnees:?}"));

// 2. Partager avec Arc : chaque tâche possède une copie du pointeur
let partage = Arc::new(vec![1, 2, 3]);
let p = Arc::clone(&partage);
std::thread::spawn(move || println!("{p:?}"));

// 3. Threads limités à une portée : l'emprunt est permis
let v = vec![1, 2, 3];
std::thread::scope(|s| {
    s.spawn(|| println!("{v:?}"));     // pas de 'static : le scope attend la fin du thread
});
```

---

## 9. Plusieurs durées de vie et relations

### 9.1 — Deux durées indépendantes

```rust
struct Analyseur<'texte, 'motif> {
    texte: &'texte str,
    motif: &'motif str,
}

impl<'texte, 'motif> Analyseur<'texte, 'motif> {
    fn premiere_occurrence(&self) -> Option<&'texte str> {
        let i = self.texte.find(self.motif)?;
        Some(&self.texte[i..i + self.motif.len()])
    }
}
```

Le résultat ne dépend que de `'texte` : l'appelant peut libérer le motif sans perdre le résultat. Avec une seule durée `'a` pour les deux, il ne le pourrait pas.

### 9.2 — `'a: 'b` : « `'a` vit au moins aussi longtemps que `'b` »

```rust
fn choisir<'long: 'court, 'court>(a: &'long str, _b: &'court str) -> &'court str {
    a                       // OK : une référence plus longue peut servir de plus courte
}
```

### 9.3 — `T: 'a` : « `T` ne contient pas de référence plus courte que `'a` »

```rust
struct Cache<'a, T: 'a> {
    valeurs: &'a [T],
}
```

Le compilateur déduit lui-même cette contrainte dans la plupart des cas ; on la voit surtout dans les messages d'erreur.

---

## 10. Durées de vie, traits et closures

### 10.1 — Trait objects

```rust
trait Regle { fn ok(&self, s: &str) -> bool; }

struct Contient<'a>(&'a str);
impl Regle for Contient<'_> {
    fn ok(&self, s: &str) -> bool { s.contains(self.0) }
}

// Box<dyn Regle> sous-entend Box<dyn Regle + 'static> : refuse les règles qui empruntent
fn regles_static() -> Vec<Box<dyn Regle>> { vec![] }

// Avec une durée explicite, une règle peut emprunter
fn regles<'a>(mot: &'a str) -> Vec<Box<dyn Regle + 'a>> {
    vec![Box::new(Contient(mot))]
}
```

### 10.2 — `impl Trait` en retour

```rust
fn mots(texte: &str) -> impl Iterator<Item = &str> {
    texte.split_whitespace()
}
```

En édition 2024, un `impl Trait` en retour **capture automatiquement** toutes les durées de vie des paramètres : l'itérateur renvoyé est lié à `texte`, sans rien écrire. En édition 2021, il fallait souvent ajouter `+ '_`. Pour restreindre ce qui est capturé : `impl Iterator<Item = u32> + use<>`.

### 10.3 — Closures qui reçoivent une référence

```rust
fn appliquer<F>(f: F) -> usize
where
    F: Fn(&str) -> usize,          // en réalité : for<'a> Fn(&'a str) -> usize
{
    let local = String::from("texte local");
    f(&local)
}

appliquer(|s| s.len());
```

`F: Fn(&str) -> usize` signifie « une closure qui accepte une `&str` de **n'importe quelle** durée de vie ». La forme longue `for<'a> Fn(&'a str)` s'appelle une contrainte de rang supérieur (HRTB) ; on la rencontre rarement écrite, mais elle apparaît dans certains messages d'erreur.

---

## 11. Erreurs courantes et corrections

### 11.1 — E0106 : durée de vie manquante

```rust
// struct Utilisateur { nom: &str }
struct Utilisateur1 { nom: String }            // 1. posséder
struct Utilisateur2<'a> { nom: &'a str }       // 2. déclarer la durée de vie
```

### 11.2 — E0515 : renvoyer une référence vers du local

```rust
// fn nom_complet(p: &str, n: &str) -> &str { &format!("{p} {n}") }
fn nom_complet(p: &str, n: &str) -> String { format!("{p} {n}") }
```

Une valeur **créée** dans la fonction se renvoie possédée.

### 11.3 — E0597 : ne vit pas assez longtemps

```rust
let resultat;
let a = String::from("abc");
{
    let b = String::from("de");
    resultat = plus_long(&a, &b);
}
// println!("{resultat}");
```

Corrections : utiliser le résultat dans le bloc ; déclarer `b` dans la même portée que `a` ; ou faire une copie (`plus_long(&a, &b).to_string()`).

### 11.4 — E0716 : valeur temporaire libérée trop tôt

```rust
// let s: &str = String::from("abc").as_str();   // la String temporaire meurt à la fin de la ligne
// println!("{s}");

let possede = String::from("abc");               // 1. nommer la valeur
let s: &str = possede.as_str();

let s2: &str = &String::from("abc");             // 2. un & direct prolonge la temporaire jusqu'à la fin du bloc
```

### 11.5 — Struct qui contient une référence vers elle-même

```rust
// struct Analyse {
//     texte: String,
//     premier_mot: &str,      // voudrait pointer dans texte : impossible
// }

struct Analyse {
    texte: String,
    premier_mot: std::ops::Range<usize>,   // stocker des positions plutôt qu'une référence
}

impl Analyse {
    fn premier_mot(&self) -> &str {
        &self.texte[self.premier_mot.clone()]
    }
}
```

Une struct ne peut pas contenir une référence vers l'un de ses propres champs (elle serait invalidée en déplaçant la struct). On stocke des index, ou on sépare le propriétaire et la vue en deux structs.

---

## 12. La même fonction écrite de plusieurs façons

Renvoyer la plus longue de deux chaînes.

```rust
use std::borrow::Cow;

// 1. Références liées : aucune copie, le résultat dépend des deux arguments
fn plus_long_v1<'a>(a: &'a str, b: &'a str) -> &'a str {
    if a.len() >= b.len() { a } else { b }
}

// 2. Valeur possédée : indépendante, mais une allocation
fn plus_long_v2(a: &str, b: &str) -> String {
    if a.len() >= b.len() { a.to_string() } else { b.to_string() }
}

// 3. Un index : aucune référence à renvoyer
fn plus_long_v3(a: &str, b: &str) -> usize {
    if a.len() >= b.len() { 0 } else { 1 }
}

// 4. Cow : emprunte quand c'est possible, possède quand il faut transformer
fn plus_long_v4<'a>(a: &'a str, b: &'a str, majuscules: bool) -> Cow<'a, str> {
    let choisi = if a.len() >= b.len() { a } else { b };
    if majuscules {
        Cow::Owned(choisi.to_uppercase())
    } else {
        Cow::Borrowed(choisi)
    }
}
```

| Version | Quand |
|---|---|
| 1 | le résultat est utilisé tout de suite, pendant que les sources existent |
| 2 | le résultat doit être stocké ou survivre aux sources |
| 3 | l'appelant a déjà les deux valeurs et veut juste savoir laquelle |
| 4 | le résultat est parfois une copie transformée (voir `borrow-avance`) |

---

## 13. Exercices

### Exercice 1 — annoter

Ajoute les durées de vie nécessaires, et seulement celles-là.

```rust
fn plus_court(a: &str, b: &str) -> &str { if a.len() < b.len() { a } else { b } }
fn prefixe(texte: &str, n: usize) -> &str { &texte[..n] }
fn trouver(texte: &str, motif: &str) -> Option<&str> { texte.find(motif).map(|i| &texte[i..]) }
```

```rust
// Solution :
fn plus_court<'a>(a: &'a str, b: &'a str) -> &'a str { if a.len() < b.len() { a } else { b } }
fn prefixe(texte: &str, n: usize) -> &str { &texte[..n] }              // règle 2 : rien à écrire
fn trouver<'a>(texte: &'a str, motif: &str) -> Option<&'a str> {       // lié à texte seulement
    texte.find(motif).map(|i| &texte[i..])
}
```

### Exercice 2 — struct qui emprunte

Écris `Mots<'a>` qui garde un `&'a str` et une méthode `plus_long(&self) -> &'a str`.

```rust
// Solution :
struct Mots<'a> {
    texte: &'a str,
}

impl<'a> Mots<'a> {
    fn plus_long(&self) -> &'a str {
        self.texte.split_whitespace().max_by_key(|m| m.chars().count()).unwrap_or("")
    }
}
```

### Exercice 3 — corriger sans annotation

Cette fonction ne compile pas. Corrige-la de deux façons sans écrire de durée de vie.

```rust
// fn saluer(nom: &str) -> &str { &format!("Bonjour {nom}") }
```

```rust
// Solution 1 : renvoyer une String
fn saluer(nom: &str) -> String { format!("Bonjour {nom}") }

// Solution 2 : écrire dans un tampon fourni par l'appelant
fn saluer_dans(nom: &str, sortie: &mut String) {
    use std::fmt::Write;
    let _ = write!(sortie, "Bonjour {nom}");
}
```

---

## 14. Aide-mémoire

```rust
fn f<'a>(x: &'a str, y: &'a str) -> &'a str    // résultat lié aux deux
fn f<'a>(x: &'a str, y: &str) -> &'a str       // lié à x seulement
fn f(x: &str) -> &str                          // élision : lié à x
fn m(&self, k: &str) -> &str                   // élision : lié à self
struct S<'a> { r: &'a str }
impl<'a> S<'a> { fn get(&self) -> &'a str { self.r } }
impl S<'_> { }
&'static str                                    // valide jusqu'à la fin du programme
T: 'static                                      // T ne contient aucun emprunt temporaire
'a: 'b                                          // 'a dure au moins autant que 'b
Box<dyn Trait + 'a>                             // trait object qui peut emprunter
std::thread::scope(|s| { s.spawn(|| ..); })     // threads qui empruntent
```

### Points clés

- Une durée de vie décrit une relation ; elle ne prolonge jamais la vie d'une valeur.
- Les trois règles d'élision couvrent la plupart des signatures.
- Annoter seulement les références réellement liées au résultat.
- Une valeur créée dans une fonction se renvoie possédée, jamais par référence.
- `T: 'static` = « ne contient pas d'emprunt », pas « vit toujours ».
- Commence par des structs qui possèdent ; emprunte quand c'est mesuré utile.
- Une struct ne peut pas pointer dans ses propres champs : stocke des index.
