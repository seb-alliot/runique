# Ébauche — opérations CRUD de l'admin (preuves typées)

> Statut : ébauche, rien n'est implémenté. Complète [ebauche-model-admin.md](ebauche-model-admin.md).

## Les trois pièces

| Pièce | Ce qu'elle décrit | Forme |
| --- | --- | --- |
| **Droits** | Ce que l'utilisateur **peut** faire sur une ressource | `CrudAction`, une struct de booléens (existe sous le nom `ResourcePerms`, renommée à la refonte ; colonnes de `eihwaz_groupes_droits`) |
| **Opérations** | Ce qu'une route **fait** : chemin, droit exigé, protections | Une struct vide par opération + le trait `Op` |
| **Preuve** | Ce que la porte **a autorisé**, pour une ligne | `Authorized<O, E>` |

## 1. Droits (ce que l'utilisateur peut faire)

Lus en base via ses groupes, une fois par requête, depuis `request.user`.

```rust
pub struct CrudAction {   // aujourd'hui `ResourcePerms`, renommé à la refonte
    pub can_create: bool,
    pub can_read: bool,
    pub can_update: bool,
    pub can_delete: bool,
    pub can_update_own: bool,
    pub can_delete_own: bool,
    pub is_superuser: bool,
}

pub enum Crud { Create, Read, Update, Delete, UpdateOwn, DeleteOwn }

impl CrudAction {
    pub fn has(&self, right: Crud) -> bool {
        match right {
            Crud::Create => self.can_create,
            Crud::Read => self.can_read,
            Crud::Update => self.can_update,
            Crud::Delete => self.can_delete,
            Crud::UpdateOwn => self.can_update_own,
            Crud::DeleteOwn => self.can_delete_own,
        }
    }
}
```

## 2. Opérations (ce qu'une route fait)

Une struct vide par opération. Elle ne contient rien : tout est dans ses constantes.

```rust
pub struct List;
pub struct View;
pub struct Create;
pub struct Update;
pub struct Delete;

pub trait Op {
    /// Pour les données : historique, templates, routes du dev.
    const KIND: CrudOperation;
    /// Chemin sous `/admin/{resource}`.
    const PATH: &'static str;
    /// Droit complet exigé.
    const RIGHT: Crud;
    /// Droit limité aux lignes de l'utilisateur, si l'opération l'admet.
    const RIGHT_OWN: Option<Crud>;
    /// Le compte d'un superuser ne peut être visé que par un superuser.
    const PROTECTS_SUPERUSER: bool;
    /// Monte ses propres handlers.
    fn mount<E: ModelMeta>(router: Router) -> Router;
}
```

| Struct | `PATH` | `RIGHT` | `RIGHT_OWN` | `PROTECTS_SUPERUSER` |
| --- | --- | --- | --- | --- |
| `List` | `/list` | `Read` | — (la liste est filtrée par `Scope::Own`) | non |
| `View` | `/{id}/detail` | `Read` | — | non |
| `Create` | `/create` | `Create` | — (pas de ligne existante) | non |
| `Update` | `/{id}/edit` | `Update` | `UpdateOwn` | oui |
| `Delete` | `/{id}/delete` | `Delete` | `DeleteOwn` | oui |

Exemple complet :

```rust
impl Op for Update {
    const KIND: CrudOperation = CrudOperation::Edit;
    const PATH: &'static str = "/{id}/edit";
    const RIGHT: Crud = Crud::Update;
    const RIGHT_OWN: Option<Crud> = Some(Crud::UpdateOwn);
    const PROTECTS_SUPERUSER: bool = true;

    fn mount<E: ModelMeta>(r: Router) -> Router {
        r.route(Self::PATH, get(edit_get::<E>).post(edit_post::<E>))
    }
}
```

L'enum `CrudOperation` (existe, `admin/resource.rs`) reste pour les données : il se stocke, se compare, s'affiche. La struct porte le comportement ; `KIND` relie les deux.

## 3. La porte (une seule, générique)

Pas de `match` : elle lit les constantes de l'opération.

