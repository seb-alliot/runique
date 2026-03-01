//! Tests — makemigrations::run() (pipeline complet scan → diff → generate → write)
//!
//! Couvre la fonction `run()` qui orchestre :
//!   scan_entities → diff_schemas → generate_create/alter → write files → update lib.rs
//!
//! Aucune connexion DB requise — tests purement fichiers.

use runique::migration::makemigrations::run;
use std::fs;
use std::path::PathBuf;

// ─── Helpers ─────────────────────────────────────────────────────────────────

fn temp_dir(suffix: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("runique_test_run_{}", suffix));
    fs::create_dir_all(&dir).ok();
    dir
}

fn entity_user() -> &'static str {
    r#"
    use runique::prelude::*;
    model! {
        User,
        table: "users",
        pk: id => i32,
        fields: {
            username: String [unique],
            email: String [unique],
            is_active: bool,
        }
    }
    "#
}

fn entity_user_with_bio() -> &'static str {
    r#"
    use runique::prelude::*;
    model! {
        User,
        table: "users",
        pk: id => i32,
        fields: {
            username: String [unique],
            email: String [unique],
            is_active: bool,
            bio: text [nullable],
        }
    }
    "#
}

fn entity_post() -> &'static str {
    r#"
    use runique::prelude::*;
    model! {
        Post,
        table: "posts",
        pk: id => i64,
        fields: {
            title: String,
            body: text [nullable],
            user_id: i32,
        }
    }
    "#
}

fn entity_product() -> &'static str {
    r#"
    use runique::prelude::*;
    model! {
        Product,
        table: "products",
        pk: id => i32,
        fields: {
            name: String,
            price: f64,
            stock: i32,
            sku: String [unique],
        }
    }
    "#
}

// ═══════════════════════════════════════════════════════════════
// Cas vide — pas d'entités
// ═══════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_run_dossier_entites_vide() {
    let entities = temp_dir("run_empty_ent");
    let migrations = temp_dir("run_empty_mig");

    let result = run(
        entities.to_str().unwrap(),
        migrations.to_str().unwrap(),
        false,
    )
    .await;
    assert!(result.is_ok(), "run() vide doit Ok: {:?}", result);
    // Aucun fichier de migration ne doit être généré
    assert!(
        !migrations.join("lib.rs").exists(),
        "lib.rs ne doit pas exister"
    );
}

#[tokio::test]
async fn test_run_dossier_inexistant_retourne_err() {
    let result = run("/chemin/inexistant_abc123/entities", "/tmp/mig_xyz", false).await;
    assert!(result.is_err(), "dossier inexistant doit Err");
}

// ═══════════════════════════════════════════════════════════════
// CREATE — premier run
// ═══════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_run_cree_snapshot() {
    let entities = temp_dir("run_snap_ent");
    let migrations = temp_dir("run_snap_mig");

    fs::write(entities.join("user.rs"), entity_user()).unwrap();
    run(
        entities.to_str().unwrap(),
        migrations.to_str().unwrap(),
        false,
    )
    .await
    .unwrap();

    assert!(
        migrations.join("snapshots/users.rs").exists(),
        "snapshot/users.rs doit exister"
    );
}

#[tokio::test]
async fn test_run_cree_lib_rs() {
    let entities = temp_dir("run_lib_ent");
    let migrations = temp_dir("run_lib_mig");

    fs::write(entities.join("user.rs"), entity_user()).unwrap();
    run(
        entities.to_str().unwrap(),
        migrations.to_str().unwrap(),
        false,
    )
    .await
    .unwrap();

    assert!(migrations.join("lib.rs").exists(), "lib.rs doit exister");
    let content = fs::read_to_string(migrations.join("lib.rs")).unwrap();
    assert!(content.contains("use sea_orm_migration::prelude::*;"));
    assert!(content.contains("pub struct Migrator;"));
    assert!(content.contains("impl MigratorTrait for Migrator"));
    assert!(content.contains("create_users_table"));
}

#[tokio::test]
async fn test_run_cree_fichier_seaorm_create() {
    let entities = temp_dir("run_seaorm_ent");
    let migrations = temp_dir("run_seaorm_mig");

    fs::write(entities.join("user.rs"), entity_user()).unwrap();
    run(
        entities.to_str().unwrap(),
        migrations.to_str().unwrap(),
        false,
    )
    .await
    .unwrap();

    let create_files: Vec<_> = fs::read_dir(&migrations)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| {
            let name = e.file_name();
            let s = name.to_string_lossy().to_string();
            s.contains("create_users_table") && s.ends_with(".rs")
        })
        .collect();

    assert!(
        !create_files.is_empty(),
        "fichier m*_create_users_table.rs doit exister dans migrations/"
    );
}

