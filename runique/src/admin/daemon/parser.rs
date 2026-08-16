//! Parser for the `src/admin.rs` file: extracts resource declarations from the `admin!{}` macro.
use crate::utils::trad::{t, tf};
use proc_macro2::{Span, TokenStream};
use syn::{
    Ident, LitBool, LitInt, LitStr, Macro, Path, Token, braced, bracketed,
    parse::{Parse, ParseStream},
    parse_file,
    punctuated::Punctuated,
    spanned::Spanned,
    visit::Visit,
};

#[derive(Debug, Clone)]
pub(crate) struct ResourceDef {
    /// Resource key (e.g., "users")
    pub key: String,

    /// SeaORM Model path (e.g., "users::Model")
    pub model_type: String,

    /// Title displayed in the admin interface
    pub title: String,

    /// Template overrides per operation (optional)
    pub template_list: Option<String>,
    pub template_create: Option<String>,
    pub template_edit: Option<String>,
    pub template_detail: Option<String>,
    pub template_delete: Option<String>,

    /// Alternative creation form (optional, full path e.g., `crate::formulaire::UserAdminCreateForm`)
    pub create_form_type: Option<String>,

    /// Alternative edition form (optional, full path e.g., `crate::formulaire::UserEditForm`)
    pub edit_form_type: Option<String>,

    /// Primary key type: "I32" (default), "I64", "Uuid"
    pub id_type: String,

    /// Custom keys for Tera context (via `extra: { "k" => "v" }`)
    pub extra_context: Vec<(String, String)>,

    /// Sidebar filters: `[("col_sql", "Display Label", limit_per_page)]`
    pub list_filter: Vec<(String, String, u64)>,

    /// List visible columns with labels (and optional FK resolution): `[("col", "Label", Option<FkDisplay>)]`
    pub list_display: Vec<(String, String, Option<FkDisplay>)>,

    /// Columns excluded from the list: `["col1", "col2"]`
    pub list_exclude: Vec<String>,

    /// Fields available for group bulk update: `[("field", "Label")]` or `[("field", "Label", "value")]`
    pub group_action: Vec<(String, String, Option<String>)>,

    /// When set, the create_fn splits this field by comma and inserts one record per value.
    /// DSL: `bulk_create: field_name`
    pub bulk_create: Option<String>,

    /// Field name used for ownership verification when `can_update_own`/`can_delete_own` is set.
    /// DSL: `own_field: "user_id"`
    pub own_field: Option<String>,

    /// Many-to-many relations to manage on create/edit.
    pub m2m: Vec<M2mFieldDef>,
}

/// FK resolution for a list_display column: display a related record's label instead of the raw ID.
/// DSL: `["menu_id", "Menu", "menus.titre"]`
#[derive(Debug, Clone)]
pub(crate) struct FkDisplay {
    /// Target table name (e.g., "menus")
    pub table: String,
    /// Column to display on the target (e.g., "titre")
    pub col: String,
}

/// One M2M relation managed by the admin.
#[derive(Debug, Clone)]
pub(crate) struct M2mFieldDef {
    /// Form field name (e.g., "allergenes") — used as context key and body prefix
    pub field_name: String,
    /// Human-readable label shown in the form
    pub label: String,
    /// Junction table name (e.g., "plat_allergene")
    pub junction_table: String,
    /// FK column in the junction table pointing to the current resource
    pub self_fk: String,
    /// FK column in the junction table pointing to the related resource
    pub target_fk: String,
    /// SeaORM Entity path for related resource (e.g., "crate::entities::allergene")
    pub target_entity: String,
    /// Column name on the target entity to use as display label (e.g., "nom")
    pub target_display: String,
}

