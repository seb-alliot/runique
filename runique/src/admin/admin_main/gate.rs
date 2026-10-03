//! The single way into an admin write — create, edit, delete, and the bulk
//! operations (`delete`, `update-submit`, `group_set`).
//!
//! Three checks, in this order; the first failure refuses the request:
//! 1. **who** — the right the operation needs, from the signed-in user's
//!    groups (`authorize` / `authorize_post`, or the bulk operation's right);
//! 2. **which rows** — under a nested route, every id belongs to the URL's
//!    parent; for `_own` rights, the row belongs to the user;
//! 3. **what** — only what the admin declares and shows: the form's fields
//!    (the bulk form for a bulk update, each value checked by its field), the
//!    configured group actions with their offered values, the parent from the
//!    URL. Never a column the user's rights alone would allow: rights say
//!    which resource, the declaration says which columns.
//!
//! The write functions downstream (`create_fn`, `update_fn`,
//! `partial_update_fn`, the generated `admin_from_form` / `admin_partial_update`)
//! set every column present in the data they get — nothing after this gate
//! filters the request again.
use super::action::{Access, MemberAction};
use super::{
    ParentBinding, ResourcePerms, check_owns_record, closure_id_of, verify_scope_ownership,
};
use crate::admin::helper::dyn_form::DynForm;
use crate::admin::helper::resource_entry::ResourceEntry;
use crate::admin::resource::CrudOperation;
use crate::context::template::Request;
use crate::utils::aliases::StrMap;
use crate::utils::session_key::session::CSRF_TOKEN_KEY;

/// The bulk operation a request asks for (`bulk_action`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum BulkOp {
    Delete,
    Update,
    GroupSet,
}

impl BulkOp {
    pub(super) fn parse(bulk_action: &str) -> Option<Self> {
        match bulk_action {
            "delete" => Some(Self::Delete),
            "update-submit" => Some(Self::Update),
            "group_set" => Some(Self::GroupSet),
            _ => None,
        }
    }
}

/// What the gate lets through.
pub(super) struct BulkGrant {
    /// Closure ids, each checked against the URL's parent.
    pub ids: Vec<String>,
    /// Columns and values the write may set — empty for a delete.
    pub data: StrMap,
}

/// Why a bulk request is refused.
#[derive(Debug, PartialEq)]
pub(super) enum BulkRefusal {
    NoSelection,
    NothingToApply,
    /// The operation's right is missing.
    Forbidden,
    /// An id outside the URL's parent.
    OutOfScope,
    /// A field or value the admin doesn't offer for this operation.
    NotOffered(String),
    /// A value its field refused: the message to show.
    Invalid(String),
}

pub(super) async fn bulk_gate(
    req: &Request,
    entry: &ResourceEntry,
    state: &super::PrototypeAdminState,
    perms: &ResourcePerms,
    parent: Option<&ParentBinding>,
    op: BulkOp,
    body: &StrMap,
) -> Result<BulkGrant, BulkRefusal> {
    let allowed = match op {
        BulkOp::Delete => perms.can_delete,
        BulkOp::Update | BulkOp::GroupSet => perms.can_update,
    };
    if !allowed {
        return Err(BulkRefusal::Forbidden);
    }

    let local_ids: Vec<&str> = body
        .get("ids")
        .map(|s| {
            s.split(',')
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .collect()
        })
        .unwrap_or_default();
    if local_ids.is_empty() {
        return Err(BulkRefusal::NoSelection);
    }
    let mut ids = Vec::with_capacity(local_ids.len());
    for local in local_ids {
        let cid = closure_id_of(parent, local);
        if let Some(p) = parent
            && !verify_scope_ownership(entry, req.engine.db.clone(), &cid, p).await
        {
            return Err(BulkRefusal::OutOfScope);
        }
        ids.push(cid);
    }

    let data = match op {
        BulkOp::Delete => StrMap::new(),
        BulkOp::GroupSet => group_set_data(entry, parent, body)?,
        BulkOp::Update => {
            let mut form = bulk_form(req, entry, state, parent).await;
            update_data(&mut form, body).await?
        }
    };
    if op != BulkOp::Delete && data.is_empty() {
        return Err(BulkRefusal::NothingToApply);
    }
    Ok(BulkGrant { ids, data })
}

