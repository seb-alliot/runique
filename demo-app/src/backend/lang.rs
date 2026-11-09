//! The language a visitor picks with the FR/EN button of the navbar, kept in a
//! cookie: `set_lang` is global to the process, so one visitor would switch it
//! for everyone.
use crate::entities::{doc_page, doc_section};
use runique::axum::http::{HeaderMap, HeaderValue, header};
use runique::prelude::*;

const ONE_YEAR: u32 = 31_536_000;

/// Only ever `"fr"` or `"en"`: the cookie is the client's to write, and this
/// value reaches templates and SQL filters.
pub fn visitor_lang(headers: &HeaderMap) -> &'static str {
    let chosen = headers
        .get_all(header::COOKIE)
        .iter()
        .filter_map(|value| value.to_str().ok())
        .flat_map(|value| value.split(';'))
        .find_map(|pair| pair.trim().strip_prefix("lang="));
    if chosen == Some("fr") { "fr" } else { "en" }
}

/// `en` or `fr`, for the visitor's language.
pub fn msg(headers: &HeaderMap, en: &'static str, fr: &'static str) -> &'static str {
    if visitor_lang(headers) == "fr" {
        fr
    } else {
        en
    }
}

/// `clean()` has no access to the request, so its errors are written in English
/// and translated here once the form has been validated.
fn clean_error_fr(en: &str) -> Option<&'static str> {
    Some(match en {
        "The username must be at least 5 characters long." => {
            "Le nom d'utilisateur doit faire au moins 5 caractères."
        }
        "The username cannot contain #, —, <, >, &, \" or '." => {
            "Le nom d'utilisateur ne peut pas contenir #, —, <, >, &, \" ou '."
        }
        "The password must be at least 10 characters long." => {
            "Le mot de passe doit faire au moins 10 caractères."
        }
        "The password must contain at least one uppercase letter, one digit and one special character." => {
            "Le mot de passe doit contenir au moins une majuscule, un chiffre et un caractère spécial."
        }
        "The title must be at least 3 characters long." => {
            "Le titre doit faire au moins 3 caractères."
        }
        "The website must start with http or https." => {
            "Le site web doit commencer par http ou https."
        }
        "Invalid email address." => "Adresse email invalide.",
        "The summary must be at least 5 characters long." => {
            "Le résumé doit faire au moins 5 caractères."
        }
        "The content must be at least 10 characters long." => {
            "Le contenu doit faire au moins 10 caractères."
        }
        _ => return None,
    })
}

pub fn localize_errors(form: &mut Forms, headers: &HeaderMap) {
    if visitor_lang(headers) != "fr" {
        return;
    }
    for field in form.fields.values_mut() {
        if let Some(error) = field.error().and_then(|error| clean_error_fr(error)) {
            field.set_error(error.to_string());
        }
    }
}

/// The page to go back to: a path of this site, never another host. `//host`
/// and `/\host` are read as another host by browsers, and `Redirect::to`
/// panics on anything that isn't a valid header value.
fn local_path(next: Option<&str>) -> &str {
    next.filter(|path| {
        path.starts_with('/')
            && !path.starts_with("//")
            && !path.contains('\\')
            && path.chars().all(|c| c.is_ascii_graphic())
    })
    .unwrap_or("/")
}

/// A course has the same slug in both languages; `/cours/{slug}` is the shape
/// from before the language was in the URL.
fn course_path_in_language(path: &str, lang: &str) -> Option<String> {
    let rest = path.strip_prefix("/cours")?;
    if !rest.is_empty() && !rest.starts_with('/') {
        return None;
    }
    let mut parts = rest.split('/').filter(|part| !part.is_empty());
    let slug = match parts.next() {
        Some("fr" | "en") => parts.next(),
        other => other,
    };
    Some(match slug {
        Some(slug) => format!("/cours/{lang}/{slug}"),
        None => format!("/cours/{lang}"),
    })
}

