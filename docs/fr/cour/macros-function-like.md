# Macros procédurales — Function-like
> `ma_macro!(...)` écrite en Rust : inventer une syntaxe, l'analyser avec `syn`, la valider à la compilation et générer du code. Avec, en exemple réel, le DSL `model!{}` de Runique.

## Objectifs

- Savoir quand une macro function-like procédurale remplace `macro_rules!`
- Écrire la signature et le point d'entrée
- Concevoir une syntaxe et l'analyser avec `Parse` : mots-clés, ponctuation, groupes, listes, choix
- Valider des valeurs à la compilation (texte, nombres, noms)
- Générer des structs, des modules et des constantes
- Partager l'analyseur entre une macro et un autre outil (CLI)
- Comparer les trois sortes de macros procédurales

## Table des matières

1. `macro_rules!` ou macro procédurale
2. Signature
3. Concevoir une syntaxe
4. Analyser avec `Parse`
5. Exemple : valider une requête à la compilation
6. Exemple : un DSL de configuration
7. Analyser des structures complexes
8. Générer du code
9. Ce qu'une macro ne doit pas faire
10. Exemple dans Runique : `model!{}` et `extend!{}`
11. Comparaison des trois sortes de macros procédurales
12. Exercices
13. Aide-mémoire

---

## 1. `macro_rules!` ou macro procédurale

| Besoin | `macro_rules!` | Function-like procédurale |
|---|---|---|
| Syntaxe régulière (listes, paires) | oui | oui |
| Fabriquer des identifiants (`get_` + nom) | non | oui |
| Calculer à la compilation (valider un texte, convertir une casse) | non | oui |
| Messages d'erreur précis, positionnés | limité (`compile_error!`) | oui |
| Lire une syntaxe très libre | possible mais pénible (tt muncher) | naturel |
| Coût | aucun | crate `proc-macro`, `syn`, `quote` |

`urlpatterns!` de Runique est une `macro_rules!` : sa syntaxe est régulière. `model!{}` est procédurale : elle convertit des noms (`cour_id` → `CourId`), valide des types et des contraintes, et génère des modules entiers.

---

## 2. Signature

```rust
use proc_macro::TokenStream;

#[proc_macro]
pub fn ma_macro(entree: TokenStream) -> TokenStream {
    // entree : tout ce qui est entre les délimiteurs de ma_macro!( ... ) / [ ... ] / { ... }
    entree
}
```

```rust
ma_macro!(a + b);
ma_macro! { config { port: 8080 } }
```

Comme pour `macro_rules!`, l'appel peut utiliser `()`, `[]` ou `{}` ; les délimiteurs ne sont pas transmis.

---

## 3. Concevoir une syntaxe

Avant d'écrire l'analyseur, écris des **exemples** d'utilisation et vérifie trois règles :

1. **Ce doit être une suite de jetons Rust valides** : identifiants, littéraux, ponctuation, groupes bien parenthésés. `ma_macro!(a @@ b)` est accepté ; une apostrophe seule (`'`) ou un guillemet non fermé ne l'est pas.
2. **Ressembler à du Rust** quand c'est possible : `nom: type [options]` se lit mieux qu'une syntaxe inventée de toutes pièces.
3. **Rester sans ambiguïté** : chaque élément doit pouvoir être reconnu en regardant un ou deux jetons à l'avance.

```rust
// Exemple de syntaxe visée
config! {
    serveur {
        port: 8080,
        hote: "0.0.0.0",
    }
    base {
        url: "postgres://localhost/app",
        connexions: 10,
    }
}
```

---

## 4. Analyser avec `Parse`

### 4.1 Le principe

```rust
use syn::parse::{Parse, ParseStream};
use syn::{Ident, LitInt, Token};

// port: 8080
struct Reglage {
    nom: Ident,
    valeur: LitInt,
}

impl Parse for Reglage {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let nom: Ident = input.parse()?;        // consomme un identifiant
        input.parse::<Token![:]>()?;            // consomme ':'
        let valeur: LitInt = input.parse()?;    // consomme un entier
        Ok(Reglage { nom, valeur })
    }
}
```

Chaque `input.parse()` **consomme** des jetons et renvoie une erreur positionnée si ce n'est pas ce qui est attendu.

