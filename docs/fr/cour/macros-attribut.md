# Macros procédurales — Attribute
> Une macro attribut reçoit un élément (fonction, struct, module...) et le **remplace** par le code qu'elle produit. Lire ses arguments, transformer une fonction ou une struct, et ajouter des vérifications à la compilation.

## Objectifs

- Comprendre la différence entre une macro derive et une macro attribut
- Écrire la signature d'une macro attribut et analyser l'élément reçu
- Lire les arguments de l'attribut de plusieurs façons (vide, liste, clé = valeur, syntaxe libre)
- Transformer une fonction : envelopper son corps, garder sa signature, gérer `async`
- Transformer une struct : ajouter des champs, générer des `impl`
- Faire vérifier des noms par le compilateur avec `quote_spanned!`
- Connaître les macros attribut courantes de l'écosystème

## Table des matières

1. Derive ou attribut
2. Signature
3. Exemple : mesurer le temps d'exécution
4. Lire les arguments de l'attribut
5. Transformer une fonction
6. Transformer une struct
7. Vérifications à la compilation et positions d'erreur
8. Ordre des attributs
9. Macros attribut de l'écosystème
10. Exemple dans Runique : `#[form]`
11. Attribut, derive ou `macro_rules!`
12. Exercices
13. Aide-mémoire

---

## 1. Derive ou attribut

| | `#[derive(X)]` | `#[x(...)]` |
|---|---|---|
| S'applique à | struct, enum, union | n'importe quel élément : fonction, struct, enum, impl, module, trait |
| Reçoit | l'élément | les arguments **et** l'élément |
| Produit | du code **ajouté** après l'élément | le code qui **remplace** l'élément |
| Peut modifier l'élément | non | oui (ou le recopier tel quel) |
| Arguments | via des attributs d'aide | directement : `#[x(a = 1, b)]` |

Une macro attribut est plus puissante, et donc plus surprenante : le code que l'utilisateur lit n'est pas forcément celui qui est compilé. Réserve-la aux cas où il faut **changer** l'élément.

---

## 2. Signature

```rust
use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn mon_attribut(attr: TokenStream, item: TokenStream) -> TokenStream {
    // attr : ce qui est entre les parenthèses de #[mon_attribut(...)]
    // item : l'élément annoté, complet
    item                // renvoyer item tel quel = l'attribut ne change rien
}
```

```rust
#[mon_attribut(niveau = "debug", repetitions = 3)]
fn calcul() {}
// attr = niveau = "debug", repetitions = 3
// item = fn calcul() {}
```

La crate est une crate `proc-macro`, avec `syn` et `quote`, comme pour une derive (voir `macros-derive`, section 3). Pour transformer des fonctions et du code arbitraire, `syn` a besoin de la feature `full`.

---

## 3. Exemple : mesurer le temps d'exécution

```rust
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

#[proc_macro_attribute]
pub fn chrono(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let fonction = parse_macro_input!(item as ItemFn);

    let attrs = &fonction.attrs;          // #[inline], ///doc... : à conserver
    let vis = &fonction.vis;
    let sig = &fonction.sig;              // nom, paramètres, retour, async, génériques
    let corps = &fonction.block;
    let nom = sig.ident.to_string();

    quote! {
        #(#attrs)*
        #vis #sig {
            let __debut = ::std::time::Instant::now();
            let __resultat = (|| #corps)();
            ::std::eprintln!("{} : {:?}", #nom, __debut.elapsed());
            __resultat
        }
    }
    .into()
}
```

```rust
#[chrono]
fn calcul(n: u64) -> u64 {
    (0..n).sum()
}

fn main() {
    calcul(1_000_000);        // calcul : 1.2ms
}
```

Points importants :

- on **recopie** les attributs, la visibilité et la signature, sinon la fonction perdrait sa documentation ou deviendrait privée ;
- le corps est placé dans une closure : un `return` dans le corps quitte la closure, et la durée est quand même affichée ;
- les variables ajoutées ont un nom improbable (`__debut`) : une macro procédurale n'est **pas** hygiénique, un `debut` pourrait entrer en conflit avec une variable de l'utilisateur.

Limite de cette version : une closure ne peut pas contenir de `.await`, et un `?` dans le corps ne fonctionne que si le type d'erreur se déduit. La section 5.3 traite les fonctions `async`.

