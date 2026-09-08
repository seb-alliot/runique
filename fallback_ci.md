failures:

---- admin::test_admin_escaping_contract::test_admin_password_never_leaks_in_list_or_detail stdout ----

thread '<unnamed>' (18552) panicked at runique/tests/helpers/db.rs:30:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

thread 'admin::test_admin_escaping_contract::test_admin_password_never_leaks_in_list_or_detail' (18548) panicked at runique/tests/helpers/admin_server.rs:390:19:
recv addr: RecvError

---- admin::test_admin_escaping_contract::test_admin_special_chars_are_escaped_in_list stdout ----

thread '<unnamed>' (18560) panicked at runique/tests/helpers/db.rs:30:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

thread 'admin::test_admin_escaping_contract::test_admin_special_chars_are_escaped_in_list' (18549) panicked at runique/tests/helpers/admin_server.rs:390:19:
recv addr: RecvError

---- admin::test_admin_groupe_droits_crud::test_groupe_and_droit_full_crud_roundtrip stdout ----

thread '<unnamed>' (18566) panicked at runique/tests/helpers/db.rs:30:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

thread 'admin::test_admin_groupe_droits_crud::test_groupe_and_droit_full_crud_roundtrip' (18550) panicked at runique/tests/helpers/admin_server.rs:390:19:
recv addr: RecvError

---- admin::test_admin_password_security::test_create_user_token_binds_to_new_user_not_creator stdout ----

thread 'admin::test_admin_password_security::test_create_user_token_binds_to_new_user_not_creator' (18555) panicked at runique/tests/helpers/db.rs:30:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- admin::test_admin_password_security::test_reset_password_binds_token_to_target_not_actor stdout ----

thread 'admin::test_admin_password_security::test_reset_password_binds_token_to_target_not_actor' (18559) panicked at runique/tests/helpers/db.rs:30:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- admin::test_admin_password_security::test_reset_password_unknown_id_creates_no_token stdout ----

thread 'admin::test_admin_password_security::test_reset_password_unknown_id_creates_no_token' (18565) panicked at runique/tests/helpers/db.rs:30:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- admin::test_admin_route_crawl::test_admin_crawl_dashboard_and_lists stdout ----

thread '<unnamed>' (18586) panicked at runique/tests/helpers/db.rs:30:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

thread 'admin::test_admin_route_crawl::test_admin_crawl_dashboard_and_lists' (18582) panicked at runique/tests/helpers/admin_server.rs:390:19:
recv addr: RecvError

---- admin::test_admin_route_crawl::test_admin_crawl_detail_create_edit_delete_bulk stdout ----

thread '<unnamed>' (18592) panicked at runique/tests/helpers/db.rs:30:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

thread 'admin::test_admin_route_crawl::test_admin_crawl_detail_create_edit_delete_bulk' (18583) panicked at runique/tests/helpers/admin_server.rs:390:19:
recv addr: RecvError

---- admin::test_admin_route_crawl::test_admin_crawl_history_views stdout ----

thread '<unnamed>' (18598) panicked at runique/tests/helpers/db.rs:30:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

thread 'admin::test_admin_route_crawl::test_admin_crawl_history_views' (18584) panicked at runique/tests/helpers/admin_server.rs:390:19:
recv addr: RecvError

---- admin::test_admin_route_crawl::test_admin_login_wrong_password_rerenders_form stdout ----

thread '<unnamed>' (18604) panicked at runique/tests/helpers/db.rs:30:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

thread 'admin::test_admin_route_crawl::test_admin_login_wrong_password_rerenders_form' (18585) panicked at runique/tests/helpers/admin_server.rs:390:19:
recv addr: RecvError

---- admin::test_admin_user_crud::test_user_bulk_delete_roundtrip stdout ----

thread '<unnamed>' (18612) panicked at runique/tests/helpers/db.rs:30:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

thread 'admin::test_admin_user_crud::test_user_bulk_delete_roundtrip' (18591) panicked at runique/tests/helpers/admin_server.rs:390:19:
recv addr: RecvError

---- admin::test_admin_user_crud::test_user_create_edit_delete_roundtrip stdout ----

thread '<unnamed>' (18625) panicked at runique/tests/helpers/db.rs:30:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

thread 'admin::test_admin_user_crud::test_user_create_edit_delete_roundtrip' (18597) panicked at runique/tests/helpers/admin_server.rs:390:19:
recv addr: RecvError

---- app::test_admin_prefix::test_chemin_admin_seul_absent_quand_prefixe stdout ----

thread 'app::test_admin_prefix::test_chemin_admin_seul_absent_quand_prefixe' (18651) panicked at runique/tests/app/test_admin_prefix.rs:66:57:
called `Result::unwrap()` on an `Err` value: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- app::test_admin_prefix::test_ordre_des_appels_sans_effet stdout ----

thread 'app::test_admin_prefix::test_ordre_des_appels_sans_effet' (18652) panicked at runique/tests/app/test_admin_prefix.rs:66:57:
called `Result::unwrap()` on an `Err` value: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- app::test_admin_prefix::test_prefix_se_compose_avec_le_chemin_admin stdout ----

thread 'app::test_admin_prefix::test_prefix_se_compose_avec_le_chemin_admin' (18653) panicked at runique/tests/app/test_admin_prefix.rs:66:57:
called `Result::unwrap()` on an `Err` value: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- app::test_admin_prefix::test_sans_prefixe_urls_inchangees stdout ----

thread 'app::test_admin_prefix::test_sans_prefixe_urls_inchangees' (18654) panicked at runique/tests/app/test_admin_prefix.rs:66:57:
called `Result::unwrap()` on an `Err` value: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- app::test_admin_prefix::test_slashs_optionnels_dans_le_prefixe stdout ----

thread 'app::test_admin_prefix::test_slashs_optionnels_dans_le_prefixe' (18655) panicked at runique/tests/app/test_admin_prefix.rs:66:57:
called `Result::unwrap()` on an `Err` value: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- app::test_engine::test_attach_middlewares_default_config stdout ----

thread 'app::test_engine::test_attach_middlewares_default_config' (18658) panicked at runique/tests/helpers/server.rs:91:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- app::test_admin_prefix::test_url_nommee_admin_login_suit_le_prefixe stdout ----

thread 'app::test_admin_prefix::test_url_nommee_admin_login_suit_le_prefixe' (18656) panicked at runique/tests/app/test_admin_prefix.rs:46:57:
called `Result::unwrap()` on an `Err` value: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- app::test_admin_prefix::test_url_nommee_dashboard_suit_le_prefixe stdout ----

thread 'app::test_admin_prefix::test_url_nommee_dashboard_suit_le_prefixe' (18657) panicked at runique/tests/app/test_admin_prefix.rs:46:57:
called `Result::unwrap()` on an `Err` value: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- app::test_engine::test_attach_middlewares_with_host_validation_enabled stdout ----

thread 'app::test_engine::test_attach_middlewares_with_host_validation_enabled' (18659) panicked at runique/tests/app/test_engine.rs:29:57:
called `Result::unwrap()` on an `Err` value: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- app::test_engine::test_attach_middlewares_with_https_redirect stdout ----

thread 'app::test_engine::test_attach_middlewares_with_https_redirect' (18660) panicked at runique/tests/app/test_engine.rs:64:57:
called `Result::unwrap()` on an `Err` value: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- app::test_robots_txt::test_robots_txt_absent_avec_no_robots_txt stdout ----

