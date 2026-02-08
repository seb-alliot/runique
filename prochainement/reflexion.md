
### 1 pseudo code

macro admin! qui prendrais le chemin du formulaire en paramatre
remplirais un registre

exemple =>

pub struct User()
admin! (forms::user)

    champs ...

remplis un vecteur de formulaire register_for_admin() par exemple avec (forms::user)

### 2 routeur admin

## 1 page de connection

middleware d'auth connecté basé sur le superuser intialement, modifiable dans la vue admin pour surcharger via des role du dev

## 2 formulaire

boucle sur register_for_admin() pour remplir sur le template admin via un for form in register_for_admin pour afficher tous les formulaires, la mise en page s'occupperai de separer les form via des selecteur par key, value

## 3 connection a la bdd

la vue admin devras etre connecter via le role superuser initialement donc avec les droits qui vont avec, la creation de role devras modifier se style de droit si moderateur est creer , ou un admin simple ?



## Gestion accés au formulaire via le role

ajouter un filtre via un role => django le gere via groupe il me semble => reproduire se principe puis dans le template ajouter un if user.is_admin && permission == "nom des permission"

## Recap actuel

1 => middleware auth, verifis si l'auth est connecter, si oui verifier les permission Admin => accé si non retour sur la page de login avec msg d'erreur vous n'avez pas les droits

2 => suivant le role des permissions, affichage des formulaires => de cette facon pas de droit a gerer sur la connection de la bdd, sa se fait via le connection => plus simple a maintenir

3 => ajouter la table user sur toutes les pages de gestion de formulaire => necessaire dans 90% des cas d'usage si ce n'est plus

4 => le js reliant le user au fomrulaire se feras pas le dev qui liera le js au formulaire via add_js() => query db sains normalement pour toutess requete fetch

5 => le formulaire d'accé au user devras avec une  requete de base pour afficher tous les users => defi => sea orm, le nom de la table dois etre imposer pour le bon fonctionnement => défi => un dev peux changer le nom de la table => solution actuel => imposer une convention de nommage pour la table user et les permissions

6 => j'ai oublié quoi?

 => requete fetch sur la table user, formulaire user personnalisé (select, recherche par nom) relier a la struct du dev
    => convention de nommage imposer table users
    => nommer les formulaires pour en ajouter un titre d'accé dans l'admin => exemple forms user titre "Compte user" html vue admin Compte user et sa afficher la vue de recherche d'un compte user

=> Gerer les erreurs renvoyé par la bdd => exemple => user supprimé => ne pas crash si d'autres tables dépendent de cet element, a uniformisé pour toutes les tables

7   => fonction de comparaison dans le register_admin_form => si la macro admin! est utilisé , la fonction analyse les champs de la struct models et le formulaire qui lui est lié, si difference de champs, alerte de couleur sinon rien, la base etant le models => source de vérité absolu
    => cargo watch ? avec un message d'erreur sur le champs en trop ou manquant ? => macro donc reactivité en temps reel
    => surveiller les doublons de formulaire declarer sur un model => interdiction de doublon ?
    => la fonction de comparaison ne s'active qu'a l'activation de la vue admin dans le router via with_admin(true)
    => macro qui lis un admin.rs qui contiendrais la liste de register_admin_form pour facilité le parse ?
    => prendre en compte que prisme ne peux accepter qu'un seul formulaire => prevoir une value differente pour chaque form
        => solution pensé
            => submit avec une value propre au formulaire a la génération de l'html
            => display="all" fait un get sur toutes les colonnes de la table
            => display="sa","ceci","cela" => filtre d'affichage des colonnes souhaité
            => edit => recuperer la liste de display avec crud update
            => delete => supprime definitement la colonnes/tables choisis en fonction des attribus choisis par le dev sur le model via sea-orm pour la 1ere et la deuxieme # Approche Admin — Architecture Runique
            => prévoir une cli pour la creation d'un super user, sea n'en a pas 
                => prevoir
                    => table user de base
                        => id
                        => email
                        => password hasher -> argon2
                        => is_superuser
                        => is_active
                        => is_staff


## Résumé

Système admin unique sans compilation ni runtime impact. Basé sur **convention stricte**, **registre centralisé** et **démon de surveillance en temps réel**.

---

## 1. Convention Stricte (Django-like)

Structure obligatoire :
```
src/
  models/
    users.rs       → struct Model { ... }
    blog.rs        → struct Model { ... }
    mod.rs

  forms/
    users.rs       → struct RegisterForm + impl RuniqueForm
    blog.rs        → struct Blog + impl RuniqueForm
    mod.rs

  admin.rs         → Déclaration des paires (registre)
```

**Un seul model et un seul form principal par fichier** — clarté maximale.

---

## 2. Registre Centralisé (`src/admin.rs`)

Fichier source unique où le dev déclare les paires model ↔ form :

```rust
// src/admin.rs

#[macro_export]
macro_rules! admin {
    ($model:path => $form:path) => {};  // no-op compile
}

admin!(crate::models::users::Model => crate::forms::users::RegisterForm);
admin!(crate::models::blog::Model => crate::forms::blog::Blog);
```