---

## 4. Lire les arguments de l'attribut

### 4.1 Aucun argument attendu

```rust
#[proc_macro_attribute]
pub fn sans_argument(attr: TokenStream, item: TokenStream) -> TokenStream {
    if !attr.is_empty() {
        return syn::Error::new(proc_macro2::Span::call_site(), "#[sans_argument] ne prend pas d'argument")
            .to_compile_error()
            .into();
    }
    item
}
```

### 4.2 Une liste de `clé = valeur` et de drapeaux

```rust
use syn::{parse::Parser, punctuated::Punctuated, Expr, Lit, Meta, Token};

struct Options { niveau: String, repetitions: u32, silencieux: bool }

fn lire_options(attr: TokenStream) -> syn::Result<Options> {
    let mut o = Options { niveau: "info".into(), repetitions: 1, silencieux: false };
    let liste = Punctuated::<Meta, Token![,]>::parse_terminated.parse(attr)?;
    for meta in liste {
        match meta {
            Meta::NameValue(nv) if nv.path.is_ident("niveau") => {
                let Expr::Lit(e) = &nv.value else {
                    return Err(syn::Error::new_spanned(&nv.value, "texte attendu"));
                };
                let Lit::Str(s) = &e.lit else {
                    return Err(syn::Error::new_spanned(&e.lit, "texte attendu"));
                };
                o.niveau = s.value();
            }
            Meta::NameValue(nv) if nv.path.is_ident("repetitions") => {
                let Expr::Lit(syn::ExprLit { lit: Lit::Int(n), .. }) = &nv.value else {
                    return Err(syn::Error::new_spanned(&nv.value, "entier attendu"));
                };
                o.repetitions = n.base10_parse()?;
            }
            Meta::Path(p) if p.is_ident("silencieux") => o.silencieux = true,
            autre => return Err(syn::Error::new_spanned(autre, "option inconnue")),
        }
    }
    Ok(o)
}
```

`Meta` couvre les trois formes : `chemin` (drapeau), `chemin = valeur`, `chemin(...)` (liste imbriquée).

### 4.3 La même lecture avec `parse_nested_meta`

```rust
use syn::meta;

fn lire_options_v2(attr: TokenStream) -> syn::Result<Options> {
    let mut o = Options { niveau: "info".into(), repetitions: 1, silencieux: false };
    let lecteur = meta::parser(|m| {
        if m.path.is_ident("niveau") {
            o.niveau = m.value()?.parse::<syn::LitStr>()?.value();
        } else if m.path.is_ident("repetitions") {
            o.repetitions = m.value()?.parse::<syn::LitInt>()?.base10_parse()?;
        } else if m.path.is_ident("silencieux") {
            o.silencieux = true;
        } else {
            return Err(m.error("option inconnue"));
        }
        Ok(())
    });
    syn::parse::Parser::parse(lecteur, attr)?;
    Ok(o)
}
```

Plus court et avec des messages d'erreur bien placés. C'est la forme recommandée par `syn` 2.

### 4.4 Une syntaxe libre : implémenter `Parse`

Pour des arguments qui ne sont pas des `Meta` (un chemin, une liste entre crochets, un mot-clé personnel), on écrit son propre analyseur :

```rust
use syn::{bracketed, parse::{Parse, ParseStream}, punctuated::Punctuated, Ident, Path, Token};

// #[table(nom = utilisateurs, colonnes = [id, email])]
struct ArgsTable { nom: Ident, colonnes: Vec<Ident> }

impl Parse for ArgsTable {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut nom = None;
        let mut colonnes = Vec::new();
        while !input.is_empty() {
            let cle: Ident = input.parse()?;
            input.parse::<Token![=]>()?;
            match cle.to_string().as_str() {
                "nom" => nom = Some(input.parse::<Ident>()?),
                "colonnes" => {
                    let contenu;
                    bracketed!(contenu in input);                 // [ ... ]
                    let liste: Punctuated<Ident, Token![,]> =
                        contenu.parse_terminated(Ident::parse, Token![,])?;
                    colonnes = liste.into_iter().collect();
                }
                _ => return Err(syn::Error::new(cle.span(), "clé inconnue : `nom` ou `colonnes`")),
            }
            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;                      // virgule facultative entre les clés
            }
        }
        let nom = nom.ok_or_else(|| input.error("`nom` est obligatoire"))?;
        Ok(ArgsTable { nom, colonnes })
    }
}

#[proc_macro_attribute]
pub fn table(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = syn::parse_macro_input!(attr as ArgsTable);
    let _ = args;
    item
}
```