thread 'app::test_robots_txt::test_robots_txt_absent_avec_no_robots_txt' (18661) panicked at runique/tests/app/test_robots_txt.rs:53:57:
called `Result::unwrap()` on an `Err` value: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- app::test_robots_txt::test_robots_txt_ne_divulgue_pas_le_prefix stdout ----

thread 'app::test_robots_txt::test_robots_txt_ne_divulgue_pas_le_prefix' (18662) panicked at runique/tests/app/test_robots_txt.rs:36:57:
called `Result::unwrap()` on an `Err` value: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- app::test_robots_txt::test_x_robots_tag_sur_login_admin stdout ----

thread 'app::test_robots_txt::test_x_robots_tag_sur_login_admin' (18664) panicked at runique/tests/app/test_robots_txt.rs:36:57:
called `Result::unwrap()` on an `Err` value: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- app::test_runique_app::test_builder_build_avec_sqlite_memory stdout ----

thread 'app::test_runique_app::test_builder_build_avec_sqlite_memory' (18665) panicked at runique/tests/app/test_runique_app.rs:60:57:
called `Result::unwrap()` on an `Err` value: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- app::test_runique_app::test_builder_build_retourne_runique_app stdout ----

thread 'app::test_runique_app::test_builder_build_retourne_runique_app' (18666) panicked at runique/tests/app/test_runique_app.rs:78:57:
called `Result::unwrap()` on an `Err` value: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- app::test_robots_txt::test_robots_txt_present_quand_admin_active stdout ----

thread 'app::test_robots_txt::test_robots_txt_present_quand_admin_active' (18663) panicked at runique/tests/app/test_robots_txt.rs:36:57:
called `Result::unwrap()` on an `Err` value: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- auth::test_default_admin_auth::test_authenticate_inactive_user stdout ----

thread 'auth::test_default_admin_auth::test_authenticate_inactive_user' (18683) panicked at runique/tests/helpers/db.rs:30:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- auth::test_default_admin_auth::test_authenticate_no_admin_access stdout ----

thread 'auth::test_default_admin_auth::test_authenticate_no_admin_access' (18684) panicked at runique/tests/helpers/db.rs:30:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- auth::test_default_admin_auth::test_authenticate_success_staff stdout ----

thread 'auth::test_default_admin_auth::test_authenticate_success_staff' (18685) panicked at runique/tests/helpers/db.rs:30:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- auth::test_default_admin_auth::test_authenticate_success_superuser stdout ----

thread 'auth::test_default_admin_auth::test_authenticate_success_superuser' (18686) panicked at runique/tests/helpers/db.rs:30:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- auth::test_default_admin_auth::test_authenticate_user_not_found stdout ----

thread 'auth::test_default_admin_auth::test_authenticate_user_not_found' (18687) panicked at runique/tests/helpers/db.rs:30:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- auth::test_default_admin_auth::test_authenticate_wrong_password stdout ----

thread 'auth::test_default_admin_auth::test_authenticate_wrong_password' (18688) panicked at runique/tests/helpers/db.rs:30:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- auth::test_login_form::test_save_avec_impl_par_defaut_retourne_ok stdout ----

thread 'auth::test_login_form::test_save_avec_impl_par_defaut_retourne_ok' (18701) panicked at runique/tests/helpers/db.rs:30:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- auth::test_middlewares::test_load_user_anonyme_pas_dextension stdout ----

thread '<unnamed>' (18705) panicked at runique/tests/auth/test_middlewares.rs:28:22:
sqlite:memory: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

thread 'auth::test_middlewares::test_load_user_anonyme_pas_dextension' (18702) panicked at runique/tests/auth/test_middlewares.rs:75:19:
called `Result::unwrap()` on an `Err` value: RecvError

---- auth::test_middlewares::test_load_user_connecte_injecte_current_user stdout ----

thread '<unnamed>' (18717) panicked at runique/tests/auth/test_middlewares.rs:28:22:
sqlite:memory: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

thread 'auth::test_middlewares::test_load_user_connecte_injecte_current_user' (18703) panicked at runique/tests/auth/test_middlewares.rs:75:19:
called `Result::unwrap()` on an `Err` value: RecvError

---- auth::test_session_auth::test_get_username_after_login stdout ----

thread 'auth::test_session_auth::test_get_username_after_login' (18744) panicked at runique/tests/auth/test_session_auth.rs:214:70:
called `Result::unwrap()` on an `Err` value: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- auth::test_session_auth::test_is_admin_authenticated_plain_user stdout ----

thread 'auth::test_session_auth::test_is_admin_authenticated_plain_user' (18746) panicked at runique/tests/auth/test_session_auth.rs:243:70:
called `Result::unwrap()` on an `Err` value: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- auth::test_session_auth::test_is_admin_authenticated_staff stdout ----

thread 'auth::test_session_auth::test_is_admin_authenticated_staff' (18747) panicked at runique/tests/auth/test_session_auth.rs:260:70:
called `Result::unwrap()` on an `Err` value: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- auth::test_session_auth::test_is_admin_authenticated_superuser stdout ----

thread 'auth::test_session_auth::test_is_admin_authenticated_superuser' (18748) panicked at runique/tests/auth/test_session_auth.rs:277:70:
called `Result::unwrap()` on an `Err` value: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- auth::test_session_auth::test_is_authenticated_after_login stdout ----

thread 'auth::test_session_auth::test_is_authenticated_after_login' (18749) panicked at runique/tests/auth/test_session_auth.rs:55:70:
called `Result::unwrap()` on an `Err` value: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- auth::test_session_auth::test_is_not_authenticated_after_logout stdout ----

thread 'auth::test_session_auth::test_is_not_authenticated_after_logout' (18751) panicked at runique/tests/auth/test_session_auth.rs:179:70:
called `Result::unwrap()` on an `Err` value: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- auth::test_session_auth::test_login_sets_all_fields stdout ----

thread 'auth::test_session_auth::test_login_sets_all_fields' (18752) panicked at runique/tests/auth/test_session_auth.rs:93:70:
called `Result::unwrap()` on an `Err` value: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- auth::test_session_auth::test_login_sets_id_and_username stdout ----

thread 'auth::test_session_auth::test_login_sets_id_and_username' (18753) panicked at runique/tests/auth/test_session_auth.rs:75:70:
called `Result::unwrap()` on an `Err` value: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- auth::test_session_auth::test_logout_clears_session_keys stdout ----

thread 'auth::test_session_auth::test_logout_clears_session_keys' (18754) panicked at runique/tests/auth/test_session_auth.rs:138:70:
called `Result::unwrap()` on an `Err` value: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- auth::test_session_security::test_deux_sessions_independantes stdout ----

thread 'auth::test_session_security::test_deux_sessions_independantes' (18758) panicked at runique/tests/auth/test_session_security.rs:166:70:
called `Result::unwrap()` on an `Err` value: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- auth::test_session_security::test_login_collision_nettoie_cache_ancien_user stdout ----

thread 'auth::test_session_security::test_login_collision_nettoie_cache_ancien_user' (18759) panicked at runique/tests/auth/test_session_security.rs:216:70:
called `Result::unwrap()` on an `Err` value: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- auth::test_session_security::test_login_meme_user_ne_reinitialise_pas_session stdout ----

