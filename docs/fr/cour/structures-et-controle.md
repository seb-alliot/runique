## ■ Structures de Contrôle en Rust

## Conditions, Boucles et Pattern Matching

Maîtriser le Flux de Contrôle 

## ■ Objectifs du cours

À la fin de ce cours, tu sauras : 

- Utiliser if, else if et else 

- Maîtriser les boucles (loop, while, for) 

- Contrôler le flux avec break et continue 

- Utiliser match pour le pattern matching 

- Appliquer if let et while let 

## ■ Table des matières

## 1. Les conditions (if/else)

- 1.1 - if basique 

1.2 - else et else if 

1.3 - if comme expression 

## 2. Les boucles

- 2.1 - loop (boucle infinie) 

- 2.2 - while (avec condition) 

- 2.3 - for (itération) 

- 2.4 - break et continue 

2.5 - Labels de boucles 

## 3. Pattern matching (match)

- 3.1 - match basique 

- 3.2 - Patterns avancés 

3.3 - Guards (conditions) 

**4. if let et while let** 

**5. Exemples pratiques** 

**6. Exercices** 

**7. Aide-mémoire** 

## 1. Les conditions (if/else)

Les conditions permettent d'exécuter du code selon qu'une expression est vraie ou fausse. En Rust, la condition doit toujours être un booléen (bool). 

## 1.1 - if basique

```
// Condition simple
fn main() {
    let nombre = 7;
    if nombre < 10 {
        println!("Le nombre est petit");
    }
}
```

```
// ■■ Important : La condition DOIT être un bool
let x = 5;
// if x { } // ■ ERREUR ! x n'est pas un bool
if x != 0 { } // ■ OK !
```

## ■ Différence avec d'autres langages : En Rust, if n'accepte que des booléens. Pas de conversion implicite comme if (nombre) en JavaScript !

## 1.2 - else et else if

```
fn main() {
    let nombre = 7;
    // if-else
    if nombre % 2 == 0 {
        println!("Pair");
    } else {
        println!("Impair");
    }
```

```
    // if-else if-else
    if nombre < 0 {
        println!("Négatif");
    } else if nombre == 0 {
        println!("Zéro");
    } else {
        println!("Positif");
    }
```

```
}
```

## 1.3 - if comme expression

En Rust, if est une expression : il retourne une valeur ! 

```
// if retourne une valeur
fn main() {
    let condition = true;
```

```
    let nombre = if condition { 5 } else { 6 };
    println!("nombre = {}", nombre); // 5
```

```
    // Exemple pratique
    let age = 20;
    let statut = if age >= 18 { "Majeur" } else { "Mineur" };
    println!("Statut : {}", statut);
}
```

```
// ■■ Les types doivent correspondre !
let x = if condition {
    5 // Type: i32
} else {
    // "six" // ■ ERREUR : types incompatibles
    6 // ■ OK : même type
};
```

**■ if comme expression : Les deux branches (if et else) doivent retourner le même type. Si une branche n'a pas de valeur de retour, elle retourne () (unit type).** 

## 2. Les boucles

Rust propose trois types de boucles : loop (infinie), while (avec condition), et for (itération). 

## 2.1 - loop (boucle infinie)

loop crée une boucle infinie. On doit utiliser break pour en sortir. 

```
// Boucle infinie simple
fn main() {
    let mut compteur = 0;
    loop {
        compteur += 1;
        println!("Compteur : {}", compteur);
        if compteur == 5 {
            break; // Sort de la boucle
        }
    }
}
// loop peut retourner une valeur !
fn main() {
    let mut compteur = 0;
    let resultat = loop {
        compteur += 1;
        if compteur == 10 {
            break compteur * 2; // Retourne 20
        }
    };
    println!("Résultat : {}", resultat); // 20
}
```

## 2.2 - while (avec condition)

while exécute une boucle tant qu'une condition est vraie. 

```
fn main() {
    let mut nombre = 3;
```

```
    // Compte à rebours
```