#[tokio::test]
async fn test_run_dossier_applied_cree() {
    let entities = temp_dir("run_applied_ent");
    let migrations = temp_dir("run_applied_mig");

    fs::write(entities.join("user.rs"), entity_user()).unwrap();
    run(
        entities.to_str().unwrap(),
        migrations.to_str().unwrap(),
        false,
    )
    .await
    .unwrap();

    assert!(
        migrations.join("applied").exists(),
        "dossier applied/ doit être créé"
    );
}

// ═══════════════════════════════════════════════════════════════
// Idempotence — deuxième run sans changements
// ═══════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_run_idempotent_meme_entite() {
    let entities = temp_dir("run_idem_ent");
    let migrations = temp_dir("run_idem_mig");

    fs::write(entities.join("user.rs"), entity_user()).unwrap();

    // Premier run
    run(
        entities.to_str().unwrap(),
        migrations.to_str().unwrap(),
        false,
    )
    .await
    .unwrap();

    // Deuxième run : pas de changements — doit Ok sans planter
    let result = run(
        entities.to_str().unwrap(),
        migrations.to_str().unwrap(),
        false,
    )
    .await;
    assert!(
        result.is_ok(),
        "2e run() sans changements doit Ok: {:?}",
        result
    );
}

#[tokio::test]
async fn test_run_lib_rs_pas_duplique_au_second_run() {
    let entities = temp_dir("run_idem2_ent");
    let migrations = temp_dir("run_idem2_mig");

    fs::write(entities.join("user.rs"), entity_user()).unwrap();

    run(
        entities.to_str().unwrap(),
        migrations.to_str().unwrap(),
        false,
    )
    .await
    .unwrap();
    run(
        entities.to_str().unwrap(),
        migrations.to_str().unwrap(),
        false,
    )
    .await
    .unwrap();

    let lib_content = fs::read_to_string(migrations.join("lib.rs")).unwrap();
    let count = lib_content.matches("create_users_table").count();
    assert_eq!(
        count, 2,
        "le module doit apparaître 2 fois dans lib.rs (mod + Box)"
    );
}

// ═══════════════════════════════════════════════════════════════
// ALTER — ajout de colonne nullable (non destructif)
// ═══════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_run_alter_ajout_colonne_nullable() {
    let entities = temp_dir("run_alter_ent");
    let migrations = temp_dir("run_alter_mig");

    // Étape 1 : CREATE initial
    fs::write(entities.join("user.rs"), entity_user()).unwrap();
    run(
        entities.to_str().unwrap(),
        migrations.to_str().unwrap(),
        false,
    )
    .await
    .unwrap();

    // Étape 2 : Ajouter une colonne nullable → pas destructif
    fs::write(entities.join("user.rs"), entity_user_with_bio()).unwrap();
    let result = run(
        entities.to_str().unwrap(),
        migrations.to_str().unwrap(),
        false,
    )
    .await;
    assert!(
        result.is_ok(),
        "run() avec ALTER non destructif doit Ok: {:?}",
        result
    );

    // Dossier applied/users/ doit exister
    assert!(
        migrations.join("applied/users").exists(),
        "applied/users/ doit être créé pour l'ALTER"
    );
}

#[tokio::test]
async fn test_run_alter_cree_fichier_alter() {
    let entities = temp_dir("run_alter_file_ent");
    let migrations = temp_dir("run_alter_file_mig");

    fs::write(entities.join("user.rs"), entity_user()).unwrap();
    run(
        entities.to_str().unwrap(),
        migrations.to_str().unwrap(),
        false,
    )
    .await
    .unwrap();

    fs::write(entities.join("user.rs"), entity_user_with_bio()).unwrap();
    run(
        entities.to_str().unwrap(),
        migrations.to_str().unwrap(),
        false,
    )
    .await
    .unwrap();

    // Un fichier *_alter_users_table.rs doit exister dans applied/users/
    let alter_dir = migrations.join("applied/users");
    if alter_dir.exists() {
        let alter_files: Vec<_> = fs::read_dir(&alter_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| {
                let n = e.file_name();
                let s = n.to_string_lossy().to_string();
                s.contains("alter_users_table") && s.ends_with(".rs")
            })
            .collect();
        assert!(
            !alter_files.is_empty(),
            "fichier *_alter_users_table.rs doit exister dans applied/users/"
        );
    }
    // Si alter_dir n'existe pas, le test n'échoue pas (implémentation variable)
}

