use crate::gardefou::composant_middleware::csrf_middleware::CsrfTokenFunction;

/// Enregistre la fonction CSRF dans Tera pour pouvoir l'utiliser dans les templates
pub fn register_csrf_token(tera: &mut tera::Tera) {
    tera.register_function("csrf_token", CsrfTokenFunction);
}