```
    while nombre != 0 {
        println!("{}!", nombre);
        nombre -= 1;
    }
    println!("Décollage !");
}
```

```
// Exemple pratique : saisie utilisateur
use std::io;
fn main() {
    let mut input = String::new();
    let mut essais = 3;
    while essais > 0 {
        println!("Entrez le mot de passe ({} essais restants):",
essais);
        input.clear();
        io::stdin().read_line(&mut input).expect("Erreur de lecture");
        if input.trim() == "secret" {
            println!("Accès autorisé !");
            break;
        }
        essais -= 1;
    }
}
```

## 2.3 - for (itération)

for permet d'itérer sur une collection ou une plage de valeurs. C'est la boucle la plus utilisée en Rust ! 

```
// Itérer sur une plage
fn main() {
    for i in 1..6 {
        println!("i = {}", i); // 1, 2, 3, 4, 5
    }
    // Plage inclusive (avec =)
    for i in 1..=5 {
        println!("i = {}", i); // 1, 2, 3, 4, 5
    }
}
// Itérer sur un tableau
fn main() {
    let nombres = [10, 20, 30, 40, 50];
    for nombre in nombres {
        println!("Nombre : {}", nombre);
    }
    // Avec index
    for (index, valeur) in nombres.iter().enumerate() {
        println!("Index {} : {}", index, valeur);
    }
}
// Itérer sur un vecteur
fn main() {
    let fruits = vec!["pomme", "banane", "orange"];
    for fruit in &fruits {
        println!("Fruit : {}", fruit);
    }
    // fruits est toujours utilisable ici
    println!("Nombre de fruits : {}", fruits.len());
```

```
}
```

**■ for vs while : Préfère toujours for quand tu itères sur une collection. C'est plus sûr (pas de risque d'index hors limite) et plus idiomatique en Rust !** 

## 2.4 - break et continue

break sort de la boucle, continue passe à l'itération suivante. 

```
// break : sortir de la boucle
fn main() {
    for i in 1..10 {
        if i == 5 {
            break; // Sort complètement de la boucle
        }
        println!("{}", i); // Affiche 1, 2, 3, 4
    }
}
```

```
// continue : passer à l'itération suivante
fn main() {
    for i in 1..10 {
        if i % 2 == 0 {
            continue; // Saute les nombres pairs
        }
        println!("{}", i); // Affiche 1, 3, 5, 7, 9
    }
}
```

```
// Exemple pratique : trouver un élément
fn main() {
    let nombres = vec![1, 5, 8, 12, 15, 20];
    let recherche = 12;
    let mut trouve = false;
    for nombre in nombres {
        if nombre == recherche {
            println!("Trouvé : {}", nombre);
            trouve = true;
            break; // Pas besoin de continuer
        }
    }
    if !trouve {
        println!("Non trouvé");
    }
```

```
}
```

## 2.5 - Labels de boucles

Les labels permettent de contrôler des boucles imbriquées. `// Boucles imbriquées avec labels fn main() { let mut compteur = 0; 'exterieur: loop { println!("Compteur extérieur = {}", compteur); let mut restant = 10; loop { println!("  Restant = {}", restant); if restant == 7 { break; // Sort de la boucle intérieure } if compteur == 2 { break 'exterieur; // Sort des DEUX boucles } restant -= 1; } compteur += 1; } }` 

**■ Labels : Les labels commencent par ' (apostrophe) et permettent de break ou continue une boucle spécifique dans des boucles imbriquées.** 

## 3. Pattern matching (match)

match est l'outil le plus puissant de Rust pour le contrôle de flux. Il vérifie tous les cas possibles et le compilateur garantit que tu n'en oublies aucun ! 

## 3.1 - match basique