/// Display configuration for a resource in the `configure {}` block
#[derive(Debug, Clone)]
pub(crate) struct ConfigureDef {
    /// Key of the resource to configure (e.g., "users", "permissions")
    pub key: String,
    /// List visible columns with labels: `[("col", "Label")]`
    pub list_display: Vec<(String, String)>,
    /// Columns excluded from the list
    pub list_exclude: Vec<String>,
    /// Sidebar filters
    pub list_filter: Vec<(String, String, u64)>,
    /// Group action fields: `[("field", "Label")]` or `[("field", "Label", "value")]`
    pub group_action: Vec<(String, String, Option<String>)>,
    /// When true, the builtin resource is removed from the registry entirely.
    pub hidden: bool,
}

/// Result of parsing `src/admin.rs`
#[derive(Debug)]
pub(crate) struct ParsedAdmin {
    pub resources: Vec<ResourceDef>,
    pub configures: Vec<ConfigureDef>,
}

/// Parses the content of `src/admin.rs` and returns the declared resources.
pub(crate) fn parse_admin_file(source: &str) -> Result<ParsedAdmin, String> {
    let syntax = parse_file(source).map_err(|e| format!("Rust syntax error: {}", e))?;

    let mut visitor = AdminMacroVisitor::new();
    visitor.visit_file(&syntax);

    if let Some(err) = visitor.error {
        return Err(err);
    }

    Ok(ParsedAdmin {
        resources: visitor.resources,
        configures: visitor.configures,
    })
}

struct AdminMacroVisitor {
    pub resources: Vec<ResourceDef>,
    pub configures: Vec<ConfigureDef>,
    /// Source line of the first `admin!{}` found — a second one is a hard error,
    /// since the generator only ever emits one registry.
    pub first_block_line: Option<usize>,
    pub error: Option<String>,
}

impl AdminMacroVisitor {
    fn new() -> Self {
        Self {
            resources: Vec::new(),
            configures: Vec::new(),
            first_block_line: None,
            error: None,
        }
    }
}

impl<'ast> Visit<'ast> for AdminMacroVisitor {
    fn visit_macro(&mut self, mac: &'ast Macro) {
        // We only look for the macro named "admin"
        let name = mac
            .path
            .segments
            .last()
            .map(|s| s.ident.to_string())
            .unwrap_or_default();

        if name != "admin" {
            return;
        }

        // Keep the first failure: reporting the last one would hide the cause.
        if self.error.is_some() {
            return;
        }

        let line = mac.path.span().start().line;

        if let Some(first) = self.first_block_line {
            self.error = Some(tf("parser.multiple_admin_blocks", &[first, line]));
            return;
        }
        self.first_block_line = Some(line);

        match parse_admin_tokens(mac.tokens.clone()) {
            Ok(parsed) => {
                self.resources = parsed.resources;
                self.configures = parsed.configures;
            }
            Err(e) => self.error = Some(e),
        }
    }
}

// Expected syntax:
//   key: path::Model => FormType {
//       title: "...",
//   }
//
// Grammar is expressed via `syn::parse::Parse` on small per-construct types
// (Punctuated<Entry, Token![,]> for every bracketed list), instead of walking
// raw `proc_macro2::TokenTree`s by hand. Unknown DSL fields and malformed
// literals now produce a real `syn::Error` (span-attached) instead of being
// silently skipped/defaulted.

fn parse_admin_tokens(tokens: TokenStream) -> Result<ParsedAdmin, String> {
    let parsed: AdminMacroInput = syn::parse2(tokens).map_err(format_syn_error)?;
    Ok(ParsedAdmin {
        resources: parsed.resources,
        configures: parsed.configures,
    })
}

/// Formats a `syn::Error` as `line:column: message` — meaningful here because
/// this parser runs outside a real proc-macro invocation (a plain binary
/// reading `src/admin.rs`), where `Span::start()` reports true source positions.
fn format_syn_error(err: syn::Error) -> String {
    let start = err.span().start();
    format!("{}:{}: {}", start.line, start.column + 1, err)
}