### 4.2 Les outils

| Outil | Rôle |
|---|---|
| `input.parse::<T>()` | consommer un `T` (`Ident`, `LitStr`, `Type`, `Expr`, `Path`...) |
| `Token![:]`, `Token![=>]`, `Token![,]` | ponctuation et mots-clés Rust (`Token![fn]`, `Token![pub]`) |
| `syn::custom_keyword!(table)` | déclarer un mot-clé à toi |
| `input.peek(T)` | regarder le prochain jeton sans le consommer |
| `input.lookahead1()` | regarder, et produire une erreur « attendu A, B ou C » |
| `braced!`, `bracketed!`, `parenthesized!` | entrer dans un groupe `{}`, `[]`, `()` |
| `Punctuated::<T, Token![,]>::parse_terminated` | une liste séparée, virgule finale permise |
| `input.is_empty()` | fin de l'entrée (ou du groupe) |
| `input.error("...")` | erreur à la position courante |

### 4.3 Mots-clés personnels

```rust
mod kw {
    syn::custom_keyword!(table);
    syn::custom_keyword!(champs);
}

// table: "utilisateurs"
struct Table { nom: syn::LitStr }

impl Parse for Table {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        input.parse::<kw::table>()?;           // le mot `table`, et rien d'autre
        input.parse::<Token![:]>()?;
        Ok(Table { nom: input.parse()? })
    }
}
```

### 4.4 Choisir entre plusieurs formes

```rust
use syn::{LitBool, LitInt, LitStr};

enum Valeur {
    Texte(LitStr),
    Entier(LitInt),
    Booleen(LitBool),
}

impl Parse for Valeur {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let l = input.lookahead1();
        if l.peek(LitStr) {
            Ok(Valeur::Texte(input.parse()?))
        } else if l.peek(LitInt) {
            Ok(Valeur::Entier(input.parse()?))
        } else if l.peek(LitBool) {
            Ok(Valeur::Booleen(input.parse()?))
        } else {
            Err(l.error())                     // « expected string literal, integer literal or boolean literal »
        }
    }
}
```

`lookahead1` mémorise tout ce qui a été testé et construit le message d'erreur automatiquement.

### 4.5 Groupes et listes

```rust
use syn::{braced, punctuated::Punctuated};

// serveur { port: 8080, hote: "0.0.0.0", }
struct Section {
    nom: Ident,
    reglages: Vec<(Ident, Valeur)>,
}

impl Parse for Section {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let nom: Ident = input.parse()?;
        let contenu;
        braced!(contenu in input);             // tout ce qui est entre { } devient un nouveau flux
        let liste = Punctuated::<Paire, Token![,]>::parse_terminated(&contenu)?;
        Ok(Section { nom, reglages: liste.into_iter().map(|p| (p.nom, p.valeur)).collect() })
    }
}

struct Paire { nom: Ident, valeur: Valeur }

impl Parse for Paire {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let nom = input.parse()?;
        input.parse::<Token![:]>()?;
        Ok(Paire { nom, valeur: input.parse()? })
    }
}
```

### 4.6 Répéter jusqu'à la fin

```rust
struct Config { sections: Vec<Section> }

impl Parse for Config {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut sections = Vec::new();
        while !input.is_empty() {
            sections.push(input.parse()?);
        }
        Ok(Config { sections })
    }
}
```

---

## 5. Exemple : valider une requête à la compilation

```rust
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, LitStr};

#[proc_macro]
pub fn requete(entree: TokenStream) -> TokenStream {
    let texte = parse_macro_input!(entree as LitStr);
    let sql = texte.value();
    let majuscules = sql.trim_start().to_uppercase();

    if !majuscules.starts_with("SELECT") {
        return syn::Error::new(texte.span(), "requete! n'accepte que des SELECT")
            .to_compile_error()
            .into();
    }
    if sql.contains(';') {
        return syn::Error::new(texte.span(), "une seule instruction, sans `;`")
            .to_compile_error()
            .into();
    }

    quote!(#texte).into()                      // le texte, désormais vérifié
}
```

