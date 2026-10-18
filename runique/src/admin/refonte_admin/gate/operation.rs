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
    /// On the account table, only reaches an account of a strictly lower
    /// rank than the user's (`Rang::cibles_gerables`).
    const PROTECTS_RANK: bool;
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
    const PROTECTS_RANK: bool = false;
}

impl AdminOperation for View {
    const CRUD_NATURE: CrudOperation = CrudOperation::View;
    const RIGHT: Crud = Crud::Read;
    const RIGHT_OWN: Option<Crud> = Some(Crud::ReadOwn);
    // Reading an account of any rank stays open.
    const PROTECTS_RANK: bool = false;
}

impl AdminOperation for Create {
    const CRUD_NATURE: CrudOperation = CrudOperation::Create;
    const RIGHT: Crud = Crud::Create;
    // No existing row to own.
    const RIGHT_OWN: Option<Crud> = None;
    const PROTECTS_RANK: bool = false;
}

impl AdminOperation for Update {
    const CRUD_NATURE: CrudOperation = CrudOperation::Edit;
    const RIGHT: Crud = Crud::Update;
    const RIGHT_OWN: Option<Crud> = Some(Crud::UpdateOwn);
    const PROTECTS_RANK: bool = true;
}

impl AdminOperation for Delete {
    const CRUD_NATURE: CrudOperation = CrudOperation::Delete;
    const RIGHT: Crud = Crud::Delete;
    const RIGHT_OWN: Option<Crud> = Some(Crud::DeleteOwn);
    const PROTECTS_RANK: bool = true;
}

impl AdminOperation for ResetPassword {
    const CRUD_NATURE: CrudOperation = CrudOperation::ResetPassword;
    const RIGHT: Crud = Crud::Update;
    const RIGHT_OWN: Option<Crud> = None;
    const PROTECTS_RANK: bool = true;
}
