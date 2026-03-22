## Les Collections en Rust

## Vec, HashMap, HashSet et Itérateurs 

Guide Complet des Structures de Données 

## Objectifs du cours

À la fin de ce cours, tu sauras : 

- Utiliser Vec<T> (vecteurs dynamiques) 

- Gérer des paires clé-valeur avec HashMap 

- Utiliser HashSet pour des ensembles 

- Maîtriser les itérateurs et leurs méthodes 

- Choisir la bonne collection pour chaque besoin 

## Table des matières

1. Les Vecteurs (Vec) 

- 1.1 - Création et initialisation 

- 1.2 - Ajouter et retirer des éléments 

- 1.3 - Accéder aux éléments 

- 1.4 - Itérer sur un vecteur 

2. Les HashMap 

- 2.1 - Création et insertion 

- 2.2 - Accès et modification 

- 2.3 - Vérifier l'existence 

- 2.4 - Itérer sur une HashMap 

3. Les HashSet 

- 3.1 - Création et ajout 

- 3.2 - Opérations d'ensemble 

- 3.3 - Vérification d'appartenance 

4. Les Itérateurs 

- 4.1 - Méthodes de base 

- 4.2 - Transformations (map, filter) 

- 4.3 - Collecte et consommation 

5. Choisir la bonne collection 

6. Exercices pratiques 

7. Aide-mémoire 

## 1. Les Vecteurs (Vec<T>)

Les **vecteurs** sont des tableaux dynamiques : leur taille peut changer. C'est la collection la plus utilisée en Rust ! 

## 1.1 - Création et initialisation

```
// Vecteur vide
let mut v: Vec<i32> = Vec::new();
// Avec la macro vec!
let v = vec![1, 2, 3, 4, 5];
// Vecteur avec capacité
let mut v: Vec<i32> = Vec::with_capacity(10);
// Vecteur de répétitions
let v = vec![0; 5];  // [0, 0, 0, 0, 0]
// Vecteur de String
let mut v: Vec<String> = Vec::new();
v.push(String::from("Hello"));
v.push(String::from("World"));
```

### Capacity vs Length :

• `len()` : Nombre d'éléments actuels • `capacity()` : Espace alloué en mémoire Utilise `with_capacity()` si tu connais la taille finale pour éviter les réallocations. 

## 1.2 - Ajouter et retirer des éléments

```
let mut v = vec![1, 2, 3];
```

```
// Ajouter à la fin
v.push(4);
println!("{:?}", v);  // [1, 2, 3, 4]
// Retirer le dernier élément
let dernier = v.pop();  // Some(4)
println!("{:?}", v);    // [1, 2, 3]
// Insérer à un index
v.insert(1, 10);
println!("{:?}", v);  // [1, 10, 2, 3]
// Retirer à un index
let element = v.remove(1);  // 10
println!("{:?}", v);        // [1, 2, 3]
// Vider le vecteur
v.clear();
println!("{:?}", v);  // []
// Ajouter plusieurs éléments
let mut v = vec![1, 2, 3];
v.extend(vec![4, 5, 6]);
println!("{:?}", v);  // [1, 2, 3, 4, 5, 6]
```

II **Attention !** `remove()` et `insert()` sont coûteux car ils déplacent tous les éléments après l'index. Préfère `push()` et `pop()` quand c'est possible. 

## 1.3 - Accéder aux éléments

```
let v = vec![1, 2, 3, 4, 5];
```

```
// Accès par index (panic si hors limites)
let troisieme = v[2];
println!("Troisième : {}", troisieme);  // 3
```

```
// Accès sécurisé avec get()
match v.get(2) {
    Some(valeur) => println!("Troisième : {}", valeur),
    None => println!("Pas d'élément à cet index"),
}
```

```
// Premier et dernier élément
let premier = v.first();   // Some(&1)
let dernier = v.last();    // Some(&5)
// Slice (portion du vecteur)
let slice = &v[1..3];  // [2, 3]
```