thread 'auth::test_session_security::test_login_meme_user_ne_reinitialise_pas_session' (18760) panicked at runique/tests/auth/test_session_security.rs:86:70:
called `Result::unwrap()` on an `Err` value: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- auth::test_session_security::test_login_user_different_nettoie_session_precedente stdout ----

thread 'auth::test_session_security::test_login_user_different_nettoie_session_precedente' (18761) panicked at runique/tests/auth/test_session_security.rs:55:70:
called `Result::unwrap()` on an `Err` value: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- auth::test_session_security::test_logout_vide_session_completement stdout ----

thread 'auth::test_session_security::test_logout_vide_session_completement' (18763) panicked at runique/tests/auth/test_session_security.rs:113:70:
called `Result::unwrap()` on an `Err` value: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- auth::test_session_security::test_logout_evicte_cache_permissions stdout ----

thread 'auth::test_session_security::test_logout_evicte_cache_permissions' (18762) panicked at runique/tests/auth/test_session_security.rs:142:70:
called `Result::unwrap()` on an `Err` value: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- auth::test_user_model::test_find_by_email_not_found stdout ----

thread 'auth::test_user_model::test_find_by_email_not_found' (18765) panicked at runique/tests/helpers/db.rs:30:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- auth::test_user_model::test_find_by_email_found stdout ----

thread 'auth::test_user_model::test_find_by_email_found' (18764) panicked at runique/tests/helpers/db.rs:30:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- auth::test_user_model::test_find_by_id_found stdout ----

thread 'auth::test_user_model::test_find_by_id_found' (18766) panicked at runique/tests/helpers/db.rs:30:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- auth::test_user_model::test_find_by_id_not_found stdout ----

thread 'auth::test_user_model::test_find_by_id_not_found' (18767) panicked at runique/tests/helpers/db.rs:30:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- auth::test_user_model::test_find_by_username_found stdout ----

thread 'auth::test_user_model::test_find_by_username_found' (18768) panicked at runique/tests/helpers/db.rs:30:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- auth::test_user_model::test_find_by_username_not_found stdout ----

thread 'auth::test_user_model::test_find_by_username_not_found' (18769) panicked at runique/tests/helpers/db.rs:30:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- auth::test_user_model::test_update_password_not_found stdout ----

thread 'auth::test_user_model::test_update_password_not_found' (18784) panicked at runique/tests/helpers/db.rs:30:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- auth::test_user_model_multi_db::test_find_by_id_found_sqlite stdout ----

thread 'auth::test_user_model_multi_db::test_find_by_id_found_sqlite' (18788) panicked at runique/tests/helpers/db.rs:30:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- auth::test_user_model::test_update_password_success stdout ----

thread 'auth::test_user_model::test_update_password_success' (18785) panicked at runique/tests/helpers/db.rs:30:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- auth::test_user_model_multi_db::test_find_by_id_not_found_sqlite stdout ----

thread 'auth::test_user_model_multi_db::test_find_by_id_not_found_sqlite' (18791) panicked at runique/tests/helpers/db.rs:30:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- config::test_builder::test_build_avec_statics_couvre_attach_static_files stdout ----

thread 'config::test_builder::test_build_avec_statics_couvre_attach_static_files' (18824) panicked at runique/tests/config/test_builder.rs:832:57:
called `Result::unwrap()` on an `Err` value: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- config::test_builder::test_build_avec_database_retourne_ok_ou_template_err stdout ----

thread 'config::test_builder::test_build_avec_database_retourne_ok_ou_template_err' (18823) panicked at runique/tests/config/test_builder.rs:785:57:
called `Result::unwrap()` on an `Err` value: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- config::test_builder::test_build_profil_production_couvre_csp_host_validation stdout ----

thread 'config::test_builder::test_build_profil_production_couvre_csp_host_validation' (18839) panicked at runique/tests/config/test_builder.rs:866:57:
called `Result::unwrap()` on an `Err` value: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- config::test_builder::test_core_staging_with_database_is_ready stdout ----

thread 'config::test_builder::test_core_staging_with_database_is_ready' (18866) panicked at runique/tests/config/test_builder.rs:323:57:
called `Result::unwrap()` on an `Err` value: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- context::test_runique_context::test_runique_context_get_200 stdout ----

thread 'context::test_runique_context::test_runique_context_get_200' (18995) panicked at runique/tests/helpers/server.rs:91:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- context::test_runique_context::test_runique_context_is_get_true stdout ----

thread 'context::test_runique_context::test_runique_context_is_get_true' (18996) panicked at runique/tests/helpers/server.rs:91:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- context::test_runique_context::test_runique_context_sans_engine_retourne_500 stdout ----

thread 'context::test_runique_context::test_runique_context_sans_engine_retourne_500' (18997) panicked at runique/tests/helpers/server.rs:91:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- context::test_runique_context::test_runique_context_secret_key_correcte stdout ----

thread 'context::test_runique_context::test_runique_context_secret_key_correcte' (18998) panicked at runique/tests/helpers/server.rs:91:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- context::test_template_request::test_insert_does_not_panic stdout ----

thread 'context::test_template_request::test_insert_does_not_panic' (19014) panicked at runique/tests/helpers/server.rs:91:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- context::test_template_request::test_render_success_returns_200 stdout ----

thread 'context::test_template_request::test_render_success_returns_200' (19015) panicked at runique/tests/helpers/server.rs:91:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- context::test_template_request::test_render_success_returns_html_content stdout ----

thread 'context::test_template_request::test_render_success_returns_html_content' (19016) panicked at runique/tests/helpers/server.rs:91:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- context::test_template_request::test_render_template_not_found_returns_500 stdout ----

thread 'context::test_template_request::test_render_template_not_found_returns_500' (19017) panicked at runique/tests/helpers/server.rs:91:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- context::test_template_request::test_request_extraction_get_200 stdout ----

thread 'context::test_template_request::test_request_extraction_get_200' (19019) panicked at runique/tests/helpers/server.rs:91:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- context::test_template_request::test_request_extraction_sans_engine_retourne_500 stdout ----

thread 'context::test_template_request::test_request_extraction_sans_engine_retourne_500' (19020) panicked at runique/tests/helpers/server.rs:91:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- context::test_template_request::test_request_is_delete_true stdout ----

thread 'context::test_template_request::test_request_is_delete_true' (19021) panicked at runique/tests/helpers/server.rs:91:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- context::test_template_request::test_render_with_template_not_found_returns_500 stdout ----

thread 'context::test_template_request::test_render_with_template_not_found_returns_500' (19018) panicked at runique/tests/helpers/server.rs:91:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- context::test_template_request::test_request_is_post_true stdout ----

thread 'context::test_template_request::test_request_is_post_true' (19023) panicked at runique/tests/helpers/server.rs:91:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- context::test_template_request::test_request_is_get_true stdout ----

thread 'context::test_template_request::test_request_is_get_true' (19022) panicked at runique/tests/helpers/server.rs:91:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- context::test_template_request::test_request_is_put_true stdout ----

thread 'context::test_template_request::test_request_is_put_true' (19024) panicked at runique/tests/helpers/server.rs:91:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- db::test_sqlite::test_connexion_sqlite_ouvre stdout ----

thread 'db::test_sqlite::test_connexion_sqlite_ouvre' (19086) panicked at runique/tests/helpers/db.rs:30:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- db::test_sqlite::test_creation_de_table stdout ----

