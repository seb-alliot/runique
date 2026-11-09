# Procedural macros — Derive
> Write your own `#[derive(...)]`: a `proc-macro` crate, `syn` to read the struct, `quote` to write the code, and the techniques to handle generics, attributes and errors.

## Objectives

- Understand what a procedural macro is and how it differs from `macro_rules!`
- Create and organize a `proc-macro` crate
- Read a struct or an enum with `syn`
- Generate code with `quote!`, its interpolations and its repetitions
- Handle named fields, tuples, enums and generics
- Read helper attributes (`#[my_attr(...)]`) on the struct and its fields
- Produce precise compile errors at the right position
- Test a procedural macro

## Table of contents

1. What a derive macro is
2. The three kinds of procedural macros
3. Structure of a `proc-macro` crate
4. `TokenStream`, `syn`, `quote`
5. A first complete derive
6. Reading the fields
7. Generating with `quote!`: interpolations and repetitions
8. Generics
9. Enums
10. Helper attributes
11. Compile errors
12. Paths in the generated code
13. Testing a procedural macro
14. Organizing a library with macros
15. Example in Runique
16. Derive, `macro_rules!` or hand-written code
17. Exercises
18. Cheat sheet

---

## 1. What a derive macro is

```rust
#[derive(Debug, Clone)]
struct Point { x: i32, y: i32 }
```

`#[derive(Debug)]` calls a **procedural macro**: a Rust program, run by the compiler, that receives the definition of `Point` and **adds** the `Debug` implementation next to it. The struct itself is not modified.

| | `macro_rules!` | Procedural macro |
|---|---|---|
| Written in | syntax patterns | ordinary Rust code |
| Can analyze arbitrary code | with difficulty | yes (with `syn`) |
| Can build identifiers (`get_` + name) | no | yes |
| Location | anywhere | a dedicated crate, of type `proc-macro` |
| Compile cost | low | `syn` and `quote` to compile, macro run on every build |

---

## 2. The three kinds of procedural macros

| Kind | Use | What it receives | What it produces |
|---|---|---|---|
| derive | `#[derive(MyMacro)]` | the struct, enum or union | code **added** after it (usually an `impl`) |
| attribute | `#[my_macro(args)]` | the arguments and the annotated item | code that **replaces** the item (course `macros-attribut`) |
| function-like | `my_macro!(...)` | the tokens between parentheses | any code (course `macros-function-like`) |

A derive is the most restricted: it can **modify nothing**, only add. It is also the most predictable for the user.

---

## 3. Structure of a `proc-macro` crate

### 3.1 Creating the crate

```bash
cargo new describe_macros --lib
```

```toml
# describe_macros/Cargo.toml
[package]
name = "describe_macros"
version = "0.1.0"
edition = "2024"

[lib]
proc-macro = true            # required: makes the crate a macro crate

[dependencies]
syn = { version = "2", features = ["full"] }   # Rust code parsing
quote = "1"                                    # code generation
proc-macro2 = "1"                              # TokenStream usable outside the macro (tests)
```

### 3.2 The constraints of a `proc-macro` crate

- It can export **only** procedural macros: no public function, struct or trait.
- It is compiled for the machine that compiles (not for the target), and run during compilation.
- The types and traits used by the generated code must therefore live in **another crate** (section 14).

### 3.3 Using the macro

```toml
# my_app/Cargo.toml
[dependencies]
describe_macros = { path = "../describe_macros" }
```

```rust
use describe_macros::Describe;

#[derive(Describe)]
struct User { name: String, age: u32 }
```

---

## 4. `TokenStream`, `syn`, `quote`

```
source code ──► TokenStream ──syn──► tree (DeriveInput) ──you──► quote! ──► TokenStream ──► compiler
```

- `proc_macro::TokenStream`: a sequence of tokens, what the compiler gives and takes back.
- `syn`: turns tokens into a typed tree (`DeriveInput`, `Field`, `Type`, `Expr`...).
- `quote!`: writes Rust code with "holes" (`#name`) filled with values, and produces tokens.
- `proc_macro2`: the same thing as `proc_macro`, but usable outside a macro (unit tests); `syn` and `quote` work with it.

### The entry point