```
// Vérifier si vide
if v.is_empty() {
    println!("Vecteur vide");
} else {
    println!("Longueur : {}", v.len());
}
```

## 1.4 - Itérer sur un vecteur

```
let v = vec![10, 20, 30];
// Itération immutable
for element in &v {
    println!("{}", element);
}
// Itération mutable
let mut v = vec![10, 20, 30];
for element in &mut v {
    *element += 10;
}
println!("{:?}", v);  // [20, 30, 40]
// Itération avec index
for (index, valeur) in v.iter().enumerate() {
    println!("v[{}] = {}", index, valeur);
}
// Consommer le vecteur
for element in v {
    println!("{}", element);
}
// v n'est plus accessible ici
```

## 2. Les HashMap<K, V>

Les **HashMap** stockent des paires clé-valeur. C'est l'équivalent des dictionnaires en Python ou des objets en JavaScript. 

## 2.1 - Création et insertion

```
use std::collections::HashMap;
// Création vide
let mut scores: HashMap<String, i32> = HashMap::new();
// Insérer des paires clé-valeur
scores.insert(String::from("Bleu"), 10);
scores.insert(String::from("Rouge"), 50);
// Créer depuis deux vecteurs
let equipes = vec![String::from("Bleu"), String::from("Rouge")];
let scores_initiaux = vec![10, 50];
let scores: HashMap<_, _> = equipes
    .iter()
    .zip(scores_initiaux.iter())
    .collect();
// HashMap avec macro (nécessite une crate externe)
// use maplit::hashmap;
// let scores = hashmap!{
//     "Bleu" => 10,
//     "Rouge" => 50,
// };
```

I **Ownership :** Quand tu insères une valeur dans une HashMap, elle prend ownership des valeurs qui n'implémentent pas `Copy` . Pour `i32` , c'est copié. Pour `String` , c'est déplacé. 

## 2.2 - Accès et modification

```
use std::collections::HashMap;
```

```
let mut scores = HashMap::new();
scores.insert(String::from("Bleu"), 10);
scores.insert(String::from("Rouge"), 50);
```

```
// Accéder à une valeur
let equipe = String::from("Bleu");
let score = scores.get(&equipe);  // Some(&10)
```

```
match score {
    Some(s) => println!("Score : {}", s),
    None => println!("Équipe inconnue"),
}
```

```
// Avec unwrap_or
let score = scores.get("Vert").unwrap_or(&0);
println!("Score : {}", score);  // 0
// Modifier une valeur existante
scores.insert(String::from("Bleu"), 25);  // Écrase l'ancienne
// Insérer seulement si la clé n'existe pas
scores.entry(String::from("Jaune")).or_insert(50);
scores.entry(String::from("Bleu")).or_insert(100);  // Pas d'effet
// Mettre à jour basé sur l'ancienne valeur
let texte = "hello world wonderful world";
let mut map = HashMap::new();
```

```
for mot in texte.split_whitespace() {
    let count = map.entry(mot).or_insert(0);
    *count += 1;
```

```
}
println!("{:?}", map);  // {"hello": 1, "world": 2, "wonderful": 1}
```

## 2.3 - Vérifier l'existence

```
let mut scores = HashMap::new();
scores.insert("Bleu", 10);
```

```
// Vérifier si une clé existe
if scores.contains_key("Bleu") {
    println!("L'équipe Bleu existe");
}
```

```
// Nombre de paires
println!("Nombre d'équipes : {}", scores.len());
```

```
// Retirer une paire
let score = scores.remove("Bleu");  // Some(10)
```

```
// Vider la HashMap
scores.clear();
```

## 2.4 - Itérer sur une HashMap

```
let mut scores = HashMap::new();
scores.insert(String::from("Bleu"), 10);
scores.insert(String::from("Rouge"), 50);
```

