use runique::auth::guard::LoginGuard;

#[test]
fn test_not_locked_initially() {
    let guard = LoginGuard::new().max_attempts(3).lockout_secs(60);
    assert!(!guard.is_locked("alice"));
    assert_eq!(guard.attempts("alice"), 0);
}

#[test]
fn test_record_failure_increments() {
    let guard = LoginGuard::new().max_attempts(3).lockout_secs(60);
    guard.record_failure("bob");
    assert_eq!(guard.attempts("bob"), 1);
    guard.record_failure("bob");
    assert_eq!(guard.attempts("bob"), 2);
    assert!(!guard.is_locked("bob")); // pas encore au max
}

#[test]
fn test_locked_after_max_attempts() {
    let guard = LoginGuard::new().max_attempts(3).lockout_secs(60);
    guard.record_failure("carol");
    guard.record_failure("carol");
    guard.record_failure("carol");
    assert!(guard.is_locked("carol"));
}

#[test]
fn test_record_success_resets() {
    let guard = LoginGuard::new().max_attempts(3).lockout_secs(60);
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
    let guard = LoginGuard::new().max_attempts(2).lockout_secs(60);
    guard.record_failure("user-a");
    guard.record_failure("user-a");
    assert!(guard.is_locked("user-a"));
    assert!(!guard.is_locked("user-b")); // autre utilisateur non affecté
}

#[test]
fn test_remaining_lockout_secs_none_when_not_locked() {
    let guard = LoginGuard::new().max_attempts(3).lockout_secs(60);
    guard.record_failure("eve");
    assert!(guard.remaining_lockout_secs("eve").is_none());
}

#[test]
fn test_remaining_lockout_secs_some_when_locked() {
    let guard = LoginGuard::new().max_attempts(1).lockout_secs(300);
    guard.record_failure("frank");
    let remaining = guard.remaining_lockout_secs("frank");
    assert!(remaining.is_some());
    let secs = remaining.unwrap();
    assert!(secs > 0 && secs <= 300);
}

#[test]
fn test_lockout_expires() {
    // Verrouillage très court (0 sec) → expiré immédiatement
    let guard = LoginGuard::new().max_attempts(1).lockout_secs(0);
    guard.record_failure("grace");
    std::thread::sleep(std::time::Duration::from_millis(10));
    assert!(!guard.is_locked("grace")); // verrouillage expiré
    assert!(guard.remaining_lockout_secs("grace").is_none());
}

#[test]
fn test_clone_shares_state() {
    let guard = LoginGuard::new().max_attempts(2).lockout_secs(60);
    let clone = guard.clone();
    guard.record_failure("henry");
    // Le clone voit l'état du guard original (Arc partagé)
    assert_eq!(clone.attempts("henry"), 1);
}

#[test]
fn test_default_values() {
    let guard = LoginGuard::new();
    assert_eq!(guard.max_attempts, 5);
    assert_eq!(guard.lockout_secs, 300);
}

#[test]
fn test_effective_key_with_username() {
    let key = LoginGuard::effective_key("alice", "1.2.3.4");
    assert_eq!(key, "alice");
}

#[test]
fn test_effective_key_empty_username_uses_ip() {
    let key = LoginGuard::effective_key("", "1.2.3.4");
    assert_eq!(key, "anonym:1.2.3.4");
}

#[test]
fn test_effective_key_whitespace_username_uses_ip() {
    let key = LoginGuard::effective_key("   ", "1.2.3.4");
    assert_eq!(key, "anonym:1.2.3.4");
}

#[test]
fn test_effective_key_anonymous_keys_are_independent() {
    let guard = LoginGuard::new().max_attempts(2).lockout_secs(60);
    let key_a = LoginGuard::effective_key("", "1.2.3.4");
    let key_b = LoginGuard::effective_key("", "5.6.7.8");
    guard.record_failure(&key_a);
    guard.record_failure(&key_a);
    assert!(guard.is_locked(&key_a));
    assert!(!guard.is_locked(&key_b)); // IP différente → non affectée
}