| Forme | Pour |
|---|---|
| `Punctuated<Meta, Token![,]>` | options simples, structure fixe |
| `meta::parser` | options simples, meilleurs messages |
| `impl Parse` | syntaxe sur mesure (chemins, listes, mots-clés) |
| `darling::FromMeta` | beaucoup d'options typées, valeurs par défaut |

---

## 5. Transformer une fonction

### 5.1 Ce que contient un `ItemFn`

```rust
pub struct ItemFn {
    pub attrs: Vec<Attribute>,     // attributs restants (dont la doc)
    pub vis: Visibility,
    pub sig: Signature,            // const, async, unsafe, fn, nom, génériques, paramètres, retour
    pub block: Box<Block>,         // le corps
}
```

```rust
let sig = &fonction.sig;
sig.ident;                 // nom
sig.asyncness.is_some();   // async ?
sig.inputs;                // paramètres : FnArg::Receiver (self) ou FnArg::Typed (nom: Type)
sig.output;                // ReturnType::Default (()) ou ReturnType::Type(_, ty)
sig.generics;
```

### 5.2 Exécuter du code avant le corps, sans closure

```rust
#[proc_macro_attribute]
pub fn tracer(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut f = parse_macro_input!(item as ItemFn);
    let nom = f.sig.ident.to_string();
    let corps = &f.block;

    // On remplace le corps par un nouveau bloc qui contient l'ancien
    let nouveau: syn::Block = syn::parse_quote!({
        ::std::eprintln!("entrée dans {}", #nom);
        #corps
    });
    f.block = Box::new(nouveau);

    quote!(#f).into()          // ItemFn s'interpole tel quel
}
```

Modifier l'arbre (`f.block = ...`) puis l'interpoler en entier garde automatiquement attributs, visibilité et signature.

### 5.3 Fonctions `async`

```rust
#[proc_macro_attribute]
pub fn chrono_async(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut f = parse_macro_input!(item as ItemFn);
    let nom = f.sig.ident.to_string();
    let corps = &f.block;

    let nouveau: syn::Block = if f.sig.asyncness.is_some() {
        syn::parse_quote!({
            let __debut = ::std::time::Instant::now();
            let __r = async move #corps.await;         // un bloc async garde .await et ?
            ::std::eprintln!("{} : {:?}", #nom, __debut.elapsed());
            __r
        })
    } else {
        syn::parse_quote!({
            let __debut = ::std::time::Instant::now();
            let __r = (|| #corps)();
            ::std::eprintln!("{} : {:?}", #nom, __debut.elapsed());
            __r
        })
    };
    f.block = Box::new(nouveau);
    quote!(#f).into()
}
```

### 5.4 Lire les paramètres

```rust
use syn::{FnArg, Pat};

let noms: Vec<_> = f.sig.inputs.iter().filter_map(|a| match a {
    FnArg::Typed(pt) => match &*pt.pat {
        Pat::Ident(i) => Some(i.ident.clone()),
        _ => None,                                 // motif déstructuré : pas de nom simple
    },
    FnArg::Receiver(_) => None,                    // self
}).collect();

// quote! { ::std::eprintln!(concat!(#(stringify!(#noms), " = {:?} "),*), #(#noms),*); }
```

### 5.5 Refuser ce qui n'est pas une fonction

```rust
let f = match syn::parse::<ItemFn>(item.clone()) {
    Ok(f) => f,
    Err(_) => {
        return syn::Error::new(proc_macro2::Span::call_site(), "#[chrono] s'applique à une fonction")
            .to_compile_error()
            .into();
    }
};
```

---

## 6. Transformer une struct

### 6.1 Ajouter un champ

