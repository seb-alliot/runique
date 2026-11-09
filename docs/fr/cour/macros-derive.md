# Macros procédurales — Derive
> Écrire son propre `#[derive(...)]` : une crate `proc-macro`, `syn` pour lire la struct, `quote` pour écrire le code, et les techniques pour gérer les génériques, les attributs et les erreurs.

## Objectifs

- Comprendre ce qu'est une macro procédurale et en quoi elle diffère de `macro_rules!`
- Créer et organiser une crate `proc-macro`
- Lire une struct ou une enum avec `syn`
- Générer du code avec `quote!`, ses interpolations et ses répétitions
- Gérer les champs nommés, les tuples, les enums et les génériques
- Lire des attributs d'aide (`#[mon_attr(...)]`) sur la struct et ses champs
- Produire des erreurs de compilation précises, à la bonne position
- Tester une macro procédurale

## Table des matières

1. Qu'est-ce qu'une macro derive
2. Les trois sortes de macros procédurales
3. Structure d'une crate `proc-macro`
4. `TokenStream`, `syn`, `quote`
5. Premier derive complet
6. Lire les champs
7. Générer avec `quote!` : interpolations et répétitions
8. Les génériques
9. Les enums
10. Attributs d'aide
11. Erreurs de compilation
12. Chemins dans le code généré
13. Tester une macro procédurale
14. Organiser une bibliothèque avec macros
15. Exemple dans Runique
16. Derive, `macro_rules!` ou code écrit à la main
17. Exercices
18. Aide-mémoire

---

## 1. Qu'est-ce qu'une macro derive

```rust
#[derive(Debug, Clone)]
struct Point { x: i32, y: i32 }
```

`#[derive(Debug)]` appelle une **macro procédurale** : un programme Rust, exécuté par le compilateur, qui reçoit la définition de `Point` et **ajoute** l'implémentation de `Debug` à côté. La struct elle-même n'est pas modifiée.

| | `macro_rules!` | Macro procédurale |
|---|---|---|
| Écrite en | motifs de syntaxe | code Rust ordinaire |
| Peut analyser du code arbitraire | difficilement | oui (avec `syn`) |
| Peut fabriquer des identifiants (`get_` + nom) | non | oui |
| Emplacement | n'importe où | une crate dédiée, de type `proc-macro` |
| Coût de compilation | faible | `syn` et `quote` à compiler, macro exécutée à chaque build |

---

## 2. Les trois sortes de macros procédurales

| Sorte | Utilisation | Ce qu'elle reçoit | Ce qu'elle produit |
|---|---|---|---|
| derive | `#[derive(MaMacro)]` | la struct, l'enum ou l'union | du code **ajouté** après (en général un `impl`) |
| attribut | `#[ma_macro(args)]` | les arguments et l'élément annoté | un code qui **remplace** l'élément (cours `macros-attribut`) |
| function-like | `ma_macro!(...)` | les jetons entre parenthèses | n'importe quel code (cours `macros-function-like`) |

Une derive est la plus restreinte : elle ne peut **rien modifier**, seulement ajouter. C'est aussi la plus prévisible pour l'utilisateur.

---

## 3. Structure d'une crate `proc-macro`

### 3.1 Créer la crate

```bash
cargo new decrire_macros --lib
```

```toml
# decrire_macros/Cargo.toml
[package]
name = "decrire_macros"
version = "0.1.0"
edition = "2024"

[lib]
proc-macro = true            # indispensable : fait de la crate une crate de macros

[dependencies]
syn = { version = "2", features = ["full"] }   # analyse du code Rust
quote = "1"                                    # génération de code
proc-macro2 = "1"                              # TokenStream utilisable hors de la macro (tests)
```

### 3.2 Les contraintes d'une crate `proc-macro`

- Elle ne peut exporter **que** des macros procédurales : pas de fonction, pas de struct, pas de trait publics.
- Elle est compilée pour la machine qui compile (pas pour la cible), et exécutée pendant la compilation.
- Les types et traits utilisés par le code généré doivent donc vivre dans **une autre crate** (section 14).

### 3.3 Utiliser la macro