struct AdminMacroInput {
    resources: Vec<ResourceDef>,
    configures: Vec<ConfigureDef>,
}

impl Parse for AdminMacroInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut resources = Vec::new();
        let mut configures = Vec::new();

        while !input.is_empty() {
            let key: Ident = input.parse()?;

            if key == "configure" {
                let content;
                braced!(content in input);
                configures.extend(parse_configure_block(&content)?);
            } else {
                input.parse::<Token![:]>()?;
                let model_type: Path = input.parse()?;
                input.parse::<Token![=>]>()?;
                let _form_type: Path = input.parse()?;

                let content;
                braced!(content in input);
                let fields = Punctuated::<ResourceField, Token![,]>::parse_terminated(&content)?;
                resources.push(build_resource_def(
                    key.to_string(),
                    path_to_string(&model_type),
                    fields,
                )?);
            }

            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(AdminMacroInput {
            resources,
            configures,
        })
    }
}

/// Parses the `configure { resource_key: { ... }, ... }` block body.
fn parse_configure_block(input: ParseStream) -> syn::Result<Vec<ConfigureDef>> {
    let mut result = Vec::new();

    while !input.is_empty() {
        let key: Ident = input.parse()?;
        input.parse::<Token![:]>()?;

        let content;
        braced!(content in input);
        let fields = Punctuated::<ConfigureField, Token![,]>::parse_terminated(&content)?;
        result.push(build_configure_def(key.to_string(), fields)?);

        if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
        }
    }

    Ok(result)
}

/// One field of a `configure { key: { ... } }` body.
enum ConfigureField {
    ListDisplay(Vec<(String, String)>),
    ListExclude(Vec<String>),
    ListFilter(Vec<(String, String, u64)>),
    GroupAction(Vec<(String, String, Option<String>)>),
    Hidden(bool),
}

impl Parse for ConfigureField {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let field: Ident = input.parse()?;
        input.parse::<Token![:]>()?;

        Ok(match field.to_string().as_str() {
            "list_display" => ConfigureField::ListDisplay(
                parse_bracketed::<PairEntry>(input)?
                    .into_iter()
                    .map(Into::into)
                    .collect(),
            ),
            "list_exclude" => ConfigureField::ListExclude(
                parse_bracketed::<LitStr>(input)?
                    .into_iter()
                    .map(|s| s.value())
                    .collect(),
            ),
            "list_filter" => ConfigureField::ListFilter(
                parse_bracketed::<ListFilterEntry>(input)?
                    .into_iter()
                    .map(Into::into)
                    .collect(),
            ),
            "group_action" => ConfigureField::GroupAction(
                parse_bracketed::<GroupActionEntry>(input)?
                    .into_iter()
                    .map(Into::into)
                    .collect(),
            ),
            "hidden" => ConfigureField::Hidden(input.parse::<LitBool>()?.value),
            other => {
                return Err(syn::Error::new(
                    field.span(),
                    format!("Unknown field in configure[]: '{other}'"),
                ));
            }
        })
    }
}

fn build_configure_def(
    key: String,
    fields: Punctuated<ConfigureField, Token![,]>,
) -> syn::Result<ConfigureDef> {
    let mut list_display = Vec::new();
    let mut list_exclude = Vec::new();
    let mut list_filter = Vec::new();
    let mut group_action = Vec::new();
    let mut hidden = false;

    for field in fields {
        match field {
            ConfigureField::ListDisplay(v) => list_display = v,
            ConfigureField::ListExclude(v) => list_exclude = v,
            ConfigureField::ListFilter(v) => list_filter = v,
            ConfigureField::GroupAction(v) => group_action = v,
            ConfigureField::Hidden(v) => hidden = v,
        }
    }

    if !list_display.is_empty() && !list_exclude.is_empty() {
        return Err(syn::Error::new(
            Span::call_site(),
            format!("configure[\"{key}\"]: list_display and list_exclude are exclusive"),
        ));
    }

    Ok(ConfigureDef {
        key,
        list_display,
        list_exclude,
        list_filter,
        group_action,
        hidden,
    })
}

