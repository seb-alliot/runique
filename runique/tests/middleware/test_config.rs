// Tests pour MiddlewareConfig
#[test]

use runique::middleware::config::MiddlewareConfig;

#[test]
fn test_middleware_config_default() {
    let config = MiddlewareConfig::default();
    assert!(config.enable_csp);
    assert!(config.enable_host_validation);
    assert!(config.enable_debug_errors);
    assert!(config.enable_cache);
}

#[test]
fn test_middleware_config_production() {
    let config = MiddlewareConfig::production();
    assert!(config.enable_csp);
    assert!(config.enable_host_validation);
    assert!(config.enable_debug_errors);
    assert!(config.enable_cache);
}

#[test]
fn test_middleware_config_development() {
    let config = MiddlewareConfig::development();
    assert!(!config.enable_csp);
    assert!(!config.enable_host_validation);
    assert!(config.enable_debug_errors);
    assert!(!config.enable_cache);
}

#[test]
fn test_middleware_config_api() {
    let config = MiddlewareConfig::api();
    assert!(!config.enable_csp);
    assert!(config.enable_host_validation);
    assert!(config.enable_debug_errors);
    assert!(config.enable_cache);
}

#[test]
fn test_middleware_config_custom_chain() {
    let config = MiddlewareConfig::custom()
        .with_csp(false)
        .with_debug_errors(false)
        .with_cache(false)
        .with_host_validation(false);
    assert!(!config.enable_csp);
    assert!(!config.enable_debug_errors);
    assert!(!config.enable_cache);
    assert!(!config.enable_host_validation);
}