```toml
# mon_app/Cargo.toml
[dependencies]
decrire_macros = { path = "../decrire_macros" }
```

```rust
use decrire_macros::Decrire;

#[derive(Decrire)]
struct Utilisateur { nom: String, age: u32 }
```

---

## 4. `TokenStream`, `syn`, `quote`

```
code source ──► TokenStream ──syn──► arbre (DeriveInput) ──vous──► quote! ──► TokenStream ──► compilateur
```

- `proc_macro::TokenStream` : une suite de jetons, ce que le compilateur donne et reprend.
- `syn` : transforme les jetons en un arbre typé (`DeriveInput`, `Field`, `Type`, `Expr`...).
- `quote!` : écrit du code Rust avec des « trous » (`#nom`) remplis par des valeurs, et produit des jetons.
- `proc_macro2` : la même chose que `proc_macro`, mais utilisable hors d'une macro (tests unitaires) ; `syn` et `quote` travaillent avec lui.

### Le point d'entrée

```rust
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Decrire)]
pub fn derive_decrire(entree: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(entree as DeriveInput);   // erreur de syntaxe -> erreur de compilation
    let nom = &ast.ident;

    let code = quote! {
        impl #nom {
            pub fn decrire() -> &'static str {
                stringify!(#nom)
            }
        }
    };

    code.into()                                            // proc_macro2::TokenStream -> proc_macro::TokenStream
}
```

### Ce que contient un `DeriveInput`

```rust
pub struct DeriveInput {
    pub attrs: Vec<Attribute>,     // #[...] posés sur le type
    pub vis: Visibility,           // pub, pub(crate)...
    pub ident: Ident,              // le nom du type
    pub generics: Generics,        // <T, 'a, const N: usize> et where
    pub data: Data,                // Struct, Enum ou Union, avec les champs
}
```

---

## 5. Premier derive complet

Un derive `Decrire` qui ajoute une méthode `decrire(&self) -> String` listant les champs et leurs valeurs.

```rust
// decrire_macros/src/lib.rs
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

#[proc_macro_derive(Decrire)]
pub fn derive_decrire(entree: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(entree as DeriveInput);
    let nom = &ast.ident;

    let Data::Struct(donnees) = &ast.data else {
        return syn::Error::new_spanned(&ast.ident, "Decrire ne s'applique qu'aux structs")
            .to_compile_error()
            .into();
    };

    let Fields::Named(champs) = &donnees.fields else {
        return syn::Error::new_spanned(&ast.ident, "Decrire demande des champs nommés")
            .to_compile_error()
            .into();
    };

    let noms: Vec<_> = champs.named.iter().map(|c| c.ident.as_ref().unwrap()).collect();

    quote! {
        impl #nom {
            pub fn decrire(&self) -> String {
                let mut parties = Vec::new();
                #(
                    parties.push(format!("{} = {:?}", stringify!(#noms), self.#noms));
                )*
                format!("{} {{ {} }}", stringify!(#nom), parties.join(", "))
            }
        }
    }
    .into()
}
```

```rust
// mon_app
#[derive(Decrire)]
struct Utilisateur { nom: String, age: u32 }

fn main() {
    let u = Utilisateur { nom: "Ada".into(), age: 36 };
    println!("{}", u.decrire());     // Utilisateur { nom = "Ada", age = 36 }
}
```

Le code généré (visible avec `cargo expand`) :

```rust
impl Utilisateur {
    pub fn decrire(&self) -> String {
        let mut parties = Vec::new();
        parties.push(format!("{} = {:?}", "nom", self.nom));
        parties.push(format!("{} = {:?}", "age", self.age));
        format!("{} {{ {} }}", "Utilisateur", parties.join(", "))
    }
}
```

---

## 6. Lire les champs

### 6.1 Les trois formes de struct

```rust
use syn::{Data, Fields};

fn decrire_forme(data: &Data) -> &'static str {
    match data {
        Data::Struct(s) => match &s.fields {
            Fields::Named(_) => "struct à champs nommés",      // struct S { a: i32 }
            Fields::Unnamed(_) => "tuple struct",              // struct S(i32, String);
            Fields::Unit => "unit struct",                     // struct S;
        },
        Data::Enum(_) => "enum",
        Data::Union(_) => "union",
    }
}
```