```rust
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Describe)]
pub fn derive_describe(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);   // syntax error -> compile error
    let name = &ast.ident;

    let code = quote! {
        impl #name {
            pub fn describe() -> &'static str {
                stringify!(#name)
            }
        }
    };

    code.into()                                           // proc_macro2::TokenStream -> proc_macro::TokenStream
}
```

### What a `DeriveInput` contains

```rust
pub struct DeriveInput {
    pub attrs: Vec<Attribute>,     // #[...] placed on the type
    pub vis: Visibility,           // pub, pub(crate)...
    pub ident: Ident,              // the name of the type
    pub generics: Generics,        // <T, 'a, const N: usize> and where
    pub data: Data,                // Struct, Enum or Union, with the fields
}
```

---

## 5. A first complete derive

A `Describe` derive that adds a method `describe(&self) -> String` listing the fields and their values.

```rust
// describe_macros/src/lib.rs
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

#[proc_macro_derive(Describe)]
pub fn derive_describe(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let name = &ast.ident;

    let Data::Struct(data) = &ast.data else {
        return syn::Error::new_spanned(&ast.ident, "Describe only applies to structs")
            .to_compile_error()
            .into();
    };

    let Fields::Named(fields) = &data.fields else {
        return syn::Error::new_spanned(&ast.ident, "Describe requires named fields")
            .to_compile_error()
            .into();
    };

    let names: Vec<_> = fields.named.iter().map(|c| c.ident.as_ref().unwrap()).collect();

    quote! {
        impl #name {
            pub fn describe(&self) -> String {
                let mut parts = Vec::new();
                #(
                    parts.push(format!("{} = {:?}", stringify!(#names), self.#names));
                )*
                format!("{} {{ {} }}", stringify!(#name), parts.join(", "))
            }
        }
    }
    .into()
}
```

```rust
// my_app
#[derive(Describe)]
struct User { name: String, age: u32 }

fn main() {
    let u = User { name: "Ada".into(), age: 36 };
    println!("{}", u.describe());     // User { name = "Ada", age = 36 }
}
```

The generated code (visible with `cargo expand`):

```rust
impl User {
    pub fn describe(&self) -> String {
        let mut parts = Vec::new();
        parts.push(format!("{} = {:?}", "name", self.name));
        parts.push(format!("{} = {:?}", "age", self.age));
        format!("{} {{ {} }}", "User", parts.join(", "))
    }
}
```

---

## 6. Reading the fields

### 6.1 The three forms of struct

```rust
use syn::{Data, Fields};

fn describe_shape(data: &Data) -> &'static str {
    match data {
        Data::Struct(s) => match &s.fields {
            Fields::Named(_) => "struct with named fields",    // struct S { a: i32 }
            Fields::Unnamed(_) => "tuple struct",              // struct S(i32, String);
            Fields::Unit => "unit struct",                     // struct S;
        },
        Data::Enum(_) => "enum",
        Data::Union(_) => "union",
    }
}
```

### 6.2 What a field contains

```rust
pub struct Field {
    pub attrs: Vec<Attribute>,     // #[...] on the field
    pub vis: Visibility,
    pub ident: Option<Ident>,      // None for a tuple field
    pub ty: Type,                  // the type of the field
    // ...
}
```

### 6.3 Tuple fields: by position

```rust
use syn::Index;

let indices: Vec<Index> = (0..fields.unnamed.len()).map(Index::from).collect();
// quote! { #( self.#indices ),* }     ->   self.0, self.1, self.2
```

An interpolated `usize` would give `0usize`; `syn::Index` gives `0`, the correct form after a dot.

### 6.4 Filtering by type

```rust
use syn::{Type, TypePath};

fn is_option(ty: &Type) -> bool {
    matches!(ty, Type::Path(TypePath { path, .. })
        if path.segments.last().is_some_and(|s| s.ident == "Option"))
}
```

A macro only sees the **syntax**: it knows that a field is written `Option<String>`, not what that type really is. An alias (`type MaybeName = Option<String>`) or a full path (`std::option::Option`) are only recognized if you handle them explicitly.

---

## 7. Generating with `quote!`: interpolations and repetitions

### 7.1 Interpolating