```rust
use syn::{parse_macro_input, parse_quote, Fields, ItemStruct};

#[proc_macro_attribute]
pub fn horodate(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut s = parse_macro_input!(item as ItemStruct);

    if let Fields::Named(champs) = &mut s.fields {
        champs.named.push(parse_quote! { pub cree_le: ::std::time::SystemTime });
        champs.named.push(parse_quote! { pub modifie_le: ::std::time::SystemTime });
    } else {
        return syn::Error::new_spanned(&s.ident, "#[horodate] demande des champs nommés")
            .to_compile_error()
            .into();
    }

    let nom = &s.ident;
    quote! {
        #s
        impl #nom {
            pub fn toucher(&mut self) { self.modifie_le = ::std::time::SystemTime::now(); }
        }
    }
    .into()
}
```

```rust
#[horodate]
struct Article { titre: String }

// devient :
// struct Article { titre: String, pub cree_le: SystemTime, pub modifie_le: SystemTime }
// impl Article { pub fn toucher(&mut self) { .. } }
```

Une derive ne pourrait pas ajouter de champ : c'est le cas où l'attribut est nécessaire. Le prix : l'utilisateur voit `struct Article { titre }` mais doit construire `Article { titre, cree_le, modifie_le }`. Une macro qui ajoute des champs doit donc fournir un constructeur et le documenter.

### 6.2 Remplir une struct vide

```rust
#[proc_macro_attribute]
pub fn formulaire(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let s = parse_macro_input!(item as ItemStruct);
    let vis = &s.vis;
    let nom = &s.ident;
    let attrs = &s.attrs;

    if !matches!(s.fields, Fields::Unit) {
        return syn::Error::new_spanned(&s.fields, "écris `struct Nom;` : les champs sont générés")
            .to_compile_error()
            .into();
    }

    quote! {
        #(#attrs)*
        #vis struct #nom {
            pub donnees: ::std::collections::HashMap<String, String>,
            pub erreurs: Vec<String>,
        }
    }
    .into()
}
```

L'utilisateur écrit `#[formulaire] pub struct Contact;` et obtient une struct complète. C'est le modèle de `#[form]` dans Runique (section 10).

---

## 7. Vérifications à la compilation et positions d'erreur

### 7.1 Faire vérifier un nom par le compilateur

Une macro ne voit que la syntaxe : elle ne sait pas si `email` est vraiment une colonne. Mais elle peut **générer** du code qui ne compile que si le nom existe :

```rust
use quote::quote_spanned;

// Pour chaque colonne nommée par l'utilisateur, une ligne inutile à l'exécution
let verifications = colonnes.iter().map(|c| {
    quote_spanned! { c.span()=>
        let _ = #schema::Column::#c;          // ERREUR si la colonne n'existe pas
    }
});

let code = quote! {
    const _: () = { #(#verifications)* };     // bloc constant : rien n'est exécuté
};
```

- `const _: () = { ... };` crée un bloc vérifié à la compilation, sans aucun coût à l'exécution.
- `quote_spanned! { span=> ... }` attribue au code généré la **position** du nom écrit par l'utilisateur : si la colonne n'existe pas, l'erreur souligne `emial` dans son attribut, pas une ligne invisible de la macro.

### 7.2 Une assertion à la compilation

```rust
let messages = colonnes.iter().map(|c| {
    let texte = c.to_string();
    let msg = format!("`{texte}` est la clé primaire : elle ne peut pas être un champ de formulaire");
    quote_spanned! { c.span()=>
        assert!(!::ma_lib::meme_nom(#schema::CLE_PRIMAIRE, #texte), #msg);
    }
});
```

Dans un bloc `const`, un `assert!` qui échoue est une **erreur de compilation** avec ce message. La fonction appelée doit être une `const fn`.

### 7.3 `quote!` ou `quote_spanned!`

| | `quote!` | `quote_spanned!` |
|---|---|---|
| Position du code généré | l'appel de la macro | celle qu'on choisit |
| Une erreur dans ce code souligne | l'attribut entier | l'élément choisi (un nom, un type) |
| Pour | le code qui ne peut pas échouer | les vérifications sur ce que l'utilisateur a écrit |

---

## 8. Ordre des attributs

Les macros attribut s'appliquent **de haut en bas** :

```rust
#[a]          // s'exécute en premier, et reçoit l'élément AVEC #[b] et #[derive] dans ses attributs
#[b]          // s'exécute ensuite, sur ce que #[a] a produit
#[derive(Debug)]
struct S;
```