### 6.2 Ce que contient un champ

```rust
pub struct Field {
    pub attrs: Vec<Attribute>,     // #[...] sur le champ
    pub vis: Visibility,
    pub ident: Option<Ident>,      // None pour un champ de tuple
    pub ty: Type,                  // le type du champ
    // ...
}
```

### 6.3 Champs de tuple : par position

```rust
use syn::Index;

let indices: Vec<Index> = (0..champs.unnamed.len()).map(Index::from).collect();
// quote! { #( self.#indices ),* }     ->   self.0, self.1, self.2
```

Un `usize` interpolé donnerait `0usize` ; `syn::Index` donne `0`, la forme correcte après un point.

### 6.4 Filtrer selon le type

```rust
use syn::{Type, TypePath};

fn est_option(ty: &Type) -> bool {
    matches!(ty, Type::Path(TypePath { path, .. })
        if path.segments.last().is_some_and(|s| s.ident == "Option"))
}
```

Une macro ne voit que la **syntaxe** : elle sait qu'un champ s'écrit `Option<String>`, pas ce que ce type est réellement. Un alias (`type MaybeName = Option<String>`) ou un chemin complet (`std::option::Option`) ne sont reconnus que si on les gère explicitement.

---

## 7. Générer avec `quote!` : interpolations et répétitions

### 7.1 Interpoler

```rust
let nom = &ast.ident;            // Ident
let ty: syn::Type = syn::parse_quote!(u32);
let n = 42usize;
let texte = "bonjour";

let code = quote! {
    const N: usize = #n;         // 42usize
    fn f() -> #ty { 0 }          // fn f() -> u32
    fn g() -> &'static str { #texte }    // "bonjour" (un littéral de chaîne)
    impl #nom {}
};
```

Tout type qui implémente `ToTokens` s'interpole : `Ident`, `Type`, `Expr`, `TokenStream`, entiers, `&str`, `bool`...

### 7.2 Répéter

```rust
let noms = vec![quote!(a), quote!(b), quote!(c)];
let types = vec![quote!(i32), quote!(String), quote!(bool)];

quote! {
    struct S { #( #noms: #types ),* }        // a: i32, b: String, c: bool
};

quote! {
    #( println!("{}", stringify!(#noms)); )*   // une ligne par élément, sans séparateur
};
```

`#( ... ),*` répète avec un séparateur, `#( ... )*` sans. Toutes les variables d'une même répétition doivent avoir la même longueur.

### 7.3 Fabriquer un identifiant

```rust
use quote::format_ident;

let champ = format_ident!("email");
let getter = format_ident!("get_{}", champ);        // get_email
let setter = format_ident!("set_{}", champ);        // set_email

quote! {
    pub fn #getter(&self) -> &str { &self.#champ }
    pub fn #setter(&mut self, v: String) { self.#champ = v; }
};
```

C'est ce que `macro_rules!` ne sait pas faire.

### 7.4 Construire le code par morceaux

```rust
let mut methodes = proc_macro2::TokenStream::new();
for c in champs.named.iter() {
    let nom = c.ident.as_ref().unwrap();
    let ty = &c.ty;
    methodes.extend(quote! {
        pub fn #nom(&self) -> &#ty { &self.#nom }
    });
}

quote! {
    impl #nom_type { #methodes }
};
```

---

## 8. Les génériques

### 8.1 Le problème

```rust
#[derive(Decrire)]
struct Boite<T> { valeur: T }

// Un impl écrit comme `impl #nom { ... }` donne `impl Boite { ... }` : ERREUR, Boite attend un T
```

### 8.2 `split_for_impl`

```rust
let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

quote! {
    impl #impl_generics #nom #ty_generics #where_clause {
        // ...
    }
}
// impl<T> Boite<T> { ... }
// impl<'a, T: Clone, const N: usize> S<'a, T, N> where T: Debug { ... }
```

`split_for_impl` sépare correctement durées de vie, paramètres de type, const generics et clause `where`.

### 8.3 Ajouter une contrainte

Le derive de la section 5 appelle `{:?}` sur chaque champ : `T` doit être `Debug`.

```rust
use syn::{parse_quote, GenericParam};