**Avantages** :
- Source unique, lisible.
- Paths explicites → zéro ambiguïté.
- Macro no-op → aucun impact compilation.
- Démon parse ce fichier pour extraire les paires.

---

## 3. Démon de Surveillance (Temps réel)

Process indépendant qui :
1. **Watcher** : surveille `src/**/*.rs` via `notify`.
2. **Parser** : extrait modèles, formulaires avec `syn`.
3. **Résolution** : match les chemins → fichiers → structs.
4. **Extraction champs** :
   - Model : `struct Model { field1, field2, ... }`
   - Form : littéraux dans `register_fields()` (ex. `TextField::...("username")`)
5. **Comparaison** : diff sets avec exclusions (`id`, `created_at`, `updated_at`).
6. **Diagnostics** : publie JSON (`.runique/diagnostics.json`) ou STDOUT.

**Temps réel** : déclenchement ~100-200ms après sauvegarde.

---

## 4. Intégration Builder Intelligent

Slot middleware dédié :
```rust
const SLOT_ADMIN_AUTH: u8 = 65;  // Entre CSRF(60) et Host(70)
```

Usage :
```rust
RuniqueApp::builder(config)
    .statics()
    .routes(app_routes)
    .with_admin(true)           // Activ/inactif
    .middleware(|m| { ... })
    .build().await?
```

Si `.with_admin(true)` :
- Routes admin activées (`/admin/*`)
- Middleware auth admin injecté (slot 65)
- Démon surveille `admin.rs`
- Registre des forms exposé au contexte Tera

---

## 5. Responsabilités AdminStaging

1. **Routing** : `/admin/login`, `/admin/dashboard`, `/admin/forms/{key}`, etc.
2. **Auth middleware** : vérifier `is_admin` + permissions par rôle.
3. **Découverte** : parser `admin.rs` une fois au démarrage + hot-reload démon.
4. **Registry** : construire map Form (clé/titre/permissions) → exposée au context Tera.

---

## 6. Templates Admin

**Dev les fournit** via include Tera :

```html
<!-- templates/admin/*.html -->
{% include "admin/login.html" %}
{% include "admin/dashboard.html" %}

{% for form_meta in admin_forms %}
  <div class="form-group">
    <h3>{{ form_meta.title }}</h3>
    <!-- form_meta.key, form_meta.permissions, etc. -->
  </div>
{% endfor %}
```

Framework = **zéro gestion template**, 100% flex Tera.

---

## 7. Comparaison Champs (Sans Compilation)

Statique et déporté du build :

- **Source de vérité** : `Model` struct.
- **Vérifications** :
  - Champs manquants dans Form (warning).
  - Champs en trop dans Form (warning).
  - Doublons de Form sur un Model (error).
  - Exclusions appliquées : `["id", "created_at", "updated_at"]`.
- **Diagnostics** : publiés JSON → affichés dans l'éditeur via problemMatcher ou extension légère.

---

## 8. Avantages Clés

✓ **Convention stricte** = prévisibilité + maintenabilité.
✓ **Zéro impact compilation** = rapidité build.
✓ **Diagnostics temps réel** = UX rapide (feedback ~100ms).
✓ **Registre centralisé** = source unique de vérité.
✓ **Extensible** : AdminRegistrable trait pour métadonnées futures.
✓ **Integration builder** : `.with_admin(true)` + slots intelligents.

---

## 9. Phase Suivante

- [ ] Implémenter AdminStaging + routing basic.
- [ ] Créer démon + parser `syn` complet.
- [ ] Intégrer au builder (flag + slot middleware).
- [ ] Tester sur demo-app (models/users + forms/users).
- [ ] Ajouter trait AdminRegistrable (optionnel, extensibilité).

---

## Notes

- **Proc-macros** : volontairement exclues (complexité, expansion nécessaire).
- **Normalisation champs** : snake_case, gestion `serde(rename)` si nécessaire.
- **Permissions** : gérées côté auth middleware (role-based).
- **Database** : imposer convention table `users` + ORM mappé. et un debut d'ebauche de fonction

```rust
 fn create_admin_views(
    mut request: Request,
    Prisme(mut admin_form): Prisme<Vec<RegisterAdminForm>>,
) -> AppResult<Response> {
    if request.is_get() {
        for form in admin_form {
            let form_id = format!("id_{}", form.id);
            form.set_id(form_id);
            form.build();
            context_update!(request => {
                "admin_form" => &form,
            });
        }
    }

    if request.is_post() {
        form = "id_{}".format(admin_form)

        // Sa coince ici !!
        Prisme(mut form): Prisme<form>,
        if form.is_valid().await {
            form.save(&request.engine.db).await?;
            return Ok(Redirect::to("/admin/success").into_response());
        } else {
            context_update!(request => {
                "admin_form" => &form,
                "messages" => flash_now!(error => "Veuillez corriger les erreurs"),
            });
            return request.render("admin/create_admin.html");
        }
    }
}