```rust
pub(crate) async fn gate<O: Op, E: ModelMeta>(
    ctx: &GateCtx<'_>,
    req: &Request,
    id: Option<PkOf<E>>,
) -> Result<Authorized<O, E>, Refusal> {
    let user = req.user.as_ref().ok_or(Refusal::ActionImpossible)?;
    let perms = CrudAction::resolve(user, E::KEY);

    let allowed = if perms.has(O::RIGHT) {
        true
    } else if let Some(own) = O::RIGHT_OWN
        && perms.has(own)
        && let Some(id) = &id
    {
        owner_of::<E>(ctx.db, id).await? == Some(user.id)   // lu en base
    } else {
        false
    };
    if !allowed {
        return Err(Refusal::ActionImpossible);
    }

    // Ressource imbriquée : la ligne doit appartenir au parent de l'URL.
    if let Some(parent) = &ctx.parent
        && let Some(id) = &id
        && !belongs_to_parent::<E>(ctx.db, id, parent).await?
    {
        return Err(Refusal::ActionImpossible);
    }

    if O::PROTECTS_SUPERUSER
        && !perms.is_superuser
        && let Some(id) = &id
        && protects_a_superuser::<E>(ctx.db, id).await?
    {
        return Err(Refusal::ActionImpossible);
    }

    Ok(Authorized::new(id))
}
```

## 4. La preuve (ce que la porte a autorisé)

```rust
/// Ni `Clone` ni `Copy` : une preuve, une écriture.
pub struct Authorized<O, E: EntityTrait> {
    id: Option<PkOf<E>>,          // l'écriture prend l'id ICI, jamais à côté
    _op: PhantomData<O>,
}

impl<O, E: EntityTrait> Authorized<O, E> {
    pub(crate) fn new(id: Option<PkOf<E>>) -> Self { .. }   // seule la porte l'appelle
}
```

Les écritures exigent la preuve de **leur** opération et la consomment :

```rust
async fn insert<E>(preuve: Authorized<Create, E>, data: .., db: &ADb) -> Result<.., Refusal>;
async fn update<E>(preuve: Authorized<Update, E>, data: .., db: &ADb) -> Result<.., Refusal>;
async fn delete<E>(preuve: Authorized<Delete, E>, db: &ADb) -> Result<.., Refusal>;
```

## 5. Un handler, de bout en bout

```rust
async fn edit_post<E: ModelMeta>(req: Request, Path(id): Path<PkOf<E>>) -> Response {
    let preuve = gate::<Update, E>(&ctx, &req, Some(id)).await?;   // refus : on s'arrête
    update(preuve, data, &db).await?;                               // insert impossible ici
    redirect_to_detail()
}
```

| Question | Réponse |
| --- | --- |
| Qui connaît la nature de l'opération ? | Le handler, par sa route (`Update::PATH`) |
| Qui vérifie les droits ? | La porte, en lisant `Update::RIGHT`, `RIGHT_OWN`, `PROTECTS_SUPERUSER` |
| Qui empêche un `update` dans une création ? | Le compilateur : le handler `create` n'a qu'une preuve `Create` |

## 6. Many-to-many

Les liens appartiennent au parent : la preuve du parent les couvre, plus une vérification de lecture sur les cibles.

```rust
pub trait WritesLinks<E> {}
impl<E> WritesLinks<E> for Authorized<Create, E> {}
impl<E> WritesLinks<E> for Authorized<Update, E> {}
// Ni Read ni Delete : la suppression passe par la cascade en base.

async fn write_links<R: M2mRelation>(
    parent: &impl WritesLinks<R::From>,
    relation: R,
    targets: ReadableIds<R::To>,   // ids que l'utilisateur peut lire ; un seul refusé → tout refusé
    txn: &DatabaseTransaction,     // même transaction que le parent
) -> Result<(), Refusal>;
```

## 7. Ajouter une opération plus tard

Une struct et ses constantes, par exemple :

```rust
pub struct Export;
impl Op for Export {
    const KIND: CrudOperation = CrudOperation::Export;
    const PATH: &'static str = "/export";
    const RIGHT: Crud = Crud::Read;
    const RIGHT_OWN: Option<Crud> = None;
    const PROTECTS_SUPERUSER: bool = false;
    fn mount<E: ModelMeta>(r: Router) -> Router { .. }
}
```

La porte ne change pas. Le `match` exhaustif sur `CrudOperation` (historique, templates) signale à la compilation chaque endroit à compléter.