/// One field of a resource body (between `{ ... }` in `admin!{ key: Model => Form { ... } }`).
enum ResourceField {
    Title(String),
    TemplateList(String),
    TemplateCreate(String),
    TemplateEdit(String),
    TemplateDetail(String),
    TemplateDelete(String),
    CreateForm(String),
    EditForm(String),
    IdType(String),
    Extra(Vec<(String, String)>),
    ListFilter(Vec<(String, String, u64)>),
    ListDisplay(Vec<(String, String, Option<FkDisplay>)>),
    ListExclude(Vec<String>),
    GroupAction(Vec<(String, String, Option<String>)>),
    BulkCreate(String),
    OwnField(String),
    M2m(Vec<M2mFieldDef>),
}

impl Parse for ResourceField {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let field: Ident = input.parse()?;
        input.parse::<Token![:]>()?;

        Ok(match field.to_string().as_str() {
            "title" => ResourceField::Title(input.parse::<LitStr>()?.value()),
            "template_list" => ResourceField::TemplateList(input.parse::<LitStr>()?.value()),
            "template_create" => ResourceField::TemplateCreate(input.parse::<LitStr>()?.value()),
            "template_edit" => ResourceField::TemplateEdit(input.parse::<LitStr>()?.value()),
            "template_detail" => ResourceField::TemplateDetail(input.parse::<LitStr>()?.value()),
            "template_delete" => ResourceField::TemplateDelete(input.parse::<LitStr>()?.value()),
            "create_form" => ResourceField::CreateForm(path_to_string(&input.parse::<Path>()?)),
            "edit_form" => ResourceField::EditForm(path_to_string(&input.parse::<Path>()?)),
            "id_type" => ResourceField::IdType(input.parse::<Ident>()?.to_string()),
            "extra" => ResourceField::Extra(parse_extra_map(input)?),
            "list_filter" => ResourceField::ListFilter(
                parse_bracketed::<ListFilterEntry>(input)?
                    .into_iter()
                    .map(Into::into)
                    .collect(),
            ),
            "list_display" => ResourceField::ListDisplay(
                parse_bracketed::<ListDisplayEntry>(input)?
                    .into_iter()
                    .map(Into::into)
                    .collect(),
            ),
            "list_exclude" => ResourceField::ListExclude(
                parse_bracketed::<LitStr>(input)?
                    .into_iter()
                    .map(|s| s.value())
                    .collect(),
            ),
            "group_action" => ResourceField::GroupAction(
                parse_bracketed::<GroupActionEntry>(input)?
                    .into_iter()
                    .map(Into::into)
                    .collect(),
            ),
            "bulk_create" => ResourceField::BulkCreate(input.parse::<Ident>()?.to_string()),
            "own_field" => ResourceField::OwnField(input.parse::<LitStr>()?.value()),
            "m2m" => ResourceField::M2m(
                parse_bracketed::<M2mEntry>(input)?
                    .into_iter()
                    .map(|e| e.0)
                    .collect(),
            ),
            other => {
                return Err(syn::Error::new(
                    field.span(),
                    format!("Unknown field in admin!{{}}: '{other}'"),
                ));
            }
        })
    }
}