```
// Match simple
fn main() {
    let nombre = 3;
    match nombre {
        1 => println!("Un"),
        2 => println!("Deux"),
        3 => println!("Trois"),
        _ => println!("Autre"), // _ = tous les autres cas
    }
}
// match retourne une valeur
fn main() {
    let nombre = 2;
    let resultat = match nombre {
        1 => "premier",
        2 => "deuxième",
        3 => "troisième",
        _ => "autre",
    };
    println!("C'est le {} nombre", resultat);
}
// Match avec Option
fn diviser(a: i32, b: i32) -> Option<i32> {
    if b == 0 {
        None
    } else {
        Some(a / b)
    }
}
```

```
fn main() {
    let resultat = diviser(10, 2);
    match resultat {
        Some(valeur) => println!("Résultat : {}", valeur),
        None => println!("Division par zéro !"),
    }
}
```

**■■ Match exhaustif : Le compilateur vérifie que tu couvres TOUS les cas possibles. Utilise _ pour capturer tous les cas restants.** 

## 3.2 - Patterns avancés

```
// Plusieurs valeurs (OR)
fn main() {
    let nombre = 5;
    match nombre {
        1 | 3 | 5 | 7 | 9 => println!("Impair"),
        2 | 4 | 6 | 8 | 10 => println!("Pair"),
        _ => println!("Hors plage"),
    }
}
// Plages de valeurs
fn main() {
    let nombre = 42;
    match nombre {
        0 => println!("Zéro"),
        1..=10 => println!("Petit"),
        11..=50 => println!("Moyen"),
        51..=100 => println!("Grand"),
        _ => println!("Très grand"),
    }
}
// Déstructuration de tuples
fn main() {
    let point = (0, 5);
    match point {
        (0, 0) => println!("Origine"),
        (0, y) => println!("Sur l'axe Y : {}", y),
        (x, 0) => println!("Sur l'axe X : {}", x),
        (x, y) => println!("Point ({}, {})", x, y),
    }
}
```

## 3.3 - Guards (conditions)

Les guards ajoutent une condition supplémentaire après un pattern. 

```
fn main() {
```

```
    let nombre = 4;
```

```
    match nombre {
        n if n < 0 => println!("Négatif : {}", n),
        n if n % 2 == 0 => println!("Pair : {}", n),
        n => println!("Impair positif : {}", n),
    }
}
```

```
// Exemple avec Option
fn main() {
    let nombre: Option<i32> = Some(7);
```

```
    match nombre {
        Some(n) if n < 5 => println!("Petit : {}", n),
        Some(n) if n >= 5 && n < 10 => println!("Moyen : {}", n),
        Some(n) => println!("Grand : {}", n),
        None => println!("Pas de valeur"),
    }
```

```
}
```

## 4. if let et while let

if let et while let sont des raccourcis pour match quand tu t'intéresses à un seul cas. 

```
// Sans if let (verbeux)
fn main() {
    let nombre: Option<i32> = Some(5);
```

```
    match nombre {
        Some(n) => println!("Valeur : {}", n),
        _ => (), // Ignore None
    }
}
// Avec if let (concis)
fn main() {
    let nombre: Option<i32> = Some(5);
    if let Some(n) = nombre {
        println!("Valeur : {}", n);
    }
}
// if let avec else
fn main() {
    let nombre: Option<i32> = None;
    if let Some(n) = nombre {
        println!("Valeur : {}", n);
    } else {
        println!("Pas de valeur");
    }
}
// while let : boucle tant que pattern match
fn main() {
    let mut pile = vec![1, 2, 3, 4, 5];
    // Vide la pile
    while let Some(sommet) = pile.pop() {
        println!("Sommet : {}", sommet); // 5, 4, 3, 2, 1
```

```
    }
```

```
}
```

**■ Quand utiliser if let ? Quand tu t'intéresses à un seul cas et que tu veux ignorer les autres. Plus lisible que match avec _ quand approprié.** 

## 5. Exemples pratiques

**Exemple 1 : Calculatrice simple** 

