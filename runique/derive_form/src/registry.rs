use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

pub enum PhantomType {
    Pk,
    I32,
    String,
    Bool,
    NaiveDateTime,
}

impl PhantomType {
    pub fn to_tokens(&self) -> TokenStream2 {
        match self {
            PhantomType::Pk => quote! { ::runique::utils::config::Pk },
            PhantomType::I32 => quote! { i32 },
            PhantomType::String => quote! { String },
            PhantomType::Bool => quote! { bool },
            PhantomType::NaiveDateTime => quote! { ::chrono::NaiveDateTime },
        }
    }
}

pub enum PkKind {
    Auto,
    Composite,
    NotPk,
}

/// How the column should be rendered and saved in the admin form.
pub enum FormWidget {
    /// Plain text input, always required.
    Text,
    /// Email input, always required.
    Email,
    /// Password input — hashed on save, NotSet if empty (preserves existing hash on edit).
    Password,
    /// Boolean checkbox — absent in form data = false.
    Bool,
    /// Auto-managed timestamp — skipped in form and in ActiveModel.
    AutoDateTime,
    /// PK and junction columns — skipped entirely.
    Skip,
}

pub struct PhantomColumn {
    pub name: &'static str,
    pub ty: PhantomType,
    pub nullable: bool,
    pub pk: PkKind,
    pub widget: FormWidget,
}

macro_rules! col {
    ($name:literal, $ty:expr, pk) => {
        PhantomColumn {
            name: $name,
            ty: $ty,
            nullable: false,
            pk: PkKind::Auto,
            widget: FormWidget::Skip,
        }
    };
    ($name:literal, $ty:expr, cpk) => {
        PhantomColumn {
            name: $name,
            ty: $ty,
            nullable: false,
            pk: PkKind::Composite,
            widget: FormWidget::Skip,
        }
    };
    ($name:literal, $ty:expr, null, $widget:expr) => {
        PhantomColumn {
            name: $name,
            ty: $ty,
            nullable: true,
            pk: PkKind::NotPk,
            widget: $widget,
        }
    };
    ($name:literal, $ty:expr, $widget:expr) => {
        PhantomColumn {
            name: $name,
            ty: $ty,
            nullable: false,
            pk: PkKind::NotPk,
            widget: $widget,
        }
    };
}

static EIHWAZ_USERS: &[PhantomColumn] = &[
    col!("id", PhantomType::Pk, pk),
    col!("username", PhantomType::String, FormWidget::Text),
    col!("email", PhantomType::String, FormWidget::Email),
    col!("password", PhantomType::String, FormWidget::Password),
    col!("is_active", PhantomType::Bool, FormWidget::Bool),
    col!("is_staff", PhantomType::Bool, FormWidget::Bool),
    col!("is_superuser", PhantomType::Bool, FormWidget::Bool),
    col!(
        "created_at",
        PhantomType::NaiveDateTime,
        null,
        FormWidget::AutoDateTime
    ),
    col!(
        "updated_at",
        PhantomType::NaiveDateTime,
        null,
        FormWidget::AutoDateTime
    ),
];

static EIHWAZ_GROUPES: &[PhantomColumn] = &[
    col!("id", PhantomType::Pk, pk),
    col!("nom", PhantomType::String, FormWidget::Text),
];

static EIHWAZ_SESSIONS: &[PhantomColumn] = &[
    col!("id", PhantomType::I32, pk),
    col!("cookie_id", PhantomType::String, FormWidget::Text),
    col!("user_id", PhantomType::Pk, FormWidget::Text),
    col!("session_id", PhantomType::String, FormWidget::Text),
    col!("session_data", PhantomType::String, null, FormWidget::Skip),
    col!(
        "expires_at",
        PhantomType::NaiveDateTime,
        FormWidget::AutoDateTime
    ),
];

static EIHWAZ_USERS_GROUPES: &[PhantomColumn] = &[
    col!("user_id", PhantomType::Pk, cpk),
    col!("groupe_id", PhantomType::Pk, cpk),
];

static EIHWAZ_GROUPES_DROITS: &[PhantomColumn] = &[
    col!("groupe_id", PhantomType::Pk, cpk),
    col!("resource_key", PhantomType::String, cpk),
    col!("can_create", PhantomType::Bool, FormWidget::Bool),
    col!("can_read", PhantomType::Bool, FormWidget::Bool),
    col!("can_update", PhantomType::Bool, FormWidget::Bool),
    col!("can_delete", PhantomType::Bool, FormWidget::Bool),
    col!("can_update_own", PhantomType::Bool, FormWidget::Bool),
    col!("can_delete_own", PhantomType::Bool, FormWidget::Bool),
];

pub fn phantom_columns(table: &str) -> &'static [PhantomColumn] {
    match table {
        "eihwaz_users" => EIHWAZ_USERS,
        "eihwaz_groupes" => EIHWAZ_GROUPES,
        "eihwaz_sessions" => EIHWAZ_SESSIONS,
        "eihwaz_users_groupes" => EIHWAZ_USERS_GROUPES,
        "eihwaz_groupes_droits" => EIHWAZ_GROUPES_DROITS,
        _ => &[],
    }
}
