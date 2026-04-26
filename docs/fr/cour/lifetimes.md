## Les Lifetimes en Rust

Comprendre et Maîtriser les Durées de Vie 

Le Concept le Plus Important de Rust 

### Objectifs du cours

À la fin de ce cours, tu sauras : 

- Comprendre ce qu'est une lifetime 

- Annoter les lifetimes correctement 

- Utiliser lifetimes dans les structs 

- Maîtriser lifetime elision 

- Résoudre les erreurs du borrow checker 

## Table des matières

1. Qu'est-ce qu'une lifetime ? 

2. Annotations de lifetime 

- 2.1 - Syntaxe de base 

- 2.2 - Dans les fonctions 

- 2.3 - Plusieurs lifetimes 

3. Lifetime Elision 

- 3.1 - Règles d'élision 

- 3.2 - Quand annoter 

4. Lifetimes dans les structs 

- 4.1 - Struct avec références 

- 4.2 - Méthodes et lifetimes 

5. 'static lifetime 

6. Patterns avancés 

7. Résoudre les erreurs 

8. Exercices 

## 1. Qu'est-ce qu'une lifetime ?

Une **lifetime** (durée de vie) est la portée pendant laquelle une référence est valide. C'est le mécanisme qui permet à Rust de garantir la sécurité mémoire sans garbage collector. 

**`// Problème sans lifetimes (hypothétique) { let r; { let x = 5; r = &x;  // x va être détruit ! } println!("{}", r);  //`** I **`Dangling pointer ! } // Rust empêche ça avec les lifetimes fn main() { let r; { let x = 5; r = &x;  //`** I **`Erreur de compilation ! // `x` does not live long enough } // println!("{}", r); } // Version correcte fn main() { let x = 5; let r = &x; println!("{}", r);  //`** I **`OK }`** 

I **Lifetime = portée** : Une référence ne peut jamais vivre plus longtemps que la donnée qu'elle référence. C'est le borrow checker qui vérifie ça ! 

## 2. Annotations de lifetime

## 2.1 - Syntaxe de base

```
// Annotation de lifetime avec '
// 'a se lit "lifetime a"
fn exemple<'a>(x: &'a str) -> &'a str {
    x
}
```

```
// Plusieurs paramètres
fn plus_long<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() {
        x
    } else {
        y
    }
}
// Convention de nommage
// 'a, 'b, 'c sont les plus courants
// Mais tu peux utiliser n'importe quel nom: 'lifetime, 'input, etc.
```

## 2.2 - Dans les fonctions

```
// Fonction qui retourne une référence
fn premier_mot<'a>(texte: &'a str) -> &'a str {
    let bytes = texte.as_bytes();
    for (i, &byte) in bytes.iter().enumerate() {
        if byte == b' ' {
            return &texte[0..i];
        }
    }
    texte
}
// Utilisation
fn main() {
    let phrase = String::from("hello world");
    let mot = premier_mot(&phrase);
    println!("{}", mot);  // "hello"
}
// Pourquoi 'a ?
// Le compilateur sait que le retour vit aussi longtemps que l'input
```

## 2.3 - Plusieurs lifetimes

**`// Deux lifetimes différentes fn comparer<'a, 'b>(x: &'a str, y: &'b str) -> bool { x.len() > y.len() } // Le retour peut dépendre d'une seule fn choisir<'a, 'b>(x: &'a str, y: &'b str, premier: bool) -> &'a str { if premier { x  //`** I **`OK, retourne 'a } else { // y  //`** I **`ERREUR ! y a lifetime 'b, pas 'a x  // Doit retourner x } } // Lifetime commune fn plus_long<'a>(x: &'a str, y: &'a str) -> &'a str { if x.len() > y.len() { x } else { y } }`** 

```
// Signifie: le retour vit aussi longtemps que
// la plus petite des deux lifetimes
```