```
fn calculer(operateur: char, a: i32, b: i32) -> Option<i32> {
    match operateur {
        '+' => Some(a + b),
        '-' => Some(a - b),
        '*' => Some(a * b),
        '/' => {
            if b == 0 {
                None
            } else {
                Some(a / b)
            }
        }
        _ => None,
    }
}
fn main() {
    let resultat = calculer('+', 10, 5);
    match resultat {
        Some(n) => println!("Résultat : {}", n),
        None => println!("Opération invalide"),
    }
}
```

## Exemple 2 : FizzBuzz

```
fn main() {
    for i in 1..=100 {
        match (i % 3, i % 5) {
            (0, 0) => println!("FizzBuzz"),
            (0, _) => println!("Fizz"),
            (_, 0) => println!("Buzz"),
            _ => println!("{}", i),
        }
    }
}
```

## Exemple 3 : Recherche dans un tableau

```
fn trouver_index(tableau: &[i32], valeur: i32) -> Option<usize> {
```

```
    for (index, &element) in tableau.iter().enumerate() {
        if element == valeur {
            return Some(index);
        }
    }
    None
}
fn main() {
    let nombres = [10, 20, 30, 40, 50];
    if let Some(index) = trouver_index(&nombres, 30) {
        println!("Trouvé à l'index {}", index);
    } else {
        println!("Non trouvé");
    }
}
```

## 6. Exercices pratiques

**■ Exercice 1 : Classification d'âge** 

Écris une fonction qui classe un âge en catégorie : 

- 0-12 : Enfant 

- 13-17 : Adolescent 

- 18-64 : Adulte 

• 65+ : Senior `// Solution : fn classifier_age(age: u32) -> &'static str { match age { 0..=12 => "Enfant", 13..=17 => "Adolescent", 18..=64 => "Adulte", _ => "Senior", } }` 

**■ Exercice 2 : Somme avec boucle** Calcule la somme des nombres de 1 à n. 

```
// Solution :
fn somme_jusqua(n: i32) -> i32 {
    let mut somme = 0;
    for i in 1..=n {
        somme += i;
    }
    somme
}
fn main() {
    println!("Somme 1 à 10 : {}", somme_jusqua(10)); // 55
}
```

**■ Exercice 3 : Nombres premiers** Vérifie si un nombre est premier. 

```
// Solution :
fn est_premier(n: u32) -> bool {
    if n < 2 {
        return false;
    }
    for i in 2..n {
        if n % i == 0 {
            return false;
```

```
        }
    }
    true
}
```

```
fn main() {
    for n in 2..=20 {
        if est_premier(n) {
            println!("{} est premier", n);
        }
    }
}
```

## 7. Aide-mémoire

## Conditions

```
if condition { } else { }
if condition { } else if condition2 { } else { }
let x = if condition { valeur1 } else { valeur2 };
```

## Boucles

```
loop { break; }                    // Infinie
while condition { }                // Avec condition
for i in 0..10 { }                 // 0 à 9
for i in 0..=10 { }                // 0 à 10 (inclusif)
for item in collection { }         // Itération
break;                             // Sort de la boucle
continue;                          // Itération suivante
```

## Pattern Matching

```
match valeur {
```

- `1 => expression,` 

- `2 | 3 => expression,` 

- `4..=10 => expression,` 

```
    n if n > 10 => expression,
```

- `_ => expression,` 

```
}
```

## Raccourcis

```
if let Some(x) = option { }        // Match un seul cas
while let Some(x) = iter.next() { } // Boucle avec pattern
```

## Points clés

- Les conditions doivent être des booléens 

- if peut retourner une valeur 

- Préfère for aux boucles while avec index 

- match doit être exhaustif 

- Utilise if let pour simplifier match avec un seul cas 

- Les labels permettent de contrôler des boucles imbriquées 

## ■ Félicitations !

## Tu maîtrises les structures de contrôle !

Tu peux maintenant : 

- Contrôler le flux de ton programme 

- • Utiliser les boucles efficacement • Maîtriser le pattern matching 

- • Écrire du code Rust idiomatique 

**■ Continue ton apprentissage avec les structures et les enums ! ■**