thread 'db::test_sqlite::test_creation_de_table' (19087) panicked at runique/tests/helpers/db.rs:30:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- db::test_sqlite::test_insertion_multiple stdout ----

thread 'db::test_sqlite::test_insertion_multiple' (19088) panicked at runique/tests/helpers/db.rs:30:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- db::test_sqlite::test_insertion_simple stdout ----

thread 'db::test_sqlite::test_insertion_simple' (19089) panicked at runique/tests/helpers/db.rs:30:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- db::test_sqlite::test_isolation_entre_tests stdout ----

thread 'db::test_sqlite::test_isolation_entre_tests' (19090) panicked at runique/tests/helpers/db.rs:30:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- db::test_sqlite::test_mise_a_jour stdout ----

thread 'db::test_sqlite::test_mise_a_jour' (19091) panicked at runique/tests/helpers/db.rs:30:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- db::test_sqlite::test_suppression stdout ----

thread 'db::test_sqlite::test_suppression' (19092) panicked at runique/tests/helpers/db.rs:30:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- formulaire::test_hooks::test_after_save_echoue_rollback_db stdout ----

thread 'formulaire::test_hooks::test_after_save_echoue_rollback_db' (19489) panicked at runique/tests/helpers/db.rs:30:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- formulaire::test_hooks::test_before_save_echoue_rollback_db stdout ----

thread 'formulaire::test_hooks::test_before_save_echoue_rollback_db' (19490) panicked at runique/tests/helpers/db.rs:30:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- formulaire::test_hooks::test_before_save_echoue_save_non_appele stdout ----

thread 'formulaire::test_hooks::test_before_save_echoue_save_non_appele' (19491) panicked at runique/tests/helpers/db.rs:30:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- formulaire::test_hooks::test_commit_si_tous_hooks_ok stdout ----

thread 'formulaire::test_hooks::test_commit_si_tous_hooks_ok' (19492) panicked at runique/tests/helpers/db.rs:30:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- formulaire::test_hooks::test_on_save_echoue_after_non_appele stdout ----

thread 'formulaire::test_hooks::test_on_save_echoue_after_non_appele' (19493) panicked at runique/tests/helpers/db.rs:30:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- formulaire::test_hooks::test_on_save_echoue_rollback_db stdout ----

thread 'formulaire::test_hooks::test_on_save_echoue_rollback_db' (19494) panicked at runique/tests/helpers/db.rs:30:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- formulaire::test_hooks::test_save_as_contexte_transmis_create stdout ----

thread 'formulaire::test_hooks::test_save_as_contexte_transmis_create' (19495) panicked at runique/tests/helpers/db.rs:30:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- formulaire::test_hooks::test_save_as_contexte_transmis_delete stdout ----

thread 'formulaire::test_hooks::test_save_as_contexte_transmis_delete' (19496) panicked at runique/tests/helpers/db.rs:30:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- formulaire::test_hooks::test_save_as_contexte_transmis_update stdout ----

thread 'formulaire::test_hooks::test_save_as_contexte_transmis_update' (19497) panicked at runique/tests/helpers/db.rs:30:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- formulaire::test_hooks::test_save_as_ordre_appel stdout ----

thread 'formulaire::test_hooks::test_save_as_ordre_appel' (19498) panicked at runique/tests/helpers/db.rs:30:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- formulaire::test_prisme_extractor::test_prisme_get_happy_path_200 stdout ----

thread 'formulaire::test_prisme_extractor::test_prisme_get_happy_path_200' (19574) panicked at runique/tests/helpers/server.rs:91:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- formulaire::test_prisme_extractor::test_prisme_get_missing_config_500 stdout ----

thread 'formulaire::test_prisme_extractor::test_prisme_get_missing_config_500' (19575) panicked at runique/tests/helpers/server.rs:91:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- formulaire::test_prisme_extractor::test_prisme_get_missing_csrf_token_500 stdout ----

thread 'formulaire::test_prisme_extractor::test_prisme_get_missing_csrf_token_500' (19576) panicked at runique/tests/helpers/server.rs:91:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- formulaire::test_prisme_extractor::test_prisme_get_with_query_params stdout ----

thread 'formulaire::test_prisme_extractor::test_prisme_get_with_query_params' (19577) panicked at runique/tests/helpers/server.rs:91:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- macros::test_register_url::test_reverse_plusieurs_urls stdout ----

thread 'macros::test_register_url::test_reverse_plusieurs_urls' (19727) panicked at runique/tests/macros/test_register_url.rs:15:10:
sqlite en mémoire: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- macros::test_register_url::test_reverse_url_enregistre stdout ----

thread 'macros::test_register_url::test_reverse_url_enregistre' (19728) panicked at runique/tests/macros/test_register_url.rs:15:10:
sqlite en mémoire: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- macros::test_register_url::test_reverse_url_inexistante_retourne_none stdout ----

thread 'macros::test_register_url::test_reverse_url_inexistante_retourne_none' (19729) panicked at runique/tests/macros/test_register_url.rs:15:10:
sqlite en mémoire: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- macros::test_register_url::test_reverse_with_parameters_multiple stdout ----

thread 'macros::test_register_url::test_reverse_with_parameters_multiple' (19730) panicked at runique/tests/macros/test_register_url.rs:15:10:
sqlite en mémoire: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- macros::test_register_url::test_reverse_with_parameters_sans_substitution stdout ----

thread 'macros::test_register_url::test_reverse_with_parameters_sans_substitution' (19731) panicked at runique/tests/macros/test_register_url.rs:15:10:
sqlite en mémoire: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- macros::test_register_url::test_reverse_with_parameters_substitue stdout ----

thread 'macros::test_register_url::test_reverse_with_parameters_substitue' (19732) panicked at runique/tests/macros/test_register_url.rs:15:10:
sqlite en mémoire: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- macros::test_register_url::test_reverse_with_parameters_url_inexistante stdout ----

thread 'macros::test_register_url::test_reverse_with_parameters_url_inexistante' (19733) panicked at runique/tests/macros/test_register_url.rs:15:10:
sqlite en mémoire: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- middleware::test_csp::test_csp_middleware_ajoute_header stdout ----

thread 'middleware::test_csp::test_csp_middleware_ajoute_header' (19800) panicked at runique/tests/helpers/server.rs:91:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- middleware::test_csp::test_csp_middleware_header_contient_default_src stdout ----

thread 'middleware::test_csp::test_csp_middleware_header_contient_default_src' (19801) panicked at runique/tests/helpers/server.rs:91:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- middleware::test_csp::test_https_redirect_disabled_par_defaut stdout ----

thread 'middleware::test_csp::test_https_redirect_disabled_par_defaut' (19802) panicked at runique/tests/helpers/server.rs:91:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- middleware::test_csp::test_https_redirect_passe_si_deja_https stdout ----

thread 'middleware::test_csp::test_https_redirect_passe_si_deja_https' (19803) panicked at runique/tests/helpers/server.rs:91:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- middleware::test_csp::test_https_redirect_redirige_quand_actif stdout ----

thread 'middleware::test_csp::test_https_redirect_redirige_quand_actif' (19804) panicked at runique/tests/helpers/server.rs:91:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- middleware::test_csp::test_security_headers_middleware_ajoute_csp_et_autres stdout ----

