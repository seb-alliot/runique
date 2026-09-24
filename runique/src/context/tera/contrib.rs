//! Re-registers the filters/functions/tests that Tera 1 shipped via its
//! `builtins` feature and that Tera 2 moved out of core into the `tera-contrib`
//! crate.
//!
//! Each item is registered under **its upstream name** and, when Tera 1 used a
//! different name, also under **the old name as an alias**. A template written
//! for Runique 2.1 therefore keeps working as-is, with no need for the
//! framework's documentation to diverge from Tera's.
//!
//! `register_filter` accepts any name, so an alias only costs one extra table
//! entry — the underlying function still exists just once.
//!
//! The only two upstream renames, whose old name is still accepted:
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

/// Registers the filters, functions and tests coming from `tera-contrib`.
///
/// Called by `register_asset_filters` before any template is loaded: Tera 2
/// checks that a filter exists when the template is added, not at render time.
pub(crate) fn register_contrib(tera: &mut Tera) {
    // ── Filters with an unchanged name since Tera 1 ──────────────────────────
    tera.register_filter("urlencode", urlencode);
    tera.register_filter("urlencode_strict", urlencode_strict);
    tera.register_filter("date", date);
    tera.register_filter("json_encode", json_encode);
    tera.register_filter("striptags", striptags);
    tera.register_filter("spaceless", spaceless);
    tera.register_filter("regex_replace", RegexReplace::default());

    // ── Filters renamed upstream: upstream name + Tera 1 alias ───────────────
    tera.register_filter("slug", slug);
    tera.register_filter("slugify", slug);
    tera.register_filter("filesize_format", filesize_format);
    tera.register_filter("filesizeformat", filesize_format);

    // ── Functions ────────────────────────────────────────────────────────────
    tera.register_function("now", now);
    tera.register_function("get_random", get_random);
    tera.register_filter("shuffle", shuffle);

    // ── Tests ────────────────────────────────────────────────────────────────
    tera.register_test("before", is_before);
    tera.register_test("after", is_after);
    tera.register_test("matching", Matching::default());
}