/// The form a bulk edit shows — and the only fields a bulk update may set.
///
/// Left out: unique fields (one value can't go on several rows), the parent
/// scope's columns (fixed by the URL), and what a bulk edit can't carry —
/// uploads, passwords, and hidden fields the admin never shows (the CSRF
/// token, a create form's injected password).
pub(super) async fn bulk_form(
    req: &Request,
    entry: &ResourceEntry,
    state: &super::PrototypeAdminState,
    parent: Option<&ParentBinding>,
) -> Box<dyn DynForm> {
    let csrf = req
        .csrf_token
        .masked()
        .unwrap_or_else(|_| req.csrf_token.clone())
        .as_str()
        .to_string();
    let resource_keys = state
        .registry
        .all()
        .map(|e| e.meta.key.to_string())
        .collect::<Vec<_>>();
    let mut form = (entry.form_builder)(
        req.engine.db.clone(),
        resource_keys,
        StrMap::new(),
        req.engine.tera.clone(),
        csrf,
        axum::http::Method::GET,
    )
    .await;

    let fields = &mut form.get_form_mut().fields;
    let scope_cols: Vec<&str> = parent
        .map(|p| p.local_key.into_iter().chain([p.fk_col]).collect())
        .unwrap_or_default();
    fields.retain(|name, field| {
        !entry.unique_fields.contains(&name.as_str())
            && !scope_cols.contains(&name.as_str())
            && !field.is_password()
            && !matches!(field.field_type(), "file" | "binary" | "hidden")
    });
    for field in fields.values_mut() {
        if field.field_type() == "select" && field.placeholder().is_empty() {
            field.set_placeholder("— sans changement —");
        }
    }
    form
}

/// The submitted bulk-form fields — an empty value means "leave as is" — each
/// read and checked by its own field, then finalized like on a single edit.
async fn update_data(form: &mut Box<dyn DynForm>, body: &StrMap) -> Result<StrMap, BulkRefusal> {
    let fields = &mut form.get_form_mut().fields;
    let mut data = StrMap::new();
    for (key, value) in body.iter().filter(|(_, v)| !v.is_empty()) {
        if matches!(key.as_str(), "ids" | "bulk_action" | CSRF_TOKEN_KEY) {
            continue;
        }
        let Some(field) = fields.get_mut(key) else {
            return Err(BulkRefusal::NotOffered(key.clone()));
        };
        field.set_submitted_value(value);
        if !field.validate().await {
            let msg = field.error().cloned().unwrap_or_default();
            return Err(BulkRefusal::Invalid(format!("{} : {msg}", field.label())));
        }
        field.finalize().await.map_err(BulkRefusal::Invalid)?;
        data.insert(key.clone(), field.value().to_string());
    }
    Ok(data)
}

/// The `ga_<field>` values of a group action: only configured actions, with
/// one of the values they offer, and never the parent scope's columns.
fn group_set_data(
    entry: &ResourceEntry,
    parent: Option<&ParentBinding>,
    body: &StrMap,
) -> Result<StrMap, BulkRefusal> {
    let mut data = StrMap::new();
    for (key, value) in body.iter().filter(|(_, v)| !v.is_empty()) {
        let Some(field) = key.strip_prefix("ga_") else {
            continue;
        };
        let in_scope = parent.is_some_and(|p| p.fk_col == field || p.local_key == Some(field));
        let offered = entry
            .group_actions
            .iter()
            .filter(|action| action.field == field)
            .flat_map(|action| &action.choices)
            .any(|(choice, _)| choice == value);
        if in_scope || !offered {
            return Err(BulkRefusal::NotOffered(field.to_string()));
        }
        data.insert(field.to_string(), value.clone());
    }
    Ok(data)
}

// ─── Single-row actions (detail, edit, delete, reset-password) ───────────────

/// The outcome of the "who" and "which rows" checks for one row.
pub(super) enum RowCheck {
    Granted,
    /// The id isn't a child of the URL's parent: answered as "not found".
    OutOfScope,
    Denied(Access),
}

/// "Which rows" then "who" for a single-row action: the row must sit under
/// the URL's parent, and the action's right — or its `_own` variant on a row
/// the user owns — must be granted.
pub(super) async fn member_gate(
    entry: &ResourceEntry,
    db: crate::utils::aliases::ADb,
    perms: &ResourcePerms,
    parent: Option<&ParentBinding>,
    user_id: crate::utils::pk::Pk,
    id: &str,
    act: &MemberAction,
) -> RowCheck {
    let closure_id = closure_id_of(parent, id);
    if let Some(p) = parent
        && !verify_scope_ownership(entry, db.clone(), &closure_id, p).await
    {
        return RowCheck::OutOfScope;
    }
    let owns_record = check_owns_record(entry, db, &closure_id, user_id).await;
    match act.authorize(perms, owns_record) {
        Access::Granted => RowCheck::Granted,
        denied => RowCheck::Denied(denied),
    }
}

// ─── Create / edit: what the write function receives ────────────────────────

