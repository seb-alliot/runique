use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

pub enum PhantomType {
    Pk,
    String,
    Bool,
    NaiveDateTime,
}

impl PhantomType {
    pub fn to_tokens(&self) -> TokenStream2 {
        match self {
            PhantomType::Pk => quote! { ::runique::utils::config::Pk },
            PhantomType::String => quote! { String },
            PhantomType::Bool => quote! { bool },
            PhantomType::NaiveDateTime => quote! { ::chrono::NaiveDateTime },
        }
    }
}

pub enum PkKind {
    Auto,
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
    // FormWidget::Skip (not Bool): an admin account must never be grantable
    // from a form, generated or handwritten — see admin/forms/mod.rs and
    // admin/builtin/user.rs for the same rule applied to the builtin form.
    // DB column defaults to false (migrations_table.rs), so NotSet on create
    // is safe; NotSet on update correctly leaves the existing value untouched.
    col!("is_superuser", PhantomType::Bool, FormWidget::Skip),
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
    // Set only by the owner's activation (`activate_account`), never by a form:
    // the database refuses `is_active` without it.
    col!(
        "activated_at",
        PhantomType::NaiveDateTime,
        null,
        FormWidget::Skip
    ),
];

static EIHWAZ_GROUPES: &[PhantomColumn] = &[
    col!("id", PhantomType::Pk, pk),
    col!("nom", PhantomType::String, FormWidget::Text),
];

pub fn phantom_columns(table: &str) -> &'static [PhantomColumn] {
    match table {
        "eihwaz_users" => EIHWAZ_USERS,
        "eihwaz_groupes" => EIHWAZ_GROUPES,
        _ => &[],
    }
}