## 8. Ce que la porte doit encore couvrir

1. **Ressources imbriquées** : sur `/admin/plat/3/commande_ligne/7/edit`, la ligne 7 doit appartenir au plat 3 (`GateCtx::parent`), sinon changer le `3` de l'URL vise la ligne d'un autre parent.
2. **Filtre métier `queryset`** : appliqué aussi aux lignes isolées (détail, édition, suppression), pas seulement à la liste ; une ligne exclue de la liste n'est pas accessible par son id.
3. **Réinitialisation du mot de passe** : sa propre struct `Op` (`/{id}/reset-password`, `RIGHT = Update`, `PROTECTS_SUPERUSER = true`).
4. **Historique** : écrit par les primitives `insert` / `update` / `delete` elles-mêmes, dans la même transaction, avec `O::KIND` ; aucun handler ne peut l'oublier.

## 9. IDOR : aucun id venu du client n'est utilisé tel quel

| Où passe l'id | Exemple | Vérification |
| --- | --- | --- |
| URL, ligne visée | `/plat/7/edit` → `/plat/8/edit` | La porte : droit, propriétaire, parent, `queryset` |
| Formulaire, clé étrangère | `theme_id` ou `parent_id` forcé vers un objet non lisible, enfant déplacé vers un autre parent | `ReadableIds<E>`, comme le M2M : la cible doit être lisible par l'utilisateur |
| Formulaire, colonne propriétaire | Un utilisateur « own » envoie `author_id = 12` : crée au nom d'un autre, ou lui transfère sa ligne | Absente de `fields` : imposée à `request.user.id`. Dans `fields` : choix du dev (transfert permis), cible vérifiée par `ReadableIds` |
| Routes annexes | `/admin/history/{id}` | Passent par la porte de la ressource visée, avec `Read` |

Garantie par le type : les écritures n'acceptent que des `ReadableIds<E>` pour les clés étrangères, jamais un id brut.

Droit manquant : `can_read_own`. Sans lui, un utilisateur limité à ses lignes en écriture lit toutes les autres via `View`. À ajouter à `CrudAction` (et `Crud::ReadOwn`, `View::RIGHT_OWN = Some(Crud::ReadOwn)`, `List` filtrée par `Scope::Own`).

## 10. Actions d'un utilisateur sur son propre compte

Un membre du staff qui a le droit de modifier `users` ne doit pas pouvoir s'en servir sur lui-même pour monter en droits.

| Action sur soi-même | Règle |
| --- | --- |
| Ajouter ses propres groupes, modifier ses droits | Refusé : les groupes et droits d'un compte se modifient par un autre compte |
| Modifier son `is_staff`, `is_active` | Refusé (se désactiver ou se retirer le staff passe par un autre compte) |
| Supprimer son propre compte depuis l'admin | Refusé pour tous (voir la hiérarchie ci-dessous) |
| Accorder à un autre un droit qu'on n'a pas soi-même | Refusé : on ne délègue que ce qu'on possède (le superuser excepté) |
| Modifier son email, son nom | Autorisé selon ses droits habituels |

**Suppression d'un compte : hiérarchie** (une suppression a un effet en cascade sur les données liées)

| Compte visé | Qui peut le supprimer depuis l'admin |
| --- | --- |
| Superuser (`is_superuser`) | **Personne.** Uniquement par une requête SQL manuelle sur la base |
| Staff (`is_staff`) | Un superuser seulement, jamais lui-même |
| Utilisateur sans staff | Un staff ayant `can_delete` sur `users`, ou un superuser |

Les rôles du type « modérateur » ne sont que des groupes créés par le dev : ils donnent des droits, pas un rang. La hiérarchie ne repose que sur les deux colonnes de `eihwaz_users`, qu'aucun groupe ne peut modifier.

Dans la porte : une constante de plus sur l'opération, ou une règle propre à la ressource `users`, évaluée quand l'id visé est celui de `request.user`. Comme la protection du superuser, elle refuse simplement de délivrer la preuve.

## 11. Trois niveaux, dans cet ordre

```text
plancher Runique ──refus──► « action impossible »
      │ oui
      ▼
droits de groupe (CrudAction) ──refus──► « action impossible »
      │ oui
      ▼
règle du dev (.access(|a| a.can_delete(..))) ──refus──► « action impossible »
      │ oui
      ▼
preuve délivrée
```

