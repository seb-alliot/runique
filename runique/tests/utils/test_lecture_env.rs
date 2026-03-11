// Tests pour env_or_default

use runique::utils::config::lecture_env::env_or_default;
use serial_test::serial;

#[test]
#[serial]
fn test_var_definie_retourne_valeur() {
    unsafe {
    std::env::set_var("_TEST_LENV_1", "valeur");
    }
    assert_eq!(env_or_default("_TEST_LENV_1", "defaut"), "valeur");
    unsafe {
    std::env::remove_var("_TEST_LENV_1");
    }
}

#[test]
#[serial]
fn test_var_absente_retourne_defaut() {
    unsafe {
    std::env::remove_var("_TEST_LENV_ABSENT");
    }
    assert_eq!(env_or_default("_TEST_LENV_ABSENT", "defaut"), "defaut");
}

#[test]
#[serial]
fn test_var_vide_retourne_chaine_vide_pas_le_defaut() {
    // Var définie mais vide → retourne "" (unwrap_or ne s'active pas)
    unsafe {
    std::env::set_var("_TEST_LENV_VIDE", "");
    }
    assert_eq!(env_or_default("_TEST_LENV_VIDE", "defaut"), "");
    unsafe {
    std::env::remove_var("_TEST_LENV_VIDE");
    }
}

#[test]
#[serial]
fn test_var_avec_espaces_conserve_espaces() {
    unsafe {
    std::env::set_var("_TEST_LENV_ESP", "  val  ");
    }
    assert_eq!(env_or_default("_TEST_LENV_ESP", "defaut"), "  val  ");
    unsafe {
    std::env::remove_var("_TEST_LENV_ESP");
    }
}

#[test]
fn test_retourne_type_string() {
    unsafe {
    std::env::remove_var("_TEST_LENV_TYPE");
    }
    let result: String = env_or_default("_TEST_LENV_TYPE", "valeur_par_defaut");
    assert_eq!(result, "valeur_par_defaut");
}