```
// Itérer sur les paires clé-valeur
for (cle, valeur) in &scores {
    println!("{}: {}", cle, valeur);
}
// Itérer seulement sur les clés
for cle in scores.keys() {
    println!("{}", cle);
}
// Itérer seulement sur les valeurs
for valeur in scores.values() {
    println!("{}", valeur);
}
// Modifier les valeurs en itérant
for valeur in scores.values_mut() {
    *valeur += 10;
}
```

II **Ordre non garanti !** Les HashMap ne maintiennent pas d'ordre. Si tu as besoin d'ordre, utilise `BTreeMap` ou garde une liste séparée des clés. 

## 3. Les HashSet<T>

Les **HashSet** sont des ensembles : collections d'éléments **uniques** sans ordre particulier. 

## 3.1 - Création et ajout

```
use std::collections::HashSet;
```

```
// Création vide
let mut nombres: HashSet<i32> = HashSet::new();
```

```
// Ajouter des éléments
nombres.insert(1);
nombres.insert(2);
nombres.insert(3);
nombres.insert(2);  // Ignoré (déjà présent)
println!("{:?}", nombres);  // {1, 2, 3}
```

```
// Créer depuis un vecteur
let v = vec![1, 2, 3, 2, 1];
let set: HashSet<i32> = v.into_iter().collect();
println!("{:?}", set);  // {1, 2, 3}
```

```
// Retirer des éléments
nombres.remove(&2);
println!("{:?}", nombres);  // {1, 3}
```

## 3.2 - Opérations d'ensemble

```
use std::collections::HashSet;
```

```
let set1: HashSet<i32> = [1, 2, 3, 4].iter().cloned().collect();
let set2: HashSet<i32> = [3, 4, 5, 6].iter().cloned().collect();
```

```
// Union (tous les éléments)
let union: HashSet<_> = set1.union(&set2).collect();
println!("Union : {:?}", union);  // {1, 2, 3, 4, 5, 6}
```

```
// Intersection (éléments communs)
let inter: HashSet<_> = set1.intersection(&set2).collect();
println!("Intersection : {:?}", inter);  // {3, 4}
```

```
// Différence (dans set1 mais pas set2)
let diff: HashSet<_> = set1.difference(&set2).collect();
println!("Différence : {:?}", diff);  // {1, 2}
```

```
// Différence symétrique (dans l'un ou l'autre mais pas les deux)
let sym_diff: HashSet<_> = set1.symmetric_difference(&set2).collect();
println!("Sym diff : {:?}", sym_diff);  // {1, 2, 5, 6}
```

## 3.3 - Vérification d'appartenance

```
let set: HashSet<i32> = [1, 2, 3].iter().cloned().collect();
```

```
// Vérifier si un élément existe
if set.contains(&2) {
    println!("2 est dans l'ensemble");
}
```

```
// Sous-ensemble et super-ensemble
let set1: HashSet<i32> = [1, 2].iter().cloned().collect();
let set2: HashSet<i32> = [1, 2, 3].iter().cloned().collect();
```

```
println!("{}", set1.is_subset(&set2));     // true
println!("{}", set2.is_superset(&set1));   // true
```

```
// Disjoints (aucun élément en commun)
let set3: HashSet<i32> = [4, 5].iter().cloned().collect();
println!("{}", set1.is_disjoint(&set3));   // true
```

## 4. Les Itérateurs

Les **itérateurs** permettent de traiter des séquences d'éléments de manière paresseuse et efficace. C'est une fonctionnalité très puissante de Rust ! 

## 4.1 - Méthodes de base

```
let v = vec![1, 2, 3, 4, 5];
```

```
// iter() - référence immutable
for val in v.iter() {
    println!("{}", val);  // &i32
}
```

```
// iter_mut() - référence mutable
let mut v = vec![1, 2, 3];
for val in v.iter_mut() {
    *val += 10;
}
```

```
// into_iter() - prend ownership
for val in v.into_iter() {
    println!("{}", val);  // i32
```

```
}
// v n'est plus accessible
```

```
// next() - obtenir le prochain élément
let v = vec![1, 2, 3];
let mut iter = v.iter();
```