let mut generics = ast.generics.clone();
for param in &mut generics.params {
    if let GenericParam::Type(t) = param {
        t.bounds.push(parse_quote!(::std::fmt::Debug));   // T devient T: Debug
    }
}
let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
```

C'est ce que fait `#[derive(Debug)]` de la bibliothèque standard : il ajoute `T: Debug` à chaque paramètre.

---

## 9. Les enums

```rust
use syn::{Data, Fields};

#[proc_macro_derive(NomVariante)]
pub fn derive_nom_variante(entree: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(entree as DeriveInput);
    let nom = &ast.ident;

    let Data::Enum(e) = &ast.data else {
        return syn::Error::new_spanned(nom, "NomVariante ne s'applique qu'aux enums")
            .to_compile_error()
            .into();
    };

    let bras = e.variants.iter().map(|v| {
        let variante = &v.ident;
        let texte = variante.to_string();
        let motif = match &v.fields {
            Fields::Unit => quote!(Self::#variante),
            Fields::Unnamed(_) => quote!(Self::#variante(..)),
            Fields::Named(_) => quote!(Self::#variante { .. }),
        };
        quote!(#motif => #texte)
    });

    let (ig, tg, wc) = ast.generics.split_for_impl();
    quote! {
        impl #ig #nom #tg #wc {
            pub fn nom_variante(&self) -> &'static str {
                match self { #(#bras),* }
            }
        }
    }
    .into()
}
```

```rust
#[derive(NomVariante)]
enum Message { Quitter, Ecrire(String), Deplacer { x: i32, y: i32 } }

Message::Ecrire("a".into()).nom_variante();    // "Ecrire"
```

Le motif dépend de la forme de chaque variante : `Self::A`, `Self::A(..)` ou `Self::A { .. }`.

---

## 10. Attributs d'aide

### 10.1 Les déclarer

```rust
#[proc_macro_derive(Decrire, attributes(decrire))]
pub fn derive_decrire(entree: TokenStream) -> TokenStream { /* ... */ }
```

`attributes(decrire)` autorise l'utilisateur à écrire `#[decrire(...)]` sur le type et sur ses champs. Sans cette déclaration, le compilateur refuserait l'attribut inconnu.

```rust
#[derive(Decrire)]
#[decrire(nom = "Compte")]
struct Utilisateur {
    nom: String,
    #[decrire(cache)]
    mot_de_passe: String,
    #[decrire(renommer = "âge")]
    age: u32,
}
```

### 10.2 Les lire avec `parse_nested_meta`

