use crate::admin::daemon::parser::ResourceDef;
use std::fmt::Write;
use std::fs;
use std::path::Path;

/// Génère `src/admins/admin_panel.rs` — seul fichier produit par le daemon.
///
/// Contient :
/// - Une impl `DynForm` par ressource (wrapping le `RuniqueForm` concret)
/// - `admin_register()` → `HashMap<String, ResourceEntry>` alimenté au boot
pub fn generate(resources: &[ResourceDef]) -> Result<(), String> {
    let admins_dir = Path::new("src/admins");
    if admins_dir.exists() {
        fs::remove_dir_all(admins_dir)
            .map_err(|e| format!("Impossible de supprimer {}: {}", admins_dir.display(), e))?;
    }
    fs::create_dir_all(admins_dir)
        .map_err(|e| format!("Impossible de créer {}: {}", admins_dir.display(), e))?;

    write_readme(admins_dir)?;
    write_admin_panel(resources, admins_dir)?;
    write_mod(admins_dir)?;

    Ok(())
}

fn write_readme(dir: &Path) -> Result<(), String> {
    let content = "<!-- AUTO-admin_panel — DO NOT EDIT MANUALLY\n     admin_panel by `runique start`. Any changes will be overwritten. -->\n";
    fs::write(dir.join("README.md"), content)
        .map_err(|e| format!("Impossible d'écrire README.md: {}", e))
}

fn write_mod(dir: &Path) -> Result<(), String> {
    let content = "pub mod admin_panel;\npub use admin_panel::{routes, admin_state};\n";
    fs::write(dir.join("mod.rs"), content).map_err(|e| format!("Impossible d'écrire mod.rs: {}", e))
}

fn write_admin_panel(resources: &[ResourceDef], dir: &Path) -> Result<(), String> {
    let mut out = String::new();

    let _ = writeln!(out, "// AUTO-admin_panel — DO NOT EDIT MANUALLY");
    let _ = writeln!(out, "// admin_panel by `runique start` from src/admin.rs");
    let _ = writeln!(out);
    let _ = writeln!(out, "use runique::prelude::*;");
    let _ = writeln!(out, "use runique::admin::resource_entry::FilterFn;");
    let _ = writeln!(out);

    // Imports des entités (AdminForm est généré par model! dans chaque module)
    for r in resources {
        let module = model_to_module(&r.model_type);
        let _ = writeln!(out, "use crate::entities::{};", module);
    }
    let _ = writeln!(out);

    // Une impl DynForm par ressource (form principal + edit_form si déclaré)
    for r in resources {
        write_dyn_form_impl(&mut out, r)?;
        if r.edit_form_type.is_some() {
            write_edit_dyn_form_impl(&mut out, r)?;
        }
    }

    // admin_register()
    write_admin_register(&mut out, resources)?;

    fs::write(dir.join("admin_panel.rs"), out)
        .map_err(|e| format!("Impossible d'écrire admin_panel.rs: {}", e))
}

fn write_dyn_form_impl(out: &mut String, r: &ResourceDef) -> Result<(), String> {
    let module = model_to_module(&r.model_type);
    let form_path = format!("{}::AdminForm", module);
    let wrapper = format!("{}AdminFormDynWrapper", pascal_case(&module));

    let _ = writeln!(out, "// ── DynForm wrapper pour {}::AdminForm ──", module);
    let _ = writeln!(out, "struct {}(pub {});", wrapper, form_path);
    let _ = writeln!(out);
    let _ = writeln!(out, "#[async_trait]");
    let _ = writeln!(out, "impl DynForm for {} {{", wrapper);
    let _ = writeln!(out, "    async fn is_valid(&mut self) -> bool {{");
    let _ = writeln!(out, "        self.0.is_valid().await");
    let _ = writeln!(out, "    }}");
    let _ = writeln!(out);
    let _ = writeln!(
        out,
        "    async fn save(&mut self, db: &DatabaseConnection) -> Result<(), DbErr> {{"
    );
    let _ = writeln!(out, "        self.0.save(db).await");
    let _ = writeln!(out, "    }}");
    let _ = writeln!(out);
    let _ = writeln!(out, "    fn get_form(&self) -> &Forms {{");
    let _ = writeln!(out, "        self.0.get_form()");
    let _ = writeln!(out, "    }}");
    let _ = writeln!(out);
    let _ = writeln!(out, "    fn get_form_mut(&mut self) -> &mut Forms {{");
    let _ = writeln!(out, "        self.0.get_form_mut()");
    let _ = writeln!(out, "    }}");
    let _ = writeln!(out, "}}");
    let _ = writeln!(out);

    Ok(())
}

