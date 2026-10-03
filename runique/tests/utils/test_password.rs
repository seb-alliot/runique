// Tests pour BaseHash, PasswordService, PasswordConfig
//
// Note: bcrypt (DEFAULT_COST=12) and scrypt are too slow to be
// tested in round-trip. We limit to Argon2 for hash+verify tests.
// Les autres algorithmes sont couverts via detect_algorithm / is_already_hashed.

use runique::utils::password::{
    AutoConfig, BaseHash, External, Manual, PasswordConfig, PasswordService,
};

// ═══════════════════════════════════════════════════════════════
// BaseHash — detect_algorithm (pure, pas d'IO)
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_detect_argon2id() {
    let bh = BaseHash::new();
    assert_eq!(bh.detect_algorithm("$argon2id$v=19$..."), Some("argon2"));
}

#[test]
fn test_detect_argon2i() {
    let bh = BaseHash::new();
    assert_eq!(bh.detect_algorithm("$argon2i$v=19$..."), Some("argon2"));
}

#[test]
fn test_detect_bcrypt_2b() {
    let bh = BaseHash::new();
    assert_eq!(bh.detect_algorithm("$2b$12$fakehash"), Some("bcrypt"));
}

#[test]
fn test_detect_bcrypt_2a() {
    let bh = BaseHash::new();
    assert_eq!(bh.detect_algorithm("$2a$12$fakehash"), Some("bcrypt"));
}

#[test]
fn test_detect_scrypt() {
    let bh = BaseHash::new();
    assert_eq!(bh.detect_algorithm("$scrypt$ln=17,..."), Some("scrypt"));
}

#[test]
fn test_detect_texte_inconnu_retourne_none() {
    let bh = BaseHash::new();
    assert!(bh.detect_algorithm("motdepasse").is_none());
}

#[test]
fn test_detect_vide_retourne_none() {
    let bh = BaseHash::new();
    assert!(bh.detect_algorithm("").is_none());
}

// ═══════════════════════════════════════════════════════════════
// BaseHash — verify avec hash invalide
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_verify_hash_invalide_retourne_false() {
    let bh = BaseHash::new();
    assert!(!bh.verify("password", "pas-un-hash"));
}

#[test]
fn test_verify_vide_retourne_false() {
    let bh = BaseHash::new();
    assert!(!bh.verify("password", ""));
}

// ═══════════════════════════════════════════════════════════════
// PasswordService — is_already_hashed
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_is_already_hashed_argon2id() {
    let svc = PasswordService::new(PasswordConfig::auto());
    assert!(svc.is_already_hashed("$argon2id$v=19$..."));
}

#[test]
fn test_is_already_hashed_argon2i() {
    let svc = PasswordService::new(PasswordConfig::auto());
    assert!(svc.is_already_hashed("$argon2i$v=19$..."));
}

#[test]
fn test_is_already_hashed_argon2d() {
    let svc = PasswordService::new(PasswordConfig::auto());
    assert!(svc.is_already_hashed("$argon2d$v=19$..."));
}

#[test]
fn test_is_already_hashed_bcrypt() {
    let svc = PasswordService::new(PasswordConfig::auto());
    assert!(svc.is_already_hashed("$2b$12$somehash"));
}

#[test]
fn test_is_already_hashed_scrypt() {
    let svc = PasswordService::new(PasswordConfig::auto());
    assert!(svc.is_already_hashed("$scrypt$ln=17,..."));
}

#[test]
fn test_is_already_hashed_plaintext_false() {
    let svc = PasswordService::new(PasswordConfig::auto());
    assert!(!svc.is_already_hashed("motdepasse"));
}

#[test]
fn test_is_already_hashed_vide_false() {
    let svc = PasswordService::new(PasswordConfig::auto());
    assert!(!svc.is_already_hashed(""));
}

// ═══════════════════════════════════════════════════════════════
// PasswordService — hash + verify Argon2 (round-trip)
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_argon2_hash_produit_prefixe_argon2() {
    let svc = PasswordService::new(PasswordConfig::auto());
    let hash = svc.hash("motdepasse_secret").unwrap();
    assert!(hash.starts_with("$argon2"));
}