II **Règle d'or :** Si une fonction retourne une référence, elle doit venir d'un des paramètres (ou être 'static). 

## 3. Lifetime Elision

Le compilateur peut souvent **inférer** les lifetimes automatiquement grâce à des règles d'élision. C'est pourquoi tu n'as pas toujours besoin de les écrire ! 

## 3.1 - Règles d'élision

```
// Ces deux fonctions sont équivalentes:
```

```
// Sans annotation (élision)
fn premier(x: &str) -> &str {
    &x[0..1]
```

```
}
```

```
// Avec annotation explicite
fn premier<'a>(x: &'a str) -> &'a str {
    &x[0..1]
```

```
}
```

```
// Règles d'élision:
// 1. Chaque paramètre référence a sa propre lifetime
fn f(x: &str, y: &str)  // devient
fn f<'a, 'b>(x: &'a str, y: &'b str)
```

```
// 2. Si un seul param référence, sa lifetime = lifetime du retour
fn f(x: &str) -> &str  // devient
fn f<'a>(x: &'a str) -> &'a str
```

```
// 3. Si &self ou &mut self, sa lifetime = lifetime du retour
impl MonType {
    fn get_data(&self) -> &str  // devient
    fn get_data<'a>(&'a self) -> &'a str
}
```

## 3.2 - Quand annoter

**`//`** I **`PAS BESOIN d'annoter fn premiere_ligne(texte: &str) -> &str { texte.lines().next().unwrap_or("") } //`** I **`BESOIN d'annoter (plusieurs inputs) fn plus_long<'a>(x: &'a str, y: &'a str) -> &'a str { if x.len() > y.len() { x } else { y } } //`** I **`BESOIN d'annoter (struct avec référence) struct Extrait<'a> { contenu: &'a str, }`** 

**`//`** I **`PAS BESOIN (méthode avec &self) impl<'a> Extrait<'a> { fn contenu(&self) -> &str { self.contenu  // lifetime inférée de &self } }`** 

I **Conseil :** Commence sans annotations. Si le compilateur se plaint, ajoute-les. Le message d'erreur te dira souvent quoi faire ! 

## 4. Lifetimes dans les structs

## 4.1 - Struct avec références

```
// Struct qui contient une référence
struct Article<'a> {
    titre: &'a str,
    contenu: &'a str,
}
impl<'a> Article<'a> {
    fn new(titre: &'a str, contenu: &'a str) -> Self {
        Article { titre, contenu }
    }
    fn apercu(&self) -> &str {
        &self.contenu[0..100.min(self.contenu.len())]
    }
}
// Utilisation
fn main() {
    let titre = String::from("Rust Lifetimes");
    let contenu = String::from("Les lifetimes sont...");
    let article = Article::new(&titre, &contenu);
    println!("{}", article.titre);
    // titre et contenu doivent vivre plus longtemps qu'article !
}
```

## 4.2 - Méthodes et lifetimes

```
struct Parser<'a> {
    texte: &'a str,
    position: usize,
}
impl<'a> Parser<'a> {
    fn new(texte: &'a str) -> Self {
        Parser { texte, position: 0 }
    }
    // Lifetime du retour = lifetime de &self
    fn reste(&self) -> &'a str {
        &self.texte[self.position..]
    }
    // Plusieurs lifetimes
    fn avec_prefixe<'b>(&'a self, prefixe: &'b str) -> String {
        format!("{}{}", prefixe, self.reste())
    }
}
```

## 5. 'static lifetime

`'static` est une lifetime spéciale qui dure pendant **toute l'exécution du programme** . 

```
// Références 'static
let s: &'static str = "Hello world";
```

```
// Littéraux de chaîne sont toujours 'static
const MESSAGE: &'static str = "Constant";
```

```
// Owned data n'a pas besoin de 'static
let s = String::from("owned");  // Pas de lifetime !
```

```
// Trait bound 'static
fn process<T: 'static>(value: T) {
    // T doit être owned ou contenir seulement 'static refs
}
```

**`// Exemples valides process(42);                          //`** I **`i32 is 'static process(String::from("hello"));       //`** I **`String is 'static process("literal");                   //`** I **`&'static str`** 

**`// Exemple invalide let s = String::from("temp"); // process(&s);  //`** I **`&s n'est pas 'static`** 

II **'static ne veut PAS dire immortel !** Un `String` peut être `T: 'static` mais être drop quand même. Ça veut juste dire "pas de références non-'static". 

## 6. Patterns avancés

```
// 1. Lifetime bounds
struct Conteneur<'a, T: 'a> {
    item: &'a T,
}
// 2. Multiple struct lifetimes
struct Context<'s, 'c> {
    source: &'s str,
    config: &'c Config,
}
// 3. Lifetime in trait
trait Parser<'a> {
    fn parse(&self, input: &'a str) -> Result<&'a str, Error>;
}
```

```
// 4. Higher-rank trait bounds (HRTB)
fn apply<F>(f: F)
where
    F: for<'a> Fn(&'a str) -> &'a str
