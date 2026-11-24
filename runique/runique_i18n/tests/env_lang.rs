//! What the environment and the `.env` file say: the language of the locale
//! (`LANG`, `LC_ALL`, `LC_MESSAGES`) and the folder of the project's
//! translations (`LOCALES_DIR`).
//!
//! The environment and the current folder are the process's: one test at a
//! time, each putting back what it found.
use runique_i18n::{Lang, project_dir};
use serial_test::serial;
use std::{
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
};

/// Environment variables set for a test, their former values back after.
struct Env(Vec<(&'static str, Option<OsString>)>);

impl Env {
    fn with(vars: &[(&'static str, Option<&str>)]) -> Self {
        let before = vars
            .iter()
            .map(|(key, _)| (*key, std::env::var_os(key)))
            .collect();
        for (key, value) in vars {
            put(key, value.map(OsString::from));
        }
        Self(before)
    }

    /// The three locale keys, each set or removed.
    fn locale(lang: Option<&str>, lc_all: Option<&str>, lc_messages: Option<&str>) -> Self {
        Self::with(&[
            ("LANG", lang),
            ("LC_ALL", lc_all),
            ("LC_MESSAGES", lc_messages),
        ])
    }
}

impl Drop for Env {
    fn drop(&mut self) {
        for (key, value) in self.0.drain(..) {
            put(key, value);
        }
    }
}

fn put(key: &str, value: Option<OsString>) {
    // The tests of this file run one at a time: nothing reads the
    // environment while it changes.
    match value {
        Some(value) => unsafe { std::env::set_var(key, value) },
        None => unsafe { std::env::remove_var(key) },
    }
}

/// A folder holding a `.env`, made the current one for a test.
struct Folder {
    root: PathBuf,
    before: PathBuf,
}

impl Folder {
    fn with_dotenv(content: &str) -> Self {
        let root = std::env::temp_dir().join(format!("rq_env_lang_{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join(".env"), content).unwrap();
        let before = std::env::current_dir().unwrap();
        std::env::set_current_dir(&root).unwrap();
        Self { root, before }
    }
}

impl Drop for Folder {
    fn drop(&mut self) {
        let _ = std::env::set_current_dir(&self.before);
        let _ = fs::remove_dir_all(&self.root);
    }
}

#[test]
#[serial]
fn the_projects_folder_is_locales_unless_the_environment_names_another() {
    {
        let _env = Env::with(&[("LOCALES_DIR", None)]);
        assert_eq!(project_dir(), Path::new("locales"));
    }
    {
        let _env = Env::with(&[("LOCALES_DIR", Some(""))]);
        assert_eq!(project_dir(), Path::new("locales"), "empty is not a folder");
    }
    let _env = Env::with(&[("LOCALES_DIR", Some("/srv/site/translations"))]);
    assert_eq!(project_dir(), Path::new("/srv/site/translations"));
}

#[test]
#[serial]
fn the_environment_gives_the_language_of_its_locale() {
    {
        let _env = Env::locale(None, None, None);
        assert_eq!(Lang::from_env(), None, "no locale, no language");
    }
    {
        let _env = Env::locale(Some("fr_FR.UTF-8"), None, None);
        assert_eq!(Lang::from_env(), Some(Lang::Fr));
    }
    {
        let _env = Env::locale(None, Some("de_DE.UTF-8"), None);
        assert_eq!(Lang::from_env(), Some(Lang::De));
    }
    {
        let _env = Env::locale(None, None, Some("it_IT"));
        assert_eq!(Lang::from_env(), Some(Lang::It));
    }
    {
        let _env = Env::locale(Some("es"), Some("de"), Some("it"));
        assert_eq!(Lang::from_env(), Some(Lang::Es), "LANG comes first");
    }
    let _env = Env::locale(None, Some("de"), Some("it"));
    assert_eq!(Lang::from_env(), Some(Lang::De), "then LC_ALL");
}

// The file's own keys, by their name, whatever the environment says.
#[test]
#[serial]
fn the_dotenv_file_gives_its_own_language_not_the_environments() {
    let _env = Env::locale(Some("de_DE.UTF-8"), None, None);
    {
        let _folder = Folder::with_dotenv("OTHER=it\nLANG=fr\nLAST=es\n");
        assert_eq!(Lang::from_dotenv(), Some(Lang::Fr));
    }
    {
        let _folder = Folder::with_dotenv("OTHER=it\nLC_MESSAGES=ja\nLC_ALL=pt\n");
        assert_eq!(Lang::from_dotenv(), Some(Lang::Pt), "LC_ALL before LC_MESSAGES");
    }
    let _folder = Folder::with_dotenv("OTHER=it\nLANGUAGE=fr\n");
    assert_eq!(Lang::from_dotenv(), None, "no such key in the file");
}
