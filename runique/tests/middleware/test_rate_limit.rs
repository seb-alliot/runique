use runique::middleware::rate_limit::RateLimiter;
use serial_test::serial;

#[test]
fn test_new_allows_first_request() {
    let limiter = RateLimiter::new(5, 60);
    assert!(limiter.is_allowed("127.0.0.1"));
}

#[test]
fn test_allows_up_to_max() {
    let limiter = RateLimiter::new(3, 60);
    assert!(limiter.is_allowed("ip1"));
    assert!(limiter.is_allowed("ip1"));
    assert!(limiter.is_allowed("ip1"));
    // 4e requête dépasse la limite
    assert!(!limiter.is_allowed("ip1"));
}

#[test]
fn test_different_keys_independent() {
    let limiter = RateLimiter::new(1, 60);
    assert!(limiter.is_allowed("ip-a"));
    assert!(limiter.is_allowed("ip-b")); // clé différente → indépendante
    assert!(!limiter.is_allowed("ip-a")); // ip-a est épuisée
}

#[test]
fn test_window_reset() {
    // Fenêtre de 0 secondes : chaque appel repart d'une nouvelle fenêtre
    let limiter = RateLimiter::new(1, 0);
    assert!(limiter.is_allowed("ip"));
    // La fenêtre est expirée dès le prochain appel
    std::thread::sleep(std::time::Duration::from_millis(10));
    assert!(limiter.is_allowed("ip")); // nouvelle fenêtre → autorisé
}

#[test]
fn test_clone_shares_state() {
    let limiter = RateLimiter::new(2, 60);
    let clone = limiter.clone();
    assert!(limiter.is_allowed("ip"));
    assert!(clone.is_allowed("ip")); // le clone partage le store (Arc)
    // 3e appel sur l'original doit être refusé
    assert!(!limiter.is_allowed("ip"));
}

#[test]
fn test_max_requests_field() {
    let limiter = RateLimiter::new(42, 30);
    assert_eq!(limiter.max_requests, 42);
    assert_eq!(limiter.window.as_secs(), 30);
}

#[test]
#[serial]
fn test_from_env_defaults() {
    unsafe {
        std::env::remove_var("RUNIQUE_RATE_LIMIT_REQUESTS");
        std::env::remove_var("RUNIQUE_RATE_LIMIT_WINDOW_SECS");
    }
    let limiter = RateLimiter::from_env();
    assert_eq!(limiter.max_requests, 60);
    assert_eq!(limiter.window.as_secs(), 60);
}

#[test]
#[serial]
fn test_from_env_custom() {
    unsafe {
        std::env::set_var("RUNIQUE_RATE_LIMIT_REQUESTS", "10");
        std::env::set_var("RUNIQUE_RATE_LIMIT_WINDOW_SECS", "120");
    }
    let limiter = RateLimiter::from_env();
    assert_eq!(limiter.max_requests, 10);
    assert_eq!(limiter.window.as_secs(), 120);
    unsafe {
        std::env::remove_var("RUNIQUE_RATE_LIMIT_REQUESTS");
        std::env::remove_var("RUNIQUE_RATE_LIMIT_WINDOW_SECS");
    }
}