thread 'middleware::test_csp::test_security_headers_middleware_ajoute_csp_et_autres' (19806) panicked at runique/tests/helpers/server.rs:91:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- middleware::test_csp::test_security_headers_middleware_hsts_present stdout ----

thread 'middleware::test_csp::test_security_headers_middleware_hsts_present' (19807) panicked at runique/tests/helpers/server.rs:91:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- middleware::test_csp::test_security_headers_middleware_nonce_injecte_dans_csp stdout ----

thread 'middleware::test_csp::test_security_headers_middleware_nonce_injecte_dans_csp' (19808) panicked at runique/tests/helpers/server.rs:91:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- middleware::test_csp::test_security_headers_middleware_x_frame_options_deny stdout ----

thread 'middleware::test_csp::test_security_headers_middleware_x_frame_options_deny' (19809) panicked at runique/tests/helpers/server.rs:91:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- middleware::test_csrf_exempt::chemin_non_exempte_reste_bloque stdout ----

thread 'middleware::test_csrf_exempt::chemin_non_exempte_reste_bloque' (19843) panicked at runique/tests/helpers/server.rs:91:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- middleware::test_csrf_exempt::exemption_ne_couvre_pas_les_sous_chemins stdout ----

thread 'middleware::test_csrf_exempt::exemption_ne_couvre_pas_les_sous_chemins' (19844) panicked at runique/tests/helpers/server.rs:91:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- middleware::test_csrf_exempt::get_sur_chemin_exempte_retourne_200 stdout ----

thread 'middleware::test_csrf_exempt::get_sur_chemin_exempte_retourne_200' (19845) panicked at runique/tests/helpers/server.rs:91:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- middleware::test_csrf_exempt::json_post_sans_exempt_bloque stdout ----

thread 'middleware::test_csrf_exempt::json_post_sans_exempt_bloque' (19846) panicked at runique/tests/helpers/server.rs:91:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- middleware::test_csrf_exempt::json_post_sur_chemin_exempte_passe stdout ----

thread 'middleware::test_csrf_exempt::json_post_sur_chemin_exempte_passe' (19847) panicked at runique/tests/helpers/server.rs:91:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- middleware::test_csrf_exempt::liste_vide_bloque_tout stdout ----

thread 'middleware::test_csrf_exempt::liste_vide_bloque_tout' (19848) panicked at runique/tests/helpers/server.rs:91:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- middleware::test_csrf_exempt::plusieurs_chemins_exempts stdout ----

thread 'middleware::test_csrf_exempt::plusieurs_chemins_exempts' (19849) panicked at runique/tests/helpers/server.rs:91:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- middleware::test_csrf_integration::test_csrf_ajax_delete_avec_token_valide stdout ----

thread '<unnamed>' (19851) panicked at runique/tests/helpers/server.rs:91:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

thread 'middleware::test_csrf_integration::test_csrf_ajax_delete_avec_token_valide' (19850) panicked at runique/tests/helpers/server.rs:71:19:
recv addr: RecvError

---- middleware::test_csrf_integration::test_csrf_ajax_post_json_avec_token_valide stdout ----

thread '<unnamed>' (19857) panicked at runique/tests/helpers/server.rs:91:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

thread 'middleware::test_csrf_integration::test_csrf_ajax_post_json_avec_token_valide' (19856) panicked at runique/tests/helpers/server.rs:71:19:
recv addr: RecvError

---- middleware::test_csrf_integration::test_csrf_ajax_post_json_sans_token_retourne_403 stdout ----

thread 'middleware::test_csrf_integration::test_csrf_ajax_post_json_sans_token_retourne_403' (19862) panicked at runique/tests/helpers/server.rs:91:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- middleware::test_csrf_integration::test_csrf_ajax_post_json_token_invalide_retourne_403 stdout ----

thread 'middleware::test_csrf_integration::test_csrf_ajax_post_json_token_invalide_retourne_403' (19863) panicked at runique/tests/helpers/server.rs:91:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- middleware::test_csrf_integration::test_csrf_ajax_x_requested_with_seul_retourne_403 stdout ----

thread 'middleware::test_csrf_integration::test_csrf_ajax_x_requested_with_seul_retourne_403' (19864) panicked at runique/tests/helpers/server.rs:91:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- middleware::test_csrf_integration::test_csrf_delete_avec_token_invalide_retourne_403 stdout ----

thread 'middleware::test_csrf_integration::test_csrf_delete_avec_token_invalide_retourne_403' (19865) panicked at runique/tests/helpers/server.rs:91:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- middleware::test_csrf_integration::test_csrf_delete_sans_header_sans_content_type_retourne_403 stdout ----

thread 'middleware::test_csrf_integration::test_csrf_delete_sans_header_sans_content_type_retourne_403' (19866) panicked at runique/tests/helpers/server.rs:91:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- middleware::test_csrf_integration::test_csrf_get_retourne_200_et_header_token stdout ----

thread 'middleware::test_csrf_integration::test_csrf_get_retourne_200_et_header_token' (19867) panicked at runique/tests/helpers/server.rs:91:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- middleware::test_csrf_integration::test_csrf_post_avec_token_invalide_retourne_403 stdout ----

thread 'middleware::test_csrf_integration::test_csrf_post_avec_token_invalide_retourne_403' (19868) panicked at runique/tests/helpers/server.rs:91:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- middleware::test_csrf_integration::test_csrf_post_sans_header_sans_content_type_retourne_403 stdout ----

thread 'middleware::test_csrf_integration::test_csrf_post_sans_header_sans_content_type_retourne_403' (19869) panicked at runique/tests/helpers/server.rs:91:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- middleware::test_csrf_integration::test_csrf_roundtrip_get_then_post_valide stdout ----

thread '<unnamed>' (19871) panicked at runique/tests/helpers/server.rs:91:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

thread 'middleware::test_csrf_integration::test_csrf_roundtrip_get_then_post_valide' (19870) panicked at runique/tests/helpers/server.rs:71:19:
recv addr: RecvError

---- middleware::test_csrf_integration::test_csrf_token_vol_autre_session_retourne_403 stdout ----

thread '<unnamed>' (19877) panicked at runique/tests/helpers/server.rs:91:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

thread 'middleware::test_csrf_integration::test_csrf_token_vol_autre_session_retourne_403' (19876) panicked at runique/tests/helpers/server.rs:71:19:
recv addr: RecvError

---- middleware::test_errors::test_200_passe_sans_modification stdout ----

thread 'middleware::test_errors::test_200_passe_sans_modification' (19886) panicked at runique/tests/helpers/server.rs:91:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- middleware::test_errors::test_404_body_contient_404 stdout ----

thread 'middleware::test_errors::test_404_body_contient_404' (19887) panicked at runique/tests/helpers/server.rs:91:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- middleware::test_errors::test_404_retourne_404 stdout ----

thread 'middleware::test_errors::test_404_retourne_404' (19888) panicked at runique/tests/helpers/server.rs:91:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- middleware::test_errors::test_404_retourne_html stdout ----

thread 'middleware::test_errors::test_404_retourne_html' (19889) panicked at runique/tests/helpers/server.rs:91:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- middleware::test_errors::test_500_body_contient_500 stdout ----

thread 'middleware::test_errors::test_500_body_contient_500' (19890) panicked at runique/tests/helpers/server.rs:91:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- middleware::test_errors::test_500_retourne_500 stdout ----

