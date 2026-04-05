//! Parser du fichier `src/admin.rs` : extrait les déclarations de ressources de la macro `admin!{}`.
use crate::utils::trad::{t, tf};
use proc_macro2::TokenStream;
use syn::{Macro, parse_file, visit::Visit};

#[derive(Debug, Clone)]
pub struct ResourceDef {
    /// Clé de la ressource (ex: "users")
    pub key: String,

    /// Chemin du Model SeaORM (ex: "users::Model")
    pub model_type: String,

    /// Nom du Form Runique (ex: "RegisterForm")
    pub form_type: String,

    /// Titre affiché dans l'interface admin
    pub title: String,

    /// Rôles autorisés
    pub permissions: Vec<String>,

    /// Surcharges de templates par opération (optionnel)
    pub template_list: Option<String>,
    pub template_create: Option<String>,
    pub template_edit: Option<String>,
    pub template_detail: Option<String>,
    pub template_delete: Option<String>,

    /// Formulaire alternatif pour la création (optionnel, chemin complet ex: crate::formulaire::UserAdminCreateForm)
    pub create_form_type: Option<String>,

    /// Formulaire alternatif pour l'édition (optionnel, chemin complet ex: crate::formulaire::UserEditForm)
    pub edit_form_type: Option<String>,

    /// Type de la clé primaire : "I32" (défaut), "I64", "Uuid"
    pub id_type: String,

    /// Clés custom pour le contexte Tera (via extra: { "k" => "v" })
    pub extra_context: Vec<(String, String)>,

    /// Filtres sidebar : [("col_sql", "Label affiché", limit_par_page)]
    pub list_filter: Vec<(String, String, u64)>,

    /// Colonnes visibles dans la liste avec labels : [("col", "Label")]
    pub list_display: Vec<(String, String)>,

    /// Colonnes exclues de la liste : ["col1", "col2"]
    pub list_exclude: Vec<String>,
}

/// Configuration d'affichage pour une ressource dans le bloc `configure {}`
#[derive(Debug, Clone)]
pub struct ConfigureDef {
    /// Clé de la ressource à configurer (ex: "users", "droits")
    pub key: String,
    /// Colonnes visibles dans la liste avec labels : [("col", "Label")]
    pub list_display: Vec<(String, String)>,
    /// Colonnes exclues de la liste
    pub list_exclude: Vec<String>,
    /// Filtres sidebar
    pub list_filter: Vec<(String, String, u64)>,
}

/// Résultat du parsing de src/admin.rs
#[derive(Debug)]
pub struct ParsedAdmin {
    pub resources: Vec<ResourceDef>,
    pub configures: Vec<ConfigureDef>,
}

/// Parse le contenu de src/admin.rs et retourne les ressources déclarées
pub fn parse_admin_file(source: &str) -> Result<ParsedAdmin, String> {
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
    pub error: Option<String>,
}

impl AdminMacroVisitor {
    fn new() -> Self {
        Self {
            resources: Vec::new(),
            configures: Vec::new(),
            error: None,
        }
    }
}

impl<'ast> Visit<'ast> for AdminMacroVisitor {
    fn visit_macro(&mut self, mac: &'ast Macro) {
        // On cherche uniquement le macro nommé "admin"
        let name = mac
            .path
            .segments
            .last()
            .map(|s| s.ident.to_string())
            .unwrap_or_default();

        if name != "admin" {
            return;
        }

        match parse_admin_tokens(mac.tokens.clone()) {
            Ok(parsed) => {
                self.resources = parsed.resources;
                self.configures = parsed.configures;
            }
            Err(e) => self.error = Some(e),
        }
    }
}

// Syntaxe attendue :
//   key: path::Model => FormType {
//       title: "...",
//       permissions: ["role1", "role2"]
//   }