```

```
{
    let s = String::from("hello");
    println!("{}", f(&s));
}
```

## 7. Résoudre les erreurs

**Erreur : "does not live long enough"** 

**`//`** I **`Problème fn dangling_ref() -> &String { let s = String::from("hello"); &s  // s est détruit ici ! } //`** I **`Solutions // 1. Retourner owned fn owned() -> String { String::from("hello") }`** 

```
// 2. Utiliser 'static
fn static_ref() -> &'static str {
    "hello"
}
// 3. Prendre un paramètre
fn borrow_from_param(s: &String) -> &String {
    s
}
```

## Erreur : "lifetime mismatch"

**`//`** I **`Problème fn choisir<'a, 'b>(x: &'a str, y: &'b str) -> &'a str { if true { x } else { y }  // y n'a pas lifetime 'a ! } //`** I **`Solution: même lifetime fn choisir<'a>(x: &'a str, y: &'a str) -> &'a str { if true { x } else { y } }`** 

## 8. Exercices pratiques

## Exercice 1 : Annoter les lifetimes

```
// Ajoute les annotations nécessaires
fn plus_court(x: &str, y: &str) -> &str {
    if x.len() < y.len() { x } else { y }
}
// Solution :
fn plus_court<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() < y.len() { x } else { y }
}
```

## Exercice 2 : Struct avec lifetime

```
// Crée une struct Citation qui contient:
// - une référence vers le texte
// - le nom de l'auteur (référence)
// Solution :
struct Citation<'a> {
    texte: &'a str,
    auteur: &'a str,
}
impl<'a> Citation<'a> {
    fn new(texte: &'a str, auteur: &'a str) -> Self {
        Citation { texte, auteur }
    }
    fn afficher(&self) {
        println!("'{}' - {}", self.texte, self.auteur);
    }
}
```

## Aide-mémoire

|**Syntaxe**|**Signification**|
|---|---|
|**`&'a T`**|Référence avec lifetime 'a|
|**`fn f<'a>`**|Fonction avec paramètre de lifetime|
|**`struct S<'a>`**|Struct avec lifetime|
|**`impl<'a> S<'a>`**|Implémentation pour S avec lifetime|
|**`T: 'a`**|T contient seulement refs qui vivent au moins 'a|
|**`'static`**|Lifetime de toute l'exécution|

- **Lifetime = portée** d'une référence 

- **Borrow checker** vérifie les lifetimes à la compilation 

- **Élision** permet d'éviter les annotations dans 90% des cas 

- **'a, 'b, 'c** sont juste des noms de variables de lifetime 

- **'static** = vit pendant toute l'exécution 

- **Références** ne peuvent jamais outlive leur donnée 

## Félicitations !

Tu as dompté les lifetimes ! 

C'est le concept le plus difficile de Rust, mais aussi le plus puissant. Maintenant tu comprends vraiment la magie du borrow checker ! 

I **Tu es un vrai Rustacean !** I
