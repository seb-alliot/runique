use runique::middleware::auth::LoginGuard;
use serial_test::serial;

#[test]
fn test_not_locked_initially() {
    let guard = LoginGuard::new(3, 60);
    assert!(!guard.is_locked("alice"));
    assert_eq!(guard.attempts("alice"), 0);
}

#[test]
fn test_record_failure_increments() {
    let guard = LoginGuard::new(3, 60);
    guard.record_failure("bob");
    assert_eq!(guard.attempts("bob"), 1);
    guard.record_failure("bob");
    assert_eq!(guard.attempts("bob"), 2);
    assert!(!guard.is_locked("bob")); // pas encore au max
}

#[test]
fn test_locked_after_max_attempts() {
    let guard = LoginGuard::new(3, 60);
    guard.record_failure("carol");
    guard.record_failure("carol");
    guard.record_failure("carol");
    assert!(guard.is_locked("carol"));
}

#[test]
fn test_record_success_resets() {
    let guard = LoginGuard::new(3, 60);
    guard.record_failure("dave");
    guard.record_failure("dave");
    guard.record_failure("dave");
    assert!(guard.is_locked("dave"));
    guard.record_success("dave");
    assert!(!guard.is_locked("dave"));
    assert_eq!(guard.attempts("dave"), 0);
}

#[test]
fn test_different_users_independent() {
    let guard = LoginGuard::new(2, 60);
    guard.record_failure("user-a");
    guard.record_failure("user-a");
    assert!(guard.is_locked("user-a"));
    assert!(!guard.is_locked("user-b")); // autre utilisateur non affecté
}

#[test]
fn test_remaining_lockout_secs_none_when_not_locked() {
    let guard = LoginGuard::new(3, 60);
    guard.record_failure("eve");
    assert!(guard.remaining_lockout_secs("eve").is_none());
}

#[test]
fn test_remaining_lockout_secs_some_when_locked() {
    let guard = LoginGuard::new(1, 300);
    guard.record_failure("frank");
    let remaining = guard.remaining_lockout_secs("frank");
    assert!(remaining.is_some());
    let secs = remaining.unwrap();
    assert!(secs > 0 && secs <= 300);
}

#[test]
fn test_lockout_expires() {
    // Verrouillage très court (0 sec) → expiré immédiatement
    let guard = LoginGuard::new(1, 0);
    guard.record_failure("grace");
    std::thread::sleep(std::time::Duration::from_millis(10));
    assert!(!guard.is_locked("grace")); // verrouillage expiré
    assert!(guard.remaining_lockout_secs("grace").is_none());
}

#[test]
fn test_clone_shares_state() {
    let guard = LoginGuard::new(2, 60);
    let clone = guard.clone();
    guard.record_failure("henry");
    // Le clone voit l'état du guard original (Arc partagé)
    assert_eq!(clone.attempts("henry"), 1);
}

#[test]
#[serial]
fn test_from_env_defaults() {
    unsafe {
        std::env::remove_var("RUNIQUE_LOGIN_MAX_ATTEMPTS");
        std::env::remove_var("RUNIQUE_LOGIN_LOCKOUT_SECS");
    }
    let guard = LoginGuard::from_env();
    assert_eq!(guard.max_attempts, 5);
    assert_eq!(guard.lockout_secs, 300);
}

#[test]
#[serial]
fn test_from_env_custom() {
    unsafe {
        std::env::set_var("RUNIQUE_LOGIN_MAX_ATTEMPTS", "10");
        std::env::set_var("RUNIQUE_LOGIN_LOCKOUT_SECS", "600");
    }
    let guard = LoginGuard::from_env();
    assert_eq!(guard.max_attempts, 10);
    assert_eq!(guard.lockout_secs, 600);
    unsafe {
        std::env::remove_var("RUNIQUE_LOGIN_MAX_ATTEMPTS");
        std::env::remove_var("RUNIQUE_LOGIN_LOCKOUT_SECS");
    }
}
