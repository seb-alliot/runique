## Créer une API Django-like en Rust

Extension ORM avec Traits, Génériques et Macros 

Framework Rusti - Documentation Complète 

### Objectif du cours

Comprendre comment créer une API Django-like en Rust pour avoir User::objects.filter() au lieu de la syntaxe verbeuse de SeaORM. 

## Table des matières

1. Le problème initial 

2. Les concepts Rust nécessaires 

- 2.1 - Les traits (interfaces) 

- 2.2 - Les génériques 

- 2.3 - PhantomData 

- 2.4 - const fn 

- 2.5 - Les macros 

3. Architecture de la solution 

4. Explication détaillée du code 

- 4.1 - objects.rs (le Manager) 

- 4.2 - query.rs (le QueryBuilder) 

- 4.3 - La macro impl_objects! 

- 4.4 - Into 

5. Exemples d'utilisation 

6. Exercices pratiques 

7. Résumé des concepts 

## 1. Le problème initial

Lorsqu'on utilise Django en Python, on a une syntaxe très intuitive pour les requêtes de base de données : 

```
# Django (Python) - Simple et intuitif
User.objects.filter(age__gte=18)
User.objects.exclude(status="banned")
User.objects.get(id=1)
```

En revanche, avec SeaORM en Rust, la syntaxe de base est plus verbeuse : 

```
// SeaORM (Rust) - Verbeux
User::find()
    .filter(user::Column::Age.gte(18))
    .all(&db)
    .await?
```

**Notre objectif :** Avoir la même syntaxe qu'en Django avec User::objects.filter() en Rust ! 

## 2. Les concepts Rust nécessaires

## 2.1 - Les traits (interfaces)

Un **trait** en Rust est similaire à une interface : c'est un ensemble de méthodes qu'un type peut implémenter. Les traits permettent d'ajouter des méthodes à des types existants. 

```
// Définir un trait
trait Parler {
    fn dire_bonjour(&self);
}
```

```
// Implémenter pour un type
struct Personne {
    nom: String,
}
```

```
impl Parler for Personne {
    fn dire_bonjour(&self) {
        println!("Bonjour, je suis {}", self.nom);
    }
}
```

```
// Utilisation
let p = Personne { nom: "Alice".to_string() };
p.dire_bonjour();  // "Bonjour, je suis Alice"
```

I **Pourquoi c'est important ?** Les traits permettent d'ajouter des méthodes à des types existants sans modifier leur code source ! 

## 2.2 - Les génériques

Les **génériques** permettent d'écrire du code qui fonctionne avec plusieurs types différents. 

```
// Sans générique (répétitif)
struct BoiteEntier { contenu: i32 }
struct BoiteString { contenu: String }
```

```
// Avec générique (réutilisable)
struct Boite<T> {
    contenu: T,
}
// Utilisation
let boite_int = Boite { contenu: 42 };
let boite_str = Boite { contenu: "Hello".to_string() };
```

```
// Avec contraintes (bounds)
fn afficher<T: std::fmt::Display>(valeur: T) {
    println!("Valeur: {}", valeur);
}
```

## 2.3 - PhantomData

`PhantomData<T>` permet de dire au compilateur "je possède un type T" **sans stocker de données réelles** . C'est un type fantôme de taille zéro. 

```
use std::marker::PhantomData;
```

```
struct Manager<E> {
    // On ne stocke PAS de E réellement
    // Mais on dit au compilateur qu'on "possède" un E
    _phantom: PhantomData<E>,
```

```
}
```

```
impl<E> Manager<E> {
    const fn new() -> Self {
        Self { _phantom: PhantomData }
    }
```

```
}
```

I **Avantages :** Le compilateur vérifie les types correctement, mais aucune donnée n'est stockée en mémoire (taille = 0 octets). 

## 2.4 - const fn

`const fn` définit une fonction qui peut être évaluée **à la compilation** plutôt qu'à l'exécution. 

```
const fn multiplier(x: i32) -> i32 {
    x * 2
```

```
}
```

```
// Calculé à la compilation !
const RESULTAT: i32 = multiplier(5);
```

```
// Pour notre cas :
pub const objects: Manager<Self> = Manager::new();
//    ^^^^^ constante, pas une fonction
```

I Cela permet de créer `objects` comme une **constante** , accessible sans parenthèses : `User::objects` 

## 2.5 - Les macros

Les **macros** permettent de générer du code automatiquement. Elles se terminent par un point d'exclamation `!` 

```
// Définir une macro
macro_rules! dire_bonjour {
    ($nom:expr) => {
        println!("Bonjour {}", $nom);
    };
}
```

```
// Utilisation
dire_bonjour!("Alice");
```

```
// Se transforme en :
println!("Bonjour {}", "Alice");
```

```
// Notre macro impl_objects! :
impl_objects!(User);
```

