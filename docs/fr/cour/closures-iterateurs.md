# Closures & Itérateurs
> Les fonctions anonymes qui capturent leur environnement, et les chaînes d'itérateurs : toutes les façons de les écrire, de les passer, de les renvoyer, et ce qu'elles coûtent.

## Objectifs

- Écrire une closure sous toutes ses formes et comprendre ce qu'elle capture
- Choisir entre `Fn`, `FnMut` et `FnOnce`, et savoir lequel une fonction doit demander
- Passer, stocker et renvoyer une closure de toutes les façons (générique, `impl`, `dyn`, pointeur de fonction)
- Comprendre qu'un itérateur est paresseux et ne fait rien sans consommateur
- Connaître les adaptateurs et les consommateurs de la bibliothèque standard
- Collecter dans n'importe quelle collection, y compris un `Result`
- Écrire ses propres itérateurs
- Choisir entre boucle et chaîne d'itérateurs

---

## Table des matières

1. Les closures : syntaxe
2. La capture de l'environnement
3. `Fn`, `FnMut`, `FnOnce`
4. Accepter une closure en paramètre
5. Stocker et renvoyer une closure
6. Closures asynchrones
7. Le trait `Iterator`
8. Obtenir un itérateur
9. La paresse
10. Adaptateurs
11. Consommateurs
12. `collect` en détail
13. Écrire un itérateur
14. La même fonction écrite de plusieurs façons
15. Pièges courants
16. Exercices
17. Aide-mémoire

---

## 1. Les closures : syntaxe

Une closure est une fonction sans nom, écrite là où on en a besoin, qui peut **utiliser les variables** de l'endroit où elle est écrite.

```rust
// Fonction
fn ajouter(x: i32, y: i32) -> i32 { x + y }

// La même chose en closure, de la plus explicite à la plus courte
let ajouter = |x: i32, y: i32| -> i32 { x + y };
let ajouter = |x: i32, y: i32| x + y;      // type de retour déduit
let ajouter = |x, y| x + y;                // types déduits du premier appel

let bonjour = || println!("Bonjour");      // sans paramètre

let traiter = |x: i32| {                   // corps sur plusieurs lignes
    let double = x * 2;
    double + 1
};

let paire = |(a, b): (i32, i32)| a * b;    // paramètre déstructuré

println!("{}", ajouter(1, 2));
```

### 1.1 — Les types sont fixés au premier usage

```rust
let identite = |x| x;
let a = identite(5);          // x est désormais i32
// let b = identite("texte"); // ERREUR : expected integer, found &str
```

Une closure n'est pas générique : ses types sont déduits une fois pour toutes.

### 1.2 — Closure ou fonction ?

| | Fonction `fn` | Closure |
|---|---|---|
| Capture des variables locales | non | oui |
| Types des paramètres | obligatoires | déduits |
| Générique | possible | non |
| Peut être récursive | oui | non (directement) |
| Usage typique | logique réutilisable | argument court passé à `map`, `filter`, `sort_by`, `spawn` |

Une fonction peut toujours être passée là où une closure est attendue : `v.iter().map(f)`.

---

## 2. La capture de l'environnement

### 2.1 — Les trois modes de capture

Une closure capture chaque variable de la façon **la moins contraignante** qui suffit à son corps :

```rust
let seuil = 5;
let est_grand = |x: i32| x > seuil;     // lit seuil : emprunt partagé (&)
println!("{}", est_grand(10));
println!("{seuil}");                     // seuil reste utilisable

let mut compteur = 0;
let mut incrementer = || compteur += 1;  // modifie compteur : emprunt mutable (&mut)
incrementer();
incrementer();
// tant qu'incrementer est utilisée plus bas, compteur ne peut pas être lu ici
println!("{compteur}");                  // 2 : OK, incrementer ne sert plus

let nom = String::from("Ada");
let consommer = || drop(nom);            // déplace nom : capture par valeur
consommer();
// println!("{nom}");                    // ERREUR : nom a été déplacée
```

### 2.2 — `move` : forcer la capture par valeur

```rust
let texte = String::from("bonjour");
let afficher = move || println!("{texte}");   // texte appartient à la closure
afficher();
// println!("{texte}");                       // ERREUR
```

À quoi sert `move` : la closure doit **vivre plus longtemps** que la fonction qui la crée. C'est le cas pour un thread, une tâche asynchrone ou une closure renvoyée.