`#[a]` reçoit `#[b] #[derive(Debug)] struct S;`. S'il recopie les attributs (`#(#attrs)*`), `#[b]` puis le derive s'appliquent ensuite sur le résultat. Une macro attribut qui **oublie** de les recopier fait disparaître tout ce qui est placé en dessous d'elle, y compris `#[derive(...)]`, `#[cfg]` et la documentation.

---

## 9. Macros attribut de l'écosystème

| Attribut | Crate | Ce qu'il fait |
|---|---|---|
| `#[tokio::main]` | tokio | transforme `async fn main` en `fn main` qui démarre un runtime |
| `#[tokio::test]` | tokio | idem pour un test |
| `#[async_trait]` | async-trait | réécrit les `async fn` d'un trait en fonctions qui renvoient un `Pin<Box<dyn Future>>` |
| `#[tracing::instrument]` | tracing | ouvre un *span* avec les arguments à chaque appel |
| `#[serde_as]` | serde_with | ajuste les attributs serde des champs |
| `#[test]`, `#[bench]` | std | marquent une fonction de test |

```rust
// Ce que fait #[tokio::main], en simplifié
#[tokio::main]
async fn main() { /* ... */ }

// devient
fn main() {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async { /* ... */ })
}
```

---

## 10. Exemple dans Runique : `#[form]`

```rust
#[form(schema = crate::entities::contact, fields = [email, message])]
pub struct ContactForm;
```

### 10.1 Les arguments : un `Parse` sur mesure

```rust
// runique/derive_form/src/schema_form.rs (extrait)
enum Columns {
    Only(Vec<Ident>),     // fields = [a, b]
    All,                  // fields = All
    Except(Vec<Ident>),   // exclude = [a, b]
}

struct FormAttrs {
    schema: Path,
    columns: Columns,
}

impl Parse for FormAttrs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // boucle clé = valeur : schema, fields, exclude
        // fields et exclude ensemble -> erreur ; schema absent -> erreur ;
        // ni fields ni exclude -> erreur « incomplete declaration »
        // ...
    }
}
```

C'est la technique de la section 4.4. Le choix de conception : `fields` ou `exclude` est **obligatoire**. Un formulaire qui prendrait toutes les colonnes sans le dire prendrait aussi toute colonne ajoutée plus tard au modèle (un champ `is_admin`, par exemple), une faille classique d'affectation de masse. `fields = All` reste possible, mais il faut l'écrire.

### 10.2 La transformation de la struct

L'utilisateur écrit une struct vide (`pub struct ContactForm;`) ; la macro génère un champ `pub form: Forms` et les implémentations `FormAccess` et `ModelForm` (section 6.2).

### 10.3 Les vérifications à la compilation

```rust
// runique/derive_form/src/schema_form.rs (extrait)
// Each name a column of the entity, checked by the compiler: the error
// lands on the name, not on a panic the first time the form is used.
let checks = fields.iter().chain(exclude.iter()).flatten().map(|field| {
    let mut variant = crate::model::utils::ident_pascal(field);
    variant.set_span(field.span());
    quote_spanned! { field.span()=> let _ = #schema_path::Column::#variant; }
});

// The key is the database's to set: listed in `fields`, refused here.
let key_checks = fields.iter().flatten().map(|field| {
    let name = field.to_string();
    let message = format!(
        "#[form]: `{name}` is the primary key; the database sets it, it is never a form field"
    );
    quote_spanned! { field.span()=>
        assert!(!::runique::forms::__same_name(#schema_path::PRIMARY_KEY, #name), #message);
    }
});

let column_check = quote! { const _: () = { #(#checks)* #(#key_checks)* }; };
```

On y retrouve exactement la section 7 :

- chaque nom de champ devient `Column::NomEnPascal` : une faute de frappe (`emial`) est une erreur de compilation soulignée sur `emial` ;
- lister la clé primaire dans `fields` est refusé à la compilation par une assertion `const`, avec un message qui explique pourquoi ;
- tout est dans un `const _: () = { ... };` : rien n'est exécuté au démarrage.

---

## 11. Attribut, derive ou `macro_rules!`

Ajouter un journal d'entrée à des fonctions.

```rust
// 1. À la main : explicite, mais à répéter dans chaque fonction
fn a() { eprintln!("entrée dans a"); /* ... */ }

// 2. macro_rules! : il faut envelopper toute la fonction
macro_rules! tracee {
    (fn $nom:ident() $corps:block) => {
        fn $nom() { eprintln!("entrée dans {}", stringify!($nom)); $corps }
    };
}
tracee!(fn b() { /* ... */ });

// 3. Attribut : une ligne au-dessus d'une fonction ordinaire
// #[tracer]
// fn c() { /* ... */ }
```