/// A create, or an edit of the row whose URL id is `local_id`.
#[derive(Clone, Copy)]
pub(super) enum FormOp<'a> {
    Create,
    Edit { local_id: &'a str },
}

/// The data a create/update function gets from a validated form: the form's
/// own values (plus its many-to-many checkboxes), the parent from the URL, and
/// on a create for an account resource, a random password.
pub(super) fn form_grant(
    form: &crate::forms::Forms,
    body: &StrMap,
    entry: &ResourceEntry,
    parent: Option<&ParentBinding>,
    op: FormOp<'_>,
) -> StrMap {
    let mut data = accepted_data(form, body);
    data.remove(CSRF_TOKEN_KEY);
    let local_id = match op {
        FormOp::Create => None,
        FormOp::Edit { local_id } => Some(local_id),
    };
    if let Some(p) = parent {
        force_scope_values(&mut data, p, local_id);
    }
    if matches!(op, FormOp::Create) && entry.meta.inject_password {
        inject_random_password(form, &mut data);
    }
    data
}

/// Forces the parent-scope identity columns into submitted data, so a nested
/// create/edit always writes the parent from the (authorized) URL path — never
/// a value the client could tamper with in the hidden field. On edit the local
/// key is also pinned so a composite child's identity can't drift.
pub(super) fn force_scope_values(
    data: &mut StrMap,
    parent: &ParentBinding,
    local_id: Option<&str>,
) {
    data.insert(parent.fk_col.to_string(), parent.parent_id.clone());
    if let (Some(col), Some(local)) = (parent.local_key, local_id) {
        data.insert(col.to_string(), local.to_string());
    }
}

/// What a create/update function receives: the values the form validated
/// (after `finalize`), plus the many-to-many checkboxes (`m2m_<field>__<id>`),
/// which the templates render outside the form and the generated code checks.
/// Any other key the client added to the request is dropped: a column the form
/// doesn't show must not become writable by adding it to the request body.
fn accepted_data(form: &crate::forms::Forms, body: &StrMap) -> StrMap {
    let mut data: StrMap = form
        .fields
        .iter()
        .map(|(name, field)| (name.clone(), field.value().to_string()))
        .collect();
    data.extend(
        body.iter()
            .filter(|(key, _)| key.starts_with("m2m_"))
            .map(|(key, value)| (key.clone(), value.clone())),
    );
    data
}

/// For a resource that creates accounts through the reset-email flow
/// (`inject_password`): the password is a random one nobody knows, unless the
/// form has a real password input the admin typed into. A hidden or missing
/// field never lets the submitted value through — the account's owner sets
/// their password from the email.
fn inject_random_password(form: &crate::forms::Forms, data: &mut StrMap) {
    let typed_by_admin = form
        .fields
        .get("password")
        .is_some_and(|f| f.field_type() == "password" && !f.value().is_empty());
    if typed_by_admin {
        return;
    }
    let temp_pw = uuid::Uuid::new_v4().to_string();
    match crate::utils::password::hash(&temp_pw) {
        Ok(hash) => {
            data.insert("password".to_string(), hash);
        }
        // Never keep the submitted value in its place.
        Err(_) => {
            data.remove("password");
        }
    }
}

// ─── Custom admin routes (`extra_routes`) ─────────────────────────────────────

impl ResourcePerms {
    /// Whether these rights cover `op` — `_own` rights need a row, so they
    /// never count for a whole route.
    fn allows(&self, op: CrudOperation) -> bool {
        match op {
            CrudOperation::List | CrudOperation::View => self.can_read,
            CrudOperation::Create => self.can_create,
            CrudOperation::Edit => self.can_update,
            CrudOperation::Delete => self.can_delete,
        }
    }
}

