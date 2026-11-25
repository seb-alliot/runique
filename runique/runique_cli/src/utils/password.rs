//! Password hashing and verification — Argon2, bcrypt, scrypt via `PasswordHasher` trait.
//! Copied from `runique::utils::password`: the algorithms, without the
//! application's configuration around them.

// === password/hasher.rs ===
// argon2 and scrypt are now both on password-hash 0.6 — a single
// shared set of imports (unlike before: scrypt was stuck on 0.5,
// an incompatible version despite identically named traits/types).
use argon2::{
    Argon2,
    password_hash::{PasswordHasher as _, PasswordVerifier as _, phc::PasswordHash},
};
use bcrypt::{DEFAULT_COST, hash as bcrypt_hash, verify as bcrypt_verify};
use scrypt::Scrypt;

use dyn_clone::DynClone;
use runique_i18n::{t, tf};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

/// Interface for a custom password hashing algorithm, plugged in via
/// `Manual::Custom`. Implementors must be `Send + Sync + Debug` and cloneable
/// behind a `Box<dyn PasswordHasher>` (via `DynClone`), since the config that
/// holds it is cloned freely.
pub trait PasswordHasher: Send + Sync + DynClone + Debug {
    /// Hashes a plaintext password, returning the encoded string to store.
    fn hash(&self, password: &str) -> Result<String, String>;
    /// Checks a plaintext password against a previously stored hash.
    fn verify(&self, password: &str, hash: &str) -> bool;
    /// Short identifier for the algorithm (e.g. `"argon2"`), for diagnostics.
    fn algorithm_name(&self) -> &str;
}

impl Clone for Box<dyn PasswordHasher> {
    fn clone(&self) -> Box<dyn PasswordHasher> {
        dyn_clone::clone_box(&**self)
    }
}

/// Built-in hasher supporting Argon2, bcrypt and Scrypt. The algorithm to use
/// for hashing is passed explicitly (via [`Manual`]); verification instead
/// auto-detects the algorithm from the stored hash's prefix.
#[derive(Debug, Clone)]
pub struct BaseHash;

/// Runs `verify_once(hash)`. If it errors — the hash doesn't even parse — re-runs it
/// against `dummy` to burn the same computation time before returning `false`: a
/// malformed stored hash must not answer faster than a well-formed one with the wrong
/// password, or timing leaks which stored hashes are well-formed (same principle as
/// `dummy_hash()` for user enumeration, applied to the parse step of each algorithm).
// A hash names its own cost, read again at each verification. One asking for
// more than these, beyond the defaults Runique hashes with, is refused before
// running: a crafted one would make a single sign-in spend unbounded memory or time.
const MAX_ARGON2_M_COST_KIB: u32 = 64 * 1024;
const MAX_ARGON2_T_COST: u32 = 8;
const MAX_ARGON2_P_COST: u32 = 4;
const MAX_SCRYPT_LOG_N: u8 = 17;
const MAX_SCRYPT_R: u32 = 8;
const MAX_SCRYPT_P: u32 = 4;
const MAX_BCRYPT_COST: u32 = 14;

fn argon2_cost_allowed(hash: &PasswordHash) -> bool {
    argon2::Params::try_from(hash).is_ok_and(|p| {
        p.m_cost() <= MAX_ARGON2_M_COST_KIB
            && p.t_cost() <= MAX_ARGON2_T_COST
            && p.p_cost() <= MAX_ARGON2_P_COST
    })
}

fn scrypt_cost_allowed(hash: &PasswordHash) -> bool {
    scrypt::Params::try_from(hash).is_ok_and(|p| {
        p.log_n() <= MAX_SCRYPT_LOG_N && p.r() <= MAX_SCRYPT_R && p.p() <= MAX_SCRYPT_P
    })
}

/// `$2b$12$…`: the cost is the two digits after the version.
fn bcrypt_cost_allowed(hash: &str) -> bool {
    hash.get(4..6)
        .and_then(|cost| cost.parse::<u32>().ok())
        .is_some_and(|cost| cost <= MAX_BCRYPT_COST)
}