```rust
let name = &ast.ident;           // Ident
let ty: syn::Type = syn::parse_quote!(u32);
let n = 42usize;
let text = "hello";

let code = quote! {
    const N: usize = #n;         // 42usize
    fn f() -> #ty { 0 }          // fn f() -> u32
    fn g() -> &'static str { #text }    // "hello" (a string literal)
    impl #name {}
};
```

Any type that implements `ToTokens` can be interpolated: `Ident`, `Type`, `Expr`, `TokenStream`, integers, `&str`, `bool`...

### 7.2 Repeating

```rust
let names = vec![quote!(a), quote!(b), quote!(c)];
let types = vec![quote!(i32), quote!(String), quote!(bool)];

quote! {
    struct S { #( #names: #types ),* }        // a: i32, b: String, c: bool
};

quote! {
    #( println!("{}", stringify!(#names)); )*   // one line per element, no separator
};
```

`#( ... ),*` repeats with a separator, `#( ... )*` without. All the variables in the same repetition must have the same length.

### 7.3 Building an identifier

```rust
use quote::format_ident;

let field = format_ident!("email");
let getter = format_ident!("get_{}", field);        // get_email
let setter = format_ident!("set_{}", field);        // set_email

quote! {
    pub fn #getter(&self) -> &str { &self.#field }
    pub fn #setter(&mut self, v: String) { self.#field = v; }
};
```

This is what `macro_rules!` cannot do.

### 7.4 Building the code in pieces

```rust
let mut methods = proc_macro2::TokenStream::new();
for c in fields.named.iter() {
    let name = c.ident.as_ref().unwrap();
    let ty = &c.ty;
    methods.extend(quote! {
        pub fn #name(&self) -> &#ty { &self.#name }
    });
}

quote! {
    impl #type_name { #methods }
};
```

---

## 8. Generics

### 8.1 The problem

```rust
#[derive(Describe)]
struct Wrapper<T> { value: T }

// An impl written as `impl #name { ... }` gives `impl Wrapper { ... }`: ERROR, Wrapper expects a T
```

### 8.2 `split_for_impl`

```rust
let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

quote! {
    impl #impl_generics #name #ty_generics #where_clause {
        // ...
    }
}
// impl<T> Wrapper<T> { ... }
// impl<'a, T: Clone, const N: usize> S<'a, T, N> where T: Debug { ... }
```

`split_for_impl` correctly separates lifetimes, type parameters, const generics and the `where` clause.

### 8.3 Adding a bound

The derive from section 5 calls `{:?}` on each field: `T` must be `Debug`.

```rust
use syn::{parse_quote, GenericParam};

let mut generics = ast.generics.clone();
for param in &mut generics.params {
    if let GenericParam::Type(t) = param {
        t.bounds.push(parse_quote!(::std::fmt::Debug));   // T becomes T: Debug
    }
}
let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
```

This is what the standard library's `#[derive(Debug)]` does: it adds `T: Debug` to each parameter.

---

## 9. Enums

```rust
use syn::{Data, Fields};

#[proc_macro_derive(VariantName)]
pub fn derive_variant_name(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let name = &ast.ident;

    let Data::Enum(e) = &ast.data else {
        return syn::Error::new_spanned(name, "VariantName only applies to enums")
            .to_compile_error()
            .into();
    };

    let arms = e.variants.iter().map(|v| {
        let variant = &v.ident;
        let text = variant.to_string();
        let pattern = match &v.fields {
            Fields::Unit => quote!(Self::#variant),
            Fields::Unnamed(_) => quote!(Self::#variant(..)),
            Fields::Named(_) => quote!(Self::#variant { .. }),
        };
        quote!(#pattern => #text)
    });

    let (ig, tg, wc) = ast.generics.split_for_impl();
    quote! {
        impl #ig #name #tg #wc {
            pub fn variant_name(&self) -> &'static str {
                match self { #(#arms),* }
            }
        }
    }
    .into()
}
```

```rust
#[derive(VariantName)]
enum Message { Quit, Write(String), Move { x: i32, y: i32 } }

Message::Write("a".into()).variant_name();    // "Write"
```

The pattern depends on the shape of each variant: `Self::A`, `Self::A(..)` or `Self::A { .. }`.

---

## 10. Helper attributes