```rust
use std::thread;

let donnees = vec![1, 2, 3];
let h = thread::spawn(move || {
    println!("{donnees:?}");      // sans move : ERREUR, le thread pourrait survivre à donnees
});
h.join().unwrap();
```

### 2.3 — `move` avec un type `Copy`

```rust
let x = 5;
let f = move || x + 1;     // x est COPIÉ dans la closure
println!("{x}");           // OK : x existe toujours
```

### 2.4 — Garder l'original et donner une copie

```rust
let config = String::from("prod");

// Écriture 1 : cloner avant, puis move
let c = config.clone();
let tache = move || println!("{c}");

// Écriture 2 : bloc qui prépare les captures
let tache2 = {
    let config = config.clone();
    move || println!("{config}")
};

println!("{config}");      // l'original est intact
```

La seconde forme est courante avec les threads et les tâches tokio : elle garde le même nom de variable.

### 2.5 — Capture de champs (édition 2021+)

```rust
struct Etat { nom: String, total: u32 }
let mut e = Etat { nom: "a".into(), total: 0 };

let mut ajouter = |n| e.total += n;    // capture seulement e.total, pas tout e
println!("{}", e.nom);                 // OK : e.nom n'est pas capturé
ajouter(5);
```

Depuis l'édition 2021, une closure capture le **champ** utilisé, pas la struct entière.

---

## 3. `Fn`, `FnMut`, `FnOnce`

Chaque closure implémente automatiquement un ou plusieurs de ces traits, selon ce que fait son corps.

| Trait | Le corps... | Appelable | Exemple |
|---|---|---|---|
| `Fn` | lit seulement ses captures | autant de fois qu'on veut, même en parallèle | `est_grand` ci-dessus |
| `FnMut` | modifie ses captures | plusieurs fois, une à la fois | `incrementer` ci-dessus |
| `FnOnce` | déplace une capture hors de la closure | une seule fois | `consommer` ci-dessus |

### 3.1 — La hiérarchie

```
Fn  ⊂  FnMut  ⊂  FnOnce
```

Toute closure `Fn` est aussi `FnMut` et `FnOnce`. Une fonction qui demande `FnOnce` accepte donc **toutes** les closures ; une fonction qui demande `Fn` n'accepte que celles qui ne modifient rien.

### 3.2 — `move` ne décide pas du trait

```rust
let nom = String::from("Ada");
let f = move || println!("{nom}");   // move, mais ne fait que lire : Fn
f();
f();                                 // appelable plusieurs fois
```

`move` décide **comment** la valeur entre dans la closure ; le trait dépend de **ce que le corps en fait**.

### 3.3 — Quel trait demander