```
// Génère automatiquement :
impl User {
    pub const objects: Objects<Self> = Objects::new();
}
```

## 3. Architecture de la solution

Notre solution utilise trois composants principaux qui travaillent ensemble : 

**==> picture [328 x 211] intentionally omitted <==**

**----- Start of picture text -----**<br>
IIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIII<br>I   User (entité SeaORM)                    I<br>I   + impl_objects!(Entity)                 I<br>IIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIII<br>I<br>M<br>IIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIII<br>I   Objects<User>                           I<br>I   - Constante créée par la macro          I<br>I   - Méthodes: filter(), exclude(), etc.   I<br>IIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIII<br>I<br>M<br>IIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIII<br>I   RustiQueryBuilder<User>                 I<br>I   - Encapsule Select<User>                I<br>I   - Méthodes chainables                   I<br>IIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIII<br>I<br>M<br>IIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIII<br>I   SeaORM Select<User>                     I<br>I   - Query SQL réelle                      I<br>IIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIII<br>**----- End of picture text -----**<br>

## Flux de données :

1. **Macro** : Génère la constante `objects` pour chaque entité 

2. **Objects** : Point d'entrée (comme Django Manager), crée des QueryBuilder 

3. **QueryBuilder** : Permet le chaînage de méthodes 

4. **Select** : Query SeaORM réelle exécutée sur la base de données 

## 4. Explication détaillée du code

## 4.1 - objects.rs (le Manager)

Le fichier `objects.rs` contient la struct `Objects<E>` qui sert de point d'entrée pour toutes les requêtes. 

```
use std::marker::PhantomData;
```

```
// Struct générique qui fonctionne avec N'IMPORTE quelle entité
pub struct Objects<E: EntityTrait> {
    //                ^^^^^^^^^^^ E doit être une entité SeaORM
```

```
    _phantom: PhantomData<E>,
```

```
    // ^^^^^^^ On stocke le type E sans données réelles
}
```

```
impl<E: EntityTrait> Objects<E> {
    // Pour chaque type E qui implémente EntityTrait
```

```
    pub const fn new() -> Self {
```

```
        // const fn = peut être appelé à la compilation
        Self { _phantom: PhantomData }
    }
```

```
    pub fn filter<C>(&self, condition: C) -> RustiQueryBuilder<E>
    //            ^^ C peut être n'importe quoi convertible en Condition
    where
```

```
        C: Into<Condition>,
```

```
        // ^^^^^^^^^^^^^^ Contrainte : C doit pouvoir devenir Condition
    {
```

```
        // 1. Créer une query SeaORM
        let query = E::find();
```

```
        // 2. L'envelopper dans notre QueryBuilder
        // 3. Appliquer le filtre
        RustiQueryBuilder::new(query).filter(condition.into())
        //                                              ^^^^^^ Conversion auto
    }
```

```
}
```

I **Analogie :** `Objects<E>` est comme une **télécommande** pour contrôler `E` . 

## 4.2 - query.rs (le QueryBuilder)

Le `RustiQueryBuilder` encapsule la query SeaORM et permet de chaîner les méthodes. 

```
pub struct RustiQueryBuilder<E: EntityTrait> {
    select: Select<E>,  // La vraie query SeaORM
}
```

```
impl<E: EntityTrait> RustiQueryBuilder<E> {
    pub fn new(select: Select<E>) -> Self {
        Self { select }
    }
    // Méthode chainable
    pub fn filter<C>(mut self, condition: C) -> Self
    //           ^^^ Prend ownership
    where
        C: Into<Condition>,
    {
        // Modifier la query interne
        self.select = self.select.filter(condition.into());
        // Retourner self pour permettre le chaînage
        self
        // ^^^^ Rend ownership
    }
```

```
    // Méthode terminale (consomme self)
    pub async fn all(self, db: &DatabaseConnection)
        -> Result<Vec<E::Model>, DbErr>
    {
        //    ^^^^ Consomme self (pas de chaînage après)
        self.select.all(db).await
    }
}
```

I **Pattern Builder :** Les méthodes qui retournent `Self` sont **chainables** , celles qui consomment `self` sont **terminales** . 

```
// Chainable car retourne Self
query.filter(...).exclude(...).limit(10)
```

```
// Terminal car consomme self
```

```
     .all(&db).await
```

## 4.3 - La macro impl_objects!

La macro génère automatiquement la constante `objects` pour chaque entité. 

```
#[macro_export]
//^^^^^^^^^^^ La macro est disponible partout
macro_rules! impl_objects {
    //        ^^^^^^^^^^^^ Nom de la macro
```

```
    ($entity:ty) => {
    // ^^^^^^^ Paramètre : un type
```

```
        impl $entity {
        //   ^^^^^^^ Utilise le paramètre
```

```
            pub const objects: $crate::orm::Objects<Self>
                = $crate::orm::Objects::new();
            //  ^^^^^^
            //  Nom, Type générique, Création const
        }
    };
}
// Utilisation :
impl_objects!(Entity);
```