### 10.1 Declaring them

```rust
#[proc_macro_derive(Describe, attributes(describe))]
pub fn derive_describe(input: TokenStream) -> TokenStream { /* ... */ }
```

`attributes(describe)` lets the user write `#[describe(...)]` on the type and on its fields. Without this declaration, the compiler would reject the unknown attribute.

```rust
#[derive(Describe)]
#[describe(name = "Account")]
struct User {
    name: String,
    #[describe(hidden)]
    password: String,
    #[describe(rename = "years")]
    age: u32,
}
```

### 10.2 Reading them with `parse_nested_meta`

```rust
struct FieldOptions {
    hidden: bool,
    rename: Option<String>,
}

fn read_options(field: &syn::Field) -> syn::Result<FieldOptions> {
    let mut opts = FieldOptions { hidden: false, rename: None };
    for attr in &field.attrs {
        if !attr.path().is_ident("describe") {
            continue;                                  // another attribute (serde, doc...): ignored
        }
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("hidden") {
                opts.hidden = true;                    // #[describe(hidden)]
                Ok(())
            } else if meta.path.is_ident("rename") {
                let value: syn::LitStr = meta.value()?.parse()?;   // #[describe(rename = "...")]
                opts.rename = Some(value.value());
                Ok(())
            } else {
                Err(meta.error("unknown option: expected `hidden` or `rename = \"...\"`"))
            }
        })?;
    }
    Ok(opts)
}
```

### 10.3 Using them in the generation

```rust
let mut lines = Vec::new();
for c in fields.named.iter() {
    let opts = match read_options(c) {
        Ok(o) => o,
        Err(e) => return e.to_compile_error().into(),
    };
    if opts.hidden {
        continue;                                      // the password never appears
    }
    let ident = c.ident.as_ref().unwrap();
    let label = opts.rename.unwrap_or_else(|| ident.to_string());
    lines.push(quote! {
        parts.push(format!("{} = {:?}", #label, self.#ident));
    });
}
```

A `hidden` attribute that removes a secret field from the output is a good example of use: it is the kind of mistake an ordinary `#[derive(Debug)]` cannot avoid.

### 10.4 With the `darling` crate

For many attributes, the `darling` crate declares the options as a struct (`#[derive(FromField)] #[darling(attributes(describe))] struct FieldOptions { #[darling(default)] hidden: bool, rename: Option<String> }`) and generates the reading and the error messages.

---

## 11. Compile errors

### 11.1 Never panic

```rust
// Bad: the user sees "proc-macro derive panicked" with no position
let fields = match &ast.data { Data::Struct(s) => &s.fields, _ => panic!("struct expected") };

// Good: a compile error, underlined in the right place
let Data::Struct(s) = &ast.data else {
    return syn::Error::new_spanned(&ast.ident, "struct expected").to_compile_error().into();
};
```

### 11.2 Choosing the position

| Construction | What is underlined |
|---|---|
| `syn::Error::new_spanned(&node, "...")` | the code of `node` (a field, a type, an attribute) |
| `syn::Error::new(ident.span(), "...")` | a precise position |
| `syn::Error::new(Span::call_site(), "...")` | the macro call (the `#[derive]`) |

Underline the **faulty** element (the field with the unsupported type, the badly written attribute), not the whole struct.

### 11.3 Several errors at once

```rust
let mut errors: Option<syn::Error> = None;
for c in fields.named.iter() {
    if let Err(e) = read_options(c) {
        match &mut errors {
            Some(all) => all.combine(e),              // accumulate
            None => errors = Some(e),
        }
    }
}
if let Some(e) = errors {
    return e.to_compile_error().into();               // all the errors in one compilation
}
```

### 11.4 Structuring with `syn::Result`

```rust
#[proc_macro_derive(Describe, attributes(describe))]
pub fn derive_describe(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    generate(&ast).unwrap_or_else(|e| e.to_compile_error()).into()
}

fn generate(ast: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    // ... ? everywhere ...
    Ok(quote! {})
}
```

The entry point only converts; all the work is in a function that uses `?`.

---

## 12. Paths in the generated code

The generated code is compiled **in the user's crate**, with the user's imports.

```rust
// Bad: depends on the user's imports
quote! { impl Display for #name { fn fmt(&self, f: &mut Formatter) -> Result { .. } } };

// Good: absolute paths
quote! {
    impl ::std::fmt::Display for #name {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result { /* .. */ Ok(()) }
    }
};
```

A procedural macro has no `$crate`. For the types of your own library, the generated code writes `::my_lib::Trait`: the user must therefore depend on `my_lib` under that name (see `macros-export`, section 7.3).

---

## 13. Testing a procedural macro

### 13.1 Usage test

```rust
// my_app/tests/describe.rs (or in the crate that re-exports the macro)
use describe_macros::Describe;

#[derive(Describe)]
struct Point { x: i32, y: i32 }

#[test]
fn describes_the_fields() {
    assert_eq!(Point { x: 1, y: 2 }.describe(), "Point { x = 1, y = 2 }");
}
```

A `proc-macro` crate cannot use its own macros in its unit tests: you test them from another crate (integration tests of the library that re-exports it, or an examples folder).

### 13.2 Testing errors with `trybuild`

```rust
// tests/errors.rs
#[test]
fn compile_errors() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/*.rs");      // each file must fail with the message in the neighboring .stderr
    t.pass("tests/ok/*.rs");
}
```

`trybuild` compiles small files and compares the error message obtained to an expected `.stderr` file: this checks that the user gets **the right message, in the right place**.

### 13.3 Testing the logic without compiling

The analysis and generation logic can live in a function that takes a `proc_macro2::TokenStream`: it can then be tested like ordinary code, without a macro.

```rust
fn generate2(input: proc_macro2::TokenStream) -> syn::Result<proc_macro2::TokenStream> {
    let ast: DeriveInput = syn::parse2(input)?;
    Ok(quote! { /* ... */ })
}

#[test]
fn rejects_an_enum() {
    let r = generate2(quote! { enum E { A } });
    assert!(r.is_err());
}
```

---

## 14. Organizing a library with macros

A `proc-macro` crate can only export macros. A library that provides some is therefore usually split like this:

```
my_lib/            ← the library: types, traits, functions; re-exports the macros
my_lib_macros/     ← proc-macro = true: only the macros
my_lib_dsl/        ← (optional) the parser, in a normal crate, reusable elsewhere
```

```rust
// my_lib/src/lib.rs
pub trait Describe { fn describe(&self) -> String; }   // the trait lives here
pub use my_lib_macros::Describe;                       // the derive macro of the same name too
```

A trait and a derive macro can have the **same name** (`serde::Serialize` is both): they live in different namespaces.

Publication order on crates.io: first the crates the others depend on (parser, then macros, then library).

---

## 15. Example in Runique

Runique follows exactly the split from section 14:

```
runique/runique_dsl/   ← parser of the model!{} / extend!{} DSL -> tree (normal crate)
runique/derive_form/   ← proc-macro = true: model!{}, extend!{}, #[form]
runique/               ← the framework; re-exports the macros (use runique::prelude::*)
```

```toml
# runique/derive_form/Cargo.toml (excerpt)
[lib]
proc-macro = true

[dependencies]
runique_dsl.workspace = true
syn = { workspace = true, features = ["extra-traits"] }
quote = "1"
proc-macro2 = "1"
```

Why is the parser in a separate crate (`runique_dsl`)? Because a `proc-macro` crate cannot export anything but macros. The `runique makemigrations` command must read the same `model!{}` declarations as the macro to generate the migrations: the parser therefore lives in an ordinary crate, used by both `derive_form` and the CLI.

Runique's macros are not derives: `model!{}` and `extend!{}` are function-like macros, `#[form]` is an attribute macro. They use the same tools as this course (`parse_macro_input!`, `syn::Error`, `to_compile_error`, `quote!`). They are presented in the next two courses.

Note: Runique uses `syn` 3; the examples in this course follow the `syn` 2 API, the most common in current resources.

---

## 16. Derive, `macro_rules!` or hand-written code

Generate getters for a struct.

```rust
// 1. By hand: nothing to learn, but repetitive
struct A { name: String, age: u32 }
impl A {
    fn name(&self) -> &String { &self.name }
    fn age(&self) -> &u32 { &self.age }
}

// 2. macro_rules!: you must repeat the list of fields at the call
macro_rules! getters {
    ($t:ident { $($c:ident : $ty:ty),* }) => {
        impl $t { $( fn $c(&self) -> &$ty { &self.$c } )* }
    };
}
struct B { name: String, age: u32 }
getters!(B { name: String, age: u32 });

// 3. Derive: the macro reads the fields itself
// #[derive(Getters)]
// struct C { name: String, age: u32 }
```

| | By hand | `macro_rules!` | Derive |
|---|---|---|---|
| Initial effort | none | low | dedicated crate, `syn`, `quote` |
| The list of fields | written twice | written twice | written once |
| Built names (`get_name`) | by hand | impossible | `format_ident!` |
| Per-field attributes | — | no | yes |
| For | a few types | a local repetition | a library, many types |

---

## 17. Exercises

### Exercise 1 — `FieldNames`

Write a derive that adds `fn field_names() -> &'static [&'static str]`.

```rust
// Solution:
#[proc_macro_derive(FieldNames)]
pub fn derive_names(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let name = &ast.ident;
    let Data::Struct(s) = &ast.data else {
        return syn::Error::new_spanned(name, "struct expected").to_compile_error().into();
    };
    let names: Vec<String> = s.fields.iter()
        .filter_map(|c| c.ident.as_ref().map(|i| i.to_string()))
        .collect();
    let (ig, tg, wc) = ast.generics.split_for_impl();
    quote! {
        impl #ig #name #tg #wc {
            pub fn field_names() -> &'static [&'static str] { &[#(#names),*] }
        }
    }
    .into()
}
```

### Exercise 2 — setters with `format_ident!`

Add a `set_<field>` for each field, except those marked `#[setters(no)]`.

```rust
// Solution (excerpt):
let setters = fields.named.iter().filter(|c| !marked_no(c)).map(|c| {
    let field = c.ident.as_ref().unwrap();
    let ty = &c.ty;
    let name = quote::format_ident!("set_{}", field);
    quote! { pub fn #name(&mut self, v: #ty) { self.#field = v; } }
});

fn marked_no(c: &syn::Field) -> bool {
    c.attrs.iter().any(|a| {
        a.path().is_ident("setters")
            && a.parse_args::<syn::Ident>().is_ok_and(|i| i == "no")
    })
}
```

### Exercise 3 — error in the right place

Make your derive reject a field of type `f64`, underlining **that field**.

```rust
// Solution (excerpt):
for c in fields.named.iter() {
    let ty = &c.ty;
    if quote::quote!(#ty).to_string() == "f64" {
        return syn::Error::new_spanned(&c.ty, "f64 is not supported: use a Decimal")
            .to_compile_error()
            .into();
    }
}
```

`new_spanned(&c.ty, ...)` puts the underline on the type of the field in the user's code.

## 18. Cheat sheet

```toml
[lib]
proc-macro = true
[dependencies]
syn = { version = "2", features = ["full"] }
quote = "1"
proc-macro2 = "1"
```

```rust
#[proc_macro_derive(Name, attributes(name))]
pub fn f(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let name = &ast.ident;
    let (ig, tg, wc) = ast.generics.split_for_impl();
    quote! { impl #ig ::my_lib::Trait for #name #tg #wc { .. } }.into()
}

Data::Struct(s) / Data::Enum(e)       Fields::Named / Unnamed / Unit
#var   #( #a: #b ),*   #( .. )*        format_ident!("get_{}", x)   syn::Index::from(i)
attr.path().is_ident("name")          attr.parse_nested_meta(|meta| ..)
syn::Error::new_spanned(&node, "..").to_compile_error()      e.combine(other)
```

### Key points

- A derive adds code after the type, without modifying it.
- A `proc-macro` crate exports only macros: types and traits go in another crate.
- `syn` reads, `quote!` writes; `split_for_impl` for generics.
- `format_ident!` builds names, which `macro_rules!` cannot do.
- Declare helper attributes with `attributes(...)`, read them with `parse_nested_meta`.
- Never `panic!`: use `syn::Error::new_spanned` on the faulty element.
- Absolute paths in the generated code.
- Test usage from another crate, errors with `trybuild`.
