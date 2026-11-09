# Les Collections en Rust

**Vec, VecDeque, HashMap, BTreeMap, HashSet, BTreeSet, BinaryHeap** — créer, remplir, lire, modifier et parcourir chaque collection, avec les différentes façons de faire la même opération et le coût de chacune.

## Objectifs du cours

À la fin de ce cours, tu sauras :

- Créer une collection de toutes les façons (`new`, macro, `from`, `collect`, capacité réservée)
- Ajouter, retirer, chercher et trier les éléments d'un `Vec`
- Lire un élément sans risquer de panique
- Utiliser l'API `entry` d'une `HashMap` pour compter, regrouper ou initialiser
- Faire des opérations d'ensemble avec `HashSet`
- Choisir la collection adaptée selon l'opération la plus fréquente
- Comprendre ce qui est déplacé ou emprunté quand on met une valeur dans une collection

## Table des matières

1. Vue d'ensemble
2. `Vec<T>` : créer
3. `Vec<T>` : ajouter et retirer
4. `Vec<T>` : lire
5. `Vec<T>` : trier, chercher, dédoublonner
6. `Vec<T>` : parcourir et transformer
7. Tranches et `Vec`
8. `VecDeque<T>`
9. `HashMap<K, V>` : créer et remplir
10. `HashMap<K, V>` : lire et modifier
11. L'API `entry`
12. `BTreeMap<K, V>`
13. `HashSet<T>` et `BTreeSet<T>`
14. `BinaryHeap<T>`
15. Ownership et collections
16. Choisir la bonne collection
17. La même tâche écrite de plusieurs façons
18. Exercices
19. Aide-mémoire

## 1. Vue d'ensemble

| Collection | Contenu | Ordre | Usage typique |
|---|---|---|---|
| `Vec<T>` | suite d'éléments | ordre d'insertion | la collection par défaut |
| `VecDeque<T>` | suite, ajout/retrait aux deux bouts | ordre d'insertion | file d'attente |
| `HashMap<K, V>` | paires clé → valeur | aucun | recherche par clé |
| `BTreeMap<K, V>` | paires clé → valeur | trié par clé | recherche par clé + parcours trié, plages |
| `HashSet<T>` | valeurs uniques | aucun | appartenance, dédoublonnage |
| `BTreeSet<T>` | valeurs uniques | trié | ensemble trié |
| `BinaryHeap<T>` | file de priorité | le plus grand en premier | traiter le plus prioritaire |