| Ta fonction appelle la closure... | Demande |
|---|---|
| une seule fois | `FnOnce` (le plus permissif pour l'appelant) |
| plusieurs fois, l'une après l'autre | `FnMut` |
| plusieurs fois en parallèle, ou par référence partagée | `Fn` |

```rust
fn executer_une_fois<F: FnOnce() -> String>(f: F) -> String { f() }
fn repeter<F: FnMut()>(mut f: F, n: usize) { for _ in 0..n { f(); } }
fn appliquer<F: Fn(i32) -> i32>(f: F, v: &[i32]) -> Vec<i32> { v.iter().map(|&x| f(x)).collect() }
```

`Option::unwrap_or_else` demande `FnOnce`, `Iterator::map` demande `FnMut`, `thread::spawn` demande `FnOnce + Send + 'static`.

---

## 4. Accepter une closure en paramètre

La même fonction, qui applique une opération à un nombre, écrite de cinq façons.

```rust
// 1. Générique avec where : le compilateur génère une version par closure (le plus rapide)
fn appliquer_v1<F>(f: F, x: i32) -> i32
where
    F: Fn(i32) -> i32,
{
    f(x)
}

// 2. Générique en ligne
fn appliquer_v2<F: Fn(i32) -> i32>(f: F, x: i32) -> i32 { f(x) }

// 3. impl Trait : même chose que 2, sans nommer le type
fn appliquer_v3(f: impl Fn(i32) -> i32, x: i32) -> i32 { f(x) }

// 4. Référence vers un trait object : une seule version compilée, appel indirect
fn appliquer_v4(f: &dyn Fn(i32) -> i32, x: i32) -> i32 { f(x) }

// 5. Pointeur de fonction : n'accepte que des closures SANS capture et des fonctions
fn appliquer_v5(f: fn(i32) -> i32, x: i32) -> i32 { f(x) }

fn main() {
    let facteur = 3;
    appliquer_v1(|x| x * facteur, 2);
    appliquer_v3(|x| x + 1, 2);
    appliquer_v4(&|x| x * facteur, 2);
    appliquer_v5(|x| x * 2, 2);                // pas de capture : accepté
    // appliquer_v5(|x| x * facteur, 2);       // ERREUR : capture facteur
}
```

| Forme | Coût | Quand l'utiliser |
|---|---|---|
| générique / `impl Fn` | aucun appel indirect, code dupliqué par type | cas par défaut |
| `&dyn Fn` | appel indirect, une seule copie du code | réduire la taille du binaire ou le temps de compilation |
| `fn(..)` | appel indirect, pas de capture | tables de fonctions, interop C |

---

## 5. Stocker et renvoyer une closure

### 5.1 — Renvoyer une closure

```rust
// 1. impl Fn : une seule sorte de closure possible, aucune allocation
fn multiplicateur(n: i32) -> impl Fn(i32) -> i32 {
    move |x| x * n                 // move obligatoire : n disparaît à la fin de la fonction
}

// 2. Box<dyn Fn> : plusieurs closures différentes selon les cas
fn operation(nom: &str) -> Box<dyn Fn(i32, i32) -> i32> {
    match nom {
        "+" => Box::new(|a, b| a + b),
        "*" => Box::new(|a, b| a * b),
        _ => Box::new(|_, _| 0),
    }
}

fn main() {
    let triple = multiplicateur(3);
    println!("{}", triple(5));          // 15
    println!("{}", operation("*")(2, 4)); // 8
}
```

`impl Fn` ne marche pas pour `operation` : chaque closure a son propre type, et `impl Trait` exige un seul type concret.

### 5.2 — Stocker dans une struct

```rust
// 1. Paramètre générique : rapide, un seul type de closure par instance
struct Bouton<F: Fn()> {
    au_clic: F,
}

// 2. Box<dyn Fn> : n'importe quelle closure, on peut en mettre plusieurs dans un Vec
struct Evenements {
    ecouteurs: Vec<Box<dyn Fn(&str)>>,
}

impl Evenements {
    fn ajouter(&mut self, f: impl Fn(&str) + 'static) {
        self.ecouteurs.push(Box::new(f));
    }
    fn emettre(&self, msg: &str) {
        for e in &self.ecouteurs {
            e(msg);
        }
    }
}

fn main() {
    let b = Bouton { au_clic: || println!("clic") };
    (b.au_clic)();                    // parenthèses : appeler le champ, pas une méthode

    let mut ev = Evenements { ecouteurs: vec![] };
    ev.ajouter(|m| println!("journal : {m}"));
    ev.ajouter(|m| println!("alerte : {m}"));
    ev.emettre("connexion");
}
```

`'static` signifie que la closure ne capture **aucune référence temporaire** : elle peut être gardée aussi longtemps que nécessaire. Pour la partager entre threads : `Box<dyn Fn(&str) + Send + Sync>`.

### 5.3 — Closure récursive

Une closure ne peut pas s'appeler elle-même. On utilise une fonction, éventuellement imbriquée :

```rust
fn factorielle(n: u64) -> u64 {
    fn fact(n: u64) -> u64 { if n <= 1 { 1 } else { n * fact(n - 1) } }
    fact(n)
}
```

---

## 6. Closures asynchrones

Depuis Rust 1.85, une closure peut être `async` :

```rust
async fn traiter_tous<F>(ids: &[u32], f: F)
where
    F: AsyncFn(u32) -> String,
{
    for &id in ids {
        println!("{}", f(id).await);
    }
}

async fn exemple() {
    let prefixe = String::from("id-");
    traiter_tous(&[1, 2], async |id| format!("{prefixe}{id}")).await;
}
```

Avant 1.85, on écrivait `|id| async move { ... }` : une closure **normale** qui renvoie un futur. Cette forme ne peut pas emprunter ses captures dans le futur, d'où des `clone` partout. `async ||` et les traits `AsyncFn`, `AsyncFnMut`, `AsyncFnOnce` règlent ce problème.

---

## 7. Le trait `Iterator`

```rust
pub trait Iterator {
    type Item;
    fn next(&mut self) -> Option<Self::Item>;
    // + plus de 70 méthodes fournies : map, filter, sum, collect...
}
```

Un itérateur est une valeur qui produit des éléments un par un : `next()` renvoie `Some(element)` tant qu'il en reste, puis `None`.

```rust
let v = vec![1, 2, 3];
let mut it = v.iter();
assert_eq!(it.next(), Some(&1));
assert_eq!(it.next(), Some(&2));
assert_eq!(it.next(), Some(&3));
assert_eq!(it.next(), None);
```

Il suffit d'écrire `next` : toutes les autres méthodes viennent gratuitement.

---

## 8. Obtenir un itérateur

### 8.1 — Depuis une collection

```rust
let mut v = vec![1, 2, 3];

let a = v.iter();          // éléments &i32 : lecture
let b = v.iter_mut();      // éléments &mut i32 : modification
let c = v.into_iter();     // éléments i32 : consomme v
```

`for x in &v` appelle `iter()`, `for x in &mut v` appelle `iter_mut()`, `for x in v` appelle `into_iter()`.

### 8.2 — Autres sources

```rust
let plage = 1..=5;
let par_pas = (0..10).step_by(2);
let caracteres = "abc".chars();
let mots = "un deux".split_whitespace();
let lignes = "a\nb".lines();
let cles = std::collections::HashMap::<i32, i32>::new().into_keys();

use std::iter;
let un_seul = iter::once(42);
let vide = iter::empty::<i32>();
let repete = iter::repeat("x").take(3);               // "x", "x", "x"
let puissances = iter::successors(Some(1), |&x| (x < 100).then(|| x * 2)); // 1, 2, 4 ... 128
let mut n = 0;
let compteur = iter::from_fn(|| { n += 1; (n <= 3).then_some(n) });       // 1, 2, 3
let carres = (1..).map(|x| x * x);                    // infini : à limiter avec take
```

### 8.3 — Itérateur sur un tableau

```rust
let t = [1, 2, 3];
for x in t { }             // édition 2021+ : x est i32 (par valeur)
for x in t.iter() { }      // x est &i32
```

---

## 9. La paresse

Un adaptateur (`map`, `filter`...) **ne fait rien** tant qu'un consommateur ne demande pas d'éléments.

```rust
let v = vec![1, 2, 3];
v.iter().map(|x| {
    println!("traitement de {x}");
    x * 2
});
// Rien n'est affiché. Le compilateur avertit : unused `Map` that must be used
```

```rust
let doubles: Vec<i32> = v.iter().map(|x| x * 2).collect();   // collect déclenche le calcul
```

### 9.1 — Ce que la paresse permet

```rust
// Un itérateur infini, utilisable parce qu'on n'en prend que le début
let premiers_pairs: Vec<u32> = (1..).filter(|x| x % 2 == 0).take(5).collect();   // [2, 4, 6, 8, 10]

// Un seul passage, sans collection intermédiaire
let somme: i32 = (1..=1_000_000).filter(|x| x % 3 == 0).map(|x| x * 2).sum();

// Arrêt anticipé : find s'arrête au premier trouvé, map n'est appelé que jusque-là
let premier = (1..).map(|x| x * x).find(|&x| x > 50);    // Some(64)
```

### 9.2 — L'ordre des opérations

Chaque élément traverse **toute la chaîne** avant que le suivant commence :

```rust
let _: Vec<_> = [1, 2].iter()
    .map(|x| { println!("map {x}"); x * 10 })
    .filter(|x| { println!("filter {x}"); true })
    .collect();
// map 1, filter 10, map 2, filter 20
```

Place les `filter` **avant** les `map` coûteux : les éléments écartés ne sont jamais transformés.

### 9.3 — Coût

Une chaîne d'itérateurs est compilée en une boucle aussi rapide qu'une boucle écrite à la main (on parle d'abstraction sans coût). Les vérifications de bornes disparaissent souvent, ce qui la rend parfois plus rapide qu'une boucle avec index.

---

## 10. Adaptateurs

Un adaptateur prend un itérateur et en renvoie un autre.

### 10.1 — Transformer

```rust
let n = vec![1, 2, 3];

n.iter().map(|x| x * 2);                       // 2, 4, 6
n.iter().copied();                             // &i32 -> i32 (types Copy)
n.iter().cloned();                             // &T -> T (types Clone)
n.iter().enumerate();                          // (0, &1), (1, &2), (2, &3)
n.iter().inspect(|x| println!("vu {x}"));      // regarder sans modifier (débogage)
n.iter().scan(0, |acc, &x| { *acc += x; Some(*acc) });   // sommes cumulées : 1, 3, 6
```

### 10.2 — Filtrer

```rust
n.iter().filter(|&&x| x > 1);                  // 2, 3
n.iter().filter_map(|&x| (x > 1).then_some(x * 10));   // 20, 30
["1", "a", "3"].iter().filter_map(|s| s.parse::<i32>().ok());   // 1, 3
n.iter().take(2);                              // 1, 2
n.iter().skip(1);                              // 2, 3
n.iter().take_while(|&&x| x < 3);              // 1, 2 puis s'arrête
n.iter().skip_while(|&&x| x < 2);              // 2, 3
["1", "2", "x", "4"].iter().map_while(|s| s.parse::<i32>().ok());   // 1, 2 puis s'arrête
n.iter().step_by(2);                           // 1, 3
```

`filter` continue après un élément refusé ; `take_while` s'arrête définitivement.

### 10.3 — Combiner

```rust
let a = [1, 2];
let b = [10, 20];

a.iter().chain(b.iter());                      // 1, 2, 10, 20
a.iter().zip(b.iter());                        // (1, 10), (2, 20)
a.iter().rev();                                // 2, 1
a.iter().cycle().take(5);                      // 1, 2, 1, 2, 1
vec![vec![1], vec![2, 3]].into_iter().flatten();          // 1, 2, 3
["a b", "c"].iter().flat_map(|s| s.split(' '));            // "a", "b", "c"
vec![Some(1), None, Some(3)].into_iter().flatten();       // 1, 3 : flatten sur des Option
```

### 10.4 — Regarder l'élément suivant : `peekable`

```rust
let mut it = [1, 1, 2, 3, 3, 3].into_iter().peekable();
let mut groupes = Vec::new();
while let Some(x) = it.next() {
    let mut nb = 1;
    while it.peek() == Some(&x) {      // regarde sans consommer
        it.next();
        nb += 1;
    }
    groupes.push((x, nb));
}
// [(1, 2), (2, 1), (3, 3)]
```

À quoi ça sert : analyseurs (lire le caractère suivant pour savoir si un nombre continue), regroupement d'éléments consécutifs. `next_if(|x| ...)` consomme seulement si la condition est vraie.

### 10.5 — `by_ref` : consommer en plusieurs fois

```rust
let mut lignes = "titre\n---\ncorps 1\ncorps 2".lines();
let entete: Vec<&str> = lignes.by_ref().take_while(|l| *l != "---").collect();
let corps: Vec<&str> = lignes.collect();     // la suite, après le séparateur
```

`take` ou `take_while` consomme l'itérateur ; `by_ref()` le prête, pour continuer à s'en servir ensuite.

---

## 11. Consommateurs

Un consommateur parcourt l'itérateur et produit un résultat.

```rust
let n = [3, 1, 4, 1, 5];

n.iter().sum::<i32>();                          // 14
n.iter().product::<i32>();                      // 60
n.iter().count();                               // 5
n.iter().min();                                 // Some(&1)
n.iter().max();                                 // Some(&5)
n.iter().last();                                // Some(&5)
n.iter().nth(2);                                // Some(&4)
n.iter().any(|&x| x > 4);                       // true
n.iter().all(|&x| x > 0);                       // true
n.iter().find(|&&x| x > 3);                     // Some(&4)
n.iter().position(|&x| x == 1);                 // Some(1)
n.iter().rposition(|&x| x == 1);                // Some(3) : depuis la fin
["a", "12", "b"].iter().find_map(|s| s.parse::<i32>().ok());   // Some(12)
n.iter().fold(0, |acc, &x| acc + x);            // 14
n.iter().copied().reduce(|a, b| a.max(b));      // Some(5) : fold sans valeur initiale
n.iter().for_each(|x| println!("{x}"));

let mots = ["rust", "go", "python"];
mots.iter().max_by_key(|m| m.len());            // Some("python")
mots.iter().min_by(|a, b| a.len().cmp(&b.len()));   // Some("go")
```

### 11.1 — Consommateurs qui peuvent échouer

```rust
// try_fold : s'arrête à la première erreur
let r: Option<i32> = [1, 2, 3].iter().try_fold(0i32, |acc, &x| acc.checked_add(x));   // Some(6)

// try_for_each : boucle avec ?
let r: Result<(), String> = ["1", "x"].iter().try_for_each(|s| {
    s.parse::<i32>().map(|_| ()).map_err(|e| e.to_string())
});
```

### 11.2 — `sum` et `max` d'éléments vides

```rust
let vide: [i32; 0] = [];
vide.iter().sum::<i32>();      // 0
vide.iter().max();             // None : pas de maximum
```

### 11.3 — Les flottants

```rust
let f = [2.5, 1.0, 3.2];
// f.iter().max();                                   // ERREUR : f64 n'est pas Ord
f.iter().copied().fold(f64::NEG_INFINITY, f64::max); // 3.2
f.iter().max_by(|a, b| a.total_cmp(b));              // Some(&3.2)
```

---

## 12. `collect` en détail

### 12.1 — Dans n'importe quelle collection

```rust
use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};

let v: Vec<i32> = (1..4).collect();
let s: HashSet<i32> = [1, 2, 2].into_iter().collect();
let d: VecDeque<i32> = (1..4).collect();
let m: HashMap<&str, i32> = [("a", 1), ("b", 2)].into_iter().collect();
let b: BTreeMap<i32, char> = "abc".char_indices().map(|(i, c)| (i as i32, c)).collect();
let t: String = ['a', 'b'].iter().collect();
let t2: String = ["un", "deux"].iter().map(|s| s.to_uppercase()).collect();
let t3 = ["un", "deux"].join(", ");        // pour joindre avec un séparateur
```

### 12.2 — Indiquer le type

```rust
let v: Vec<_> = (1..4).collect();          // annotation sur la variable
let v = (1..4).collect::<Vec<_>>();        // turbofish
fn premiers() -> Vec<i32> { (1..4).collect() }   // type déduit du retour
```

### 12.3 — Collecter des `Result` et des `Option`

```rust
let textes = ["1", "2", "x"];

// Un Vec de Result : chaque élément garde son résultat
let tous: Vec<Result<i32, _>> = textes.iter().map(|s| s.parse::<i32>()).collect();

// Un Result de Vec : Ok si TOUS réussissent, sinon la première erreur
let r: Result<Vec<i32>, _> = textes.iter().map(|s| s.parse::<i32>()).collect();   // Err

// Garder seulement les réussites
let ok: Vec<i32> = textes.iter().filter_map(|s| s.parse().ok()).collect();       // [1, 2]

// Séparer réussites et échecs
let (bons, mauvais): (Vec<_>, Vec<_>) = textes.iter().map(|s| s.parse::<i32>()).partition(|r| r.is_ok());

// Même chose avec Option
let o: Option<Vec<i32>> = [Some(1), None].into_iter().collect();   // None
```

`collect::<Result<Vec<_>, _>>()` s'arrête dès la première erreur : c'est la façon de valider toute une liste.

### 12.4 — Séparer en deux

```rust
let (pairs, impairs): (Vec<i32>, Vec<i32>) = (1..=6).partition(|x| x % 2 == 0);
let (noms, ages): (Vec<&str>, Vec<u32>) = [("a", 1), ("b", 2)].into_iter().unzip();
```

### 12.5 — Ajouter à une collection existante

```rust
let mut v = vec![1];
v.extend((2..4).map(|x| x * 10));          // [1, 20, 30]
```

---

## 13. Écrire un itérateur

### 13.1 — Implémenter `Iterator`

```rust
struct Compte { valeur: u32, max: u32 }

impl Compte {
    fn new(max: u32) -> Self { Compte { valeur: 0, max } }
}

impl Iterator for Compte {
    type Item = u32;

    fn next(&mut self) -> Option<u32> {
        if self.valeur < self.max {
            self.valeur += 1;
            Some(self.valeur)
        } else {
            None
        }
    }
}

let somme: u32 = Compte::new(5)
    .zip(Compte::new(5).skip(1))
    .map(|(a, b)| a * b)
    .filter(|x| x % 3 == 0)
    .sum();
// Compte a toutes les méthodes d'Iterator gratuitement
```

### 13.2 — Sans struct : `from_fn` et `successors`

```rust
use std::iter;

// Fibonacci avec une struct implicite
fn fibonacci() -> impl Iterator<Item = u64> {
    let (mut a, mut b) = (0u64, 1u64);
    iter::from_fn(move || {
        let r = a;
        (a, b) = (b, a.checked_add(b)?);    // s'arrête avant de déborder
        Some(r)
    })
}

// Chaque élément calculé à partir du précédent
fn collatz(n: u64) -> impl Iterator<Item = u64> {
    iter::successors(Some(n), |&x| match x {
        1 => None,
        x if x % 2 == 0 => Some(x / 2),
        x => Some(3 * x + 1),
    })
}

let f: Vec<u64> = fibonacci().take(8).collect();   // [0, 1, 1, 2, 3, 5, 8, 13]
let c: Vec<u64> = collatz(6).collect();            // [6, 3, 10, 5, 16, 8, 4, 2, 1]
```

### 13.3 — Renvoyer un itérateur

```rust
// 1. impl Iterator : le type exact reste caché, aucun coût
fn pairs(v: &[i32]) -> impl Iterator<Item = &i32> {
    v.iter().filter(|x| *x % 2 == 0)
}

// 2. Box<dyn Iterator> : quand plusieurs chaînes différentes peuvent être renvoyées
fn selection(v: &[i32], inverse: bool) -> Box<dyn Iterator<Item = &i32> + '_> {
    if inverse { Box::new(v.iter().rev()) } else { Box::new(v.iter()) }
}

// 3. Collecter : simple, mais alloue un Vec même si l'appelant ne veut que le premier
fn pairs_vec(v: &[i32]) -> Vec<i32> {
    v.iter().copied().filter(|x| x % 2 == 0).collect()
}
```

### 13.4 — Rendre son type utilisable dans un `for`

```rust
struct Panier { articles: Vec<String> }

impl<'a> IntoIterator for &'a Panier {
    type Item = &'a String;
    type IntoIter = std::slice::Iter<'a, String>;
    fn into_iter(self) -> Self::IntoIter {
        self.articles.iter()
    }
}

let p = Panier { articles: vec!["pomme".into()] };
for a in &p {
    println!("{a}");
}
```

### 13.5 — Traits complémentaires

| Trait | Ce qu'il ajoute | Exemple |
|---|---|---|
| `DoubleEndedIterator` | `next_back`, donc `rev()` | itérateurs de `Vec`, plages |
| `ExactSizeIterator` | `len()` exact | `v.iter()` |
| `FusedIterator` | garantit `None` pour toujours après le premier `None` | la plupart des itérateurs standard |

---

## 14. La même fonction écrite de plusieurs façons

### 14.1 — Les emails valides en minuscules

```rust
struct Utilisateur { email: String, actif: bool }

// 1. Boucle
fn emails_v1(u: &[Utilisateur]) -> Vec<String> {
    let mut r = Vec::new();
    for x in u {
        if x.actif && x.email.contains('@') {
            r.push(x.email.to_lowercase());
        }
    }
    r
}

// 2. filter + map
fn emails_v2(u: &[Utilisateur]) -> Vec<String> {
    u.iter()
        .filter(|x| x.actif && x.email.contains('@'))
        .map(|x| x.email.to_lowercase())
        .collect()
}

// 3. filter_map
fn emails_v3(u: &[Utilisateur]) -> Vec<String> {
    u.iter()
        .filter_map(|x| (x.actif && x.email.contains('@')).then(|| x.email.to_lowercase()))
        .collect()
}
```

### 14.2 — Compter les mots

```rust
use std::collections::HashMap;

// 1. Boucle + entry
fn compter_v1(t: &str) -> HashMap<&str, usize> {
    let mut m = HashMap::new();
    for mot in t.split_whitespace() {
        *m.entry(mot).or_insert(0) += 1;
    }
    m
}

// 2. fold
fn compter_v2(t: &str) -> HashMap<&str, usize> {
    t.split_whitespace().fold(HashMap::new(), |mut m, mot| {
        *m.entry(mot).or_insert(0) += 1;
        m
    })
}
```

Ici la boucle est plus lisible : `fold` avec un accumulateur mutable n'apporte rien.

### 14.3 — Choisir

- Une **transformation** de données (filtrer, convertir, calculer un total) se lit mieux en chaîne d'itérateurs.
- Une boucle avec des **effets** (afficher, écrire en base), plusieurs sorties anticipées, `?`, `break` ou `continue` reste plus claire en `for`.
- Au-delà de cinq ou six étapes, une chaîne gagne à être coupée en variables nommées ou en fonctions.

---

## 15. Pièges courants

### 15.1 — Les `&` en trop ou en moins

```rust
let v = vec![1, 2, 3];

v.iter().filter(|x| **x > 1);          // filter reçoit &&i32
v.iter().filter(|&&x| x > 1);          // déstructuration des deux &
v.iter().copied().filter(|&x| x > 1);  // copied d'abord : un seul &
v.iter().map(|x| x * 2);               // map reçoit &i32 ; * fonctionne sur &i32
```

`filter` reçoit **une référence vers l'élément** de l'itérateur ; si l'itérateur produit déjà des `&i32`, on obtient `&&i32`.

### 15.2 — Utiliser après `into_iter`

```rust
let v = vec![String::from("a")];
let m: Vec<String> = v.into_iter().map(|s| s + "!").collect();
// println!("{v:?}");                  // ERREUR : v consommé
```

### 15.3 — Modifier la source pendant l'itération

```rust
let mut v = vec![1, 2, 3];
// for x in v.iter() { v.push(*x); }  // ERREUR : v empruntée
let copie: Vec<i32> = v.clone();
v.extend(copie);
```

### 15.4 — `collect` sans type

```rust
// let v = (1..4).collect();           // ERREUR : type annotations needed
let v: Vec<i32> = (1..4).collect();
```

### 15.5 — `return` dans une closure

```rust
fn premier_negatif(v: &[i32]) -> Option<i32> {
    v.iter().for_each(|&x| {
        if x < 0 {
            return;                    // quitte la CLOSURE, pas la fonction
        }
    });
    v.iter().copied().find(|&x| x < 0) // la bonne façon
}
```

---

## 16. Exercices

### Exercice 1 — compteur

Écris une fonction qui renvoie une closure comptant ses appels (`FnMut() -> u32`).

```rust
// Solution :
fn compteur() -> impl FnMut() -> u32 {
    let mut n = 0;
    move || {
        n += 1;
        n
    }
}

let mut c = compteur();
assert_eq!(c(), 1);
assert_eq!(c(), 2);
```

### Exercice 2 — valider une liste

Convertis `&[&str]` en `Vec<u32>`, en renvoyant la première erreur si un élément n'est pas un nombre.

```rust
// Solution :
fn convertir(t: &[&str]) -> Result<Vec<u32>, std::num::ParseIntError> {
    t.iter().map(|s| s.trim().parse::<u32>()).collect()
}
```

### Exercice 3 — moyenne glissante

Pour `[1.0, 2.0, 3.0, 4.0]` et une fenêtre de 2, renvoie `[1.5, 2.5, 3.5]`.

```rust
// Solution :
fn moyenne_glissante(v: &[f64], n: usize) -> Vec<f64> {
    v.windows(n).map(|w| w.iter().sum::<f64>() / n as f64).collect()
}
```

### Exercice 4 — itérateur de dates

Écris un itérateur qui produit les numéros de jour 1 à 31, sauf les dimanches (tous les 7 jours à partir du jour 7).

```rust
// Solution :
fn jours_ouvres() -> impl Iterator<Item = u32> {
    (1..=31).filter(|j| j % 7 != 0)
}
```

---

## 17. Aide-mémoire

### Closures

```rust
|x| x + 1                   |x: i32| -> i32 { x + 1 }       || println!()
move || utilise(valeur)     async |x| f(x).await
fn f(g: impl Fn(i32) -> i32)      fn f<F: FnMut()>(g: F)      fn f(g: &dyn Fn())
fn f() -> impl Fn(i32) -> i32     Box<dyn Fn(i32) -> i32>     fn(i32) -> i32
```

### Itérateurs

```rust
v.iter()  v.iter_mut()  v.into_iter()  (a..b)  iter::once  iter::repeat  iter::from_fn  iter::successors
.map  .filter  .filter_map  .flat_map  .flatten  .enumerate  .zip  .chain  .rev  .take  .skip
.take_while  .skip_while  .map_while  .step_by  .peekable  .scan  .inspect  .copied  .cloned  .by_ref
.collect  .sum  .product  .count  .min  .max  .min_by_key  .max_by  .fold  .reduce  .any  .all
.find  .find_map  .position  .last  .nth  .partition  .unzip  .for_each  .try_fold  .try_for_each
```

### Points clés

- Une closure capture de la façon la moins contraignante ; `move` force la capture par valeur.
- Le trait (`Fn`, `FnMut`, `FnOnce`) dépend de ce que fait le corps, pas de `move`.
- Demande `FnOnce` si tu appelles une fois, `FnMut` plusieurs fois, `Fn` en parallèle.
- `impl Fn` pour une closure, `Box<dyn Fn>` pour plusieurs sortes ou pour en stocker une liste.
- Un itérateur ne fait rien sans consommateur.
- `collect::<Result<Vec<_>, _>>()` valide toute une liste et s'arrête à la première erreur.
- Une chaîne d'itérateurs coûte autant qu'une boucle écrite à la main.