fn write_edit_dyn_form_impl(out: &mut String, r: &ResourceDef) -> Result<(), String> {
    let edit_form_path = r.edit_form_type.as_ref().unwrap();
    let type_name = edit_form_path.split("::").last().unwrap_or(edit_form_path);
    let module = model_to_module(&r.model_type);
    let wrapper = format!("{}EditFormDynWrapper", pascal_case(&module));

    let _ = writeln!(out, "// ── DynForm edit wrapper pour {} ──", edit_form_path);
    let _ = writeln!(out, "struct {}(pub {});", wrapper, edit_form_path);
    let _ = writeln!(out);
    let _ = writeln!(out, "#[async_trait]");
    let _ = writeln!(out, "impl DynForm for {} {{", wrapper);
    let _ = writeln!(out, "    async fn is_valid(&mut self) -> bool {{");
    let _ = writeln!(out, "        self.0.is_valid().await");
    let _ = writeln!(out, "    }}");
    let _ = writeln!(out);
    let _ = writeln!(
        out,
        "    async fn save(&mut self, _db: &DatabaseConnection) -> Result<(), DbErr> {{"
    );
    let _ = writeln!(out, "        Ok(()) // update_fn gère la persistance");
    let _ = writeln!(out, "    }}");
    let _ = writeln!(out);
    let _ = writeln!(out, "    fn get_form(&self) -> &Forms {{");
    let _ = writeln!(out, "        self.0.get_form()");
    let _ = writeln!(out, "    }}");
    let _ = writeln!(out);
    let _ = writeln!(out, "    fn get_form_mut(&mut self) -> &mut Forms {{");
    let _ = writeln!(out, "        self.0.get_form_mut()");
    let _ = writeln!(out, "    }}");
    let _ = writeln!(out, "}}");
    let _ = writeln!(out);

    let _ = type_name;

    Ok(())
}

fn write_admin_register(out: &mut String, resources: &[ResourceDef]) -> Result<(), String> {
    // Collecte tous les rôles uniques déclarés dans permissions: [...]
    let mut all_roles: Vec<String> = resources
        .iter()
        .flat_map(|r| r.permissions.iter().cloned())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    all_roles.sort();
    let roles_str = all_roles
        .iter()
        .map(|p| format!("\"{}\".to_string()", p))
        .collect::<Vec<_>>()
        .join(", ");

    let _ = writeln!(out, "/// Construit le registre admin au boot.");
    let _ = writeln!(out, "/// Appelé par le builder de l'application.");
    let _ = writeln!(out, "pub fn admin_register() -> AdminRegistry {{");
    let _ = writeln!(
        out,
        "    runique::admin::register_roles(vec![{}]);",
        roles_str
    );
    let _ = writeln!(out, "    let mut registry = AdminRegistry::new();");
    let _ = writeln!(out);

    for r in resources {
        write_resource_entry(out, r)?;
    }

    let _ = writeln!(out, "    registry");
    let _ = writeln!(out, "}}");
    let _ = writeln!(out);

    // Fonction routes() — retourne le Router axum du prototype admin
    let _ = writeln!(
        out,
        "/// Construit le Router axum du prototype admin pour le préfixe donné."
    );
    let _ = writeln!(
        out,
        "/// À passer à `.with_admin(|a| a.routes(admins::routes(\"/admin\")))` dans main.rs."
    );
    let _ = writeln!(
        out,
        "pub fn routes(prefix: &str) -> runique::axum::Router {{"
    );
    let _ = writeln!(out, "    let p = prefix.trim_end_matches('/');");
    let _ = writeln!(out, "    runique::axum::Router::new()");
    let _ = writeln!(
        out,
        "        .route(&format!(\"{{}}/{{{{resource}}}}/{{{{action}}}}\", p), get(admin_get).post(admin_post))"
    );
    let _ = writeln!(
        out,
        "        .route(&format!(\"{{}}/{{{{resource}}}}/{{{{id}}}}/{{{{action}}}}\", p), get(admin_get_id).post(admin_post_id))"
    );
    let _ = writeln!(out, "}}");
    let _ = writeln!(out);
    let _ = writeln!(
        out,
        "/// Retourne l'état partagé du prototype admin (pour le dashboard)."
    );
    let _ = writeln!(
        out,
        "pub fn admin_state() -> std::sync::Arc<PrototypeAdminState> {{"
    );
    let _ = writeln!(out, "    let config = Arc::new(AdminConfig::new());");
    let _ = writeln!(out, "    std::sync::Arc::new(PrototypeAdminState {{");
    let _ = writeln!(out, "        registry: Arc::new(admin_register()),");
    let _ = writeln!(out, "        config,");
    let _ = writeln!(out, "    }})");
    let _ = writeln!(out, "}}");

    Ok(())
}

