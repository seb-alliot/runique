
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

---

## Déclaration des formulaires

Dans `src/admin.rs` :

```rust
admin!(UserModel => UserForm);
admin!(BlogModel => BlogForm);
```

Le parser extrait automatiquement ces paires pour générer les routes admin.

---

## Flux d’une requête `/admin/{form}`

```
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
```

### Étapes détaillées

1. Le router reçoit la requête.
2. Le handler générique `admin_form_handler::<T>` est appelé.
3. `Prisme<T>` instancie le formulaire typé.
4. Selon la méthode HTTP :

   * `GET` → rend le formulaire vide ou liste
   * `POST` → appelle `form.is_valid()` puis `form.save()`
   * `PUT` / `DELETE` → logique correspondante
5. Les **signals du modèle** s’exécutent :

   * `before_save` → calculs inter-tables
   * `after_save` → notifications/audit
6. Le handler renvoie la réponse HTTP (render ou redirect).

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

## Exemple minimal d’intégration

```rust
RuniqueApp::builder(config)
    .with_admin(true)                     // active l’admin
    .routes(app_routes)                    // routes normales
    .nest("/admin", admin::admin_routes()) // routes auto-générées
    .build().await?;
```

---

## Notes

* La logique métier complexe doit rester **dans les formulaires ou modèles**.
* Les templates admin sont séparés dans `admin/templates/`.
* Les formulaires doivent implémenter `RuniqueForm` et éventuellement `AdminRunnable`.

---

## Version de l’admin

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

---

## CLI pour SeaORM — Créer un superuser

```rust
// src/bin/create-superuser.rs
use sea_orm::{Database, EntityTrait, ActiveValue::Set};
use runique::models::user::{ActiveModel, Entity as User};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db_url = std::env::var("DATABASE_URL")?;
    let db = Database::connect(db_url).await?;

    let admin = ActiveModel {
        username: Set("admin".to_string()),
        password_hash: Set("HASH_PLACEHOLDER".to_string()), // remplacer par Argon2 ou fonction de hashage des formulaires
        is_superuser: Set(true),
        ..Default::default()
    };

    admin.insert(&db).await?;
    println!("Superuser 'admin' créé !");

    Ok(())
}
```

---

## Conventions pour le dev

* **Nom de table user** : obligatoire pour la cohérence
* **Nom du formulaire** : utilisé pour générer la route `/admin/{form}`
* **Pas de doublon de formulaire pour un même modèle**
* **Tous les champs du modèle doivent être couverts par le formulaire** (sauf id, created_at, updated_at)
* **Logic métier inter-table** → via signals, jamais dans le handler
