// ═══════════════════════════════════════════════════════════════
// Parser — Analyse de src/admin.rs
// ═══════════════════════════════════════════════════════════════
//
// Lit le fichier admin.rs du projet développeur et extrait
// les déclarations de ressources du macro admin!{}.
//
// Syntaxe parsée :
//
//   admin! {
//       users: users::Model => RegisterForm {
//           title: "Utilisateurs",
//           permissions: ["admin"]
//       }
//       blog: blog::Model => BlogForm {
//           title: "Articles",
//           permissions: ["admin", "editor"]
//       }
//   }
//
// Stratégie : syn::File pour localiser le macro admin!,
// puis parsing manuel du TokenStream interne.
// ═══════════════════════════════════════════════════════════════

use proc_macro2::TokenStream;
use syn::{parse_file, visit::Visit, Macro};

/// Définition d'une ressource extraite du macro admin!
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
}

/// Résultat du parsing de src/admin.rs
#[derive(Debug)]
pub struct ParsedAdmin {
    pub resources: Vec<ResourceDef>,
}

/// Parse le contenu de src/admin.rs et retourne les ressources déclarées
pub fn parse_admin_file(source: &str) -> Result<ParsedAdmin, String> {
    let syntax = parse_file(source).map_err(|e| format!("Erreur de syntaxe Rust: {}", e))?;

    let mut visitor = AdminMacroVisitor::new();
    visitor.visit_file(&syntax);

    if let Some(err) = visitor.error {
        return Err(err);
    }

    Ok(ParsedAdmin {
        resources: visitor.resources,
    })
}

// ───────────────────────────────────────────────
// Visitor syn — localise le macro admin!
// ───────────────────────────────────────────────

struct AdminMacroVisitor {
    pub resources: Vec<ResourceDef>,
    pub error: Option<String>,
}

impl AdminMacroVisitor {
    fn new() -> Self {
        Self {
            resources: Vec::new(),
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
            Ok(resources) => self.resources = resources,
            Err(e) => self.error = Some(e),
        }
    }
}

// ───────────────────────────────────────────────
// Parser du TokenStream interne de admin!{}
// ───────────────────────────────────────────────
//
// Syntaxe attendue :
//   key: path::Model => FormType {
//       title: "...",
//       permissions: ["role1", "role2"]
//   }

fn parse_admin_tokens(tokens: TokenStream) -> Result<Vec<ResourceDef>, String> {
    use proc_macro2::TokenTree;

    let mut resources = Vec::new();
    let mut iter = tokens.into_iter().peekable();

    while iter.peek().is_some() {
        // 1. key (ident)
        let key = match iter.next() {
            Some(TokenTree::Ident(id)) => id.to_string(),
            Some(other) => return Err(format!("Attendu nom de ressource, trouvé: {}", other)),
            None => break,
        };

        // 2. ':'
        expect_punct(&mut iter, ':')?;

        // 3. model_type (path: ident :: ident...)
        let model_type = parse_path(&mut iter)?;

        // 4. '=>'
        expect_punct(&mut iter, '=')?;
        expect_punct(&mut iter, '>')?;

        // 5. form_type (ident simple)
        let form_type = match iter.next() {
            Some(TokenTree::Ident(id)) => id.to_string(),
            Some(other) => return Err(format!("Attendu nom de Form, trouvé: {}", other)),
            None => return Err("Attendu nom de Form, fin de fichier".to_string()),
        };

        // 6. { title: "...", permissions: [...] }
        let (title, permissions) = match iter.next() {
            Some(TokenTree::Group(group)) => parse_resource_body(group.stream())?,
            Some(other) => return Err(format!("Attendu '{{', trouvé: {}", other)),
            None => return Err("Attendu '{{', fin de fichier".to_string()),
        };

        resources.push(ResourceDef {
            key,
            model_type,
            form_type,
            title,
            permissions,
        });

        // Virgule optionnelle entre ressources
        skip_optional_punct(&mut iter, ',');
    }

    Ok(resources)
}

// ───────────────────────────────────────────────
// Parser du corps d'une ressource { title, permissions }
// ───────────────────────────────────────────────

fn parse_resource_body(tokens: TokenStream) -> Result<(String, Vec<String>), String> {
    use proc_macro2::TokenTree;

    let mut iter = tokens.into_iter().peekable();
    let mut title = String::new();
    let mut permissions = Vec::new();

    while iter.peek().is_some() {
        let field = match iter.next() {
            Some(TokenTree::Ident(id)) => id.to_string(),
            Some(TokenTree::Punct(p)) if p.as_char() == ',' => continue,
            _ => continue,
        };

        // ':'
        expect_punct(&mut iter, ':')?;

        match field.as_str() {
            "title" => {
                title = parse_string_literal(&mut iter)?;
            }
            "permissions" => {
                permissions = parse_permissions_array(&mut iter)?;
            }
            other => {
                // Champ inconnu → on skip jusqu'à la prochaine virgule
                skip_until_punct(&mut iter, ',');
                eprintln!("⚠️  Champ inconnu dans admin!{{}}: '{}'", other);
            }
        }
    }

    if title.is_empty() {
        return Err("Champ 'title' manquant dans la déclaration admin!{}".to_string());
    }
    if permissions.is_empty() {
        return Err("Champ 'permissions' manquant dans la déclaration admin!{}".to_string());
    }

    Ok((title, permissions))
}

// ───────────────────────────────────────────────
// Helpers de parsing
// ───────────────────────────────────────────────

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
        Err("Attendu chemin de type (ex: users::Model)".to_string())
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
                Err(format!("Attendu chaîne littérale, trouvé: {}", s))
            }
        }
        Some(other) => Err(format!("Attendu chaîne littérale, trouvé: {}", other)),
        None => Err("Attendu chaîne littérale, fin de fichier".to_string()),
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
                Err("Au moins un rôle requis dans permissions: [...]".to_string())
            } else {
                Ok(roles)
            }
        }
        Some(other) => Err(format!("Attendu [...] pour permissions, trouvé: {}", other)),
        None => Err("Attendu [...] pour permissions, fin de fichier".to_string()),
    }
}

/// Vérifie et consomme la ponctuation attendue
fn expect_punct(iter: &mut TokenIter, expected: char) -> Result<(), String> {
    use proc_macro2::TokenTree;

    match iter.next() {
        Some(TokenTree::Punct(p)) if p.as_char() == expected => Ok(()),
        Some(other) => Err(format!("Attendu '{}', trouvé: {}", expected, other)),
        None => Err(format!("Attendu '{}', fin de fichier", expected)),
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