fn verify_constant_time<E>(
    verify_once: impl Fn(&str) -> Result<bool, E>,
    hash: &str,
    dummy: &str,
) -> bool {
    match verify_once(hash) {
        Ok(matched) => matched,
        Err(_) => {
            let _ = verify_once(dummy);
            false
        }
    }
}

impl BaseHash {
    /// Creates a new `BaseHash`. Stateless — equivalent to `Default::default()`.
    pub fn new() -> Self {
        Self
    }

    /// Hashes `password` with the given `algorithm` (or delegates to the
    /// wrapped hasher for `Manual::Custom`). Returns an error if `password` is
    /// empty, or if the underlying algorithm implementation fails.
    pub fn hash(&self, password: &str, algorithm: &Manual) -> Result<String, String> {
        match algorithm {
            Manual::Argon2 => self.hash_argon2(password),
            Manual::Bcrypt => self.hash_bcrypt(password),
            Manual::Scrypt => self.hash_scrypt(password),
            Manual::Custom(hasher) => hasher.hash(password),
        }
    }
    /// Verifies `password` against `hash`, auto-detecting the algorithm from
    /// the hash's prefix (`$argon2*`, `$2*` for bcrypt, `$scrypt$`). Returns
    /// `false` for an unrecognized prefix, after an Argon2 check against
    /// `dummy_hash()`. Each algorithm's check runs through
    /// `verify_constant_time`, so a malformed stored hash takes the same time
    /// to reject as a well-formed one with the wrong password.
    #[must_use]
    pub fn verify(&self, password: &str, hash: &str) -> bool {
        if hash.starts_with("$argon2") {
            self.verify_argon2(password, hash)
        } else if hash.starts_with("$2") {
            self.verify_bcrypt(password, hash)
        } else if hash.starts_with("$scrypt") {
            self.verify_scrypt(password, hash)
        } else {
            // Corrupted or plain stored value: burn an Argon2 check like a
            // missing account does, or answering at once tells it apart.
            let _ = self.verify_argon2(password, dummy_hash());
            false
        }
    }
    /// Returns the algorithm name (`"argon2"`, `"bcrypt"` or `"scrypt"`)
    /// inferred from the hash's prefix, or `None` if it matches none of them.
    #[must_use]
    pub fn detect_algorithm(&self, hash: &str) -> Option<&'static str> {
        if hash.starts_with("$argon2") {
            Some("argon2")
        } else if hash.starts_with("$2") {
            Some("bcrypt")
        } else if hash.starts_with("$scrypt") {
            Some("scrypt")
        } else {
            None
        }
    }

    fn hash_argon2(&self, password: &str) -> Result<String, String> {
        if password.is_empty() {
            return Err(t(runique_i18n::keys::forms::error::PASSWORD_EMPTY).into_owned());
        }
        // argon2 0.6: `hash_password` generates the salt internally (via `getrandom`),
        // it no longer takes an explicit `SaltString`/RNG.
        Argon2::default()
            .hash_password(password.as_bytes())
            .map(|h| h.to_string())
            .map_err(|e| tf(runique_i18n::keys::forms::error::HASH, &[&e.to_string()]).clone())
    }

    fn verify_argon2(&self, password: &str, hash: &str) -> bool {
        verify_constant_time(
            |h| {
                let p = PasswordHash::new(h).map_err(|_| ())?;
                if !argon2_cost_allowed(&p) {
                    return Err(());
                }
                Ok(Argon2::default()
                    .verify_password(password.as_bytes(), &p)
                    .is_ok())
            },
            hash,
            dummy_hash(),
        )
    }

    fn hash_bcrypt(&self, password: &str) -> Result<String, String> {
        if password.is_empty() {
            return Err(t(runique_i18n::keys::forms::error::PASSWORD_EMPTY).into_owned());
        }
        bcrypt_hash(password, DEFAULT_COST)
            .map_err(|e| tf(runique_i18n::keys::forms::error::HASH, &[&e.to_string()]).clone())
    }

    fn verify_bcrypt(&self, password: &str, hash: &str) -> bool {
        // `bcrypt::verify` only returns `Err` when the hash fails to parse (`split_hash`) —
        // never for a wrong password, which yields `Ok(false)`. Same handling as
        // the other algorithms: burn the same time on a malformed hash.
        verify_constant_time(
            |h| {
                if !bcrypt_cost_allowed(h) {
                    return Err(());
                }
                bcrypt_verify(password, h).map_err(|_| ())
            },
            hash,
            &DUMMY_HASH_BCRYPT,
        )
    }

    fn hash_scrypt(&self, password: &str) -> Result<String, String> {
        if password.is_empty() {
            return Err(t(runique_i18n::keys::forms::error::PASSWORD_EMPTY).into_owned());
        }
        // Like argon2 0.6, `hash_password` generates the salt internally (`getrandom`).
        // `Scrypt` 0.12 is no longer a unit struct (private `params` field) — go through `default()`.
        Scrypt::default()
            .hash_password(password.as_bytes())
            .map(|h| h.to_string())
            .map_err(|e| tf(runique_i18n::keys::forms::error::HASH, &[&e.to_string()]).clone())
    }

    fn verify_scrypt(&self, password: &str, hash: &str) -> bool {
        verify_constant_time(
            |h| {
                let p = PasswordHash::new(h).map_err(|_| ())?;
                if !scrypt_cost_allowed(&p) {
                    return Err(());
                }
                Ok(Scrypt::default()
                    .verify_password(password.as_bytes(), &p)
                    .is_ok())
            },
            hash,
            &DUMMY_HASH_SCRYPT,
        )
    }
}