fn parse_admin_tokens(tokens: TokenStream) -> Result<ParsedAdmin, String> {
    use proc_macro2::TokenTree;

    let mut resources = Vec::new();
    let mut configures = Vec::new();
    let mut iter = tokens.into_iter().peekable();

    while iter.peek().is_some() {
        // 1. key (ident)
        let key = match iter.next() {
            Some(TokenTree::Ident(id)) => id.to_string(),
            Some(other) => return Err(format!("Expected resource name, found: {}", other)),
            None => break,
        };

        // Bloc configure { ... } — configuration d'affichage pour toute ressource (builtin ou déclarée)
        if key == "configure" {
            let cfg = parse_configure_block(&mut iter)?;
            configures.extend(cfg);
            skip_optional_punct(&mut iter, ',');
            continue;
        }

        // 2. ':'
        expect_punct(&mut iter, ':')?;

        // 3. model_type (path: ident :: ident...)
        let model_type = parse_path(&mut iter)?;

        // 4. '=>'
        expect_punct(&mut iter, '=')?;
        expect_punct(&mut iter, '>')?;

        // 5. form_type (chemin complet possible : crate::formulaire::MyForm ou simple ident)
        let form_type = parse_path(&mut iter)?;

        // 6. { title: "...", permissions: [...], template_*: "...", extra: { ... } }
        let body = match iter.next() {
            Some(TokenTree::Group(group)) => parse_resource_body(group.stream())?,
            Some(other) => return Err(format!("Expected '{{', found: {}", other)),
            None => return Err("Expected '{{', end of file".to_string()),
        };
        let title = body.title.clone();
        let permissions = body.permissions.clone();

        resources.push(ResourceDef {
            key,
            model_type,
            form_type,
            title,
            permissions,
            template_list: body.template_list,
            template_create: body.template_create,
            template_edit: body.template_edit,
            template_detail: body.template_detail,
            template_delete: body.template_delete,
            extra_context: body.extra_context,
            create_form_type: body.create_form_type,
            edit_form_type: body.edit_form_type,
            id_type: body.id_type,
            list_filter: body.list_filter,
            list_display: body.list_display,
            list_exclude: body.list_exclude,
        });

        // Virgule optionnelle entre ressources
        skip_optional_punct(&mut iter, ',');
    }

    Ok(ParsedAdmin {
        resources,
        configures,
    })
}

/// Parse le bloc `configure { resource_key: { ... }, ... }`
fn parse_configure_block(iter: &mut TokenIter) -> Result<Vec<ConfigureDef>, String> {
    use proc_macro2::TokenTree;

    let group = match iter.next() {
        Some(TokenTree::Group(g)) => g,
        Some(other) => return Err(format!("Expected '{{' after configure, found: {}", other)),
        None => return Err("Expected '{{' after configure, end of file".to_string()),
    };

    let mut result = Vec::new();
    let mut inner: TokenIter = group.stream().into_iter().peekable();

    while inner.peek().is_some() {
        let key = match inner.next() {
            Some(TokenTree::Ident(id)) => id.to_string(),
            Some(TokenTree::Punct(p)) if p.as_char() == ',' => continue,
            Some(_) | None => continue,
        };

        expect_punct(&mut inner, ':')?;

        let body_group = match inner.next() {
            Some(TokenTree::Group(g)) => g,
            Some(other) => {
                return Err(format!(
                    "Expected '{{' for configure[\"{}\"] body, found: {}",
                    key, other
                ));
            }
            None => {
                return Err(format!(
                    "Expected '{{' for configure[\"{}\"] body, end of file",
                    key
                ));
            }
        };

        result.push(parse_configure_body(key, body_group.stream())?);
        skip_optional_punct(&mut inner, ',');
    }

    Ok(result)
}

