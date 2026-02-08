
# Runique Admin — README

## Présentation

Runique Admin est un **système admin dynamique et typé** pour le framework Runique.
Il permet de :

* déclarer simplement les formulaires à gérer via une macro `admin!`
* générer automatiquement les routes `/admin/{form}`
* utiliser des handlers génériques typés avec `Prisme<T>`
* déléguer la logique métier aux formulaires et aux modèles
* exploiter les **signals SeaORM** pour calculs inter-tables, notifications ou audit

Le concept : **“mini-projet admin dans le projet”**, isolé, extensible et maintenable.

---

## Structure recommandée

src/
 ├── models/
 ├── forms/
 ├── admin.rs       ← registre Model ↔ Form
admin/
 ├── mod.rs         ← assemble les routes
 ├── router.rs      ← génère /admin/{form}
 ├── handlers.rs    ← handlers génériques
 └── templates/     ← templates admin (form.html, list.html)

## Déclaration des formulaires

Dans `src/admin.rs` :

```rust
admin!(UserModel => UserForm);
admin!(BlogModel => BlogForm);
```

Le parser extrait automatiquement ces paires pour générer les routes admin.

---

## Flux d’une requête `/admin/{form}`

Dev -> admin.rs declaration
       │
       ▼
Parser admin -> Registre admin
       │
       ▼
Routes générées /admin/{form}
       │
       ▼
Router principal
       │
       ▼
admin_form_handler::<T>
       │
       ▼
Prisme<T> instancie le formulaire
       │
       ▼
form.is_valid() / form.save()
       │
       ▼
Signals du modèle (before_save / after_save)
       │
       ▼
Render / Redirect

---

## Exemple de signal SeaORM

```rust
#[signals]
impl UserModel {
    async fn before_save(&mut self, db: &Database) -> Result<()> {
        self.total_points = PointsEntity::find().all(db).await?.iter().map(|p| p.value).sum();
        Ok(())
    }

    async fn after_save(&self, db: &Database) -> Result<()> {
        log::info!("Utilisateur {} sauvegardé", self.username);
        Ok(())
    }
}
```

---

## Avantages

* **Typé statiquement** : zéro dynamique ou trait objet complexe
* **Handler générique** : code réutilisable pour tous les formulaires
* **Mini-projet admin** : isolé, modulaire et maintenable
* **Logique métier déportée** : formulaires et modèles gèrent la validation et les calculs
* **Extensible** : ajoutez de nouveaux formulaires sans toucher aux handlers
* **Compatible Prisme** : form handling typé et sécurisé
* **Signals SeaORM** : déclenche logique automatique inter-table

---

## Intégration dans RuniqueApp

```rust
RuniqueApp::builder(config)
    .with_admin(true)                     // active l’admin
    .routes(app_routes)                    // routes normales
    .nest("/admin", admin::admin_routes()) // routes auto-générées
    .build().await?;
```

### `with_admin` — détails

* `prefix` : préfixe pour toutes les routes admin (défaut : `/admin`)
* `enable_dev_tools` : active le démon de surveillance pour feedback temps réel

```rust
pub fn with_admin(
    mut self,
    prefix: impl Into<String> = "/admin",
    enable_dev_tools: bool = false,
) -> Self {
    let prefix = prefix.into();

    self = self.nest(&prefix, admin::admin_router());
    self = self.middleware(admin_auth_middleware);

    if enable_dev_tools {
        // spawn démon
        tokio::spawn(async { admin::dev_tools::start_watcher().await });
    }

    self
}
```

Flux :

Builder
   │
   ├── Nest routes → /admin/*
   ├── Middleware admin → contrôle permissions
   └── Démon dev → hot reload & diagnostics

## CLI pour créer un superuser

```rust
use clap::Parser;
use sea_orm::{Database, DatabaseConnection, EntityTrait, ActiveValue::Set};
use runique::models::user::{self, ActiveModel, Entity as User};
use bcrypt::{hash, DEFAULT_COST};
use std::env;

#[derive(Parser)]
#[command(name = "runique createsuperuser")]
struct Args {
    #[arg(short, long, default_value = "admin")]
    username: String,

    #[arg(short, long)]
    email: Option<String>,

    #[arg(short, long)]
    password: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let db_url = env::var("DATABASE_URL")?;
    let db: DatabaseConnection = Database::connect(db_url).await?;

    let password = if let Some(pw) = args.password {
        pw
    } else {
        rpassword::prompt_password("Mot de passe pour le superuser : ")?
    };

    let hashed = hash(&password, DEFAULT_COST)?;

    let admin = ActiveModel {
        username: Set(args.username.clone()),
        email: Set(args.email.unwrap_or_default()),
        password_hash: Set(hashed),
        is_superuser: Set(true),
        is_staff: Set(true),
        is_active: Set(true),
        ..Default::default()
    };

    match admin.insert(&db).await {
        Ok(_) => println!("Superuser '{}' créé avec succès !", args.username),
        Err(e) => eprintln!("Erreur : {}", e),
    }

    Ok(())
}
```

* **Arguments CLI** :

  * `--username` / `-u` : nom du superuser
  * `--email` / `-e` : email (optionnel)
  * `--password` / `-p` : mot de passe (sinon prompt interactif)

---

## Conventions pour le dev

* **Nom de table user** : obligatoire pour la cohérence
* **Nom du formulaire** : utilisé pour générer la route `/admin/{form}`
* **Pas de doublon de formulaire pour un même modèle**
* **Tous les champs du modèle doivent être couverts par le formulaire** (sauf `id`, `created_at`, `updated_at`)
* **Logique métier inter-table** → via signals, jamais dans le handler

---

Si tu veux, je peux maintenant te créer **un schéma graphique unique** combinant **flow des requêtes /admin + signals + CLI** à insérer directement dans ce README pour que tout soit visuel et lisible.