#[test]
fn test_argon2_verify_correct() {
    let svc = PasswordService::new(PasswordConfig::auto());
    let hash = svc.hash("correct").unwrap();
    assert!(svc.verify("correct", &hash));
}

#[test]
fn test_argon2_verify_incorrect_false() {
    let svc = PasswordService::new(PasswordConfig::auto());
    let hash = svc.hash("bon").unwrap();
    assert!(!svc.verify("mauvais", &hash));
}

#[test]
fn test_hash_vide_retourne_erreur_allow_empty_false() {
    let svc = PasswordService::new(PasswordConfig::auto()); // allow_empty=false by default
    assert!(svc.hash("").is_err());
}

#[test]
fn test_hash_ressemblant_a_argon2_retourne_erreur() {
    let svc = PasswordService::new(PasswordConfig::auto());
    assert!(svc.hash("$argon2id$fake_input").is_err());
}

#[test]
fn test_hash_ressemblant_a_bcrypt_retourne_erreur() {
    let svc = PasswordService::new(PasswordConfig::auto());
    assert!(svc.hash("$2b$12$deja_hache").is_err());
}

// ═══════════════════════════════════════════════════════════════
// PasswordService — is_algorithm_current
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_is_algorithm_current_argon2_correspond() {
    let svc = PasswordService::new(PasswordConfig::auto()); // algo=Argon2
    let hash = svc.hash("test").unwrap();
    assert!(svc.is_algorithm_current(&hash));
}

#[test]
fn test_is_algorithm_current_bcrypt_ne_correspond_pas_a_argon2() {
    let svc = PasswordService::new(PasswordConfig::auto()); // algo=Argon2
    assert!(!svc.is_algorithm_current("$2b$12$fakebcrypthash"));
}

#[test]
fn test_is_algorithm_current_manual_toujours_true() {
    let svc = PasswordService::new(PasswordConfig::manual(Manual::Argon2));
    // Manual mode → always true (no detection)
    assert!(svc.is_algorithm_current("$2b$12$nimportequoi"));
}

// ═══════════════════════════════════════════════════════════════
// PasswordService — mode Manual(Argon2)
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_manual_argon2_roundtrip() {
    let svc = PasswordService::new(PasswordConfig::manual(Manual::Argon2));
    let hash = svc.hash("secret").unwrap();
    assert!(svc.verify("secret", &hash));
    assert!(!svc.verify("wrong", &hash));
}

// ═══════════════════════════════════════════════════════════════
// PasswordService — mode Delegated (OAuth)
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_delegated_hash_retourne_erreur() {
    let svc = PasswordService::new(PasswordConfig::oauth(External::GoogleOAuth));
    assert!(svc.hash("password").is_err());
}

#[test]
fn test_delegated_verify_retourne_false() {
    let svc = PasswordService::new(PasswordConfig::oauth(External::GoogleOAuth));
    assert!(!svc.verify("password", "$argon2id$fake"));
}

// ═══════════════════════════════════════════════════════════════
// PasswordConfig — constructeurs
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_password_config_default_est_auto() {
    assert!(matches!(PasswordConfig::default(), PasswordConfig::Auto(_)));
}

#[test]
fn test_password_config_auto_est_auto() {
    assert!(matches!(PasswordConfig::auto(), PasswordConfig::Auto(_)));
}

#[test]
fn test_password_config_auto_with_bcrypt() {
    let cfg = PasswordConfig::auto_with(Manual::Bcrypt);
    assert!(matches!(cfg, PasswordConfig::Auto(_)));
}

#[test]
fn test_password_config_manual_scrypt() {
    let cfg = PasswordConfig::manual(Manual::Scrypt);
    assert!(matches!(cfg, PasswordConfig::Manual(Manual::Scrypt)));
}