| Niveau | Contenu | Qui l'écrit |
| --- | --- | --- |
| Plancher | Tables `eihwaz_*`, `is_superuser`, colonnes secrètes, protection superuser, actions sur soi-même, hiérarchie de suppression des comptes | Runique, `pub(crate)`, non configurable |
| Droits | `can_*` et `can_*_own` par groupe | L'administrateur, depuis l'interface |
| Métier | Règles du domaine (plat déjà commandé, rang entre groupes…) | Le dev, dans le builder |

Le dev ne peut que **restreindre** : sa règle n'est évaluée que si les deux niveaux précédents ont dit oui. Un `can_delete(|_, _| true)` n'autorise rien que Runique refuse. Différence assumée avec Django, où ces protections sont laissées au dev.

**Limite de responsabilité.** Tout ce qui passe par le builder (formulaire généré, `form::<F>()`) reste borné par le plancher. Une `extra_route` avec son propre handler sort de ce cadre : c'est un choix délibéré du dev, son code écrit librement via SeaORM, et ce qu'il y fait (y compris toucher `is_superuser`) relève de sa responsabilité. La route reste vérifiée par `extra_route_gate` ; sa doc le rappelle.

## 12. Trous relevés à la relecture (2026-10-10)

1. **Monter en droits par la table des droits.** Un staff avec `can_update` sur `eihwaz_groupes_droits` coche `can_delete` sur son propre groupe ; ou ajoute des membres à un groupe plus puissant depuis `groupes`. Plancher : on ne modifie ni les droits d'un groupe dont on est membre, ni la composition d'un groupe, au-delà des droits qu'on possède (superuser excepté).
2. **Vérification et écriture séparées (TOCTOU).** Le propriétaire ou le statut superuser peuvent changer entre la porte et l'écriture. Écriture conditionnelle (`UPDATE … WHERE id = ? AND owner_id = ?`, puis vérifier qu'une ligne a été touchée), ou porte et écriture dans une même transaction.
3. **Constructeur de la preuve** : `pub(in crate::admin::gate)`, pas `pub(crate)`. Le module `admin/gate/` peut garder plusieurs fichiers (un par règle) ; seul ce dossier fabrique des preuves.
4. **Existence qui fuit** : même réponse pour « n'existe pas » et « pas le droit », sinon les ids s'énumèrent.
5. **Colonnes structurelles** (tranché) : clé primaire jamais dans `fields` (vérifié au démarrage) ; clé du parent d'une route imbriquée imposée par l'URL. Toute autre clé étrangère, colonne propriétaire comprise, suit la logique de Django : modifiable seulement si le dev la met dans `fields` (transférer une commande à un autre utilisateur est un choix du projet), absente ou `readonly` sinon. La cible reste vérifiée par `ReadableIds`.

Déjà corrigé : les droits sont relus à chaque requête, la porte revérifie au POST.

## 13. Retour de relecture (Grok, 2026-10-10)

1. **Propriétaire typé, via `ModelMeta`** : `owner_of` lit la colonne propriétaire déclarée par `ModelMeta` (`const OWNER: Option<Self::Column>`), comparée en `PkOf<User>`, jamais en chaîne (`own_field` + `to_string()` d'aujourd'hui disparaissent). Pas de colonne propriétaire → `RIGHT_OWN` n'ouvre jamais. Colonne `NULL` → pas propriétaire.
2. **Parent dans la porte et dans la preuve** : `GateCtx::parent` (ressource + id) est vérifié par la porte (ci-dessus) et rejoué par `Guarded::conditions()` (`AND parent_id = ?`).
3. **Frontière** : la porte autorise **l'opération sur une ligne** ; le plancher et `fields` autorisent **les colonnes**. Un `create` ne peut pas poser `is_superuser` parce que la colonne est refusée, pas parce que la porte le voit.
4. **`ResetPassword`** : déjà prévue (section 8, point 3) ; sa preuve est `Authorized<ResetPassword, User>`.
5. **Réponse unique** : « n'existe pas », « hors du parent », « pas le droit », « superuser protégé » donnent tous la même réponse (section 12, point 4). Laquelle (404 ou message général) : à trancher une fois pour toutes.
6. **Vocabulaire unique** : `CrudAction` (droits), `Crud` (un droit), `CrudOperation` (enum des opérations, données), structs `Op` (comportement), `Authorized`, `Refusal`. `ResourcePerms` et `Refus` disparaissent à la refonte.
7. **Tests** : les tests fabriquent leurs preuves par un module `gate::test_support` sous `#[cfg(test)]`, jamais en ouvrant `Authorized::new`.
8. **Action groupée** : la colonne visée doit être dans `fields`, jamais secrète ni `server_only` (vérifié au démarrage).
9. **`ModelMeta` reste fin** : clé de ressource, clé primaire, propriétaire, parent, secrets, colonnes obligatoires, relations. L'affichage, les formulaires et les inlines restent dans le builder `ModelAdmin`. La preuve ne connaît que `E` et `O`, jamais le formulaire.

