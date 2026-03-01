//! Tests — utils/init_error/init.rs
//! Couvre : init_logging (appels simples, idempotence)

use runique::utils::init_error::init::init_logging;

#[test]
fn test_init_logging_ne_panique_pas() {
    // Premier appel : initialise le logger
    init_logging();
}

#[test]
fn test_init_logging_idempotent() {
    // Appels multiples ne doivent pas paniquer (le logger est déjà init)
    init_logging();
    init_logging();
    init_logging();
}
