//! What the admin needs to know about a model, on the SeaORM entity itself.
//!
//! A trait, not a second model. For a project's models it will be generated
//! by `model!{}`, from the same source as the forms; Runique implements it by
//! hand for its own tables. Kept thin: display belongs to the builder.
use sea_orm::{EntityTrait, PrimaryKeyTrait};

pub trait ModelMeta:
    EntityTrait<PrimaryKey: PrimaryKeyTrait<ValueType: Clone + PartialEq>>
{
    /// Resource key, also the rights key in `eihwaz_groupes_droits`.
    const KEY: &'static str;
    /// Column holding the owner's id. Without one, `can_*_own` rights never
    /// open anything on this model.
    const OWNER: Option<Self::Column> = None;
    /// `eihwaz_users`: its superuser accounts are protected by the gate.
    const IS_ACCOUNT_TABLE: bool = false;
}

impl ModelMeta for crate::auth::user::Entity {
    const KEY: &'static str = "users";
    const IS_ACCOUNT_TABLE: bool = true;
}