## Écarté (et pourquoi)

| Piste | Raison |
| --- | --- |
| Signature de route par un middleware | Inutile : le handler connaît déjà sa nature par sa route |
| Hook SeaORM (`before_save`) + `task_local` | Laisse passer en silence si l'intention n'est pas posée ; bloque l'historique et les liens M2M ; vit dans le modèle du dev |
| Rapport après écriture + annulation | Écrit pour rien, consomme des identifiants |
| Lint Clippy sur les méthodes SeaORM | Écarté par choix : l'ORM n'est pas bridé |
| Preuve portant un enum (`op: CrudOperation`) | Vérification à l'exécution dans chaque écriture, oubliable |

## Tranché (2026-10-10)

- Noms : `gate`, `Refusal`, `Authorized` ; `bulk_gate` → `action_gate` à la refonte.
- Actions groupées : la modification en groupe actuelle est reprise ; une preuve par ligne (`Vec<Authorized<Update, E>>`). Même logique que le CSRF avant l'upload : **toutes** les lignes passent par la porte avant la moindre écriture (protection superuser comprise, un refus refuse tout le lot) ; ensuite seulement, toutes les écritures dans une seule transaction, annulée à la première erreur.
- Une action groupée ne vise qu'une colonne présente dans `fields` (vérifié au démarrage).
- Secrets : un champ `password` est secret d'office ; tout autre champ se déclare secret explicitement dans le DSL (marqueur booléen sur le champ, ex. `api_key: text [secret]`). `ModelMeta` publie la liste ; le plancher l'applique.
- `surcharge_js` retenu.
- Export (historique, ou une table en CSV) : pas encore prévu, plus tard. Contraintes le jour venu : une struct `Export` (`RIGHT = Read`), même projection que la liste (jamais de colonne secrète ni `server_only`), même `Scope`, et l'export de l'historique filtré par les droits de lecture de chaque ressource. Esquisse : génération en tâche Tokio (lecture par paquets, `.csv.gz`) dans un dossier d'export du serveur, jamais servi en statique ; une table d'exports (fichier, propriétaire, date, expiration, état du lien) ; lien à jeton aléatoire **et** réservé au propriétaire connecté ; servi par la porte avec `Range` (reprise) ; lien invalidé une fois le téléchargement **complet** (pas à la première requête, sinon la reprise casse) ; fichier conservé pour régénérer un lien à la demande ; email = un lien, jamais une pièce jointe. À prévoir : limite du nombre d'exports lancés (la génération est le coût réel), emplacement et rétention des fichiers : choix du dev (dossier configurable, disque annexe, cloud) ; Runique fournit le réglage, pas la politique.
- TOCTOU : trait générique appliqué à tout le CRUD. La porte ne délivre pas seulement la preuve, elle y joint les conditions qu'elle a vérifiées (propriétaire = `user.id`, cible non superuser, parent attendu). L'écriture les rejoue dans sa clause `WHERE` (`UPDATE … WHERE id = ? AND owner_id = ? AND is_superuser = false`) et exige une ligne touchée ; zéro ligne → `Refusal`, même message général. `Create` n'a pas de ligne existante, donc pas de condition.

```rust
pub trait Guarded<E: EntityTrait> {
    /// Les conditions vérifiées par la porte, rejouées par l'écriture.
    fn conditions(&self) -> Condition;
}
```