thread 'middleware::test_errors::test_500_retourne_500' (19891) panicked at runique/tests/helpers/server.rs:91:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- middleware::test_errors::test_500_retourne_html stdout ----

thread 'middleware::test_errors::test_500_retourne_html' (19892) panicked at runique/tests/helpers/server.rs:91:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- middleware::test_errors::test_debug_200_passe_sans_modification stdout ----

thread 'middleware::test_errors::test_debug_200_passe_sans_modification' (19893) panicked at runique/tests/helpers/server.rs:91:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- middleware::test_errors::test_debug_404_retourne_html stdout ----

thread 'middleware::test_errors::test_debug_404_retourne_html' (19894) panicked at runique/tests/helpers/server.rs:91:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- middleware::test_errors::test_debug_500_retourne_html stdout ----

thread 'middleware::test_errors::test_debug_500_retourne_html' (19895) panicked at runique/tests/helpers/server.rs:91:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- middleware::test_errors::test_debug_body_contient_info_erreur stdout ----

thread 'middleware::test_errors::test_debug_body_contient_info_erreur' (19896) panicked at runique/tests/helpers/server.rs:91:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- middleware::test_open_redirect::allowed_host_redirect_passes stdout ----

thread 'middleware::test_open_redirect::allowed_host_redirect_passes' (19915) panicked at runique/tests/helpers/server.rs:91:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- middleware::test_open_redirect::allowed_wildcard_subdomain_passes stdout ----

thread 'middleware::test_open_redirect::allowed_wildcard_subdomain_passes' (19916) panicked at runique/tests/helpers/server.rs:91:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- middleware::test_open_redirect::external_absolute_redirect_blocked stdout ----

thread 'middleware::test_open_redirect::external_absolute_redirect_blocked' (19917) panicked at runique/tests/helpers/server.rs:91:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- middleware::test_open_redirect::external_http_redirect_blocked stdout ----

thread 'middleware::test_open_redirect::external_http_redirect_blocked' (19918) panicked at runique/tests/helpers/server.rs:91:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- middleware::test_open_redirect::localhost_127_redirect_passes stdout ----

thread 'middleware::test_open_redirect::localhost_127_redirect_passes' (19919) panicked at runique/tests/helpers/server.rs:91:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- middleware::test_open_redirect::localhost_absolute_redirect_passes stdout ----

thread 'middleware::test_open_redirect::localhost_absolute_redirect_passes' (19920) panicked at runique/tests/helpers/server.rs:91:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- middleware::test_open_redirect::lookalike_host_blocked stdout ----

thread 'middleware::test_open_redirect::lookalike_host_blocked' (19921) panicked at runique/tests/helpers/server.rs:91:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- middleware::test_open_redirect::non_redirect_response_passes_unchanged stdout ----

error: test failed, to rerun pass `--test mod`
thread 'middleware::test_open_redirect::non_redirect_response_passes_unchanged' (19922) panicked at runique/tests/helpers/server.rs:91:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- middleware::test_open_redirect::not_in_allowed_hosts_blocked stdout ----

thread 'middleware::test_open_redirect::not_in_allowed_hosts_blocked' (19923) panicked at runique/tests/helpers/server.rs:91:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- middleware::test_open_redirect::protocol_relative_redirect_blocked stdout ----

thread 'middleware::test_open_redirect::protocol_relative_redirect_blocked' (19924) panicked at runique/tests/helpers/server.rs:91:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- middleware::test_open_redirect::relative_redirect_passes stdout ----

thread 'middleware::test_open_redirect::relative_redirect_passes' (19925) panicked at runique/tests/helpers/server.rs:91:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- middleware::test_open_redirect::relative_redirect_with_query_passes stdout ----

thread 'middleware::test_open_redirect::relative_redirect_with_query_passes' (19926) panicked at runique/tests/helpers/server.rs:91:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- middleware::test_open_redirect::subdomain_spoof_blocked stdout ----

thread 'middleware::test_open_redirect::subdomain_spoof_blocked' (19927) panicked at runique/tests/helpers/server.rs:91:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- middleware::test_session_db::test_session_db_create_and_find stdout ----

thread 'middleware::test_session_db::test_session_db_create_and_find' (19969) panicked at runique/tests/helpers/db.rs:30:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- middleware::test_session_db::test_session_db_delete stdout ----

thread 'middleware::test_session_db::test_session_db_delete' (19970) panicked at runique/tests/helpers/db.rs:30:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- middleware::test_session_db::test_session_db_delete_nonexistent_ok stdout ----

thread 'middleware::test_session_db::test_session_db_delete_nonexistent_ok' (19971) panicked at runique/tests/helpers/db.rs:30:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- middleware::test_session_db::test_session_db_find_absent_returns_none stdout ----

thread 'middleware::test_session_db::test_session_db_find_absent_returns_none' (19972) panicked at runique/tests/helpers/db.rs:30:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- middleware::test_session_db::test_session_db_find_by_user stdout ----

thread 'middleware::test_session_db::test_session_db_find_by_user' (19973) panicked at runique/tests/helpers/db.rs:30:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- middleware::test_session_db::test_session_db_find_by_user_excludes_expired stdout ----

thread 'middleware::test_session_db::test_session_db_find_by_user_excludes_expired' (19974) panicked at runique/tests/helpers/db.rs:30:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- middleware::test_session_db::test_session_db_find_expired_returns_none stdout ----

thread 'middleware::test_session_db::test_session_db_find_expired_returns_none' (19975) panicked at runique/tests/helpers/db.rs:30:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- middleware::test_session_db::test_session_db_invalidate_all stdout ----

thread 'middleware::test_session_db::test_session_db_invalidate_all' (19976) panicked at runique/tests/helpers/db.rs:30:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- middleware::test_session_db::test_session_db_invalidate_other_sessions stdout ----

thread 'middleware::test_session_db::test_session_db_invalidate_other_sessions' (19977) panicked at runique/tests/helpers/db.rs:30:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- middleware::test_session_db::test_session_db_spawn_cleanup_actually_purges_expired stdout ----

thread 'middleware::test_session_db::test_session_db_spawn_cleanup_actually_purges_expired' (19978) panicked at runique/tests/helpers/db.rs:30:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- middleware::test_session_db::test_session_db_upsert_inserts_when_absent stdout ----

thread 'middleware::test_session_db::test_session_db_upsert_inserts_when_absent' (19979) panicked at runique/tests/helpers/db.rs:30:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- middleware::test_session_db::test_session_db_upsert_refreshes_expiry_not_frozen stdout ----

thread 'middleware::test_session_db::test_session_db_upsert_refreshes_expiry_not_frozen' (19980) panicked at runique/tests/helpers/db.rs:30:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- middleware::test_session_db::test_session_db_upsert_updates_data_on_existing stdout ----

thread 'middleware::test_session_db::test_session_db_upsert_updates_data_on_existing' (19981) panicked at runique/tests/helpers/db.rs:30:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- migration::test_model_schema::test_schema_to_migration_creates_real_table_sqlite stdout ----

thread 'migration::test_model_schema::test_schema_to_migration_creates_real_table_sqlite' (20335) panicked at runique/tests/helpers/db.rs:30:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- migration::test_model_schema::test_schema_to_migration_fk_is_enforced_sqlite stdout ----

