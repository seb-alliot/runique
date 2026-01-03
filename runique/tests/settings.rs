use runique::Settings;

#[test]
fn test_settings_default_values() {
    let settings = Settings::default_values();

    // Vérifier que les valeurs par défaut sont définies
    assert!(!settings.base_dir.is_empty());
    assert!(!settings.templates_dir.is_empty());
    assert!(!settings.staticfiles_dirs.is_empty());
}

#[test]
fn test_settings_allowed_hosts_default() {
    let settings = Settings::default_values();

    // Les allowed_hosts devraient avoir des valeurs par défaut
    assert!(!settings.allowed_hosts.is_empty());
}

#[test]
fn test_settings_debug_mode() {
    let settings = Settings::default_values();

    // En mode debug, debug devrait être true
    // (ou false selon la configuration)
    assert!(settings.debug); // Toujours vrai, juste pour vérifier l'accès
}

#[test]
fn test_settings_builder() {
    let settings = Settings::builder()
        .debug(true)
        .templates_dir(vec!["custom/templates".to_string()])
        .build();

    assert!(settings.debug);
    assert_eq!(settings.templates_dir, vec!["custom/templates".to_string()]);
}

#[test]
fn test_settings_validate_allowed_hosts_debug() {
    let mut settings = Settings::default_values();
    settings.debug = true;
    settings.allowed_hosts = vec![];

    // En mode debug, ne devrait pas paniquer
    settings.validate_allowed_hosts();
}

#[test]
fn test_settings_static_urls() {
    let settings = Settings::default_values();

    // Vérifier que les URLs statiques sont définies
    assert!(!settings.static_url.is_empty());
    assert!(!settings.media_url.is_empty());
    assert!(!settings.static_runique_url.is_empty());
    assert!(!settings.media_runique_url.is_empty());
}

#[test]
fn test_settings_paths() {
    let settings = Settings::default_values();

    // Vérifier que les chemins sont définis
    assert!(!settings.staticfiles_dirs.is_empty());
    assert!(!settings.media_root.is_empty());
    assert!(!settings.static_runique_path.is_empty());
    assert!(!settings.media_runique_path.is_empty());
}

#[test]
fn test_settings_server_config() {
    let settings = Settings::default_values();

    // Vérifier que la configuration serveur est définie
    assert!(!settings.server.ip_server.is_empty());
    assert!(settings.server.port > 0);
    assert!(!settings.server.secret_key.is_empty());
}

#[test]
fn test_settings_clone() {
    let settings1 = Settings::default_values();
    let settings2 = settings1.clone();

    // Vérifier que le clone fonctionne
    assert_eq!(settings1.debug, settings2.debug);
    assert_eq!(settings1.allowed_hosts, settings2.allowed_hosts);
}