| | À la main | `macro_rules!` | Attribut |
|---|---|---|---|
| Lisibilité du code annoté | parfaite | moyenne (tout est dans la macro) | bonne |
| Gère génériques, `async`, `pub`, attributs | oui | difficilement (un bras par cas) | oui, avec `syn` |
| Coût | aucun | aucun | crate `proc-macro` |

---

## 12. Exercices

### Exercice 1 — `#[deprecie_vers]`

Écris `#[deprecie_vers("nouvelle_fonction")]` qui ajoute `#[deprecated(note = "utilise nouvelle_fonction")]` à la fonction, sans rien changer d'autre.

```rust
// Solution :
#[proc_macro_attribute]
pub fn deprecie_vers(attr: TokenStream, item: TokenStream) -> TokenStream {
    let cible = parse_macro_input!(attr as syn::LitStr);
    let f = parse_macro_input!(item as ItemFn);
    let note = format!("utilise {}", cible.value());
    quote! {
        #[deprecated(note = #note)]
        #f
    }
    .into()
}
```

### Exercice 2 — `#[repeter(n = 3)]`

Écris un attribut qui exécute le corps d'une fonction sans valeur de retour `n` fois.

```rust
// Solution :
#[proc_macro_attribute]
pub fn repeter(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut n: u32 = 1;
    let lecteur = syn::meta::parser(|m| {
        if m.path.is_ident("n") {
            n = m.value()?.parse::<syn::LitInt>()?.base10_parse()?;
            Ok(())
        } else {
            Err(m.error("seule l'option `n` est acceptée"))
        }
    });
    if let Err(e) = syn::parse::Parser::parse(lecteur, attr) {
        return e.to_compile_error().into();
    }
    let mut f = parse_macro_input!(item as ItemFn);
    if !matches!(f.sig.output, syn::ReturnType::Default) {
        return syn::Error::new_spanned(&f.sig.output, "#[repeter] demande une fonction sans retour")
            .to_compile_error()
            .into();
    }
    let corps = &f.block;
    f.block = Box::new(syn::parse_quote!({ for _ in 0..#n #corps }));
    quote!(#f).into()
}
```

### Exercice 3 — nom vérifié

Dans un attribut `#[route(handler = nom)]`, fais vérifier par le compilateur que `nom` est une fonction existante, avec l'erreur soulignée sur `nom`.

```rust
// Solution (extrait) :
let handler: syn::Ident = /* lu dans les arguments */ syn::parse_quote!(index);
let verif = quote::quote_spanned! { handler.span()=>
    const _: () = { let _ = #handler; };           // n'existe pas : erreur soulignée sur le nom
};
```

## 13. Aide-mémoire

```rust
#[proc_macro_attribute]
pub fn nom(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as MesArgs);       // impl Parse for MesArgs
    let mut f = parse_macro_input!(item as ItemFn);       // ou ItemStruct, Item...
    f.block = Box::new(parse_quote!({ /* avant */ #corps }));
    quote!(#f).into()
}

Punctuated::<Meta, Token![,]>::parse_terminated.parse(attr)
syn::meta::parser(|m| { m.path.is_ident("x"); m.value()?.parse::<LitStr>()?; Ok(()) })
bracketed!(contenu in input)   contenu.parse_terminated(Ident::parse, Token![,])
quote_spanned! { ident.span()=> let _ = #chemin::#ident; }
const _: () = { /* vérifications */ };
```

### Points clés

- Un attribut **remplace** l'élément : recopie attributs, visibilité et signature.
- Une macro procédurale n'est pas hygiénique : préfixe les variables ajoutées (`__debut`).
- Gère `async` séparément (bloc `async move`), une closure n'accepte pas `.await`.
- `meta::parser` pour des options simples, `impl Parse` pour une syntaxe sur mesure.
- Ajouter des champs est possible, mais l'utilisateur doit savoir les construire.
- `quote_spanned!` + `const _: () = { .. }` : faire vérifier des noms par le compilateur, avec l'erreur sur le nom écrit par l'utilisateur.