/// Parse `{ list_display: [...], list_exclude: [...], list_filter: [...] }` pour un `configure` item
fn parse_configure_body(key: String, tokens: TokenStream) -> Result<ConfigureDef, String> {
    use proc_macro2::TokenTree;

    let mut iter: TokenIter = tokens.into_iter().peekable();
    let mut list_display = Vec::new();
    let mut list_exclude = Vec::new();
    let mut list_filter = Vec::new();

    while iter.peek().is_some() {
        let field = match iter.next() {
            Some(TokenTree::Ident(id)) => id.to_string(),
            Some(TokenTree::Punct(p)) if p.as_char() == ',' => continue,
            _ => continue,
        };

        expect_punct(&mut iter, ':')?;

        match field.as_str() {
            "list_display" => {
                list_display = parse_list_display(&mut iter)?;
            }
            "list_exclude" => {
                list_exclude = parse_list_exclude(&mut iter)?;
            }
            "list_filter" => {
                list_filter = parse_list_filter(&mut iter)?;
            }
            other => {
                skip_until_punct(&mut iter, ',');
                eprintln!("  Unknown field in configure[\"{}\"]: '{}'", key, other);
            }
        }
    }

    if !list_display.is_empty() && !list_exclude.is_empty() {
        return Err(format!(
            "configure[\"{key}\"]: list_display et list_exclude sont exclusifs"
        ));
    }

    Ok(ConfigureDef {
        key,
        list_display,
        list_exclude,
        list_filter,
    })
}

struct ResourceBody {
    title: String,
    permissions: Vec<String>,
    template_list: Option<String>,
    template_create: Option<String>,
    template_edit: Option<String>,
    template_detail: Option<String>,
    template_delete: Option<String>,
    extra_context: Vec<(String, String)>,
    create_form_type: Option<String>,
    edit_form_type: Option<String>,
    id_type: String,
    list_filter: Vec<(String, String, u64)>,
    list_display: Vec<(String, String)>,
    list_exclude: Vec<String>,
}

fn parse_resource_body(tokens: TokenStream) -> Result<ResourceBody, String> {
    use proc_macro2::TokenTree;

    let mut iter = tokens.into_iter().peekable();
    let mut body = ResourceBody {
        title: String::new(),
        permissions: Vec::new(),
        template_list: None,
        template_create: None,
        template_edit: None,
        template_detail: None,
        template_delete: None,
        extra_context: Vec::new(),
        create_form_type: None,
        edit_form_type: None,
        id_type: "I32".to_string(),
        list_filter: Vec::new(),
        list_display: Vec::new(),
        list_exclude: Vec::new(),
    };

    while iter.peek().is_some() {
        let field = match iter.next() {
            Some(TokenTree::Ident(id)) => id.to_string(),
            Some(TokenTree::Punct(p)) if p.as_char() == ',' => continue,
            _ => continue,
        };

        expect_punct(&mut iter, ':')?;

        match field.as_str() {
            "title" => {
                body.title = parse_string_literal(&mut iter)?;
            }
            "permissions" => {
                body.permissions = parse_permissions_array(&mut iter)?;
            }
            "template_list" => {
                body.template_list = Some(parse_string_literal(&mut iter)?);
            }
            "template_create" => {
                body.template_create = Some(parse_string_literal(&mut iter)?);
            }
            "template_edit" => {
                body.template_edit = Some(parse_string_literal(&mut iter)?);
            }
            "template_detail" => {
                body.template_detail = Some(parse_string_literal(&mut iter)?);
            }
            "template_delete" => {
                body.template_delete = Some(parse_string_literal(&mut iter)?);
            }
            "create_form" => {
                body.create_form_type = Some(parse_path(&mut iter)?);
            }
            "edit_form" => {
                body.edit_form_type = Some(parse_path(&mut iter)?);
            }
            "id_type" => {
                body.id_type = parse_ident(&mut iter)?;
            }
            "extra" => {
                body.extra_context = parse_extra_map(&mut iter)?;
            }
            "list_filter" => {
                body.list_filter = parse_list_filter(&mut iter)?;
            }
            "list_display" => {
                body.list_display = parse_list_display(&mut iter)?;
            }
            "list_exclude" => {
                body.list_exclude = parse_list_exclude(&mut iter)?;
            }
            other => {
                skip_until_punct(&mut iter, ',');
                eprintln!("  Unknown field in admin!{{}}: '{}'", other);
            }
        }
    }

    if body.title.is_empty() {
        return Err(t("parser.title_required").to_string());
    }
    if body.permissions.is_empty() {
        return Err(t("parser.permissions_required").to_string());
    }
    if !body.list_display.is_empty() && !body.list_exclude.is_empty() {
        return Err(t("parser.list_display_exclude_exclusive").to_string());
    }

    Ok(body)
}