fn write_resource_entry(out: &mut String, r: &ResourceDef) -> Result<(), String> {
    let key = &r.key;
    let model = full_model_path(&r.model_type);
    let title = &r.title;
    let module = model_to_module(&r.model_type);
    let form_path = format!("{}::AdminForm", module);
    let wrapper = format!("{}AdminFormDynWrapper", pascal_case(&module));
    let roles: Vec<String> = r
        .permissions
        .iter()
        .map(|p| format!("\"{}\".to_string()", p))
        .collect();
    let roles_str = roles.join(", ");

    // Code de conversion de l'ID depuis String selon id_type
    let id_parse_code = match r.id_type.as_str() {
        "I64" => {
            "let id = id.parse::<i64>().map_err(|_| DbErr::Custom(\"id invalide\".to_string().to_string()))?"
        }
        "Uuid" => {
            "let id = uuid::Uuid::parse_str(&id).map_err(|_| DbErr::Custom(\"id invalide\".to_string().to_string()))?"
        }
        _ => {
            "let id = id.parse::<i32>().map_err(|_| DbErr::Custom(\"id invalide\".to_string().to_string()))?"
        }
    };

    // AdminResource
    let _ = writeln!(out, "    // ── Ressource : {} ──", key);
    let _ = writeln!(out, "    let meta = AdminResource::new(");
    let _ = writeln!(out, "        \"{}\",", key);
    let _ = writeln!(out, "        \"{}\",", model);
    let _ = writeln!(out, "        \"AdminForm\",");
    let _ = writeln!(out, "        \"{}\",", title);
    let _ = writeln!(out, "        vec![{}],", roles_str);
    let _ = writeln!(out, "    );");

    // Template overrides
    if let Some(ref t) = r.template_list {
        let _ = writeln!(out, "    let meta = meta.template_list(\"{}\");", t);
    }
    if let Some(ref t) = r.template_create {
        let _ = writeln!(out, "    let meta = meta.template_create(\"{}\");", t);
    }
    if let Some(ref t) = r.template_edit {
        let _ = writeln!(out, "    let meta = meta.template_edit(\"{}\");", t);
    }
    if let Some(ref t) = r.template_detail {
        let _ = writeln!(out, "    let meta = meta.template_detail(\"{}\");", t);
    }
    if let Some(ref t) = r.template_delete {
        let _ = writeln!(out, "    let meta = meta.template_delete(\"{}\");", t);
    }

    // Extra context
    for (k, v) in &r.extra_context {
        let _ = writeln!(out, "    let meta = meta.extra(\"{}\", \"{}\");", k, v);
    }

    // FormBuilder closure
    let _ = writeln!(
        out,
        "    let form_builder: FormBuilder = Arc::new(|data: StrMap, tera: ATera, csrf: String, method: Method| {{"
    );
    let _ = writeln!(out, "        Box::pin(async move {{");
    let _ = writeln!(
        out,
        "            let form = {}::build_with_data(&data, tera, &csrf, method).await;",
        form_path
    );
    let _ = writeln!(
        out,
        "            Box::new({}(form)) as Box<dyn DynForm>",
        wrapper
    );
    let _ = writeln!(out, "        }})");
    let _ = writeln!(out, "    }});");
    let _ = writeln!(out);
    let _ = writeln!(
        out,
        "    let list_fn: ListFn = Arc::new(|db: ADb, params: ListParams| {{"
    );
    let _ = writeln!(out, "        Box::pin(async move {{");
    let _ = writeln!(
        out,
        "            use sea_orm::{{QueryFilter, sea_query::{{Alias, Expr, Order}}}};"
    );
    let _ = writeln!(
        out,
        "            let mut query = {}::Entity::find();",
        module
    );
    let _ = writeln!(out, "            if let Some(ref col) = params.sort_by {{");
    let _ = writeln!(
        out,
        "                let order = if params.sort_dir == SortDir::Desc {{ Order::Desc }} else {{ Order::Asc }};"
    );
    let _ = writeln!(
        out,
        "                query = query.order_by(Expr::col(Alias::new(col.as_str())), order);"
    );
    let _ = writeln!(out, "            }}");
    let _ = writeln!(
        out,
        "            for (col, val) in &params.column_filters {{"
    );
    let _ = writeln!(
        out,
        "                let escaped = val.replace('\\'', \"''\");"
    );
    let _ = writeln!(
        out,
        "                query = query.filter(Expr::cust(format!(\"CAST({{}} AS TEXT) = '{{}}'\", col, escaped)));"
    );
    let _ = writeln!(out, "            }}");
    let _ = writeln!(
        out,
        "            let rows = query.offset(params.offset).limit(params.limit).all(&*db).await?;"
    );
    let _ = writeln!(out, "            Ok(rows.into_iter()");
    let _ = writeln!(
        out,
        "                .map(|r| serde_json::to_value(r).unwrap_or(serde_json::Value::Null))"
    );
    let _ = writeln!(out, "                .collect())");
    let _ = writeln!(out, "        }})");
    let _ = writeln!(out, "    }});");
    let _ = writeln!(out);

    // CountFn closure
    let _ = writeln!(
        out,
        "    let count_fn: CountFn = Arc::new(|db: ADb, _search: Option<String>| {{"
    );
    let _ = writeln!(out, "        Box::pin(async move {{");
    let _ = writeln!(
        out,
        "            {}::Entity::find().count(&*db).await",
        module
    );
    let _ = writeln!(out, "        }})");
    let _ = writeln!(out, "    }});");
    let _ = writeln!(out);

    // GetFn closure
    let _ = writeln!(
        out,
        "    let get_fn: GetFn = Arc::new(|db: ADb, id: String| {{"
    );
    let _ = writeln!(out, "        Box::pin(async move {{");
    let _ = writeln!(out, "            {};", id_parse_code);
    let _ = writeln!(
        out,
        "            let row = {}::Entity::find_by_id(id).one(&*db).await?;",
        module
    );
    let _ = writeln!(
        out,
        "            Ok(row.map(|r| serde_json::to_value(r).unwrap_or(serde_json::Value::Null)))"
    );
    let _ = writeln!(out, "        }})");
    let _ = writeln!(out, "    }});");
    let _ = writeln!(out);

    // DeleteFn closure
    let _ = writeln!(
        out,
        "    let delete_fn: DeleteFn = Arc::new(|db: ADb, id: String| {{"
    );
    let _ = writeln!(out, "        Box::pin(async move {{");
    let _ = writeln!(out, "            {};", id_parse_code);
    let _ = writeln!(
        out,
        "            {}::Entity::delete_by_id(id).exec(&*db).await.map(|_| ())",
        module
    );
    let _ = writeln!(out, "        }})");
    let _ = writeln!(out, "    }});");
    let _ = writeln!(out);

    // CreateFn closure — utilise admin_from_form() généré par le proc-macro
    let _ = writeln!(
        out,
        "    let create_fn: CreateFn = Arc::new(|db: ADb, data: StrMap| {{"
    );
    let _ = writeln!(out, "        Box::pin(async move {{");
    let _ = writeln!(out, "            {}::admin_from_form(&data, None)", module);
    let _ = writeln!(out, "                .insert(&*db).await.map(|_| ())");
    let _ = writeln!(out, "        }})");
    let _ = writeln!(out, "    }});");
    let _ = writeln!(out);

    // UpdateFn closure — utilise admin_from_form() généré par le proc-macro
    let _ = writeln!(
        out,
        "    let update_fn: UpdateFn = Arc::new(|db: ADb, id: String, data: StrMap| {{"
    );
    let _ = writeln!(out, "        Box::pin(async move {{");
    let _ = writeln!(out, "            {};", id_parse_code);
    let _ = writeln!(
        out,
        "            {}::admin_from_form(&data, Some(id))",
        module
    );
    let _ = writeln!(out, "                .update(&*db).await.map(|_| ())");
    let _ = writeln!(out, "        }})");
    let _ = writeln!(out, "    }});");
    let _ = writeln!(out);

    // EditFormBuilder closure (optionnel)
    if let Some(ref edit_form_path) = r.edit_form_type {
        let edit_wrapper = format!("{}EditFormDynWrapper", pascal_case(&module));
        let _ = writeln!(
            out,
            "    let edit_form_builder: FormBuilder = Arc::new(|data: StrMap, tera: ATera, csrf: String, method: Method| {{"
        );
        let _ = writeln!(out, "        Box::pin(async move {{");
        let _ = writeln!(
            out,
            "            let form = {}::build_with_data(&data, tera, &csrf, method).await;",
            edit_form_path
        );
        let _ = writeln!(
            out,
            "            Box::new({}(form)) as Box<dyn DynForm>",
            edit_wrapper
        );
        let _ = writeln!(out, "        }})");
        let _ = writeln!(out, "    }});");
        let _ = writeln!(out);
    }

    // DisplayConfig avec list_display et/ou list_filter si configurés
    if !r.list_display.is_empty() || !r.list_filter.is_empty() {
        let mut display_chain = "DisplayConfig::new()".to_string();
        if !r.list_display.is_empty() {
            let cols_str = r
                .list_display
                .iter()
                .map(|(c, l)| format!("(\"{}\", \"{}\")", c, l))
                .collect::<Vec<_>>()
                .join(", ");
            display_chain.push_str(&format!(".columns_include(vec![{}])", cols_str));
        }
        if !r.list_filter.is_empty() {
            let filter_str = r
                .list_filter
                .iter()
                .map(|(col, label, limit)| format!("(\"{}\", \"{}\", {}u64)", col, label, limit))
                .collect::<Vec<_>>()
                .join(", ");
            display_chain.push_str(&format!(".list_filter(vec![{}])", filter_str));
        }
        let _ = writeln!(out, "    let meta = meta.display({});", display_chain);
    }

    // FilterFn closure (valeurs distinctes par colonne configurée)
    if !r.list_filter.is_empty() {
        let _ = writeln!(
            out,
            "    let filter_fn: FilterFn = Arc::new(|db: ADb, pages: std::collections::HashMap<String, u64>| {{"
        );
        let _ = writeln!(out, "        Box::pin(async move {{");
        let _ = writeln!(
            out,
            "            use sea_orm::{{ConnectionTrait, ExprTrait}};"
        );
        let _ = writeln!(
            out,
            "            use sea_orm::sea_query::{{Query, Alias, Expr, Order}};"
        );
        let _ = writeln!(
            out,
            "            let mut result: std::collections::HashMap<String, (Vec<String>, u64)> = std::collections::HashMap::new();"
        );
        for (col, _label, limit) in &r.list_filter {
            let _ = writeln!(
                out,
                "            let page_size_{col} = {limit}u64;",
                col = col,
                limit = limit
            );
            let _ = writeln!(
                out,
                "            let cur_page_{col} = pages.get(\"{col}\").copied().unwrap_or(0);",
                col = col
            );
            let _ = writeln!(
                out,
                "            let count_stmt_{col} = Query::select().expr(Expr::cust(\"COUNT(DISTINCT {col})\")).from(Alias::new({module}::Entity.table_name())).and_where(Expr::col(Alias::new(\"{col}\")).is_not_null()).to_owned();",
                col = col,
                module = module
            );
            let _ = writeln!(
                out,
                "            let count_row_{col} = match db.query_one(&count_stmt_{col}).await {{ Ok(r) => r, Err(e) => {{ tracing::error!(\"[runique admin] list_filter `{key}.{col}` : colonne introuvable en DB — {{}}\", e); None }} }};",
                col = col,
                key = key
            );
            let _ = writeln!(
                out,
                "            let total_{col} = count_row_{col}.and_then(|r| r.try_get_by_index::<i64>(0).ok()).unwrap_or(0) as u64;",
                col = col
            );
            let _ = writeln!(
                out,
                "            let stmt_{col} = Query::select().distinct().expr(Expr::cust(\"CAST({col} AS TEXT)\")).from(Alias::new({module}::Entity.table_name())).and_where(Expr::col(Alias::new(\"{col}\")).is_not_null()).order_by(Alias::new(\"{col}\"), Order::Asc).limit(page_size_{col}).offset(cur_page_{col} * page_size_{col}).to_owned();",
                col = col,
                module = module
            );
            let _ = writeln!(
                out,
                "            let rows_{col} = match db.query_all(&stmt_{col}).await {{ Ok(r) => r, Err(e) => {{ tracing::error!(\"[runique admin] list_filter `{key}.{col}` : colonne introuvable en DB — {{}}\", e); vec![] }} }};",
                col = col,
                key = key
            );
            let _ = writeln!(
                out,
                "            let vals_{col}: Vec<String> = rows_{col}.iter().filter_map(|r| r.try_get_by_index::<String>(0).ok()).collect();",
                col = col
            );
            let _ = writeln!(
                out,
                "            result.insert(\"{col}\".to_string(), (vals_{col}, total_{col}));",
                col = col
            );
        }
        let _ = writeln!(out, "            Ok(result)");
        let _ = writeln!(out, "        }})");
        let _ = writeln!(out, "    }});");
        let _ = writeln!(out);
    }

    let _ = writeln!(out, "    registry.register(");
    let has_edit_form = r.edit_form_type.is_some();
    if has_edit_form {
        let _ = writeln!(out, "        ResourceEntry::new(meta, form_builder)");
        let _ = writeln!(
            out,
            "            .with_edit_form_builder(edit_form_builder)"
        );
    } else {
        let _ = writeln!(out, "        ResourceEntry::new(meta, form_builder)");
    }
    let _ = writeln!(out, "            .with_list_fn(list_fn)");
    let _ = writeln!(out, "            .with_get_fn(get_fn)");
    let _ = writeln!(out, "            .with_delete_fn(delete_fn)");
    let _ = writeln!(out, "            .with_create_fn(create_fn)");
    let _ = writeln!(out, "            .with_update_fn(update_fn)");
    let _ = writeln!(out, "            .with_count_fn(count_fn)");
    if !r.list_filter.is_empty() {
        let _ = writeln!(out, "            .with_filter_fn(filter_fn)");
    }
    let _ = writeln!(out, "    );");
    let _ = writeln!(out);

    Ok(())
}