fn build_resource_def(
    key: String,
    model_type: String,
    fields: Punctuated<ResourceField, Token![,]>,
) -> syn::Result<ResourceDef> {
    let mut title = String::new();
    let mut template_list = None;
    let mut template_create = None;
    let mut template_edit = None;
    let mut template_detail = None;
    let mut template_delete = None;
    let mut create_form_type = None;
    let mut edit_form_type = None;
    let mut id_type = "Pk".to_string();
    let mut extra_context = Vec::new();
    let mut list_filter = Vec::new();
    let mut list_display = Vec::new();
    let mut list_exclude = Vec::new();
    let mut group_action = Vec::new();
    let mut bulk_create = None;
    let mut own_field = None;
    let mut m2m = Vec::new();

    for field in fields {
        match field {
            ResourceField::Title(v) => title = v,
            ResourceField::TemplateList(v) => template_list = Some(v),
            ResourceField::TemplateCreate(v) => template_create = Some(v),
            ResourceField::TemplateEdit(v) => template_edit = Some(v),
            ResourceField::TemplateDetail(v) => template_detail = Some(v),
            ResourceField::TemplateDelete(v) => template_delete = Some(v),
            ResourceField::CreateForm(v) => create_form_type = Some(v),
            ResourceField::EditForm(v) => edit_form_type = Some(v),
            ResourceField::IdType(v) => id_type = v,
            ResourceField::Extra(v) => extra_context = v,
            ResourceField::ListFilter(v) => list_filter = v,
            ResourceField::ListDisplay(v) => list_display = v,
            ResourceField::ListExclude(v) => list_exclude = v,
            ResourceField::GroupAction(v) => group_action = v,
            ResourceField::BulkCreate(v) => bulk_create = Some(v),
            ResourceField::OwnField(v) => own_field = Some(v),
            ResourceField::M2m(v) => m2m = v,
        }
    }

    if title.is_empty() {
        return Err(syn::Error::new(
            Span::call_site(),
            t("parser.title_required"),
        ));
    }
    if !list_display.is_empty() && !list_exclude.is_empty() {
        return Err(syn::Error::new(
            Span::call_site(),
            t("parser.list_display_exclude_exclusive"),
        ));
    }

    Ok(ResourceDef {
        key,
        model_type,
        title,
        template_list,
        template_create,
        template_edit,
        template_detail,
        template_delete,
        create_form_type,
        edit_form_type,
        id_type,
        extra_context,
        list_filter,
        list_display,
        list_exclude,
        group_action,
        bulk_create,
        own_field,
        m2m,
    })
}

/// Parses `[ entry, entry, ... ]` — outer brackets + comma-separated `T`, trailing comma optional.
fn parse_bracketed<T: Parse>(input: ParseStream) -> syn::Result<Vec<T>> {
    let content;
    bracketed!(content in input);
    let items = Punctuated::<T, Token![,]>::parse_terminated(&content)?;
    Ok(items.into_iter().collect())
}

/// Parses the `"col", "label"` prefix shared by every `[col, label, ...]` entry below.
fn parse_pair(content: ParseStream) -> syn::Result<(String, String)> {
    let col: LitStr = content.parse()?;
    content.parse::<Token![,]>()?;
    let label: LitStr = content.parse()?;
    Ok((col.value(), label.value()))
}

/// `["col", "Label"]` — used by `configure { list_display: [...] }` (no FK resolution).
struct PairEntry(String, String);

impl Parse for PairEntry {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        bracketed!(content in input);
        let (col, label) = parse_pair(&content)?;
        Ok(PairEntry(col, label))
    }
}

impl From<PairEntry> for (String, String) {
    fn from(e: PairEntry) -> Self {
        (e.0, e.1)
    }
}

/// `["col_sql", "Label"]` or `["col_sql", "Label", 10]` — `list_filter` entries.
struct ListFilterEntry(String, String, u64);

impl Parse for ListFilterEntry {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        bracketed!(content in input);
        let (col, label) = parse_pair(&content)?;
        let limit = if content.peek(Token![,]) {
            content.parse::<Token![,]>()?;
            content.parse::<LitInt>()?.base10_parse::<u64>()?
        } else {
            10
        };
        Ok(ListFilterEntry(col, label, limit))
    }
}

impl From<ListFilterEntry> for (String, String, u64) {
    fn from(e: ListFilterEntry) -> Self {
        (e.0, e.1, e.2)
    }
}