/// Parse list_display: [["col", "Label"], ...]
fn parse_list_display(iter: &mut TokenIter) -> Result<Vec<(String, String)>, String> {
    parse_str_pair_array(iter, "list_display")
}

/// Parse list_exclude: ["col1", "col2", ...]
fn parse_list_exclude(iter: &mut TokenIter) -> Result<Vec<String>, String> {
    use proc_macro2::TokenTree;

    match iter.next() {
        Some(TokenTree::Group(outer)) => {
            let mut cols = Vec::new();
            let mut inner = outer.stream().into_iter().peekable();
            while inner.peek().is_some() {
                match inner.next() {
                    Some(TokenTree::Punct(p)) if p.as_char() == ',' => continue,
                    Some(TokenTree::Literal(lit)) => {
                        let s = lit.to_string();
                        if s.starts_with('"') && s.ends_with('"') {
                            cols.push(s[1..s.len() - 1].to_string());
                        } else {
                            return Err(format!(
                                "Expected string literal in list_exclude, found: {}",
                                s
                            ));
                        }
                    }
                    Some(other) => {
                        return Err(format!(
                            "Expected string literal in list_exclude, found: {}",
                            other
                        ));
                    }
                    None => break,
                }
            }
            Ok(cols)
        }
        Some(other) => Err(format!("Expected [...] for list_exclude, got {:?}", other)),
        None => Err("Expected [...] for list_exclude, end of file".to_string()),
    }
}

/// Parse list_filter: [["col_sql", "Label"], ...] ou [["col_sql", "Label", 10], ...]
fn parse_list_filter(iter: &mut TokenIter) -> Result<Vec<(String, String, u64)>, String> {
    use proc_macro2::TokenTree;

    match iter.next() {
        Some(TokenTree::Group(outer)) => {
            let mut entries = Vec::new();
            let mut inner = outer.stream().into_iter().peekable();
            while inner.peek().is_some() {
                match inner.next() {
                    Some(TokenTree::Punct(p)) if p.as_char() == ',' => continue,
                    Some(TokenTree::Group(pair)) => {
                        let mut t = pair.stream().into_iter().peekable();
                        let col = parse_string_literal(&mut t)?;
                        match t.next() {
                            Some(TokenTree::Punct(p)) if p.as_char() == ',' => {}
                            _ => return Err("Expected ',' after col in list_filter".to_string()),
                        }
                        let label = parse_string_literal(&mut t)?;
                        // 3ème élément optionnel : limit
                        let limit = match t.next() {
                            Some(TokenTree::Punct(p)) if p.as_char() == ',' => {
                                // virgule présente → lire le literal
                                parse_integer_literal(&mut t).unwrap_or(10)
                            }
                            _ => 10, // pas de 3ème élément → défaut 10
                        };
                        entries.push((col, label, limit));
                    }
                    Some(other) => {
                        return Err(format!(
                            "Expected [col, label] in list_filter, found: {}",
                            other
                        ));
                    }
                    None => break,
                }
            }
            Ok(entries)
        }
        other => Err(format!("Expected [...] for list_filter, got {:?}", other)),
    }
}

fn parse_integer_literal(iter: &mut TokenIter) -> Result<u64, String> {
    use proc_macro2::TokenTree;
    match iter.next() {
        Some(TokenTree::Literal(lit)) => lit
            .to_string()
            .parse::<u64>()
            .map_err(|_| format!("Expected integer literal, got '{}'", lit)),
        other => Err(format!("Expected integer literal, got {:?}", other)),
    }
}