```
println!("{:?}", iter.next());  // Some(&1)
println!("{:?}", iter.next());  // Some(&2)
println!("{:?}", iter.next());  // Some(&3)
println!("{:?}", iter.next());  // None
```

## 4.2 - Transformations (map, filter)

```
let v = vec![1, 2, 3, 4, 5];
```

```
// map - transformer chaque élément
let doubles: Vec<i32> = v.iter()
    .map(|x| x * 2)
    .collect();
println!("{:?}", doubles);  // [2, 4, 6, 8, 10]
```

```
// filter - garder seulement certains éléments
let pairs: Vec<i32> = v.iter()
    .filter(|x| *x % 2 == 0)
    .cloned()
    .collect();
println!("{:?}", pairs);  // [2, 4]
```

```
// Chaînage de méthodes
let resultat: Vec<i32> = v.iter()
    .filter(|x| *x % 2 == 0)  // Garder pairs
    .map(|x| x * 2)            // Doubler
    .collect();
println!("{:?}", resultat);    // [4, 8]
```

```
// find - trouver le premier élément
let trouve = v.iter().find(|&&x| x > 3);
println!("{:?}", trouve);  // Some(&4)
```

```
// any / all - tester des conditions
println!("{}", v.iter().any(|&x| x > 4));  // true
println!("{}", v.iter().all(|&x| x > 0));  // true
```

```
// take / skip - prendre/sauter des éléments
let premiers: Vec<i32> = v.iter()
    .take(3)
    .cloned()
    .collect();
println!("{:?}", premiers);  // [1, 2, 3]
```

## 4.3 - Collecte et consommation

```
let v = vec![1, 2, 3, 4, 5];
```

```
// collect() - transformer en collection
let doubles: Vec<i32> = v.iter().map(|x| x * 2).collect();
// sum() - additionner tous les éléments
let somme: i32 = v.iter().sum();
println!("Somme : {}", somme);  // 15
```

```
// product() - multiplier tous les éléments
let produit: i32 = v.iter().product();
println!("Produit : {}", produit);  // 120
```

```
// max() / min() - trouver max/min
let max = v.iter().max();
println!("Max : {:?}", max);  // Some(&5)
```

```
// count() - compter les éléments
let compte = v.iter().count();
println!("Compte : {}", compte);  // 5
```

```
// fold() - réduction personnalisée
let somme = v.iter().fold(0, |acc, x| acc + x);
println!("Somme avec fold : {}", somme);  // 15
```

```
// enumerate() - avec index
for (i, val) in v.iter().enumerate() {
    println!("v[{}] = {}", i, val);
}
```

```
// zip() - combiner deux itérateurs
let noms = vec!["Alice", "Bob", "Charlie"];
let ages = vec![25, 30, 35];
```

```
for (nom, age) in noms.iter().zip(ages.iter()) {
    println!("{} a {} ans", nom, age);
}
```

I **Itérateurs paresseux :** Les itérateurs ne font rien tant que tu n'appelles pas une méthode de consommation comme `collect()` , `sum()` , ou une boucle `for` . 

## 5. Choisir la bonne collection

|**Collection**|**Quand utiliser**|**Complexité**|
|---|---|---|
|Vec<T>|Liste ordonnée, accès par index|O(1) accès, O(n) insert|
|HashMap<K,V>|Paires clé-valeur, recherche rapide|O(1) moyen|
|HashSet<T>|Ensemble unique, appartenance|O(1) moyen|
|BTreeMap<K,V>|Clés ordonnées|O(log n)|
|BTreeSet<T>|Ensemble ordonné|O(log n)|
|VecDeque<T>|File FIFO/LIFO|O(1) aux extrémités|

## Guide de décision :

- **Besoin d'ordre ?** → Vec ou VecDeque 

- **Recherche par clé ?** → HashMap ou BTreeMap 

- **Éléments uniques ?** → HashSet ou BTreeSet 

- **Accès fréquent par index ?** → Vec 

- **Insertion/suppression au début ?** → VecDeque 