```rust
let q = requete!("SELECT id, email FROM utilisateurs WHERE id = $1");
// let r = requete!("DELETE FROM utilisateurs");   // ERREUR à la compilation, soulignée sur le texte
```

La vérification n'a lieu **qu'une fois**, à la compilation : aucun coût à l'exécution. C'est le principe de `sqlx::query!`, qui va plus loin en interrogeant une vraie base pendant la compilation pour vérifier les colonnes et les types.

Attention : une telle macro valide la **forme** de la requête, pas sa sûreté. Les valeurs venant de l'utilisateur passent toujours par des paramètres (`$1`), jamais par une chaîne assemblée.

---

## 6. Exemple : un DSL de configuration

Avec les analyseurs de la section 4, la macro `config!` génère une struct par section et un constructeur :

```rust
#[proc_macro]
pub fn config(entree: TokenStream) -> TokenStream {
    let cfg = parse_macro_input!(entree as Config);

    let structs = cfg.sections.iter().map(|s| {
        let nom_struct = quote::format_ident!("{}", majuscule(&s.nom.to_string()));
        let champs = s.reglages.iter().map(|(n, v)| {
            let ty = match v {
                Valeur::Texte(_) => quote!(&'static str),
                Valeur::Entier(_) => quote!(i64),
                Valeur::Booleen(_) => quote!(bool),
            };
            quote!(pub #n: #ty)
        });
        let valeurs = s.reglages.iter().map(|(n, v)| {
            let lit = match v {
                Valeur::Texte(l) => quote!(#l),
                Valeur::Entier(l) => quote!(#l),
                Valeur::Booleen(l) => quote!(#l),
            };
            quote!(#n: #lit)
        });
        quote! {
            #[derive(Debug, Clone)]
            pub struct #nom_struct { #(#champs),* }
            impl #nom_struct {
                pub const fn defaut() -> Self { Self { #(#valeurs),* } }
            }
        }
    });

    quote!(#(#structs)*).into()
}

fn majuscule(s: &str) -> String {
    let mut c = s.chars();
    c.next().map(|p| p.to_uppercase().collect::<String>() + c.as_str()).unwrap_or_default()
}
```

```rust
config! {
    serveur { port: 8080, hote: "0.0.0.0" }
    base { connexions: 10, ssl: true }
}

const SERVEUR: Serveur = Serveur::defaut();     // Serveur { port: 8080, hote: "0.0.0.0" }
```

Un nom de struct fabriqué à partir d'un nom de section (`serveur` → `Serveur`) : impossible avec `macro_rules!`.

---

## 7. Analyser des structures complexes

### 7.1 Éléments facultatifs

```rust
// nom: type [option1, option2]     les crochets sont facultatifs
struct Champ {
    nom: Ident,
    ty: Ident,
    options: Vec<Ident>,
}

impl Parse for Champ {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let nom = input.parse()?;
        input.parse::<Token![:]>()?;
        let ty = input.parse()?;
        let options = if input.peek(syn::token::Bracket) {        // crochets présents ?
            let contenu;
            syn::bracketed!(contenu in input);
            Punctuated::<Ident, Token![,]>::parse_terminated(&contenu)?.into_iter().collect()
        } else {
            Vec::new()
        };
        Ok(Champ { nom, ty, options })
    }
}
```

### 7.2 Clés dans n'importe quel ordre, chacune une seule fois

```rust
struct Meta { table: Option<syn::LitStr>, ordre: Option<syn::LitStr> }

impl Parse for Meta {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut m = Meta { table: None, ordre: None };
        while !input.is_empty() {
            let cle: Ident = input.parse()?;
            input.parse::<Token![:]>()?;
            let valeur: syn::LitStr = input.parse()?;
            let emplacement = match cle.to_string().as_str() {
                "table" => &mut m.table,
                "ordre" => &mut m.ordre,
                _ => return Err(syn::Error::new(cle.span(), "clé inconnue : `table` ou `ordre`")),
            };
            if emplacement.replace(valeur).is_some() {
                return Err(syn::Error::new(cle.span(), format!("`{cle}` est déjà défini")));
            }
            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }
        Ok(m)
    }
}
```

`Option::replace` renvoie l'ancienne valeur : si elle existait, la clé est en double.

### 7.3 Réutiliser les analyseurs de `syn`