fn full_model_path(model_type: &str) -> String {
    if model_type.starts_with("crate::") || model_type.starts_with("runique::") {
        model_type.to_string()
    } else {
        format!("crate::entities::{}", model_type)
    }
}

/// Convertit snake_case → PascalCase (ex: `blog_post` → `BlogPost`)
fn pascal_case(s: &str) -> String {
    s.split('_')
        .map(|word| {
            let mut c = word.chars();
            match c.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + c.as_str(),
            }
        })
        .collect()
}

/// Extrait le nom du module depuis un chemin de model.
/// Ex: `users::Model` → `users`, `crate::entities::blog_post::Model` → `blog_post`
/// Le dernier segment est le nom du struct (Model), l'avant-dernier est le module.
fn model_to_module(model_type: &str) -> String {
    let segments: Vec<&str> = model_type.split("::").collect();
    if segments.len() >= 2 {
        segments[segments.len() - 2].to_string()
    } else {
        // Fallback : PascalCase → snake_case du seul segment
        let base = segments.last().copied().unwrap_or(model_type);
        let mut result = String::new();
        for (i, c) in base.chars().enumerate() {
            if c.is_uppercase() && i > 0 {
                result.push('_');
            }
            result.extend(c.to_lowercase());
        }
        result
    }
}