/// `["col", "Label"]` or `["col", "Label", "table.col"]` — `list_display` entries.
struct ListDisplayEntry(String, String, Option<FkDisplay>);

impl Parse for ListDisplayEntry {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        bracketed!(content in input);
        let (col, label) = parse_pair(&content)?;
        let fk = if content.peek(Token![,]) {
            content.parse::<Token![,]>()?;
            let spec: LitStr = content.parse()?;
            let value = spec.value();
            let parts: Vec<&str> = value.splitn(2, '.').collect();
            if parts.len() != 2 {
                return Err(syn::Error::new(
                    spec.span(),
                    format!("FK spec '{value}' must be 'table.col'"),
                ));
            }
            Some(FkDisplay {
                table: parts[0].to_string(),
                col: parts[1].to_string(),
            })
        } else {
            None
        };
        Ok(ListDisplayEntry(col, label, fk))
    }
}

impl From<ListDisplayEntry> for (String, String, Option<FkDisplay>) {
    fn from(e: ListDisplayEntry) -> Self {
        (e.0, e.1, e.2)
    }
}

/// `["field", "Label"]` or `["field", "Label", "value"]` — `group_action` entries.
struct GroupActionEntry(String, String, Option<String>);

impl Parse for GroupActionEntry {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        bracketed!(content in input);
        let (field, label) = parse_pair(&content)?;
        let value = if content.peek(Token![,]) {
            content.parse::<Token![,]>()?;
            Some(content.parse::<LitStr>()?.value())
        } else {
            None
        };
        Ok(GroupActionEntry(field, label, value))
    }
}

impl From<GroupActionEntry> for (String, String, Option<String>) {
    fn from(e: GroupActionEntry) -> Self {
        (e.0, e.1, e.2)
    }
}

/// `["field", "Label", "junction_table", "self_fk", "target_fk", "entity::path", "target_display"]`
struct M2mEntry(M2mFieldDef);

impl Parse for M2mEntry {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        bracketed!(content in input);
        let field_name: LitStr = content.parse()?;
        content.parse::<Token![,]>()?;
        let label: LitStr = content.parse()?;
        content.parse::<Token![,]>()?;
        let junction_table: LitStr = content.parse()?;
        content.parse::<Token![,]>()?;
        let self_fk: LitStr = content.parse()?;
        content.parse::<Token![,]>()?;
        let target_fk: LitStr = content.parse()?;
        content.parse::<Token![,]>()?;
        let target_entity: LitStr = content.parse()?;
        content.parse::<Token![,]>()?;
        let target_display: LitStr = content.parse()?;

        Ok(M2mEntry(M2mFieldDef {
            field_name: field_name.value(),
            label: label.value(),
            junction_table: junction_table.value(),
            self_fk: self_fk.value(),
            target_fk: target_fk.value(),
            target_entity: target_entity.value(),
            target_display: target_display.value(),
        }))
    }
}

/// Parses `extra: { "key" => "value", ... }`.
fn parse_extra_map(input: ParseStream) -> syn::Result<Vec<(String, String)>> {
    let content;
    braced!(content in input);
    let items = Punctuated::<ExtraEntry, Token![,]>::parse_terminated(&content)?;
    Ok(items.into_iter().map(|e| (e.0, e.1)).collect())
}

struct ExtraEntry(String, String);

impl Parse for ExtraEntry {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let key: LitStr = input.parse()?;
        input.parse::<Token![=>]>()?;
        let value: LitStr = input.parse()?;
        Ok(ExtraEntry(key.value(), value.value()))
    }
}

/// Joins a `syn::Path`'s segments with `::` (drops generic args, same as before — the DSL never uses them).
fn path_to_string(path: &Path) -> String {
    path.segments
        .iter()
        .map(|s| s.ident.to_string())
        .collect::<Vec<_>>()
        .join("::")
}