impl Default for BaseHash {
    fn default() -> Self {
        Self::new()
    }
}

/// The hashing algorithm of a password.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Manual {
    Argon2,
    Bcrypt,
    Scrypt,
    /// A user-supplied algorithm, plugged in via a [`PasswordHasher`] implementation.
    #[serde(skip)]
    Custom(Box<dyn PasswordHasher>),
}

// Pre-computed dummy hash — used to run a full verify even when the user is not found,
// preventing user enumeration via timing differences (constant-time authentication flow).
static DUMMY_HASH: std::sync::LazyLock<String> = std::sync::LazyLock::new(|| {
    BaseHash::new()
        .hash("__runique_dummy__", &Manual::Argon2)
        .unwrap_or_else(|_| {
            "$argon2id$v=19$m=19456,t=2,p=1$c29tZXNhbHQ$RdescudvJCsgt3ub+b+dWRWJTmaaJObG"
                .to_string()
        })
});

/// Returns a pre-computed dummy hash for constant-time authentication.
/// Always use this when the user is not found — never short-circuit before `verify()`.
#[must_use]
pub fn dummy_hash() -> &'static str {
    &DUMMY_HASH
}

// Dummy hashes for bcrypt/scrypt — used only by `verify_constant_time` to burn the
// same computation time when a stored hash fails to parse for that algorithm.
// Unlike `dummy_hash()` (public, algorithm-agnostic, used for user-enumeration
// defense), these are private: each `verify_*` needs a dummy in its own format.
static DUMMY_HASH_BCRYPT: std::sync::LazyLock<String> = std::sync::LazyLock::new(|| {
    bcrypt_hash("__runique_dummy__", DEFAULT_COST)
        .expect("bcrypt hashing a fixed short string should never fail")
});

static DUMMY_HASH_SCRYPT: std::sync::LazyLock<String> = std::sync::LazyLock::new(|| {
    Scrypt::default()
        .hash_password(b"__runique_dummy__")
        .map(|h| h.to_string())
        .expect("scrypt hashing a fixed short string should never fail")
});