Toutes sont dans `std::collections`, sauf `Vec` (importé d'office).

```rust
use std::collections::{BTreeMap, BTreeSet, BinaryHeap, HashMap, HashSet, VecDeque};
```

## 2. `Vec<T>` : créer

### 2.1 — Les différentes façons

```rust
let v1: Vec<i32> = Vec::new();             // vide, type annoté
let mut v2 = Vec::new();                    // vide, type déduit plus loin
v2.push(1);

let v3 = vec![1, 2, 3];                     // macro avec éléments
let v4 = vec![0; 10];                       // dix zéros
let v5 = Vec::from([1, 2, 3]);              // depuis un tableau
let v6: Vec<i32> = (1..=5).collect();       // depuis un itérateur
let v7 = vec!["a"; 3];                      // ["a", "a", "a"]
let v8: Vec<char> = "abc".chars().collect();
let v9 = [1, 2].to_vec();                   // copie d'une tranche
```

### 2.2 — Capacité et longueur

```rust
let mut v = Vec::with_capacity(100);   // réserve la place de 100 éléments, longueur 0
println!("{} {}", v.len(), v.capacity());  // 0 100
for i in 0..100 {
    v.push(i);                          // aucune réallocation
}
v.reserve(50);                          // garantit 50 places de plus
v.shrink_to_fit();                      // rend la mémoire inutilisée
```

- `len()` : nombre d'éléments.
- `capacity()` : place allouée.

Quand `len` atteint `capacity`, `push` réalloue un bloc environ deux fois plus grand et y recopie tout. À quoi sert `with_capacity` : quand tu connais la taille finale, il évite ces copies successives. `collect` le fait tout seul quand l'itérateur connaît sa taille.

### 2.3 — `vec![x; n]` et le clonage

```rust
let lignes = vec![Vec::<i32>::new(); 3];   // trois Vec indépendants (chacun cloné)
```

`vec![x; n]` **clone** `x` : le type doit implémenter `Clone`.

## 3. `Vec<T>` : ajouter et retirer

### 3.1 — À la fin (rapide)

```rust
let mut v = vec![1, 2, 3];
v.push(4);                     // [1, 2, 3, 4]
let dernier = v.pop();         // Some(4) ; None si vide
```

### 3.2 — Au milieu (lent : décale les éléments)

```rust
let mut v = vec![1, 2, 3];
v.insert(1, 10);               // [1, 10, 2, 3]
let retire = v.remove(0);      // 1 ; [10, 2, 3] — panique si l'index n'existe pas
```

### 3.3 — Retirer sans garder l'ordre (rapide)

```rust
let mut v = vec![1, 2, 3, 4];
let x = v.swap_remove(0);      // 1 ; le dernier prend sa place : [4, 2, 3]
```

`remove` décale tous les éléments suivants (coût proportionnel à la taille), `swap_remove` n'en déplace qu'un. À utiliser quand l'ordre n'a pas d'importance.

### 3.4 — Plusieurs éléments

```rust
let mut v = vec![1, 2, 3];
v.extend([4, 5]);                        // ajoute depuis n'importe quel itérateur
v.extend_from_slice(&[6, 7]);            // copie depuis une tranche

let mut autre = vec![8, 9];
v.append(&mut autre);                    // déplace tout : autre devient vide

let fin = v.split_off(5);                // v garde [0..5), fin reçoit le reste
v.truncate(2);                           // garde les 2 premiers
let extraits: Vec<i32> = v.drain(..1).collect();   // retire une plage et la renvoie
v.clear();                               // vide (garde la capacité)
```

### 3.5 — Retirer selon une condition

```rust
let mut v = vec![1, 2, 3, 4, 5, 6];

// 1. retain : garde ceux qui vérifient la condition (sur place)
v.retain(|&x| x % 2 == 0);              // [2, 4, 6]

// 2. filter + collect : nouvelle collection, l'original intact
let v = vec![1, 2, 3, 4];
let pairs: Vec<i32> = v.iter().copied().filter(|x| x % 2 == 0).collect();

// 3. extract_if (Rust 1.87+) : retire ET récupère ce qui est retiré
let mut v = vec![1, 2, 3, 4];
let impairs: Vec<i32> = v.extract_if(.., |x| *x % 2 == 1).collect();   // v = [2, 4]
```

Ne retire jamais des éléments par index dans une boucle `for i in 0..v.len()` : les index se décalent et la fin de boucle déborde.

## 4. `Vec<T>` : lire

### 4.1 — Par index ou par `get`

```rust
let v = vec![10, 20, 30];

let a = v[0];                 // 10 — panique si l'index n'existe pas
let b = v.get(0);             // Some(&10)
let c = v.get(10);            // None, pas de panique

match v.get(5) {
    Some(x) => println!("{x}"),
    None => println!("pas d'élément"),
}
let x = v.get(5).copied().unwrap_or(0);
```

| Écriture | Index absent | Quand l'utiliser |
|---|---|---|
| `v[i]` | panique | l'index est garanti valide (calculé à partir de `len`) |
| `v.get(i)` | `None` | l'index vient de l'extérieur (utilisateur, fichier, URL) |

### 4.2 — Premier, dernier, morceaux

```rust
let v = vec![1, 2, 3, 4];
v.first();               // Some(&1)
v.last();                // Some(&4)
v.split_first();         // Some((&1, &[2, 3, 4]))
v.get(1..3);             // Some(&[2, 3])
v.is_empty();
v.len();
```

### 4.3 — Modifier un élément

```rust
let mut v = vec![1, 2, 3];
v[0] = 10;

if let Some(x) = v.get_mut(1) {
    *x += 5;
}

if let Some(dernier) = v.last_mut() {
    *dernier = 0;
}

v.swap(0, 2);            // échange deux éléments
v.fill(7);               // remplace tout
```

### 4.4 — Emprunt et modification

```rust
let mut v = vec![1, 2, 3];
let premier = &v[0];
// v.push(4);            // ERREUR : v est empruntée par `premier`
println!("{premier}");
v.push(4);               // OK : `premier` ne sert plus
```

`push` peut réallouer le bloc mémoire, ce qui rendrait `premier` invalide : le compilateur interdit donc de modifier le `Vec` tant qu'une référence vers un élément est utilisée.

## 5. `Vec<T>` : trier, chercher, dédoublonner

### 5.1 — Trier

```rust
let mut v = vec![3, 1, 2];
v.sort();                                  // [1, 2, 3], stable
v.sort_unstable();                         // plus rapide, n'assure pas l'ordre des égaux
v.reverse();                               // [3, 2, 1]
v.sort_by(|a, b| b.cmp(a));                // décroissant

struct P { nom: String, age: u32 }
let mut gens = vec![P { nom: "B".into(), age: 30 }, P { nom: "A".into(), age: 20 }];
gens.sort_by_key(|p| p.age);                          // par âge
gens.sort_by(|a, b| a.nom.cmp(&b.nom));               // par nom
gens.sort_by(|a, b| b.age.cmp(&a.age).then_with(|| a.nom.cmp(&b.nom))); // âge décroissant puis nom
gens.sort_by_key(|p| std::cmp::Reverse(p.age));       // décroissant, autre écriture

let mut f = vec![2.5, 1.0, 3.2];
f.sort_by(|a, b| a.total_cmp(b));          // les flottants n'ont pas Ord
```

`sort_by_key` avec une clé coûteuse (une `String` créée à chaque comparaison) : préfère `sort_by_cached_key`, qui la calcule une fois par élément.

### 5.2 — Chercher

```rust
let v = vec![10, 20, 30, 40];

v.contains(&20);                          // true
v.iter().position(|&x| x == 30);          // Some(2)
v.iter().find(|&&x| x > 15);              // Some(&20)
v.iter().any(|&x| x > 35);                // true
v.binary_search(&30);                     // Ok(2) — v doit être TRIÉ
v.binary_search(&25);                     // Err(2) : position où l'insérer pour garder l'ordre
```

`contains` et `position` parcourent tout (coût proportionnel à la taille). `binary_search` divise par deux à chaque étape (1 000 000 d'éléments : 20 comparaisons), mais exige un `Vec` trié.

### 5.3 — Dédoublonner

```rust
let mut v = vec![1, 1, 2, 3, 3, 3, 1];
v.dedup();                     // [1, 2, 3, 1] : retire les doublons CONSÉCUTIFS

let mut v = vec![3, 1, 3, 2, 1];
v.sort();
v.dedup();                     // [1, 2, 3] : trier d'abord pour tout dédoublonner

// En gardant l'ordre d'origine
use std::collections::HashSet;
let v = vec![3, 1, 3, 2, 1];
let mut vus = HashSet::new();
let uniques: Vec<i32> = v.into_iter().filter(|x| vus.insert(*x)).collect();   // [3, 1, 2]
```

## 6. `Vec<T>` : parcourir et transformer

### 6.1 — Les trois parcours

```rust
let mut v = vec![1, 2, 3];

for x in &v { }              // &i32 : lecture
for x in &mut v { *x += 1; } // &mut i32 : modification
for x in v { }               // i32 : consomme v
```

Le détail de ces trois formes est dans le cours `structures-et-controle`.

### 6.2 — Transformer

```rust
let v = vec![1, 2, 3];

let doubles: Vec<i32> = v.iter().map(|x| x * 2).collect();
let textes: Vec<String> = v.iter().map(|x| x.to_string()).collect();
let somme: i32 = v.iter().sum();
let joint = v.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(", ");   // "1, 2, 3"
let (pairs, impairs): (Vec<i32>, Vec<i32>) = v.iter().copied().partition(|x| x % 2 == 0);
```

Les itérateurs sont détaillés dans le cours `closures-iterateurs`.

### 6.3 — Aplatir et concaténer

```rust
let matrice = vec![vec![1, 2], vec![3]];
let plat: Vec<i32> = matrice.concat();                          // [1, 2, 3]
let plat2: Vec<i32> = matrice.into_iter().flatten().collect();  // même résultat

let mots = vec!["a", "b", "c"];
let phrase = mots.join("-");                                    // "a-b-c"
```

## 7. Tranches et `Vec`

### 7.1 — Accepter une tranche

```rust
// Écriture 1 : &Vec<i32> — n'accepte qu'un Vec
fn somme_vec(v: &Vec<i32>) -> i32 { v.iter().sum() }

// Écriture 2 : &[i32] — accepte Vec, tableau, partie de l'un ou de l'autre
fn somme(v: &[i32]) -> i32 { v.iter().sum() }

fn main() {
    let v = vec![1, 2, 3];
    let t = [4, 5];
    somme(&v);           // &Vec<i32> devient &[i32] automatiquement
    somme(&t);
    somme(&v[1..]);
}
```

`clippy` signale `&Vec<T>` en paramètre (`ptr_arg`) : `&[T]` fait la même chose en acceptant plus de types. De même, préfère `&str` à `&String`.

### 7.2 — Méthodes des tranches

Toutes les méthodes de lecture, de tri et de recherche de `Vec` viennent en réalité des tranches : `sort`, `contains`, `iter`, `first`, `binary_search`, `windows`, `chunks`, `split_at`, `starts_with`, `concat`, `join`... Elles fonctionnent donc aussi sur un tableau ou un morceau de `Vec`.

```rust
let v = [1, 2, 3, 4, 5, 6];
let (gauche, droite) = v.split_at(2);     // [1, 2] et [3, 4, 5, 6]
for paire in v.windows(2) { }             // fenêtres glissantes
for lot in v.chunks(4) { }                // [1, 2, 3, 4], [5, 6]
v.starts_with(&[1, 2]);                   // true
v.iter().rev();                           // parcours à l'envers
```

## 8. `VecDeque<T>`

Une file à double entrée : ajout et retrait rapides **aux deux bouts**.

```rust
use std::collections::VecDeque;

let mut file = VecDeque::new();
file.push_back(1);            // à la fin
file.push_back(2);
file.push_front(0);           // au début
let premier = file.pop_front();   // Some(0)
let dernier = file.pop_back();    // Some(2)
file.front();                 // Some(&1)
```

À quoi ça sert : une file d'attente (premier arrivé, premier servi). Avec un `Vec`, retirer au début (`v.remove(0)`) décale tous les éléments à chaque fois ; avec `VecDeque`, c'est immédiat.

```rust
// File de tâches limitée aux 3 dernières entrées
let mut historique = VecDeque::with_capacity(3);
for evenement in ["a", "b", "c", "d"] {
    if historique.len() == 3 {
        historique.pop_front();
    }
    historique.push_back(evenement);
}
// ["b", "c", "d"]
```

## 9. `HashMap<K, V>` : créer et remplir

### 9.1 — Créer

```rust
use std::collections::HashMap;

let mut scores: HashMap<String, u32> = HashMap::new();
let scores2 = HashMap::from([("Alice", 10), ("Bob", 7)]);
let scores3: HashMap<&str, u32> = vec![("Alice", 10), ("Bob", 7)].into_iter().collect();
let noms = ["a", "b"];
let longueurs: HashMap<&str, usize> = noms.iter().map(|n| (*n, n.len())).collect();
let reserve: HashMap<u32, String> = HashMap::with_capacity(1000);
```

### 9.2 — Conditions sur la clé

La clé doit implémenter `Eq` et `Hash`. Conséquences :

- `String`, `&str`, entiers, `char`, `bool`, tuples et structs dérivant `Eq + Hash` : OK ;
- `f64` : **impossible** (pas de `Eq` à cause de `NaN`) ; on stocke les flottants en entiers (centimes, millièmes) ou avec une crate comme `ordered-float`.

```rust
#[derive(PartialEq, Eq, Hash)]
struct Coord { x: i32, y: i32 }

let mut grille: HashMap<Coord, char> = HashMap::new();
grille.insert(Coord { x: 0, y: 0 }, '#');
```

### 9.3 — Insérer

```rust
let mut scores = HashMap::new();
let ancien = scores.insert("Alice", 10);   // None : nouvelle clé
let ancien = scores.insert("Alice", 15);   // Some(10) : la valeur est remplacée
```

`insert` **remplace** toujours. Pour n'insérer que si la clé est absente : `entry(...).or_insert(...)` (section 11).

### 9.4 — L'ordre n'est pas garanti

L'ordre de parcours d'une `HashMap` est **aléatoire** et peut changer d'une exécution à l'autre (le hachage est volontairement imprévisible, ce qui protège contre les attaques par collisions). Pour un ordre stable : `BTreeMap` (trié) ou trier les clés avant d'afficher.

## 10. `HashMap<K, V>` : lire et modifier

### 10.1 — Lire

```rust
let mut scores = HashMap::from([("Alice".to_string(), 10)]);

let s = scores.get("Alice");                 // Some(&10) — une &str suffit pour une clé String
let s = scores.get("Zoé").copied().unwrap_or(0);
let existe = scores.contains_key("Alice");
let s = scores["Alice"];                     // panique si la clé est absente
```

`get` accepte une forme empruntée de la clé : avec des clés `String`, on cherche avec une `&str`, sans créer de `String`.

### 10.2 — Modifier

```rust
if let Some(s) = scores.get_mut("Alice") {
    *s += 5;
}

let retire = scores.remove("Alice");                 // Some(15)
let paire = scores.remove_entry("Bob");              // Some(("Bob", v)) ou None
scores.retain(|_nom, score| *score > 5);             // garde selon une condition
```

### 10.3 — Parcourir

```rust
let scores = HashMap::from([("Alice", 10), ("Bob", 7)]);

for (nom, score) in &scores { }          // (&K, &V)
for nom in scores.keys() { }
for score in scores.values() { }

let mut scores = scores;
for score in scores.values_mut() {
    *score *= 2;
}
for (nom, score) in scores { }           // consomme la map

// Dans un ordre stable
let mut paires: Vec<_> = HashMap::from([("b", 2), ("a", 1)]).into_iter().collect();
paires.sort();
```

## 11. L'API `entry`

`entry(cle)` donne accès à l'emplacement d'une clé, **présente ou non**, en une seule recherche.

### 11.1 — Compter : quatre écritures

```rust
use std::collections::HashMap;

let texte = "le chat et le chien";

// 1. get puis insert : deux recherches, plus long
let mut c1: HashMap<&str, u32> = HashMap::new();
for mot in texte.split_whitespace() {
    let n = c1.get(mot).copied().unwrap_or(0);
    c1.insert(mot, n + 1);
}

// 2. match sur get_mut
let mut c2: HashMap<&str, u32> = HashMap::new();
for mot in texte.split_whitespace() {
    match c2.get_mut(mot) {
        Some(n) => *n += 1,
        None => { c2.insert(mot, 1); }
    }
}

// 3. entry + or_insert : une seule recherche (forme idiomatique)
let mut c3: HashMap<&str, u32> = HashMap::new();
for mot in texte.split_whitespace() {
    *c3.entry(mot).or_insert(0) += 1;
}

// 4. entry + or_default
let mut c4: HashMap<&str, u32> = HashMap::new();
for mot in texte.split_whitespace() {
    *c4.entry(mot).or_default() += 1;
}
```

### 11.2 — Toutes les méthodes d'`entry`

```rust
let mut m: HashMap<&str, Vec<u32>> = HashMap::new();

m.entry("a").or_insert(Vec::new());          // valeur créée même si la clé existe
m.entry("b").or_insert_with(Vec::new);       // créée seulement si absente
m.entry("c").or_default();                   // Default du type
m.entry("a").or_default().push(1);           // regrouper : renvoie &mut V

m.entry("a")
    .and_modify(|v| v.push(2))               // si présente : modifier
    .or_insert_with(|| vec![2]);             // si absente : insérer

let cle = "d";
m.entry(cle).or_insert_with_key(|k| vec![k.len() as u32]);   // la valeur dépend de la clé
```

### 11.3 — Regrouper

```rust
let mots = ["arbre", "abri", "bateau", "bus", "car"];
let mut par_initiale: HashMap<char, Vec<&str>> = HashMap::new();
for mot in mots {
    let initiale = mot.chars().next().unwrap();
    par_initiale.entry(initiale).or_default().push(mot);
}
// {'a': ["arbre", "abri"], 'b': ["bateau", "bus"], 'c': ["car"]}
```

### 11.4 — Matcher l'entrée

```rust
use std::collections::hash_map::Entry;

let mut stock: HashMap<&str, u32> = HashMap::new();
match stock.entry("pomme") {
    Entry::Occupied(mut e) => { *e.get_mut() += 1; }
    Entry::Vacant(e) => { e.insert(1); }
}
```

À quoi ça sert : quand les deux cas demandent un traitement différent et long, ou pour retirer l'entrée (`e.remove()`) selon sa valeur.

## 12. `BTreeMap<K, V>`

Même API qu'une `HashMap`, mais les clés restent **triées**. La clé doit implémenter `Ord` (et non `Hash`).

```rust
use std::collections::BTreeMap;

let mut notes = BTreeMap::new();
notes.insert("Charlie", 12);
notes.insert("Alice", 15);
notes.insert("Bob", 9);

for (nom, note) in &notes {
    println!("{nom} : {note}");          // Alice, Bob, Charlie : toujours dans l'ordre
}

notes.first_key_value();                 // Some((&"Alice", &15))
notes.last_key_value();                  // Some((&"Charlie", &12))

let mut evenements = BTreeMap::new();
evenements.insert(1990, "a");
evenements.insert(2005, "b");
evenements.insert(2020, "c");
for (annee, e) in evenements.range(2000..2021) {
    println!("{annee} {e}");             // 2005, 2020
}
```

| | `HashMap` | `BTreeMap` |
|---|---|---|
| Recherche, insertion | très rapide (temps constant en moyenne) | rapide (logarithmique) |
| Parcours | ordre aléatoire | trié |
| Plages (`range`), min, max | non | oui |
| Clé | `Eq + Hash` | `Ord` |

Choisis `BTreeMap` quand l'ordre compte (affichage, sortie stable pour des tests, export) ou pour des requêtes par plage.

## 13. `HashSet<T>` et `BTreeSet<T>`

Un ensemble : des valeurs **uniques**, sans valeur associée.

### 13.1 — Créer et ajouter

```rust
use std::collections::HashSet;

let mut s = HashSet::new();
let nouveau = s.insert("rust");      // true
let nouveau = s.insert("rust");      // false : déjà présent, rien ne change
s.remove("rust");

let s2: HashSet<i32> = [1, 2, 2, 3].into_iter().collect();   // {1, 2, 3}
let s3 = HashSet::from([1, 2, 3]);
```

`insert` renvoie `false` pour un doublon : pratique pour détecter les doublons au passage.

### 13.2 — Appartenance : `Vec` ou `HashSet`

```rust
let interdits_vec = vec!["admin", "root", "system"];
let interdits_set: HashSet<&str> = interdits_vec.iter().copied().collect();

let nom = "root";
interdits_vec.contains(&nom);     // parcourt tout le Vec
interdits_set.contains(nom);      // une recherche par hachage
```

Pour quelques éléments, la différence est nulle. Pour des milliers de vérifications sur des milliers d'éléments, le `HashSet` est incomparablement plus rapide.

### 13.3 — Opérations d'ensemble

```rust
let a: HashSet<i32> = [1, 2, 3, 4].into();
let b: HashSet<i32> = [3, 4, 5].into();

let union: HashSet<_> = a.union(&b).copied().collect();                 // {1, 2, 3, 4, 5}
let inter: HashSet<_> = a.intersection(&b).copied().collect();          // {3, 4}
let diff: HashSet<_> = a.difference(&b).copied().collect();             // {1, 2}
let sym: HashSet<_> = a.symmetric_difference(&b).copied().collect();    // {1, 2, 5}

let inter2 = &a & &b;      // mêmes opérations avec des opérateurs
let union2 = &a | &b;
let diff2 = &a - &b;

a.is_subset(&b);           // false
a.is_disjoint(&b);         // false
```

### 13.4 — `BTreeSet`

```rust
use std::collections::BTreeSet;

let s: BTreeSet<i32> = [5, 1, 3].into();
for x in &s { }            // 1, 3, 5
s.first();                 // Some(&1)
s.range(2..).next();       // Some(&3) : plus petite valeur >= 2
```

## 14. `BinaryHeap<T>`

Une file de priorité : `pop` renvoie toujours **le plus grand** élément.

```rust
use std::collections::BinaryHeap;
use std::cmp::Reverse;

let mut tas = BinaryHeap::new();
tas.push(3);
tas.push(10);
tas.push(1);
tas.pop();             // Some(10)
tas.peek();            // Some(&3)

// Le plus PETIT d'abord : envelopper dans Reverse
let mut min_tas = BinaryHeap::new();
min_tas.push(Reverse(3));
min_tas.push(Reverse(1));
let Some(Reverse(plus_petit)) = min_tas.pop() else { return };   // 1

// Priorité + données : le tuple est comparé élément par élément
let mut taches = BinaryHeap::new();
taches.push((2, "moyenne"));
taches.push((9, "urgente"));
taches.pop();          // Some((9, "urgente"))
```

À quoi ça sert : traiter les tâches par priorité, garder les N plus grandes valeurs d'un flux, algorithme de Dijkstra.

## 15. Ownership et collections

### 15.1 — Insérer déplace la valeur

```rust
let nom = String::from("Alice");
let mut noms = Vec::new();
noms.push(nom);
// println!("{nom}");    // ERREUR : nom a été déplacée dans le Vec

let cle = String::from("k");
let mut m = std::collections::HashMap::new();
m.insert(cle, 1);
// println!("{cle}");    // ERREUR aussi
```

Trois solutions selon le besoin :

```rust
let nom = String::from("Alice");
let mut a = Vec::new();
a.push(nom.clone());      // 1. copier : nom reste utilisable

let mut b: Vec<&str> = Vec::new();
b.push(&nom);             // 2. stocker une référence : b ne doit pas vivre plus longtemps que nom

let mut c = Vec::new();
c.push(nom);              // 3. déplacer, et accéder à la valeur par la collection ensuite
println!("{}", c[0]);
```

### 15.2 — Sortir une valeur d'une collection

```rust
let mut v = vec![String::from("a"), String::from("b")];

// let s = v[0];                    // ERREUR : cannot move out of index
let s = &v[0];                      // emprunter
let s = v[0].clone();               // copier
let s = v.remove(0);                // retirer (et décaler)
let s = std::mem::take(&mut v[0]);  // prendre la valeur, laisser String::new() à la place
let s = std::mem::replace(&mut v[0], String::from("z"));   // échanger
```

## 16. Choisir la bonne collection

| Besoin principal | Collection |
|---|---|
| Liste, ordre d'insertion, accès par position | `Vec` |
| File d'attente, ajout et retrait aux deux bouts | `VecDeque` |
| Retrouver une valeur par une clé | `HashMap` |
| Pareil, avec parcours trié ou plages de clés | `BTreeMap` |
| Savoir si une valeur est présente, dédoublonner | `HashSet` |
| Ensemble trié | `BTreeSet` |
| Toujours prendre le plus prioritaire | `BinaryHeap` |

### Coût des opérations

| Opération | `Vec` | `VecDeque` | `HashMap`/`HashSet` | `BTreeMap`/`BTreeSet` |
|---|---|---|---|---|
| Accès par index | immédiat | immédiat | — | — |
| Ajout à la fin | immédiat (amorti) | immédiat | — | — |
| Ajout au début | proportionnel à n | immédiat | — | — |
| Recherche d'une valeur | proportionnel à n | proportionnel à n | immédiat en moyenne | logarithmique |
| Insertion par clé | — | — | immédiat en moyenne | logarithmique |
| Parcours trié | trier d'abord | trier d'abord | trier d'abord | gratuit |

En cas de doute, commence par un `Vec` : c'est la plus simple et, pour de petites tailles, souvent la plus rapide (mémoire contiguë).

## 17. La même tâche écrite de plusieurs façons

### 17.1 — Statistiques sur un `Vec`

```rust
fn stats_boucle(v: &[f64]) -> Option<(f64, f64, f64)> {
    if v.is_empty() {
        return None;
    }
    let mut min = v[0];
    let mut max = v[0];
    let mut somme = 0.0;
    for &x in v {
        if x < min { min = x; }
        if x > max { max = x; }
        somme += x;
    }
    Some((min, max, somme / v.len() as f64))
}

fn stats_iter(v: &[f64]) -> Option<(f64, f64, f64)> {
    let premier = *v.first()?;
    let min = v.iter().copied().fold(premier, f64::min);
    let max = v.iter().copied().fold(premier, f64::max);
    let moyenne = v.iter().sum::<f64>() / v.len() as f64;
    Some((min, max, moyenne))
}
```

`f64` n'a pas `Ord`, donc pas de `.min()` sur l'itérateur : on passe par `fold` avec `f64::min`.

### 17.2 — Retirer les doublons

```rust
use std::collections::{BTreeSet, HashSet};

let v = vec![3, 1, 3, 2, 1];

// 1. Trier + dedup : ordre trié
let mut a = v.clone();
a.sort();
a.dedup();                                             // [1, 2, 3]

// 2. Passer par un BTreeSet : ordre trié
let b: Vec<i32> = v.iter().copied().collect::<BTreeSet<_>>().into_iter().collect();

// 3. HashSet comme filtre : ordre d'origine conservé
let mut vus = HashSet::new();
let c: Vec<i32> = v.iter().copied().filter(|x| vus.insert(*x)).collect();   // [3, 1, 2]
```

### 17.3 — Le mot le plus fréquent

```rust
use std::collections::HashMap;

fn plus_frequent(texte: &str) -> Option<(&str, u32)> {
    let mut compte: HashMap<&str, u32> = HashMap::new();
    for mot in texte.split_whitespace() {
        *compte.entry(mot).or_default() += 1;
    }
    compte.into_iter().max_by_key(|&(_, n)| n)
}
```

En cas d'égalité, le résultat dépend de l'ordre de la `HashMap`, donc il peut changer d'une exécution à l'autre. Pour un résultat stable, utiliser `BTreeMap` ou départager par le mot : `max_by(|a, b| a.1.cmp(&b.1).then(b.0.cmp(a.0)))`.

## 18. Exercices

### Exercice 1 — statistiques

Écris `moyenne`, `mediane` et `mode` (valeur la plus fréquente) pour un `&[i32]`, en renvoyant `None` sur une liste vide.

```rust
// Solution :
use std::collections::HashMap;

fn moyenne(v: &[i32]) -> Option<f64> {
    if v.is_empty() {
        return None;
    }
    Some(v.iter().sum::<i32>() as f64 / v.len() as f64)
}

fn mediane(v: &[i32]) -> Option<f64> {
    if v.is_empty() {
        return None;
    }
    let mut tri = v.to_vec();
    tri.sort();
    let m = tri.len() / 2;
    Some(if tri.len() % 2 == 0 {
        (tri[m - 1] + tri[m]) as f64 / 2.0
    } else {
        tri[m] as f64
    })
}

fn mode(v: &[i32]) -> Option<i32> {
    let mut compte = HashMap::new();
    for &x in v {
        *compte.entry(x).or_insert(0) += 1;
    }
    compte.into_iter().max_by_key(|&(_, n)| n).map(|(x, _)| x)
}
```

### Exercice 2 — anagrammes

Regroupe des mots par anagrammes (`["chien", "niche", "chat"]` → `[["chien", "niche"], ["chat"]]`).

```rust
// Solution :
use std::collections::HashMap;

fn anagrammes<'a>(mots: &[&'a str]) -> Vec<Vec<&'a str>> {
    let mut groupes: HashMap<Vec<char>, Vec<&str>> = HashMap::new();
    for &mot in mots {
        let mut cle: Vec<char> = mot.chars().collect();
        cle.sort_unstable();
        groupes.entry(cle).or_default().push(mot);
    }
    groupes.into_values().collect()
}
```

### Exercice 3 — intersection de listes

Renvoie les éléments communs à deux `Vec<String>`, sans doublon et triés.

```rust
// Solution :
use std::collections::BTreeSet;

fn communs(a: &[String], b: &[String]) -> Vec<String> {
    let a: BTreeSet<&String> = a.iter().collect();
    let b: BTreeSet<&String> = b.iter().collect();
    a.intersection(&b).map(|s| (*s).clone()).collect()
}
```

## 19. Aide-mémoire

### Vec

```rust
vec![1, 2]   Vec::with_capacity(n)   iter.collect::<Vec<_>>()
v.push(x)   v.pop()   v.insert(i, x)   v.remove(i)   v.swap_remove(i)
v.extend(it)   v.append(&mut w)   v.truncate(n)   v.clear()   v.drain(a..b)
v.retain(|x| ..)   v.dedup()   v.sort()   v.sort_by_key(|x| ..)   v.reverse()
v[i]   v.get(i)   v.first()   v.last()   v.get_mut(i)   v.contains(&x)
v.binary_search(&x)   v.iter().position(|x| ..)   v.windows(2)   v.chunks(n)
```

### HashMap

```rust
HashMap::from([(k, v)])   m.insert(k, v)   m.get(&k)   m.get_mut(&k)
m.remove(&k)   m.contains_key(&k)   m.keys()   m.values()   m.values_mut()
*m.entry(k).or_insert(0) += 1
m.entry(k).or_default().push(x)
m.entry(k).and_modify(|v| ..).or_insert(v)
```

### HashSet

```rust
s.insert(x)  // false si déjà présent
s.contains(&x)   a.union(&b)   a.intersection(&b)   a.difference(&b)
&a | &b   &a & &b   &a - &b
```

### Points clés

- `Vec` est la collection par défaut ; `with_capacity` quand la taille est connue.
- `v[i]` panique, `v.get(i)` renvoie `None` : utilise `get` pour tout index venu de l'extérieur.
- `remove` décale, `swap_remove` non ; retirer au début d'une file : `VecDeque`.
- `HashMap` ne garantit aucun ordre ; `BTreeMap` est triée.
- `entry` fait une seule recherche pour lire et écrire.
- Les clés de `HashMap` doivent être `Eq + Hash` : pas de `f64`.
- Insérer dans une collection déplace la valeur.
- En paramètre, `&[T]` plutôt que `&Vec<T>`.