#[test]
fn test_password_config_oauth_google() {
    let cfg = PasswordConfig::oauth(External::GoogleOAuth);
    assert!(matches!(
        cfg,
        PasswordConfig::Delegated(External::GoogleOAuth)
    ));
}

// ═══════════════════════════════════════════════════════════════
// AutoConfig — default values
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_auto_config_default_algo_argon2() {
    let cfg = AutoConfig::default();
    assert!(matches!(cfg.algorithm, Manual::Argon2));
}

#[test]
fn test_auto_config_default_allow_empty_false() {
    let cfg = AutoConfig::default();
    assert!(!cfg.allow_empty);
}

#[test]
fn test_auto_config_default_pas_de_hook() {
    let cfg = AutoConfig::default();
    assert!(cfg.pre_hash_hook.is_none());
}

// ── Written from cargo-mutants survivors (2026-10-02) ───────────────────────

mod guarantees {
    use runique::forms::FormField;
    use runique::forms::fields::TextField;
    use runique::utils::password::{
        AutoConfig, BaseHash, PasswordConfig, PasswordHandler, PasswordService, dummy_hash,
    };

    /// A scrypt hash of "s3cret" with the cheapest parameters (ln=4), made
    /// outside Rust (Python `hashlib.scrypt`): verification reads its cost
    /// from the hash, so this runs in milliseconds where hashing with the
    /// default cost takes ~30 s in a debug build.
    const SCRYPT_S3CRET: &str =
        "$scrypt$ln=4,r=8,p=1$cnVuaXF1ZXRlc3RzYWx0IQ$IrYGuYkWi1qLJdbUB8hBYvQLZ+VwWD821t29b3yqt9s";

    #[test]
    fn scrypt_verifies_the_right_password_only() {
        let base = BaseHash::new();
        assert!(base.verify("s3cret", SCRYPT_S3CRET), "right password");
        assert!(!base.verify("other", SCRYPT_S3CRET), "wrong password");
    }

    /// The hash an unknown username is checked against, so the response takes
    /// as long as for a real account (no user enumeration by timing).
    #[test]
    fn the_dummy_hash_is_a_real_argon2_hash() {
        assert!(dummy_hash().starts_with("$argon2"), "{}", dummy_hash());
        assert!(!BaseHash::new().verify("anything", dummy_hash()));
    }

    #[test]
    fn an_empty_password_is_refused_by_default() {
        let service = PasswordService::new(PasswordConfig::auto());
        let err = service.hash("").unwrap_err();
        assert!(
            err.contains("Empty password not allowed"),
            "refused by the policy: {err}"
        );
    }

    #[test]
    fn auto_config_clone_and_debug_keep_its_settings() {
        let config = AutoConfig {
            allow_empty: true,
            ..Default::default()
        };
        assert!(config.clone().allow_empty);
        assert!(format!("{config:?}").contains("allow_empty: true"));
        assert!(matches!(
            PasswordConfig::auto(),
            PasswordConfig::Auto(AutoConfig {
                allow_empty: false,
                ..
            })
        ));
    }

    #[derive(Clone)]
    struct Plain;

    impl PasswordHandler for Plain {
        fn name(&self) -> &str {
            "plain-handler"
        }
        fn create_field(&self, name: &str) -> Box<dyn FormField> {
            Box::new(TextField::password(name))
        }
        fn validate_input(&self, _input: &str) -> Result<(), String> {
            Ok(())
        }
        fn transform(&self, input: &str) -> Result<String, String> {
            Ok(format!("plain:{input}"))
        }
        fn verify(&self, input: &str, stored: &str) -> bool {
            stored == format!("plain:{input}")
        }
    }

    #[test]
    fn a_custom_handler_is_used_and_named() {
        let config = PasswordConfig::custom(Plain);
        let PasswordConfig::Custom(handler) = &config else {
            panic!("custom() gives a Custom config");
        };
        assert!(format!("{handler:?}").contains("plain-handler"));
        let service = PasswordService::new(config);
        assert_eq!(service.hash("pw").unwrap(), "plain:pw");
    }
}