Inutile de réécrire l'analyse d'un type ou d'une expression : `input.parse::<syn::Type>()`, `input.parse::<syn::Expr>()`, `input.parse::<syn::Path>()` acceptent toute la syntaxe Rust correspondante, génériques compris.

### 7.4 Tester l'analyseur sans macro

```rust
#[test]
fn refuse_une_cle_en_double() {
    let r: syn::Result<Meta> = syn::parse_str(r#"table: "a", table: "b""#);
    assert!(r.is_err());
}
```

`syn::parse_str` et `syn::parse2` permettent de tester `Parse` comme du code ordinaire, si l'analyseur vit dans une crate normale (section 10).

---

## 8. Générer du code

### 8.1 Plusieurs éléments

Une macro function-like peut produire **n'importe quoi** de valide à l'endroit de l'appel : une expression, des instructions, ou une suite d'éléments (structs, `impl`, modules, constantes).

```rust
quote! {
    pub mod utilisateur {
        pub struct Model { /* ... */ }
        pub enum Column { Id, Email }
        pub const TABLE: &str = "utilisateurs";
        impl Model { /* ... */ }
    }
}
```

### 8.2 Expression ou éléments

```rust
let x = ma_macro!(...);       // doit produire une expression
ma_macro! { ... }             // en position d'élément : peut produire des déclarations
```

Une macro conçue pour les deux usages ne marche que dans l'un : décide lequel, et documente-le.

### 8.3 Positions

Le code généré prend par défaut la position de l'appel. Pour qu'une erreur dans un morceau précis souligne le code de l'utilisateur, utilise `quote_spanned!` avec la position du jeton concerné (voir `macros-attribut`, section 7).

---

## 9. Ce qu'une macro ne doit pas faire

### 9.1 Dépendre de fichiers ou de l'environnement sans le dire

Une macro peut lire un fichier ou une variable d'environnement pendant la compilation, mais :

- Cargo ne sait pas que la macro dépend de ce fichier : s'il change, la crate **n'est pas recompilée** ;
- le résultat dépend de la machine qui compile (CI, poste du développeur, serveur) ;
- `cargo check` et rust-analyzer exécutent la macro en permanence : un accès réseau ou base à chaque frappe ralentit tout.

```rust
// Ce que la macro reçoit est la seule entrée fiable et reproductible
// let contenu = std::fs::read_to_string("schema.txt");   // à éviter
```

Si c'est indispensable, préfère générer `include_str!("chemin")` dans le code produit : c'est alors le compilateur qui lit le fichier, et il suit ses modifications.

### 9.2 Paniquer

Toujours une `syn::Error` positionnée, jamais `panic!` ni `unwrap()` sur une entrée de l'utilisateur.

### 9.3 Cacher trop de choses

Une macro qui génère des centaines de lignes rend les erreurs difficiles à comprendre. Le code généré doit rester **visible** (`cargo expand`) et **documenté** : quels types, quelles fonctions, quels noms apparaissent.

---

## 10. Exemple dans Runique : `model!{}` et `extend!{}`

### 10.1 La syntaxe

```rust
// demo-app/src/entities/chapitre.rs
model! {
    Chapitre,
    table: "chapitre",
    pk: id => Pk,
    {
        cour_id:    Pk [required],
        slug:       text [required],
        title:      text [required],
        lead:       text [nullable],
        sort_order: int [required],
    },
    relations: {
        belongs_to: Cour via cour_id [cascade],
    }
}
```

On y retrouve les techniques de ce cours : un mot-clé suivi de `:` (`table:`, `pk:`, `relations:`), un groupe `{ ... }` de champs, des listes facultatives entre crochets (`[required]`, `[cascade]`), des mots-clés personnels (`via`, `belongs_to`).

### 10.2 Le découpage en trois crates

```
model! { ... }
   │  jetons
   ▼
runique_dsl      analyse le DSL → arbre (AST)          crate normale
   │
   ├──► derive_form   génère l'entité SeaORM, les colonnes, le formulaire   (proc-macro)
   │
   └──► runique (CLI) makemigrations : compare l'arbre au schéma et écrit les migrations
```