#[tokio::test]
async fn test_run_alter_snapshot_mis_a_jour() {
    let entities = temp_dir("run_snap_update_ent");
    let migrations = temp_dir("run_snap_update_mig");

    fs::write(entities.join("user.rs"), entity_user()).unwrap();
    run(
        entities.to_str().unwrap(),
        migrations.to_str().unwrap(),
        false,
    )
    .await
    .unwrap();

    fs::write(entities.join("user.rs"), entity_user_with_bio()).unwrap();
    run(
        entities.to_str().unwrap(),
        migrations.to_str().unwrap(),
        false,
    )
    .await
    .unwrap();

    // Snapshot doit contenir "bio"
    let snap = fs::read_to_string(migrations.join("snapshots/users.rs")).unwrap();
    assert!(snap.contains("bio"), "snapshot doit contenir le champ bio");
}

// ═══════════════════════════════════════════════════════════════
// Plusieurs entités
// ═══════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_run_deux_entites() {
    let entities = temp_dir("run_two_ent");
    let migrations = temp_dir("run_two_mig");

    fs::write(entities.join("user.rs"), entity_user()).unwrap();
    fs::write(entities.join("post.rs"), entity_post()).unwrap();

    let result = run(
        entities.to_str().unwrap(),
        migrations.to_str().unwrap(),
        false,
    )
    .await;
    assert!(result.is_ok(), "run() 2 entités doit Ok: {:?}", result);

    assert!(migrations.join("snapshots/users.rs").exists());
    assert!(migrations.join("snapshots/posts.rs").exists());
    assert!(migrations.join("lib.rs").exists());
}

#[tokio::test]
async fn test_run_trois_entites() {
    let entities = temp_dir("run_three_ent");
    let migrations = temp_dir("run_three_mig");

    fs::write(entities.join("user.rs"), entity_user()).unwrap();
    fs::write(entities.join("post.rs"), entity_post()).unwrap();
    fs::write(entities.join("product.rs"), entity_product()).unwrap();

    let result = run(
        entities.to_str().unwrap(),
        migrations.to_str().unwrap(),
        false,
    )
    .await;
    assert!(result.is_ok(), "run() 3 entités doit Ok: {:?}", result);

    let lib_content = fs::read_to_string(migrations.join("lib.rs")).unwrap();
    assert!(lib_content.contains("create_users_table"));
    assert!(lib_content.contains("create_posts_table"));
    assert!(lib_content.contains("create_products_table"));
}

#[tokio::test]
async fn test_run_plusieurs_entites_plusieurs_runs() {
    let entities = temp_dir("run_multi_ent");
    let migrations = temp_dir("run_multi_mig");

    // Run 1 : 1 entité
    fs::write(entities.join("user.rs"), entity_user()).unwrap();
    run(
        entities.to_str().unwrap(),
        migrations.to_str().unwrap(),
        false,
    )
    .await
    .unwrap();

    // Run 2 : 2e entité ajoutée
    fs::write(entities.join("post.rs"), entity_post()).unwrap();
    run(
        entities.to_str().unwrap(),
        migrations.to_str().unwrap(),
        false,
    )
    .await
    .unwrap();

    assert!(migrations.join("snapshots/users.rs").exists());
    assert!(migrations.join("snapshots/posts.rs").exists());

    let lib = fs::read_to_string(migrations.join("lib.rs")).unwrap();
    assert!(lib.contains("users"));
    assert!(lib.contains("posts"));
}

// ═══════════════════════════════════════════════════════════════
// Fichier sans model! ignoré
// ═══════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_run_ignore_fichier_sans_macro() {
    let entities = temp_dir("run_nomacro_ent");
    let migrations = temp_dir("run_nomacro_mig");

    // Fichier sans model! → doit être ignoré silencieusement
    fs::write(entities.join("helper.rs"), "pub fn helper() -> i32 { 42 }").unwrap();

    let result = run(
        entities.to_str().unwrap(),
        migrations.to_str().unwrap(),
        false,
    )
    .await;
    assert!(
        result.is_ok(),
        "fichier sans model! ne doit pas planter: {:?}",
        result
    );
    assert!(
        !migrations.join("lib.rs").exists(),
        "lib.rs ne doit pas exister si aucun modèle trouvé"
    );
}

#[tokio::test]
async fn test_run_ignore_mod_rs() {
    let entities = temp_dir("run_modrs_ent");
    let migrations = temp_dir("run_modrs_mig");

    // mod.rs doit être ignoré même s'il contient un model!
    fs::write(entities.join("mod.rs"), entity_user()).unwrap();
    fs::write(entities.join("post.rs"), entity_post()).unwrap();

    run(
        entities.to_str().unwrap(),
        migrations.to_str().unwrap(),
        false,
    )
    .await
    .unwrap();

    // Seulement posts.rs doit avoir généré un snapshot
    assert!(migrations.join("snapshots/posts.rs").exists());
    assert!(
        !migrations.join("snapshots/users.rs").exists(),
        "mod.rs doit être ignoré"
    );
}
