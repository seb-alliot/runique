//! What a route DOES. One empty struct per operation, its rules declared once
//! in its constants; [`CrudOperation`] is the data form of the same thing
//! (history, templates, `extra_routes`), linked by [`AdminOperation::CRUD_NATURE`].
use super::rights::Crud;
use crate::admin::resource::CrudOperation;

pub trait AdminOperation {
    const CRUD_NATURE: CrudOperation;
    /// Full right required.
    const RIGHT: Crud;
    /// Right limited to the user's own rows, if the operation allows it.
    const RIGHT_OWN: Option<Crud>;
    /// A non-superuser can't target a superuser's account. Stricter account
    /// rules (superuser to superuser, staff deletion, self-actions) are not
    /// expressed by this flag.
    const PROTECTS_SUPERUSER: bool;
}

pub struct List;
pub struct View;
pub struct Create;
pub struct Update;
pub struct Delete;
pub struct ResetPassword;

impl AdminOperation for List {
    const CRUD_NATURE: CrudOperation = CrudOperation::List;
    const RIGHT: Crud = Crud::Read;
    const RIGHT_OWN: Option<Crud> = Some(Crud::ReadOwn);
    const PROTECTS_SUPERUSER: bool = false;
}

impl AdminOperation for View {
    const CRUD_NATURE: CrudOperation = CrudOperation::View;
    const RIGHT: Crud = Crud::Read;
    const RIGHT_OWN: Option<Crud> = Some(Crud::ReadOwn);
    // Reading a superuser's account stays open.
    const PROTECTS_SUPERUSER: bool = false;
}

impl AdminOperation for Create {
    const CRUD_NATURE: CrudOperation = CrudOperation::Create;
    const RIGHT: Crud = Crud::Create;
    // No existing row to own.
    const RIGHT_OWN: Option<Crud> = None;
    const PROTECTS_SUPERUSER: bool = false;
}

impl AdminOperation for Update {
    const CRUD_NATURE: CrudOperation = CrudOperation::Edit;
    const RIGHT: Crud = Crud::Update;
    const RIGHT_OWN: Option<Crud> = Some(Crud::UpdateOwn);
    const PROTECTS_SUPERUSER: bool = true;
}

impl AdminOperation for Delete {
    const CRUD_NATURE: CrudOperation = CrudOperation::Delete;
    const RIGHT: Crud = Crud::Delete;
    const RIGHT_OWN: Option<Crud> = Some(Crud::DeleteOwn);
    const PROTECTS_SUPERUSER: bool = true;
}

impl AdminOperation for ResetPassword {
    const CRUD_NATURE: CrudOperation = CrudOperation::ResetPassword;
    const RIGHT: Crud = Crud::Update;
    const RIGHT_OWN: Option<Crud> = None;
    const PROTECTS_SUPERUSER: bool = true;
}
