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
    /// The account's password — never a form field: a new account gets a
    /// random hash nobody knows (its owner sets theirs from the email link),
    /// an edit leaves it untouched. Changing it is the reset flow's job.
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
    // Skip, like the ranks below: activating, deactivating or banning an
    // account goes through the gate's account rules (not oneself, not a higher
    // rank, only an account its owner activated), never a profile form. New
    // accounts start inactive (DB default), waiting for the owner's link.
    col!("is_active", PhantomType::Bool, FormWidget::Skip),
    // FormWidget::Skip (not Bool) on the three rank flags: a rank is never
    // a form field, generated or handwritten. The account page changes it
    // through `set_rank`. `create-superuser` sets `is_superuser` by a direct
    // insert. `write::insert` refuses `Set(true)` on these columns;
    // `write::update` refuses any `Set`. DB columns default to false
    // (migrations_table.rs), so NotSet on create is a lambda; NotSet on
    // update leaves the existing value untouched.
    col!("is_dirigeant", PhantomType::Bool, FormWidget::Skip),
    col!("is_admin", PhantomType::Bool, FormWidget::Skip),
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
    col!("is_staff", PhantomType::Bool, FormWidget::Bool),
];

pub fn phantom_columns(table: &str) -> &'static [PhantomColumn] {
    match table {
        "eihwaz_users" => EIHWAZ_USERS,
        "eihwaz_groupes" => EIHWAZ_GROUPES,
        _ => &[],
    }
}