/// "Who" for a custom admin route: the right its declared operation needs on
/// its declared resource, from the signed-in user's groups. Which rows and
/// columns the handler then touches is the handler's own business.
pub(in crate::admin) async fn extra_route_gate(
    axum::extract::State((resource, op)): axum::extract::State<(String, CrudOperation)>,
    axum::Extension(admin): axum::Extension<std::sync::Arc<crate::admin::AdminState>>,
    session: tower_sessions::Session,
    request: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    let user = request
        .extensions()
        .get::<crate::auth::session::CurrentUser>();
    let granted = user.is_some_and(|u| ResourcePerms::resolve(u, &resource).allows(op));
    if let Some(level) = crate::utils::runique_log::get_log()
        .admin
        .as_ref()
        .and_then(|a| a.auth)
    {
        crate::runique_log!(
            level,
            resource = %resource,
            operation = ?op,
            user = user.map_or("-", |u| u.username.as_str()),
            granted,
            "extra route access check"
        );
    }
    if granted {
        return next.run(request).await;
    }
    let notices = crate::flash::flash_manager::Message { session };
    super::permission_denied_dashboard(&notices, &admin.config.prefix).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::admin::helper::resource_entry::{FormBuilder, GroupAction};
    use crate::admin::resource::AdminResource;
    use std::sync::Arc;

    fn entry() -> ResourceEntry {
        let form_builder: FormBuilder =
            Arc::new(|_, _, _, _, _, _| Box::pin(async { unreachable!() }));
        let mut e = ResourceEntry::new(
            AdminResource::new("lignes", "M", "F", "Lignes"),
            form_builder,
        );
        e.group_actions = vec![
            GroupAction::bool("is_active", "Actif"),
            GroupAction::val("statut", "Archiver", "archive"),
            GroupAction::val("commande_id", "Déplacer", "9"),
        ];
        e
    }

    fn body(pairs: &[(&str, &str)]) -> StrMap {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    fn parent() -> ParentBinding {
        ParentBinding {
            parent_key: "commandes".into(),
            parent_id: "5".into(),
            fk_col: "commande_id",
            local_key: None,
        }
    }

    #[test]
    fn group_set_takes_only_configured_actions_and_values() {
        let e = entry();
        let ok = group_set_data(&e, None, &body(&[("ga_statut", "archive"), ("ids", "1")]));
        assert_eq!(ok.unwrap(), body(&[("statut", "archive")]));
        assert_eq!(
            group_set_data(&e, None, &body(&[("ga_statut", "supprime")])),
            Err(BulkRefusal::NotOffered("statut".into())),
            "a value the action doesn't offer"
        );
        assert_eq!(
            group_set_data(&e, None, &body(&[("ga_is_staff", "true")])),
            Err(BulkRefusal::NotOffered("is_staff".into())),
            "a field with no action"
        );
        assert!(
            group_set_data(&e, None, &body(&[("ga_statut", "")]))
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn group_set_never_moves_rows_to_another_parent() {
        let e = entry();
        assert!(group_set_data(&e, None, &body(&[("ga_commande_id", "9")])).is_ok());
        assert_eq!(
            group_set_data(&e, Some(&parent()), &body(&[("ga_commande_id", "9")])),
            Err(BulkRefusal::NotOffered("commande_id".into()))
        );
    }

    #[test]
    fn bulk_ops_parse() {
        assert_eq!(BulkOp::parse("delete"), Some(BulkOp::Delete));
        assert_eq!(BulkOp::parse("update-submit"), Some(BulkOp::Update));
        assert_eq!(BulkOp::parse("group_set"), Some(BulkOp::GroupSet));
        assert_eq!(BulkOp::parse("drop"), None);
    }
}

#[cfg(test)]
mod form_tests {
    use super::{accepted_data, inject_random_password};
    use crate::forms::Forms;
    use crate::forms::fields::{HiddenField, TextField};
    use crate::utils::aliases::StrMap;

    fn body(pairs: &[(&str, &str)]) -> StrMap {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    #[test]
    fn accepted_data_keeps_form_fields_and_m2m_only() {
        let mut form = Forms::new("csrf");
        form.field(&TextField::text("username"));
        form.fields
            .get_mut("username")
            .expect("field")
            .set_value("alice");
        let raw = body(&[
            ("username", "tampered"),
            ("is_active", "true"),
            ("m2m_tags__3", "on"),
        ]);
        let data = accepted_data(&form, &raw);
        assert_eq!(data.get("username").map(String::as_str), Some("alice"));
        assert!(!data.contains_key("is_active"), "not a form field");
        assert!(
            data.contains_key("m2m_tags__3"),
            "m2m checkboxes live outside the form"
        );
    }

    #[test]
    fn hidden_password_is_always_replaced() {
        let mut form = Forms::new("csrf");
        form.field(&HiddenField::new("password"));
        let mut data = body(&[("password", "$argon2id$chosen-by-the-client")]);
        inject_random_password(&form, &mut data);
        let stored = data.get("password").expect("password set");
        assert_ne!(stored, "$argon2id$chosen-by-the-client");
        assert!(stored.starts_with("$argon2"));
    }

    #[test]
    fn typed_password_is_kept() {
        let mut form = Forms::new("csrf");
        form.field(&TextField::password("password"));
        form.fields
            .get_mut("password")
            .expect("field")
            .set_value("$argon2id$hashed-by-finalize");
        let mut data = body(&[("password", "$argon2id$hashed-by-finalize")]);
        inject_random_password(&form, &mut data);
        assert_eq!(
            data.get("password").map(String::as_str),
            Some("$argon2id$hashed-by-finalize")
        );
    }
}