L'analyseur est dans `runique_dsl`, une crate **ordinaire** : une crate `proc-macro` ne peut rien exporter d'autre que des macros, et la commande `runique makemigrations` doit lire exactement les mêmes déclarations que la macro. Le même `impl Parse` sert donc à la compilation (génération du code) et à la CLI (génération des migrations SQL).

Autre avantage : l'analyseur se teste comme du code ordinaire (section 7.4). `runique_dsl` a des tests de propriétés (`proptest`) qui lui envoient des entrées générées : il doit répondre par un arbre ou une erreur, **jamais** paniquer.

### 10.3 `extend!{}` : valider à la compilation

```rust
// runique/derive_form/src/lib.rs (extrait)
#[proc_macro]
pub fn extend(input: TokenStream) -> TokenStream {
    const FRAMEWORK_TABLES: &[&str] = &["eihwaz_users", "eihwaz_groupes"];

    let dsl = match syn::parse::<extend_schema::ExtendDsl>(input) {
        Ok(d) => d,
        Err(e) => return e.to_compile_error().into(),
    };

    if !FRAMEWORK_TABLES.contains(&dsl.table.as_str()) {
        let tables_list = FRAMEWORK_TABLES.join(", ");
        let msg = format!(
            "extend!{{}}: \"{}\" is not a known framework table. Allowed: {}",
            dsl.table, tables_list
        );
        return quote::quote! { compile_error!(#msg); }.into();
    }
    // ...
}
```

`extend!{}` ajoute des colonnes aux tables du framework. Une table inconnue est refusée **à la compilation**, avec la liste des tables autorisées : la faute de frappe ne peut pas atteindre une migration.

### 10.4 Les gardes de features dans la crate de macros

```rust
// runique/derive_form/src/lib.rs (début)
#[cfg(all(not(doc), not(feature = "all-databases"), feature = "postgres", feature = "mysql"))]
compile_error!("features `postgres` and `mysql` are mutually exclusive");
```

La crate de macros reçoit les mêmes features que `runique` (transmises par `runique/Cargo.toml` : `postgres = [..., "derive_form/postgres"]`). Elle connaît ainsi le moteur de base à la compilation et applique les mêmes règles d'exclusion (voir `cargo-dependances`, section 9).

L'ancien mécanisme lisait le moteur dans `.env` (`DB_ENGINE`, `DATABASE_URL`) pendant la compilation, avec exactement les défauts de la section 9.1 : un `.env` modifié ne déclenchait pas de recompilation, et un fichier oublié pouvait désigner un autre moteur que celui réellement compilé. Il reste en repli, seulement pour les projets dont le `Cargo.toml` ne transmet pas encore la feature (`runique/derive_form/src/model/utils/db_engine.rs`) ; la feature, elle, ne peut pas diverger de ce qui est compilé.

---

## 11. Comparaison des trois sortes de macros procédurales

| | Derive | Attribut | Function-like |
|---|---|---|---|
| Déclaration | `#[proc_macro_derive(Nom)]` | `#[proc_macro_attribute]` | `#[proc_macro]` |
| Signature | `fn(item) -> TokenStream` | `fn(attr, item) -> TokenStream` | `fn(entree) -> TokenStream` |
| Utilisation | `#[derive(Nom)]` sur un type | `#[nom(...)]` sur un élément | `nom!(...)` n'importe où |
| Reçoit | un type valide (struct, enum, union) | un élément Rust valide + des arguments | n'importe quels jetons |
| Produit | du code ajouté | le remplacement de l'élément | tout ce qui est valide à l'endroit |
| Modifie le code existant | non | oui | il n'y a pas de code existant |
| Exemples | `Serialize`, `Debug`, `Error` | `#[tokio::main]`, `#[form]`, `#[async_trait]` | `sqlx::query!`, `model!{}`, `html!` |

Choix :

- ajouter un comportement à un type → **derive** ;
- modifier un élément Rust existant (fonction, struct) → **attribut** ;
- une syntaxe qui n'est pas du Rust → **function-like**.

---

## 12. Exercices

### Exercice 1 — `couleur!`

Écris `couleur!("#ff8800")` qui produit `(255u8, 136u8, 0u8)` et refuse à la compilation un texte qui n'est pas une couleur hexadécimale de 6 chiffres.