- **Itération dans l'ordre des clés ?** → BTreeMap/BTreeSet 

## 6. Exercices pratiques

## Exercice 1 : Statistiques sur un Vec

```
// Écris une fonction qui prend un Vec<i32> et retourne :
// - La moyenne
```

```
// - Le minimum
// - Le maximum
```

```
// Solution :
fn statistiques(v: &Vec<i32>) -> (f64, i32, i32) {
    let somme: i32 = v.iter().sum();
    let moyenne = somme as f64 / v.len() as f64;
    let min = *v.iter().min().unwrap();
    let max = *v.iter().max().unwrap();
```

```
    (moyenne, min, max)
```

```
}
```

```
fn main() {
```

```
    let nombres = vec![1, 5, 3, 9, 2];
    let (moy, min, max) = statistiques(&nombres);
    println!("Moy: {}, Min: {}, Max: {}", moy, min, max);
}
```

## Exercice 2 : Compter les occurrences

```
// Écris une fonction qui compte les occurrences de chaque mot
// dans une phrase
```

```
// Solution :
use std::collections::HashMap;
```

```
fn compter_mots(texte: &str) -> HashMap<String, u32> {
    let mut compteur = HashMap::new();
```

```
    for mot in texte.split_whitespace() {
        let count = compteur.entry(mot.to_string()).or_insert(0);
        *count += 1;
    }
```

```
    compteur
}
```

```
fn main() {
    let texte = "hello world hello rust world";
    let compte = compter_mots(texte);
    for (mot, freq) in &compte {
        println!("{}: {}", mot, freq);
    }
```

```
}
```

## Exercice 3 : Supprimer les doublons

```
// Écris une fonction qui enlève les doublons d'un Vec
```

```
// Solution :
use std::collections::HashSet;
```

```
fn sans_doublons(v: Vec<i32>) -> Vec<i32> {
    let set: HashSet<i32> = v.into_iter().collect();
    set.into_iter().collect()
}
```

```
fn main() {
    let nombres = vec![1, 2, 2, 3, 1, 4, 5, 3];
    let uniques = sans_doublons(nombres);
    println!("{:?}", uniques);  // Ordre non garanti !
}
```

## 7. Aide-mémoire

## Vec<T>

|**Méthode**|**Description**|**Exemple**|
|---|---|---|
|**`push(val)`**|Ajouter à la fin|v.push(5)|
|**`pop()`**|Retirer de la fin|v.pop()|
|**`len()`**|Nombre d'éléments|v.len()|
|**`is_empty()`**|Vérifier si vide|v.is_empty()|
|**`get(i)`**|Accès sécurisé|v.get(2)|
|**`clear()`**|Vider|v.clear()|

## HashMap<K, V>

|**Méthode**|**Description**|**Exemple**|
|---|---|---|
|**`insert(k, v)`**|Ajouter/modifier|map.insert("a", 1)|
|**`get(k)`**|Obtenir valeur|map.get("a")|
|**`remove(k)`**|Supprimer|map.remove("a")|
|**`contains_key(k)`**|Vérifier clé|map.contains_key("a")|
|**`entry(k).or_insert(v)`**|Insert si absent|map.entry("a").or_insert(0)|

## Itérateurs

|**Méthode**|**Description**|**Type retour**|
|---|---|---|
|**`map(f)`**|Transformer|Iterator|
|**`filter(f)`**|Filtrer|Iterator|
|**`collect()`**|Collecter|Collection|
|**`sum()`**|Additionner|Nombre|
|**`any(f)`**|Tester si au moins un|bool|
|**`all(f)`**|Tester si tous|bool|
|**`find(f)`**|Trouver premier|Option<T>|

### Bravo !

Tu maîtrises maintenant les collections en Rust ! 

## Prochaines étapes : 

• Pratiquer avec des projets réels • Explorer les lifetimes • Apprendre les traits avancés • Découvrir la programmation asynchrone 

I **Tu es maintenant un vrai Rustacean !** I