thread 'migration::test_model_schema::test_schema_to_migration_fk_is_enforced_sqlite' (20338) panicked at runique/tests/helpers/db.rs:30:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- migration::test_model_schema::test_schema_to_migration_ignored_column_is_really_absent stdout ----

thread 'migration::test_model_schema::test_schema_to_migration_ignored_column_is_really_absent' (20339) panicked at runique/tests/helpers/db.rs:30:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- utils::test_reset_token::test_consume_returns_user_id stdout ----

thread 'utils::test_reset_token::test_consume_returns_user_id' (20712) panicked at runique/tests/helpers/db.rs:30:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- utils::test_reset_token::test_consume_single_use stdout ----

thread 'utils::test_reset_token::test_consume_single_use' (20713) panicked at runique/tests/helpers/db.rs:30:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- utils::test_reset_token::test_consume_unknown_token stdout ----

thread 'utils::test_reset_token::test_consume_unknown_token' (20714) panicked at runique/tests/helpers/db.rs:30:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- utils::test_reset_token::test_generate_peek_valid stdout ----

thread 'utils::test_reset_token::test_generate_peek_valid' (20721) panicked at runique/tests/helpers/db.rs:30:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- utils::test_reset_token::test_peek_after_consume_false stdout ----

thread 'utils::test_reset_token::test_peek_after_consume_false' (20722) panicked at runique/tests/helpers/db.rs:30:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))

---- utils::test_reset_token::test_peek_unknown_token stdout ----

thread 'utils::test_reset_token::test_peek_unknown_token' (20723) panicked at runique/tests/helpers/db.rs:30:10:
sqlite::memory: connect: Conn(Internal("The connection string 'sqlite::memory:' has no supporting driver."))