```rust
// Solution :
#[proc_macro]
pub fn couleur(entree: TokenStream) -> TokenStream {
    let lit = parse_macro_input!(entree as syn::LitStr);
    let v = lit.value();
    let hex = v.strip_prefix('#').unwrap_or(&v);
    if hex.len() != 6 || !hex.chars().all(|c| c.is_ascii_hexdigit()) {
        return syn::Error::new(lit.span(), "attendu : \"#rrggbb\"").to_compile_error().into();
    }
    let canal = |i: usize| u8::from_str_radix(&hex[i..i + 2], 16).unwrap();   // vérifié ci-dessus
    let (r, g, b) = (canal(0), canal(2), canal(4));
    quote!((#r, #g, #b)).into()
}
```

L'`unwrap` est sûr ici : la ligne précédente a vérifié les six chiffres hexadécimaux.

### Exercice 2 — `enum_texte!`

Écris `enum_texte! { Statut: brouillon, publie, archive }` qui génère `enum Statut { Brouillon, Publie, Archive }` avec `fn as_str(&self) -> &'static str` renvoyant le nom en minuscules.

```rust
// Solution :
struct EnumTexte { nom: syn::Ident, variantes: Vec<syn::Ident> }

impl Parse for EnumTexte {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let nom = input.parse()?;
        input.parse::<Token![:]>()?;
        let v = Punctuated::<syn::Ident, Token![,]>::parse_terminated(input)?;
        Ok(EnumTexte { nom, variantes: v.into_iter().collect() })
    }
}

#[proc_macro]
pub fn enum_texte(entree: TokenStream) -> TokenStream {
    let e = parse_macro_input!(entree as EnumTexte);
    let nom = &e.nom;
    let variantes: Vec<_> = e.variantes.iter()
        .map(|v| quote::format_ident!("{}", majuscule(&v.to_string())))
        .collect();
    let textes: Vec<String> = e.variantes.iter().map(|v| v.to_string()).collect();
    quote! {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum #nom { #(#variantes),* }
        impl #nom {
            pub fn as_str(&self) -> &'static str {
                match self { #( Self::#variantes => #textes ),* }
            }
        }
    }
    .into()
}
```

### Exercice 3 — clé en double

Teste, sans macro, que l'analyseur de la section 7.2 refuse `ordre: "a", ordre: "b"`.

```rust
// Solution :
#[test]
fn ordre_en_double() {
    let r: syn::Result<Meta> = syn::parse_str(r#"ordre: "a", ordre: "b""#);
    let e = r.err().expect("doit échouer");
    assert!(e.to_string().contains("déjà défini"));
}
```

## 13. Aide-mémoire

```rust
#[proc_macro]
pub fn nom(entree: TokenStream) -> TokenStream {
    let dsl = parse_macro_input!(entree as MonDsl);   // impl Parse for MonDsl
    generer(&dsl).unwrap_or_else(|e| e.to_compile_error()).into()
}

input.parse::<Ident>()?        input.parse::<Token![:]>()?      input.parse::<LitStr>()?
syn::custom_keyword!(table);   input.parse::<kw::table>()?
input.peek(Token![,])          input.peek(syn::token::Bracket)  input.is_empty()
let l = input.lookahead1(); if l.peek(LitStr) { .. } else { Err(l.error()) }
let c; braced!(c in input);    bracketed!   parenthesized!
Punctuated::<T, Token![,]>::parse_terminated(&c)?
syn::Error::new(lit.span(), "..")   syn::parse_str::<T>("..")   format_ident!("{}", x)
```

### Points clés

- Une macro function-like procédurale sert quand il faut calculer, fabriquer des noms ou lire une syntaxe libre.
- Écris d'abord des exemples de la syntaxe, puis un `impl Parse` par élément.
- `lookahead1` produit des messages « attendu A ou B » ; `Option::replace` détecte les doublons.
- Valider à la compilation coûte zéro à l'exécution, mais ne remplace pas les paramètres de requête.
- La macro ne doit dépendre que de ses jetons : pas de fichier, de réseau ni d'environnement cachés.
- Mets l'analyseur dans une crate normale pour le tester et le réutiliser (comme `runique_dsl`).