/// The same page in `lang` for pages whose language is in their URL. Doc pages
/// don't always have the same name in both languages: without an equivalent,
/// the section, then the documentation's index.
async fn path_in_language(path: &str, lang: &str, request: &Request) -> String {
    if path == "/readme/fr" || path == "/readme/en" {
        return format!("/readme/{lang}");
    }
    if let Some(course) = course_path_in_language(path, lang) {
        return course;
    }
    let Some(rest) = path.strip_prefix("/docs/") else {
        return path.to_string();
    };
    let mut parts = rest.split('/').filter(|part| !part.is_empty()).skip(1);
    let home = format!("/docs/{lang}");
    let Some(section) = parts.next() else {
        return home;
    };
    let db = request.engine.db.clone();
    let Ok(Some(found)) = search!(doc_section::Entity => Lang eq lang, Slug eq section)
        .first(&db)
        .await
    else {
        return home;
    };
    let section_url = format!("{home}/{section}");
    let Some(page) = parts.next() else {
        return section_url;
    };
    let full_slug = format!("{section}-{page}");
    match search!(doc_page::Entity => SectionId eq found.id, Slug eq full_slug)
        .first(&db)
        .await
    {
        Ok(Some(_)) => format!("{section_url}/{page}"),
        _ => section_url,
    }
}

fn lang_cookie(lang: &str, dev: bool) -> String {
    let secure = if dev { "" } else { "; Secure" };
    // Lax, not Strict: a visitor coming from another site keeps their language.
    format!("lang={lang}; Path=/; Max-Age={ONE_YEAR}; SameSite=Lax; HttpOnly{secure}")
}

/// Remembers the language and goes back to the page. A plain link, so a GET: the
/// worst another site can do with it is change a visitor's language.
pub async fn switch_language(code: &str, next: Option<&str>, request: &Request) -> Response {
    let lang = match code {
        "fr" => "fr",
        "en" => "en",
        _ => return StatusCode::NOT_FOUND.into_response(),
    };
    let target = path_in_language(local_path(next), lang, request).await;
    let mut response = Redirect::to(&target).into_response();
    if let Ok(cookie) = HeaderValue::from_str(&lang_cookie(lang, request.engine.config.dev)) {
        response.headers_mut().append(header::SET_COOKIE, cookie);
    }
    response
}

#[cfg(test)]
mod tests {
    use super::*;

    fn headers(cookie: &str) -> HeaderMap {
        let mut map = HeaderMap::new();
        map.insert(header::COOKIE, HeaderValue::from_str(cookie).unwrap());
        map
    }

    #[test]
    fn the_cookie_only_selects_french_or_english() {
        assert_eq!(visitor_lang(&headers("lang=fr")), "fr");
        assert_eq!(visitor_lang(&headers("id=abc; lang=fr")), "fr");
        assert_eq!(visitor_lang(&headers("lang=en")), "en");
        assert_eq!(visitor_lang(&headers("lang=<script>")), "en");
        assert_eq!(visitor_lang(&headers("id=abc")), "en");
        assert_eq!(visitor_lang(&HeaderMap::new()), "en");
    }

    #[test]
    fn only_a_path_of_this_site_is_kept() {
        assert_eq!(local_path(Some("/docs/en/orm")), "/docs/en/orm");
        for refused in [
            "https://evil.example",
            "//evil.example",
            "/\\evil.example",
            "evil.example",
            "/a b",
            "/a\r\nSet-Cookie: x=y",
            "/é",
            "",
        ] {
            assert_eq!(local_path(Some(refused)), "/", "{refused:?}");
        }
        assert_eq!(local_path(None), "/");
    }

    #[test]
    fn a_course_page_is_swapped_to_the_other_language() {
        assert_eq!(
            course_path_in_language("/cours", "en").as_deref(),
            Some("/cours/en")
        );
        assert_eq!(
            course_path_in_language("/cours/", "en").as_deref(),
            Some("/cours/en")
        );
        assert_eq!(
            course_path_in_language("/cours/fr", "en").as_deref(),
            Some("/cours/en")
        );
        assert_eq!(
            course_path_in_language("/cours/fr/serde", "en").as_deref(),
            Some("/cours/en/serde")
        );
        assert_eq!(
            course_path_in_language("/cours/en/serde", "fr").as_deref(),
            Some("/cours/fr/serde")
        );
        assert_eq!(
            course_path_in_language("/cours/serde", "fr").as_deref(),
            Some("/cours/fr/serde")
        );
        assert_eq!(course_path_in_language("/coursx", "fr"), None);
        assert_eq!(course_path_in_language("/docs/fr/cour", "en"), None);
    }

    #[test]
    fn the_cookie_is_secure_outside_development() {
        assert!(lang_cookie("fr", false).ends_with("; Secure"));
        assert!(!lang_cookie("fr", true).contains("Secure"));
        assert!(lang_cookie("en", true).starts_with("lang=en; Path=/;"));
    }
}
