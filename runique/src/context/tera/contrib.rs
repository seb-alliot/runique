//! Réenregistrement des filtres/fonctions/tests que Tera 1 fournissait via sa
//! feature `builtins` et que Tera 2 a sortis du cœur vers le crate `tera-contrib`.
//!
//! Chaque élément est enregistré sous **son nom amont** et, quand Tera 1 utilisait
//! un nom différent, sous **l'ancien nom en alias**. Un template écrit pour
//! Runique 2.1 continue donc de fonctionner tel quel, sans que la documentation
//! du framework ait à diverger de celle de Tera.
//!
//! `register_filter` accepte n'importe quel nom, donc un alias ne coûte qu'une
//! entrée de plus dans la table — la fonction, elle, n'existe qu'une fois.
//!
//! Les deux seuls renommages amont, dont l'ancien nom reste accepté :
//! `slugify` → `slug`, `filesizeformat` → `filesize_format`.

use tera::Tera;
use tera_contrib::{
    dates::{date, is_after, is_before, now},
    filesize_format::filesize_format,
    json::json_encode,
    rand::{get_random, shuffle},
    regex::{Matching, RegexReplace, spaceless, striptags},
    slug::slug,
    urlencode::{urlencode, urlencode_strict},
};

/// Enregistre les filtres, fonctions et tests issus de `tera-contrib`.
///
/// Appelé par `register_asset_filters` avant tout chargement de template : Tera 2
/// vérifie l'existence des filtres au moment où le template est ajouté, pas au
/// rendu.
pub(crate) fn register_contrib(tera: &mut Tera) {
    // ── Filtres au nom inchangé depuis Tera 1 ────────────────────────────────
    tera.register_filter("urlencode", urlencode);
    tera.register_filter("urlencode_strict", urlencode_strict);
    tera.register_filter("date", date);
    tera.register_filter("json_encode", json_encode);
    tera.register_filter("striptags", striptags);
    tera.register_filter("spaceless", spaceless);
    tera.register_filter("regex_replace", RegexReplace::default());

    // ── Filtres renommés en amont : nom amont + alias Tera 1 ─────────────────
    tera.register_filter("slug", slug);
    tera.register_filter("slugify", slug);
    tera.register_filter("filesize_format", filesize_format);
    tera.register_filter("filesizeformat", filesize_format);

    // ── Fonctions ────────────────────────────────────────────────────────────
    tera.register_function("now", now);
    tera.register_function("get_random", get_random);
    tera.register_filter("shuffle", shuffle);

    // ── Tests ────────────────────────────────────────────────────────────────
    tera.register_test("before", is_before);
    tera.register_test("after", is_after);
    tera.register_test("matching", Matching::default());
}