```
// Se transforme en :
impl Entity {
    pub const objects: rusti::orm::Objects<Self>
        = rusti::orm::Objects::new();
```

```
}
```

## 4.4 - Into : La conversion magique

Le trait `Into<Condition>` permet la conversion automatique des expressions SeaORM en conditions. 

```
pub fn filter<C>(&self, condition: C) -> RustiQueryBuilder<E>
where
```

```
    C: Into<Condition>,
    //^^^^^^^^^^^^^^^^ Le secret !
```

```
// SeaORM retourne Expr pour les comparaisons :
Column::Age.gte(18)  // Type: Expr
```

```
// Mais filter() attend Condition
```

```
// Into<Condition> permet la conversion automatique :
```

```
// L'utilisateur écrit :
```

```
.filter(Column::Age.gte(18))
```

```
// Rust convertit automatiquement :
```

```
.filter(Column::Age.gte(18).into())
```

```
//                          ^^^^^^ Ajouté automatiquement
```

## 5. Exemples d'utilisation

Une fois configuré, voici comment utiliser l'API : 

```
// 1. Dans ton entité SeaORM
use rusti::impl_objects;
```

```
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub username: String,
    pub age: i32,
}
```

```
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
```

```
impl ActiveModelBehavior for ActiveModel {}
```

**`//`** I **`Ajouter le support objects impl_objects!(Entity);`** 

```
// 2. Utilisation dans le code
```

```
// Tous les utilisateurs
let users = User::objects.all().all(&db).await?;
```

```
// Filtrer
let adults = User::objects
    .filter(user::Column::Age.gte(18))
    .all(&db)
    .await?;
// Exclure
let active = User::objects
    .exclude(user::Column::Status.eq("banned"))
    .all(&db)
    .await?;
```

```
// Get par ID
let user = User::objects.get(&db, 1).await?;
// Compter
let count = User::objects.count(&db).await?;
```

```
// Query complexe avec chaînage
let results = User::objects
    .filter(user::Column::Age.gte(18))
    .exclude(user::Column::Status.eq("banned"))
    .order_by_desc(user::Column::CreatedAt)
    .limit(10)
    .offset(20)
    .all(&db)
    .await?;
```

## 6. Exercices pratiques

Pour approfondir ta compréhension, voici quelques exercices : 

## Exercice 1 : Ajouter first()

Ajoute une méthode `first()` qui retourne le premier résultat. 

```
// Dans objects.rs
```

```
pub async fn first(&self, db: &DatabaseConnection)
    -> Result<Option<E::Model>, DbErr>
{
```

```
    E::find().one(db).await
}
```

```
// Utilisation :
let premier = User::objects.first(&db).await?;
```

## Exercice 2 : Ajouter exists()

Crée une méthode `exists()` qui vérifie si des résultats existent. 

```
// Dans query.rs
```

- **`pub async fn exists(self, db: &DatabaseConnection) -> Result<bool, DbErr>`** 

```
{
    let count = self.count(db).await?;
    Ok(count > 0)
}
```

```
// Utilisation :
let existe = User::objects
    .filter(user::Column::Username.eq("alice"))
    .exists(&db)
    .await?;
```

## 7. Résumé des concepts

Voici un tableau récapitulatif des concepts Rust utilisés : 

|**Concept**|**Utilité**|**Exemple**|
|---|---|---|
|**Trait**|Ajouter méthodes à types|impl MonTrait for MaStruct|
|**Générique**|Code réutilisable|struct Box<T>|
|**PhantomData**|Type sans données|PhantomData<E>|
|**const fn**|Eval à compilation|const fn new()|
|**Macro**|Générer code|macro_rules! impl_objects|
|**Into<T>**|Conversion auto|C: Into<Condition>|
|**Builder**|Méthodes chainables|filter().exclude()|

## Points clés à retenir :

- **Traits** : Permettent d'étendre des types existants 

- **Génériques** : Rendent le code réutilisable pour plusieurs types 

- **PhantomData** : Type fantôme de taille zéro pour la vérification de types 

- **const fn** : Évaluation à la compilation pour créer des constantes 

- **Macros** : Génération automatique de code répétitif 

- **Into<T>** : Conversions automatiques entre types 

- **Builder pattern** : Chaînage de méthodes pour une API fluide 

## Félicitations !

Tu as maintenant compris comment créer une API Django-like en Rust ! Continue à expérimenter, à casser des choses et à apprendre. I **La communauté Rust est là pour t'aider !** I 

## Ressources pour aller plus loin :

I The Rust Book : https://doc.rust-lang.org/book/ 

I Rust by Example : https://doc.rust-lang.org/rust-by-example/ 

I SeaORM Docs : https://www.sea-ql.org/SeaORM/ 

I Forum Rust : https://users.rust-lang.org/