failures:
    admin::test_admin_escaping_contract::test_admin_password_never_leaks_in_list_or_detail
    admin::test_admin_escaping_contract::test_admin_special_chars_are_escaped_in_list
    admin::test_admin_groupe_droits_crud::test_groupe_and_droit_full_crud_roundtrip
    admin::test_admin_password_security::test_create_user_token_binds_to_new_user_not_creator
    admin::test_admin_password_security::test_reset_password_binds_token_to_target_not_actor
    admin::test_admin_password_security::test_reset_password_unknown_id_creates_no_token
    admin::test_admin_route_crawl::test_admin_crawl_dashboard_and_lists
    admin::test_admin_route_crawl::test_admin_crawl_detail_create_edit_delete_bulk
    admin::test_admin_route_crawl::test_admin_crawl_history_views
    admin::test_admin_route_crawl::test_admin_login_wrong_password_rerenders_form
    admin::test_admin_user_crud::test_user_bulk_delete_roundtrip
    admin::test_admin_user_crud::test_user_create_edit_delete_roundtrip
    app::test_admin_prefix::test_chemin_admin_seul_absent_quand_prefixe
    app::test_admin_prefix::test_ordre_des_appels_sans_effet
    app::test_admin_prefix::test_prefix_se_compose_avec_le_chemin_admin
    app::test_admin_prefix::test_sans_prefixe_urls_inchangees
    app::test_admin_prefix::test_slashs_optionnels_dans_le_prefixe
    app::test_admin_prefix::test_url_nommee_admin_login_suit_le_prefixe
    app::test_admin_prefix::test_url_nommee_dashboard_suit_le_prefixe
    app::test_engine::test_attach_middlewares_default_config
    app::test_engine::test_attach_middlewares_with_host_validation_enabled
    app::test_engine::test_attach_middlewares_with_https_redirect
    app::test_robots_txt::test_robots_txt_absent_avec_no_robots_txt
    app::test_robots_txt::test_robots_txt_ne_divulgue_pas_le_prefix
    app::test_robots_txt::test_robots_txt_present_quand_admin_active
    app::test_robots_txt::test_x_robots_tag_sur_login_admin
    app::test_runique_app::test_builder_build_avec_sqlite_memory
    app::test_runique_app::test_builder_build_retourne_runique_app
    auth::test_default_admin_auth::test_authenticate_inactive_user
    auth::test_default_admin_auth::test_authenticate_no_admin_access
    auth::test_default_admin_auth::test_authenticate_success_staff
    auth::test_default_admin_auth::test_authenticate_success_superuser
    auth::test_default_admin_auth::test_authenticate_user_not_found
    auth::test_default_admin_auth::test_authenticate_wrong_password
    auth::test_login_form::test_save_avec_impl_par_defaut_retourne_ok
    auth::test_middlewares::test_load_user_anonyme_pas_dextension
    auth::test_middlewares::test_load_user_connecte_injecte_current_user
    auth::test_session_auth::test_get_username_after_login
    auth::test_session_auth::test_is_admin_authenticated_plain_user
    auth::test_session_auth::test_is_admin_authenticated_staff
    auth::test_session_auth::test_is_admin_authenticated_superuser
    auth::test_session_auth::test_is_authenticated_after_login
    auth::test_session_auth::test_is_not_authenticated_after_logout
    auth::test_session_auth::test_login_sets_all_fields
    auth::test_session_auth::test_login_sets_id_and_username
    auth::test_session_auth::test_logout_clears_session_keys
    auth::test_session_security::test_deux_sessions_independantes
    auth::test_session_security::test_login_collision_nettoie_cache_ancien_user
    auth::test_session_security::test_login_meme_user_ne_reinitialise_pas_session
    auth::test_session_security::test_login_user_different_nettoie_session_precedente
    auth::test_session_security::test_logout_evicte_cache_permissions
    auth::test_session_security::test_logout_vide_session_completement
    auth::test_user_model::test_find_by_email_found
    auth::test_user_model::test_find_by_email_not_found
    auth::test_user_model::test_find_by_id_found
    auth::test_user_model::test_find_by_id_not_found
    auth::test_user_model::test_find_by_username_found
    auth::test_user_model::test_find_by_username_not_found
    auth::test_user_model::test_update_password_not_found
    auth::test_user_model::test_update_password_success
    auth::test_user_model_multi_db::test_find_by_id_found_sqlite
    auth::test_user_model_multi_db::test_find_by_id_not_found_sqlite
    config::test_builder::test_build_avec_database_retourne_ok_ou_template_err
    config::test_builder::test_build_avec_statics_couvre_attach_static_files
    config::test_builder::test_build_profil_production_couvre_csp_host_validation
    config::test_builder::test_core_staging_with_database_is_ready
    context::test_runique_context::test_runique_context_get_200
    context::test_runique_context::test_runique_context_is_get_true
    context::test_runique_context::test_runique_context_sans_engine_retourne_500
    context::test_runique_context::test_runique_context_secret_key_correcte
    context::test_template_request::test_insert_does_not_panic
    context::test_template_request::test_render_success_returns_200
    context::test_template_request::test_render_success_returns_html_content
    context::test_template_request::test_render_template_not_found_returns_500
    context::test_template_request::test_render_with_template_not_found_returns_500
    context::test_template_request::test_request_extraction_get_200
    context::test_template_request::test_request_extraction_sans_engine_retourne_500
    context::test_template_request::test_request_is_delete_true
    context::test_template_request::test_request_is_get_true
    context::test_template_request::test_request_is_post_true
    context::test_template_request::test_request_is_put_true
    db::test_sqlite::test_connexion_sqlite_ouvre
    db::test_sqlite::test_creation_de_table
    db::test_sqlite::test_insertion_multiple
    db::test_sqlite::test_insertion_simple
    db::test_sqlite::test_isolation_entre_tests
    db::test_sqlite::test_mise_a_jour
    db::test_sqlite::test_suppression
    formulaire::test_hooks::test_after_save_echoue_rollback_db
    formulaire::test_hooks::test_before_save_echoue_rollback_db
    formulaire::test_hooks::test_before_save_echoue_save_non_appele
    formulaire::test_hooks::test_commit_si_tous_hooks_ok
    formulaire::test_hooks::test_on_save_echoue_after_non_appele
    formulaire::test_hooks::test_on_save_echoue_rollback_db
    formulaire::test_hooks::test_save_as_contexte_transmis_create
    formulaire::test_hooks::test_save_as_contexte_transmis_delete
    formulaire::test_hooks::test_save_as_contexte_transmis_update
    formulaire::test_hooks::test_save_as_ordre_appel
    formulaire::test_prisme_extractor::test_prisme_get_happy_path_200
    formulaire::test_prisme_extractor::test_prisme_get_missing_config_500
    formulaire::test_prisme_extractor::test_prisme_get_missing_csrf_token_500
    formulaire::test_prisme_extractor::test_prisme_get_with_query_params
    macros::test_register_url::test_reverse_plusieurs_urls
    macros::test_register_url::test_reverse_url_enregistre
    macros::test_register_url::test_reverse_url_inexistante_retourne_none
    macros::test_register_url::test_reverse_with_parameters_multiple
    macros::test_register_url::test_reverse_with_parameters_sans_substitution
    macros::test_register_url::test_reverse_with_parameters_substitue
    macros::test_register_url::test_reverse_with_parameters_url_inexistante
    middleware::test_csp::test_csp_middleware_ajoute_header
    middleware::test_csp::test_csp_middleware_header_contient_default_src
    middleware::test_csp::test_https_redirect_disabled_par_defaut
    middleware::test_csp::test_https_redirect_passe_si_deja_https
    middleware::test_csp::test_https_redirect_redirige_quand_actif
    middleware::test_csp::test_security_headers_middleware_ajoute_csp_et_autres
    middleware::test_csp::test_security_headers_middleware_hsts_present
    middleware::test_csp::test_security_headers_middleware_nonce_injecte_dans_csp
    middleware::test_csp::test_security_headers_middleware_x_frame_options_deny
    middleware::test_csrf_exempt::chemin_non_exempte_reste_bloque
    middleware::test_csrf_exempt::exemption_ne_couvre_pas_les_sous_chemins
    middleware::test_csrf_exempt::get_sur_chemin_exempte_retourne_200
    middleware::test_csrf_exempt::json_post_sans_exempt_bloque
    middleware::test_csrf_exempt::json_post_sur_chemin_exempte_passe
    middleware::test_csrf_exempt::liste_vide_bloque_tout
    middleware::test_csrf_exempt::plusieurs_chemins_exempts
    middleware::test_csrf_integration::test_csrf_ajax_delete_avec_token_valide
    middleware::test_csrf_integration::test_csrf_ajax_post_json_avec_token_valide
    middleware::test_csrf_integration::test_csrf_ajax_post_json_sans_token_retourne_403
    middleware::test_csrf_integration::test_csrf_ajax_post_json_token_invalide_retourne_403
    middleware::test_csrf_integration::test_csrf_ajax_x_requested_with_seul_retourne_403
    middleware::test_csrf_integration::test_csrf_delete_avec_token_invalide_retourne_403
    middleware::test_csrf_integration::test_csrf_delete_sans_header_sans_content_type_retourne_403
    middleware::test_csrf_integration::test_csrf_get_retourne_200_et_header_token
    middleware::test_csrf_integration::test_csrf_post_avec_token_invalide_retourne_403
    middleware::test_csrf_integration::test_csrf_post_sans_header_sans_content_type_retourne_403
    middleware::test_csrf_integration::test_csrf_roundtrip_get_then_post_valide
    middleware::test_csrf_integration::test_csrf_token_vol_autre_session_retourne_403
    middleware::test_errors::test_200_passe_sans_modification
    middleware::test_errors::test_404_body_contient_404
    middleware::test_errors::test_404_retourne_404
    middleware::test_errors::test_404_retourne_html
    middleware::test_errors::test_500_body_contient_500
    middleware::test_errors::test_500_retourne_500
    middleware::test_errors::test_500_retourne_html
    middleware::test_errors::test_debug_200_passe_sans_modification
    middleware::test_errors::test_debug_404_retourne_html
    middleware::test_errors::test_debug_500_retourne_html
    middleware::test_errors::test_debug_body_contient_info_erreur
    middleware::test_open_redirect::allowed_host_redirect_passes
    middleware::test_open_redirect::allowed_wildcard_subdomain_passes
    middleware::test_open_redirect::external_absolute_redirect_blocked
    middleware::test_open_redirect::external_http_redirect_blocked
    middleware::test_open_redirect::localhost_127_redirect_passes
    middleware::test_open_redirect::localhost_absolute_redirect_passes
    middleware::test_open_redirect::lookalike_host_blocked
    middleware::test_open_redirect::non_redirect_response_passes_unchanged
    middleware::test_open_redirect::not_in_allowed_hosts_blocked
    middleware::test_open_redirect::protocol_relative_redirect_blocked
    middleware::test_open_redirect::relative_redirect_passes
    middleware::test_open_redirect::relative_redirect_with_query_passes
    middleware::test_open_redirect::subdomain_spoof_blocked
    middleware::test_session_db::test_session_db_create_and_find
    middleware::test_session_db::test_session_db_delete
    middleware::test_session_db::test_session_db_delete_nonexistent_ok
    middleware::test_session_db::test_session_db_find_absent_returns_none
    middleware::test_session_db::test_session_db_find_by_user
    middleware::test_session_db::test_session_db_find_by_user_excludes_expired
    middleware::test_session_db::test_session_db_find_expired_returns_none
    middleware::test_session_db::test_session_db_invalidate_all
    middleware::test_session_db::test_session_db_invalidate_other_sessions
    middleware::test_session_db::test_session_db_spawn_cleanup_actually_purges_expired
    middleware::test_session_db::test_session_db_upsert_inserts_when_absent
    middleware::test_session_db::test_session_db_upsert_refreshes_expiry_not_frozen
    middleware::test_session_db::test_session_db_upsert_updates_data_on_existing
    migration::test_model_schema::test_schema_to_migration_creates_real_table_sqlite
    migration::test_model_schema::test_schema_to_migration_fk_is_enforced_sqlite
    migration::test_model_schema::test_schema_to_migration_ignored_column_is_really_absent
    utils::test_reset_token::test_consume_returns_user_id
    utils::test_reset_token::test_consume_single_use
    utils::test_reset_token::test_consume_unknown_token
    utils::test_reset_token::test_generate_peek_valid
    utils::test_reset_token::test_peek_after_consume_false
    utils::test_reset_token::test_peek_unknown_token

test result: FAILED. 1923 passed; 183 failed; 2 ignored; 0 measured; 0 filtered out; finished in 16.85s

Error: Process completed with exit code 101.
0s
0s
0s
0s
Footer
© 2026 GitHub, Inc.
Footer navigation

    Terms
    Privacy
    Security
    Status
    Community
    Docs
    Contact