```rust
struct OptionsChamp {
    cache: bool,
    renommer: Option<String>,
}

fn lire_options(champ: &syn::Field) -> syn::Result<OptionsChamp> {
    let mut opts = OptionsChamp { cache: false, renommer: None };
    for attr in &champ.attrs {
        if !attr.path().is_ident("decrire") {
            continue;                                  // un autre attribut (serde, doc...) : ignoré
        }
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("cache") {
                opts.cache = true;                     // #[decrire(cache)]
                Ok(())
            } else if meta.path.is_ident("renommer") {
                let valeur: syn::LitStr = meta.value()?.parse()?;   // #[decrire(renommer = "...")]
                opts.renommer = Some(valeur.value());
                Ok(())
            } else {
                Err(meta.error("option inconnue : attendu `cache` ou `renommer = \"...\"`"))
            }
        })?;
    }
    Ok(opts)
}
```

### 10.3 S'en servir dans la génération

```rust
let mut lignes = Vec::new();
for c in champs.named.iter() {
    let opts = match lire_options(c) {
        Ok(o) => o,
        Err(e) => return e.to_compile_error().into(),
    };
    if opts.cache {
        continue;                                      // le mot de passe n'apparaît jamais
    }
    let ident = c.ident.as_ref().unwrap();
    let libelle = opts.renommer.unwrap_or_else(|| ident.to_string());
    lignes.push(quote! {
        parties.push(format!("{} = {:?}", #libelle, self.#ident));
    });
}
```

Un attribut `cache` qui retire un champ secret de l'affichage est un bon exemple d'usage : c'est le genre d'erreur qu'un `#[derive(Debug)]` ordinaire ne sait pas éviter.

### 10.4 Avec la crate `darling`

Pour des attributs nombreux, la crate `darling` déclare les options comme une struct (`#[derive(FromField)] #[darling(attributes(decrire))] struct OptionsChamp { #[darling(default)] cache: bool, renommer: Option<String> }`) et génère la lecture et les messages d'erreur.

---

## 11. Erreurs de compilation

### 11.1 Ne jamais paniquer

```rust
// Mauvais : l'utilisateur voit « proc-macro derive panicked » sans position
let champs = match &ast.data { Data::Struct(s) => &s.fields, _ => panic!("struct attendue") };

// Bon : une erreur de compilation, soulignée au bon endroit
let Data::Struct(s) = &ast.data else {
    return syn::Error::new_spanned(&ast.ident, "struct attendue").to_compile_error().into();
};
```

### 11.2 Choisir la position

| Construction | Ce qui est souligné |
|---|---|
| `syn::Error::new_spanned(&noeud, "...")` | le code de `noeud` (un champ, un type, un attribut) |
| `syn::Error::new(ident.span(), "...")` | une position précise |
| `syn::Error::new(Span::call_site(), "...")` | l'appel de la macro (le `#[derive]`) |

Souligne l'élément **fautif** (le champ au type non supporté, l'attribut mal écrit), pas toute la struct.

### 11.3 Plusieurs erreurs à la fois

```rust
let mut erreurs: Option<syn::Error> = None;
for c in champs.named.iter() {
    if let Err(e) = lire_options(c) {
        match &mut erreurs {
            Some(tout) => tout.combine(e),            // accumuler
            None => erreurs = Some(e),
        }
    }
}
if let Some(e) = erreurs {
    return e.to_compile_error().into();               // toutes les erreurs en une compilation
}
```

### 11.4 Structurer avec `syn::Result`

```rust
#[proc_macro_derive(Decrire, attributes(decrire))]
pub fn derive_decrire(entree: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(entree as DeriveInput);
    generer(&ast).unwrap_or_else(|e| e.to_compile_error()).into()
}

fn generer(ast: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    // ... ? partout ...
    Ok(quote! {})
}
```

Le point d'entrée ne fait que convertir ; tout le travail est dans une fonction qui utilise `?`.

---

## 12. Chemins dans le code généré

Le code généré est compilé **chez l'utilisateur**, avec ses imports à lui.

```rust
// Mauvais : dépend des imports de l'utilisateur
quote! { impl Display for #nom { fn fmt(&self, f: &mut Formatter) -> Result { .. } } };

// Bon : chemins absolus
quote! {
    impl ::std::fmt::Display for #nom {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result { /* .. */ Ok(()) }
    }
};
```

Une macro procédurale n'a pas de `$crate`. Pour les types de ta propre bibliothèque, le code généré écrit `::ma_lib::Trait` : l'utilisateur doit donc dépendre de `ma_lib` sous ce nom (voir `macros-export`, section 7.3).

---

## 13. Tester une macro procédurale

### 13.1 Test d'utilisation

```rust
// mon_app/tests/decrire.rs (ou dans la crate qui réexporte la macro)
use decrire_macros::Decrire;

#[derive(Decrire)]
struct Point { x: i32, y: i32 }

#[test]
fn decrit_les_champs() {
    assert_eq!(Point { x: 1, y: 2 }.decrire(), "Point { x = 1, y = 2 }");
}
```

Une crate `proc-macro` ne peut pas utiliser ses propres macros dans ses tests unitaires : on les teste depuis une autre crate (tests d'intégration de la bibliothèque qui la réexporte, ou un dossier d'exemples).

### 13.2 Tester les erreurs avec `trybuild`

```rust
// tests/erreurs.rs
#[test]
fn erreurs_de_compilation() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/*.rs");      // chaque fichier doit échouer avec le message du .stderr voisin
    t.pass("tests/ok/*.rs");
}
```

`trybuild` compile des petits fichiers et compare le message d'erreur obtenu à un fichier `.stderr` attendu : on vérifie ainsi que l'utilisateur reçoit **le bon message, au bon endroit**.

### 13.3 Tester la logique sans compiler

La logique d'analyse et de génération peut vivre dans une fonction qui prend un `proc_macro2::TokenStream` : elle se teste alors comme du code ordinaire, sans macro.

```rust
fn generer2(entree: proc_macro2::TokenStream) -> syn::Result<proc_macro2::TokenStream> {
    let ast: DeriveInput = syn::parse2(entree)?;
    Ok(quote! { /* ... */ })
}

#[test]
fn refuse_une_enum() {
    let r = generer2(quote! { enum E { A } });
    assert!(r.is_err());
}
```

---

## 14. Organiser une bibliothèque avec macros

Une crate `proc-macro` ne peut exporter que des macros. Une bibliothèque qui en fournit est donc en général découpée ainsi :

```
ma_lib/            ← la bibliothèque : types, traits, fonctions ; réexporte les macros
ma_lib_macros/     ← proc-macro = true : seulement les macros
ma_lib_dsl/        ← (facultatif) l'analyseur, dans une crate normale, réutilisable ailleurs
```

```rust
// ma_lib/src/lib.rs
pub trait Decrire { fn decrire(&self) -> String; }   // le trait vit ici
pub use ma_lib_macros::Decrire;                      // la macro derive de même nom aussi
```

Un trait et une macro derive peuvent porter le **même nom** (`serde::Serialize` est les deux) : ils vivent dans des espaces de noms différents.

Ordre de publication sur crates.io : d'abord les crates dont les autres dépendent (analyseur, puis macros, puis bibliothèque).

---

## 15. Exemple dans Runique

Runique suit exactement le découpage de la section 14 :

```
runique/runique_dsl/   ← analyseur du DSL model!{} / extend!{} -> arbre (crate normale)
runique/derive_form/   ← proc-macro = true : model!{}, extend!{}, #[form]
runique/               ← le framework ; réexporte les macros (use runique::prelude::*)
```

```toml
# runique/derive_form/Cargo.toml (extrait)
[lib]
proc-macro = true

[dependencies]
runique_dsl.workspace = true
syn = { workspace = true, features = ["extra-traits"] }
quote = "1"
proc-macro2 = "1"
```

Pourquoi l'analyseur est-il dans une crate séparée (`runique_dsl`) ? Parce qu'une crate `proc-macro` ne peut rien exporter d'autre que des macros. La commande `runique makemigrations` doit lire les mêmes déclarations `model!{}` que la macro pour générer les migrations : l'analyseur vit donc dans une crate ordinaire, utilisée à la fois par `derive_form` et par la CLI.

Les macros de Runique ne sont pas des derives : `model!{}` et `extend!{}` sont des macros function-like, `#[form]` est une macro attribut. Elles utilisent les mêmes outils que ce cours (`parse_macro_input!`, `syn::Error`, `to_compile_error`, `quote!`). Elles sont présentées dans les deux cours suivants.

Note : Runique utilise `syn` 3 ; les exemples de ce cours suivent l'API de `syn` 2, la plus répandue dans les ressources actuelles.

---

## 16. Derive, `macro_rules!` ou code écrit à la main

Générer des getters pour une struct.

```rust
// 1. À la main : rien à apprendre, mais répétitif
struct A { nom: String, age: u32 }
impl A {
    fn nom(&self) -> &String { &self.nom }
    fn age(&self) -> &u32 { &self.age }
}

// 2. macro_rules! : il faut répéter la liste des champs à l'appel
macro_rules! getters {
    ($t:ident { $($c:ident : $ty:ty),* }) => {
        impl $t { $( fn $c(&self) -> &$ty { &self.$c } )* }
    };
}
struct B { nom: String, age: u32 }
getters!(B { nom: String, age: u32 });

// 3. Derive : la macro lit les champs elle-même
// #[derive(Getters)]
// struct C { nom: String, age: u32 }
```

| | À la main | `macro_rules!` | Derive |
|---|---|---|---|
| Effort initial | aucun | faible | crate dédiée, `syn`, `quote` |
| La liste des champs | écrite deux fois | écrite deux fois | écrite une fois |
| Noms fabriqués (`get_nom`) | à la main | impossible | `format_ident!` |
| Attributs par champ | — | non | oui |
| Pour | quelques types | une répétition locale | une bibliothèque, beaucoup de types |

---

## 17. Exercices

### Exercice 1 — `NomsDesChamps`

Écris un derive qui ajoute `fn noms_des_champs() -> &'static [&'static str]`.

```rust
// Solution :
#[proc_macro_derive(NomsDesChamps)]
pub fn derive_noms(entree: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(entree as DeriveInput);
    let nom = &ast.ident;
    let Data::Struct(s) = &ast.data else {
        return syn::Error::new_spanned(nom, "struct attendue").to_compile_error().into();
    };
    let noms: Vec<String> = s.fields.iter()
        .filter_map(|c| c.ident.as_ref().map(|i| i.to_string()))
        .collect();
    let (ig, tg, wc) = ast.generics.split_for_impl();
    quote! {
        impl #ig #nom #tg #wc {
            pub fn noms_des_champs() -> &'static [&'static str] { &[#(#noms),*] }
        }
    }
    .into()
}
```

### Exercice 2 — setters avec `format_ident!`

Ajoute un `set_<champ>` pour chaque champ, sauf ceux marqués `#[setters(non)]`.

```rust
// Solution (extrait) :
let setters = champs.named.iter().filter(|c| !marque_non(c)).map(|c| {
    let champ = c.ident.as_ref().unwrap();
    let ty = &c.ty;
    let nom = quote::format_ident!("set_{}", champ);
    quote! { pub fn #nom(&mut self, v: #ty) { self.#champ = v; } }
});

fn marque_non(c: &syn::Field) -> bool {
    c.attrs.iter().any(|a| {
        a.path().is_ident("setters")
            && a.parse_args::<syn::Ident>().is_ok_and(|i| i == "non")
    })
}
```

### Exercice 3 — erreur au bon endroit

Fais en sorte que ton derive refuse un champ de type `f64` en soulignant **ce champ**.

```rust
// Solution (extrait) :
for c in champs.named.iter() {
    let ty = &c.ty;
    if quote::quote!(#ty).to_string() == "f64" {
        return syn::Error::new_spanned(&c.ty, "f64 n'est pas supporté : utilise un Decimal")
            .to_compile_error()
            .into();
    }
}
```

`new_spanned(&c.ty, ...)` place le soulignement sur le type du champ dans le code de l'utilisateur.

## 18. Aide-mémoire

```toml
[lib]
proc-macro = true
[dependencies]
syn = { version = "2", features = ["full"] }
quote = "1"
proc-macro2 = "1"
```

```rust
#[proc_macro_derive(Nom, attributes(nom))]
pub fn f(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let nom = &ast.ident;
    let (ig, tg, wc) = ast.generics.split_for_impl();
    quote! { impl #ig ::ma_lib::Trait for #nom #tg #wc { .. } }.into()
}

Data::Struct(s) / Data::Enum(e)       Fields::Named / Unnamed / Unit
#var   #( #a: #b ),*   #( .. )*        format_ident!("get_{}", x)   syn::Index::from(i)
attr.path().is_ident("nom")           attr.parse_nested_meta(|meta| ..)
syn::Error::new_spanned(&noeud, "..").to_compile_error()      e.combine(autre)
```

### Points clés

- Un derive ajoute du code après le type, sans le modifier.
- Une crate `proc-macro` n'exporte que des macros : types et traits vont dans une autre crate.
- `syn` lit, `quote!` écrit ; `split_for_impl` pour les génériques.
- `format_ident!` fabrique des noms, ce que `macro_rules!` ne peut pas faire.
- Déclare les attributs d'aide avec `attributes(...)`, lis-les avec `parse_nested_meta`.
- Jamais de `panic!` : `syn::Error::new_spanned` sur l'élément fautif.
- Chemins absolus dans le code généré.
- Teste l'usage depuis une autre crate, les erreurs avec `trybuild`.