/// Parseur générique pour [["str1", "str2"], ...] — utilisé par list_display et list_filter
fn parse_str_pair_array(
    iter: &mut TokenIter,
    field: &str,
) -> Result<Vec<(String, String)>, String> {
    use proc_macro2::TokenTree;

    match iter.next() {
        Some(TokenTree::Group(outer)) => {
            let mut pairs = Vec::new();
            let mut inner = outer.stream().into_iter().peekable();
            while inner.peek().is_some() {
                match inner.next() {
                    Some(TokenTree::Punct(p)) if p.as_char() == ',' => continue,
                    Some(TokenTree::Group(pair)) => {
                        let mut t = pair.stream().into_iter().peekable();
                        let col = parse_string_literal(&mut t)?;
                        match t.next() {
                            Some(TokenTree::Punct(p)) if p.as_char() == ',' => {}
                            _ => {
                                return Err(format!(
                                    "Expected ',' between col and label in {}",
                                    field
                                ));
                            }
                        }
                        let label = parse_string_literal(&mut t)?;
                        pairs.push((col, label));
                    }
                    Some(other) => {
                        return Err(format!(
                            "Expected [col, label] in {}, found: {}",
                            field, other
                        ));
                    }
                    None => break,
                }
            }
            Ok(pairs)
        }
        Some(other) => Err(format!("Expected [...] for {}, found: {}", field, other)),
        None => Err(format!("Expected [...] for {}, end of file", field)),
    }
}

/// Parse extra: { "key" => "value", ... }
fn parse_extra_map(iter: &mut TokenIter) -> Result<Vec<(String, String)>, String> {
    use proc_macro2::TokenTree;

    match iter.next() {
        Some(TokenTree::Group(group)) => {
            let mut pairs = Vec::new();
            let mut inner = group.stream().into_iter().peekable();

            while inner.peek().is_some() {
                // "key"
                let key = match inner.next() {
                    Some(TokenTree::Literal(lit)) => {
                        let s = lit.to_string();
                        if s.starts_with('"') && s.ends_with('"') {
                            s[1..s.len() - 1].to_string()
                        } else {
                            continue;
                        }
                    }
                    Some(TokenTree::Punct(p)) if p.as_char() == ',' => continue,
                    _ => continue,
                };

                // =>
                // consume '='
                match inner.next() {
                    Some(TokenTree::Punct(p)) if p.as_char() == '=' => {}
                    _ => return Err("Expected '=>' in extra map".to_string()),
                }
                // consume '>'
                match inner.next() {
                    Some(TokenTree::Punct(p)) if p.as_char() == '>' => {}
                    _ => return Err("Expected '>' after '=' in extra map".to_string()),
                }

                // "value"
                let value = match inner.next() {
                    Some(TokenTree::Literal(lit)) => {
                        let s = lit.to_string();
                        if s.starts_with('"') && s.ends_with('"') {
                            s[1..s.len() - 1].to_string()
                        } else {
                            return Err(format!(
                                "Expected string value in extra map, found: {}",
                                s
                            ));
                        }
                    }
                    Some(other) => {
                        return Err(format!(
                            "Expected string value in extra map, found: {}",
                            other
                        ));
                    }
                    None => {
                        return Err("Expected string value in extra map, end of file".to_string());
                    }
                };

                pairs.push((key, value));
            }

            Ok(pairs)
        }
        Some(other) => Err(format!(
            "Expected '{{...}}' for extra map, found: {}",
            other
        )),
        None => Err("Expected '{{...}}' for extra map, end of file".to_string()),
    }
}

type TokenIter = std::iter::Peekable<proc_macro2::token_stream::IntoIter>;

