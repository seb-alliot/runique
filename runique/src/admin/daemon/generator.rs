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
    let content = "pub mod admin_panel;\npub use admin_panel::{routes, admin_proto_state};\n";
    fs::write(dir.join("mod.rs"), content).map_err(|e| format!("Impossible d'écrire mod.rs: {}", e))
}

fn write_admin_panel(resources: &[ResourceDef], dir: &Path) -> Result<(), String> {
    let mut out = String::new();

    let _ = writeln!(out, "// AUTO-admin_panel — DO NOT EDIT MANUALLY");
    let _ = writeln!(out, "// admin_panel by `runique start` from src/admin.rs");
    let _ = writeln!(out);
    let _ = writeln!(out, "use runique::prelude::*;");
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
    let _ = writeln!(out, "/// Construit le registre admin au boot.");
    let _ = writeln!(out, "/// Appelé par le builder de l'application.");
    let _ = writeln!(out, "pub fn admin_register() -> AdminRegistry {{");
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
    let _ = writeln!(
        out,
        "    let config = Arc::new(AdminConfig::new().prefix(prefix));"
    );
    let _ = writeln!(out, "    let state = Arc::new(PrototypeAdminState {{");
    let _ = writeln!(out, "        registry: Arc::new(admin_register()),");
    let _ = writeln!(out, "        config: config.clone(),");
    let _ = writeln!(out, "    }});");
    let _ = writeln!(out, "    runique::axum::Router::new()");
    let _ = writeln!(out, "        .route(&format!(\"{{}}/{{{{resource}}}}/{{{{action}}}}\", p), get(admin_get).post(admin_post))");
    let _ = writeln!(out, "        .route(&format!(\"{{}}/{{{{resource}}}}/{{{{id}}}}/{{{{action}}}}\", p), get(admin_get_id).post(admin_post_id))");
    let _ = writeln!(out, "        .layer(Extension(state))");
    let _ = writeln!(out, "}}");
    let _ = writeln!(out);
    let _ = writeln!(
        out,
        "/// Retourne l'état partagé du prototype admin (pour le dashboard)."
    );
    let _ = writeln!(
        out,
        "pub fn admin_proto_state() -> std::sync::Arc<PrototypeAdminState> {{"
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
    let _ = writeln!(out, "    let form_builder: FormBuilder = Arc::new(|data: StrMap, tera: ATera, csrf: String, method: Method| {{");
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
    let _ = writeln!(out, "    let list_fn: ListFn = Arc::new(|db: ADb| {{");
    let _ = writeln!(out, "        Box::pin(async move {{");
    let _ = writeln!(
        out,
        "            let rows = {}::Entity::find().all(&*db).await?;",
        module
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
    let _ = writeln!(out, "    let count_fn: CountFn = Arc::new(|db: ADb| {{");
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
        "    let get_fn: GetFn = Arc::new(|db: ADb, id: i32| {{"
    );
    let _ = writeln!(out, "        Box::pin(async move {{");
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
        "    let delete_fn: DeleteFn = Arc::new(|db: ADb, id: i32| {{"
    );
    let _ = writeln!(out, "        Box::pin(async move {{");
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
        "    let update_fn: UpdateFn = Arc::new(|db: ADb, id: i32, data: StrMap| {{"
    );
    let _ = writeln!(out, "        Box::pin(async move {{");
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
        let _ = writeln!(out, "    let edit_form_builder: FormBuilder = Arc::new(|data: StrMap, tera: ATera, csrf: String, method: Method| {{");
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