/// Parse un chemin de type (ex: users::Model, crate::models::users::Model)
fn parse_path(iter: &mut TokenIter) -> Result<String, String> {
    use proc_macro2::TokenTree;

    let mut path = String::new();

    loop {
        match iter.peek() {
            Some(TokenTree::Ident(_)) => {
                if let Some(TokenTree::Ident(id)) = iter.next() {
                    path.push_str(&id.to_string());
                }
            }
            Some(TokenTree::Punct(p)) if p.as_char() == ':' => {
                iter.next(); // premier ':'
                // Vérifie le deuxième ':'
                match iter.peek() {
                    Some(TokenTree::Punct(p2)) if p2.as_char() == ':' => {
                        iter.next();
                        path.push_str("::");
                    }
                    _ => break, // c'était le ':' de "key:"
                }
            }
            _ => break,
        }
    }

    if path.is_empty() {
        Err("Expected type path (e.g., crate::users::Model)".to_string())
    } else {
        Ok(path)
    }
}

/// Parse une chaîne littérale "..."
fn parse_string_literal(iter: &mut TokenIter) -> Result<String, String> {
    use proc_macro2::TokenTree;

    match iter.next() {
        Some(TokenTree::Literal(lit)) => {
            let s = lit.to_string();
            // Retire les guillemets
            if s.starts_with('"') && s.ends_with('"') {
                Ok(s[1..s.len() - 1].to_string())
            } else {
                Err(tf("parser.string_expected", &[&s]))
            }
        }
        Some(other) => Err(tf("parser.string_expected", &[&other.to_string()])),
        None => Err(t("parser.string_eof").to_string()),
    }
}

/// Parse un identifiant simple (ex: I32, I64, Uuid)
fn parse_ident(iter: &mut TokenIter) -> Result<String, String> {
    use proc_macro2::TokenTree;

    match iter.next() {
        Some(TokenTree::Ident(id)) => Ok(id.to_string()),
        Some(other) => Err(format!("Expected identifier, found: {}", other)),
        None => Err("Expected identifier, end of file".to_string()),
    }
}

/// Parse un tableau de permissions ["role1", "role2"]
fn parse_permissions_array(iter: &mut TokenIter) -> Result<Vec<String>, String> {
    use proc_macro2::TokenTree;

    match iter.next() {
        Some(TokenTree::Group(group)) => {
            let mut roles = Vec::new();
            let mut inner = group.stream().into_iter().peekable();

            while inner.peek().is_some() {
                match inner.next() {
                    Some(TokenTree::Literal(lit)) => {
                        let s = lit.to_string();
                        if s.starts_with('"') && s.ends_with('"') {
                            roles.push(s[1..s.len() - 1].to_string());
                        }
                    }
                    Some(TokenTree::Punct(p)) if p.as_char() == ',' => continue,
                    _ => continue,
                }
            }

            if roles.is_empty() {
                Err(t("parser.role_required").to_string())
            } else {
                Ok(roles)
            }
        }
        Some(other) => Err(tf("parser.array_expected", &[&other.to_string()])),
        None => Err(t("parser.array_eof").to_string()),
    }
}

/// Vérifie et consomme la ponctuation attendue
fn expect_punct(iter: &mut TokenIter, expected: char) -> Result<(), String> {
    use proc_macro2::TokenTree;

    match iter.next() {
        Some(TokenTree::Punct(p)) if p.as_char() == expected => Ok(()),
        Some(other) => Err(tf(
            "parser.punct_expected",
            &[&expected.to_string(), &other.to_string()],
        )),
        None => Err(tf("parser.punct_eof", &[&expected.to_string()])),
    }
}

/// Skip une ponctuation si présente (non bloquant)
fn skip_optional_punct(iter: &mut TokenIter, ch: char) {
    use proc_macro2::TokenTree;

    if let Some(TokenTree::Punct(p)) = iter.peek() {
        if p.as_char() == ch {
            iter.next();
        }
    }
}

/// Skip les tokens jusqu'à trouver une ponctuation donnée
fn skip_until_punct(iter: &mut TokenIter, ch: char) {
    use proc_macro2::TokenTree;

    while let Some(token) = iter.peek() {
        if let TokenTree::Punct(p) = token {
            if p.as_char() == ch {
                iter.next();
                return;
            }
        }
        iter.next();
    }
}
