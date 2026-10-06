//! Tests — makemigrations::run() (pipeline complet scan → diff → generate → write)
//!
//! Couvre la fonction `run()` qui orchestre :
//!   scan_entities → diff_schemas → generate_create/alter → write files → update lib.rs
//!
//! Aucune connexion DB requise — tests purement fichiers.

use crate::utils::clean_tpm_test::TestTempDir;
use crate::utils::env::{del_env, set_env};
use runique::cli::makemigration::run;
use std::fs;

// ─── Helpers ─────────────────────────────────────────────────────────────────

fn temp_dir(suffix: &str) -> TestTempDir {
    TestTempDir::new("runique_test_run", suffix)
}

fn entity_user() -> &'static str {
    r#"
    use runique::prelude::*;
    model! {
        User,
        table: "users",
        pk: id => i32,
        {
            username: text [required, unique],
            email: text [required, unique],
            is_active: bool [required],
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
        {
            username: text [required, unique],
            email: text [required, unique],
            is_active: bool [required],
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
        {
            title: text [required],
            body: text [nullable],
            user_id: int [required],
            description: text [nullable],
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
        {
            name: text [required],
            price: float [required],
            stock: int [required],
            sku: text [required, unique],
        }
    }
    "#
}

// ═══════════════════════════════════════════════════════════════
// Cas vide — pas d'entités
// ═══════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_run_dossier_entites_vide() {
    set_env("RUNIQUE_TEST", "1");
    let entities = temp_dir("run_empty_ent");
    let migrations = temp_dir("run_empty_mig");

    let result = run(
        entities.to_str().unwrap(),
        migrations.to_str().unwrap(),
        false,
    );
    assert!(result.is_ok(), "run() vide doit Ok: {:?}", result);
    assert!(
        !migrations.join("lib.rs").exists(),
        "lib.rs ne doit pas exister"
    );

    del_env("RUNIQUE_TEST");
    std::fs::remove_dir_all(&entities).ok();
    std::fs::remove_dir_all(&migrations).ok();
}

#[tokio::test]
async fn test_run_dossier_inexistant_retourne_err() {
    set_env("RUNIQUE_TEST", "1");
    let result = run("/chemin/inexistant_abc123/entities", "/tmp/mig_xyz", false);
    assert!(result.is_err(), "dossier inexistant doit Err");
    del_env("RUNIQUE_TEST");
}

// ═══════════════════════════════════════════════════════════════
// CREATE — premier run
// ═══════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_run_cree_snapshot() {
    set_env("RUNIQUE_TEST", "1");
    let entities = temp_dir("run_snap_ent");
    let migrations = temp_dir("run_snap_mig");

    fs::write(entities.join("user.rs"), entity_user()).unwrap();
    run(
        entities.to_str().unwrap(),
        migrations.to_str().unwrap(),
        false,
    )
    .unwrap();

    assert!(
        migrations.join("snapshots/users.rs").exists(),
        "snapshot/users.rs doit exister"
    );
    del_env("RUNIQUE_TEST");
    std::fs::remove_dir_all(&entities).ok();
    std::fs::remove_dir_all(&migrations).ok();
}

#[tokio::test]
async fn test_run_cree_lib_rs() {
    set_env("RUNIQUE_TEST", "1");
    let entities = temp_dir("run_lib_ent");
    let migrations = temp_dir("run_lib_mig");

    fs::write(entities.join("user.rs"), entity_user()).unwrap();
    run(
        entities.to_str().unwrap(),
        migrations.to_str().unwrap(),
        false,
    )
    .unwrap();

    assert!(migrations.join("lib.rs").exists(), "lib.rs doit exister");
    let content = fs::read_to_string(migrations.join("lib.rs")).unwrap();
    assert!(content.contains("use sea_orm_migration::prelude::*;"));
    assert!(content.contains("pub struct Migrator;"));
    assert!(content.contains("impl MigratorTrait for Migrator"));
    assert!(content.contains("create_users_table"));
    del_env("RUNIQUE_TEST");
    std::fs::remove_dir_all(&entities).ok();
    std::fs::remove_dir_all(&migrations).ok();
}

#[tokio::test]
async fn test_run_cree_fichier_seaorm_create() {
    set_env("RUNIQUE_TEST", "1");
    let entities = temp_dir("run_seaorm_ent");
    let migrations = temp_dir("run_seaorm_mig");

    fs::write(entities.join("user.rs"), entity_user()).unwrap();
    run(
        entities.to_str().unwrap(),
        migrations.to_str().unwrap(),
        false,
    )
    .unwrap();

    let entries: Vec<_> = fs::read_dir(&migrations)
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();
    println!("Contenu du dossier migrations :");
    for entry in &entries {
        println!("- {}", entry.file_name().to_string_lossy());
    }
    let create_files: Vec<_> = entries
        .iter()
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
    del_env("RUNIQUE_TEST");
    std::fs::remove_dir_all(&entities).ok();
    std::fs::remove_dir_all(&migrations).ok();
}

// ═══════════════════════════════════════════════════════════════
// Idempotence — deuxième run sans changements
// ═══════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_run_idempotent_meme_entite() {
    set_env("RUNIQUE_TEST", "1");
    let entities = temp_dir("run_idem_ent");
    let migrations = temp_dir("run_idem_mig");

    fs::write(entities.join("user.rs"), entity_user()).unwrap();

    // Premier run
    run(
        entities.to_str().unwrap(),
        migrations.to_str().unwrap(),
        false,
    )
    .unwrap();

    // Deuxième run : pas de changements — doit Ok sans planter
    let result = run(
        entities.to_str().unwrap(),
        migrations.to_str().unwrap(),
        false,
    );
    assert!(
        result.is_ok(),
        "2e run() sans changements doit Ok: {:?}",
        result
    );
    del_env("RUNIQUE_TEST");
    std::fs::remove_dir_all(&entities).ok();
    std::fs::remove_dir_all(&migrations).ok();
}

#[tokio::test]
async fn test_run_lib_rs_pas_duplique_au_second_run() {
    set_env("RUNIQUE_TEST", "1");
    let entities = temp_dir("run_idem2_ent");
    let migrations = temp_dir("run_idem2_mig");

    fs::write(entities.join("user.rs"), entity_user()).unwrap();

    run(
        entities.to_str().unwrap(),
        migrations.to_str().unwrap(),
        false,
    )
    .unwrap();
    run(
        entities.to_str().unwrap(),
        migrations.to_str().unwrap(),
        false,
    )
    .unwrap();

    let lib_content = fs::read_to_string(migrations.join("lib.rs")).unwrap();
    let count = lib_content.matches("create_users_table").count();
    assert_eq!(
        count, 2,
        "le module doit apparaître 2 fois dans lib.rs (mod + Box)"
    );
    del_env("RUNIQUE_TEST");
    std::fs::remove_dir_all(&entities).ok();
    std::fs::remove_dir_all(&migrations).ok();
}

// ═══════════════════════════════════════════════════════════════
// ALTER — ajout de colonne nullable (non destructif)
// ═══════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_run_alter_ajout_colonne_nullable() {
    set_env("RUNIQUE_TEST", "1");
    let entities = temp_dir("run_alter_ent");
    let migrations = temp_dir("run_alter_mig");

    // Étape 1 : CREATE initial
    fs::write(entities.join("user.rs"), entity_user()).unwrap();
    run(
        entities.to_str().unwrap(),
        migrations.to_str().unwrap(),
        false,
    )
    .unwrap();

    // Étape 2 : Ajouter une colonne nullable → pas destructif
    fs::write(entities.join("user.rs"), entity_user_with_bio()).unwrap();
    let result = run(
        entities.to_str().unwrap(),
        migrations.to_str().unwrap(),
        false,
    );
    assert!(
        result.is_ok(),
        "run() avec ALTER non destructif doit Ok: {:?}",
        result
    );

    assert!(
        fs::read_dir(&migrations).unwrap().any(|e| e
            .unwrap()
            .file_name()
            .to_string_lossy()
            .ends_with("_alter_users_table.rs")),
        "une migration *_alter_users_table.rs doit être créée"
    );
    del_env("RUNIQUE_TEST");
    std::fs::remove_dir_all(&entities).ok();
    std::fs::remove_dir_all(&migrations).ok();
}

#[tokio::test]
async fn test_run_alter_cree_fichier_alter() {
    set_env("RUNIQUE_TEST", "1");
    let entities = temp_dir("run_alter_file_ent");
    let migrations = temp_dir("run_alter_file_mig");

    fs::write(entities.join("user.rs"), entity_user()).unwrap();
    run(
        entities.to_str().unwrap(),
        migrations.to_str().unwrap(),
        false,
    )
    .unwrap();

    fs::write(entities.join("user.rs"), entity_user_with_bio()).unwrap();
    run(
        entities.to_str().unwrap(),
        migrations.to_str().unwrap(),
        false,
    )
    .unwrap();

    let alter_files: Vec<_> = fs::read_dir(&migrations)
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
        "fichier *_alter_users_table.rs doit exister dans migrations/"
    );
    assert!(
        !migrations.join("applied").exists(),
        "plus de copie dans applied/"
    );
    del_env("RUNIQUE_TEST");
    std::fs::remove_dir_all(&entities).ok();
    std::fs::remove_dir_all(&migrations).ok();
}

#[tokio::test]
async fn test_run_alter_snapshot_mis_a_jour() {
    set_env("RUNIQUE_TEST", "1");
    let entities = temp_dir("run_snap_update_ent");
    let migrations = temp_dir("run_snap_update_mig");

    fs::write(entities.join("user.rs"), entity_user()).unwrap();
    run(
        entities.to_str().unwrap(),
        migrations.to_str().unwrap(),
        false,
    )
    .unwrap();

    fs::write(entities.join("user.rs"), entity_user_with_bio()).unwrap();
    run(
        entities.to_str().unwrap(),
        migrations.to_str().unwrap(),
        false,
    )
    .unwrap();

    // Snapshot doit contenir "bio"
    let snap = fs::read_to_string(migrations.join("snapshots/users.rs")).unwrap();
    assert!(snap.contains("bio"), "snapshot doit contenir le champ bio");
    del_env("RUNIQUE_TEST");
    std::fs::remove_dir_all(&entities).ok();
    std::fs::remove_dir_all(&migrations).ok();
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
    );
    assert!(result.is_ok(), "run() 2 entités doit Ok: {:?}", result);

    assert!(migrations.join("snapshots/users.rs").exists());
    assert!(migrations.join("snapshots/posts.rs").exists());
    assert!(migrations.join("lib.rs").exists());
    std::fs::remove_dir_all(&entities).ok();
    std::fs::remove_dir_all(&migrations).ok();
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
    );
    assert!(result.is_ok(), "run() 3 entités doit Ok: {:?}", result);

    let lib_content = fs::read_to_string(migrations.join("lib.rs")).unwrap();
    assert!(lib_content.contains("create_users_table"));
    assert!(lib_content.contains("create_posts_table"));
    assert!(lib_content.contains("create_products_table"));
    std::fs::remove_dir_all(&entities).ok();
    std::fs::remove_dir_all(&migrations).ok();
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
    .unwrap();

    // Run 2 : 2e entité ajoutée
    fs::write(entities.join("post.rs"), entity_post()).unwrap();
    run(
        entities.to_str().unwrap(),
        migrations.to_str().unwrap(),
        false,
    )
    .unwrap();

    assert!(migrations.join("snapshots/users.rs").exists());
    assert!(migrations.join("snapshots/posts.rs").exists());

    let lib = fs::read_to_string(migrations.join("lib.rs")).unwrap();
    assert!(lib.contains("users"));
    assert!(lib.contains("posts"));
    std::fs::remove_dir_all(&entities).ok();
    std::fs::remove_dir_all(&migrations).ok();
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
    );
    assert!(
        result.is_ok(),
        "fichier sans model! ne doit pas planter: {:?}",
        result
    );
    assert!(
        !migrations.join("lib.rs").exists(),
        "lib.rs ne doit pas exister si aucun modèle trouvé"
    );
    std::fs::remove_dir_all(&entities).ok();
    std::fs::remove_dir_all(&migrations).ok();
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
    .unwrap();

    // Seulement posts.rs doit avoir généré un snapshot
    assert!(migrations.join("snapshots/posts.rs").exists());
    assert!(
        !migrations.join("snapshots/users.rs").exists(),
        "mod.rs doit être ignoré"
    );
    std::fs::remove_dir_all(&entities).ok();
    std::fs::remove_dir_all(&migrations).ok();
}

// ═══════════════════════════════════════════════════════════════
// ALTER — suppression de colonne (destructif)
// ═══════════════════════════════════════════════════════════════

fn entity_user_without_email() -> &'static str {
    r#"
    use runique::prelude::*;
    model! {
        User,
        table: "users",
        pk: id => i32,
        {
            username: text [required, unique],
            is_active: bool [required],
        }
    }
    "#
}

#[tokio::test]
async fn test_run_drop_colonne_sans_force_retourne_err() {
    set_env("RUNIQUE_TEST", "1");
    let entities = temp_dir("run_drop_noforce_ent");
    let migrations = temp_dir("run_drop_noforce_mig");

    fs::write(entities.join("user.rs"), entity_user()).unwrap();
    run(
        entities.to_str().unwrap(),
        migrations.to_str().unwrap(),
        false,
    )
    .unwrap();

    fs::write(entities.join("user.rs"), entity_user_without_email()).unwrap();
    let result = run(
        entities.to_str().unwrap(),
        migrations.to_str().unwrap(),
        false,
    );
    assert!(
        result.is_err(),
        "DROP COLUMN sans --force doit retourner Err"
    );
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("--force") || err.contains("force"),
        "message d'erreur doit mentionner --force, got: {err}"
    );

    let snap = fs::read_to_string(migrations.join("snapshots/users.rs")).unwrap();
    assert!(
        snap.contains("email"),
        "snapshot ne doit pas être mis à jour après un bail"
    );

    del_env("RUNIQUE_TEST");
    std::fs::remove_dir_all(&entities).ok();
    std::fs::remove_dir_all(&migrations).ok();
}

#[tokio::test]
async fn test_run_drop_colonne_avec_force_genere_migration() {
    set_env("RUNIQUE_TEST", "1");
    let entities = temp_dir("run_drop_force_ent");
    let migrations = temp_dir("run_drop_force_mig");

    fs::write(entities.join("user.rs"), entity_user()).unwrap();
    run(
        entities.to_str().unwrap(),
        migrations.to_str().unwrap(),
        false,
    )
    .unwrap();

    fs::write(entities.join("user.rs"), entity_user_without_email()).unwrap();
    let result = run(
        entities.to_str().unwrap(),
        migrations.to_str().unwrap(),
        true,
    );
    assert!(
        result.is_ok(),
        "DROP COLUMN avec --force doit Ok: {:?}",
        result
    );

    let alter_files: Vec<_> = fs::read_dir(&migrations)
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
        "fichier ALTER doit exister dans migrations/"
    );

    let alter_content = fs::read_to_string(alter_files[0].path()).unwrap();
    assert!(
        alter_content.contains("drop_column"),
        "ALTER doit contenir drop_column"
    );
    assert!(
        alter_content.contains("email"),
        "ALTER doit mentionner la colonne supprimée"
    );

    let snap = fs::read_to_string(migrations.join("snapshots/users.rs")).unwrap();
    assert!(
        !snap.contains("\"email\""),
        "snapshot mis à jour ne doit plus contenir email"
    );

    del_env("RUNIQUE_TEST");
    std::fs::remove_dir_all(&entities).ok();
    std::fs::remove_dir_all(&migrations).ok();
}

// ═══════════════════════════════════════════════════════════════
// Longueurs de colonnes — snapshot antérieur aux longueurs
// ═══════════════════════════════════════════════════════════════

fn post_with_title_length(len: u32) -> String {
    format!(
        r#"model! {{ Post, table: "posts", pk: id => i32, {{ title: text [required, max_length: {len}] }} }}"#
    )
}

fn migration_files(dir: &std::path::Path) -> usize {
    fs::read_dir(dir)
        .unwrap()
        .filter(|e| {
            let name = e.as_ref().unwrap().file_name();
            let name = name.to_string_lossy();
            name.starts_with('m') && name.ends_with(".rs")
        })
        .count()
}

#[tokio::test]
async fn test_run_snapshot_ancien_rafraichi_puis_longueur_suivie() {
    use runique::migration::utils::generators::SNAPSHOT_LENGTHS_MARKER;
    set_env("RUNIQUE_TEST", "1");
    let entities = temp_dir("run_len_ent");
    let migrations = temp_dir("run_len_mig");
    let (ent, mig) = (entities.to_str().unwrap(), migrations.to_str().unwrap());

    fs::write(entities.join("post.rs"), post_with_title_length(80)).unwrap();
    run(ent, mig, false).unwrap();
    let snapshot = migrations.join("snapshots/posts.rs");
    let first = migration_files(&migrations);

    // Snapshot écrit avant les longueurs : sans marqueur ni longueur.
    let old = fs::read_to_string(&snapshot)
        .unwrap()
        .replacen(&format!("{SNAPSHOT_LENGTHS_MARKER}\n"), "", 1)
        .replace(".string_len(80)", ".string()");
    fs::write(&snapshot, old).unwrap();

    run(ent, mig, false).unwrap();
    assert_eq!(migration_files(&migrations), first, "aucune migration");
    let refreshed = fs::read_to_string(&snapshot).unwrap();
    assert!(
        refreshed.starts_with(SNAPSHOT_LENGTHS_MARKER),
        "snapshot rafraîchi"
    );
    assert!(refreshed.contains(".string_len(80)"));

    // La longueur est désormais suivie : l'agrandir produit une migration.
    fs::write(entities.join("post.rs"), post_with_title_length(120)).unwrap();
    run(ent, mig, false).unwrap();
    assert_eq!(migration_files(&migrations), first + 1);

    del_env("RUNIQUE_TEST");
}

// ═══════════════════════════════════════════════════════════════
// FK circulaires et index de FK
// ═══════════════════════════════════════════════════════════════

fn create_file(dir: &std::path::Path, table: &str) -> String {
    let name = fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().into_string().unwrap())
        .find(|n| n.ends_with(&format!("_create_{table}_table.rs")))
        .unwrap_or_else(|| panic!("no CREATE file for {table}"));
    fs::read_to_string(dir.join(name)).unwrap()
}

#[tokio::test]
async fn test_run_cycle_de_fk_place_la_cle_selon_le_moteur() {
    set_env("RUNIQUE_TEST", "1");
    let entities = temp_dir("run_cycle_ent");
    let migrations = temp_dir("run_cycle_mig");
    fs::write(
        entities.join("author.rs"),
        r#"model! { Author, table: "authors", pk: id => i32, { best_book_id: int [nullable] },
            relations: { belongs_to: book via best_book_id [set_null] } }"#,
    )
    .unwrap();
    fs::write(
        entities.join("book.rs"),
        r#"model! { Book, table: "books", pk: id => i32, { author_id: int [required] },
            relations: { belongs_to: author via author_id [cascade] } }"#,
    )
    .unwrap();
    run(
        entities.to_str().unwrap(),
        migrations.to_str().unwrap(),
        false,
    )
    .unwrap();

    // `authors` comes first (smallest name): its key to `books` is inline on
    // SQLite only.
    let authors = create_file(&migrations, "authors");
    let sqlite_branch = authors
        .split("DbBackend::Sqlite {")
        .nth(1)
        .expect("SQLite branch");
    assert!(
        sqlite_branch.contains("authors_best_book_id_books_fkey"),
        "{authors}"
    );
    assert!(authors.contains("manager.create_table(table.to_owned())"));

    // `books` closes the cycle elsewhere, and undoes it before its DROP.
    let books = create_file(&migrations, "books");
    let (up, down) = books.split_once("async fn down").unwrap();
    assert!(up.contains("!= sea_orm::DbBackend::Sqlite"), "{books}");
    assert!(up.contains(".create_foreign_key("));
    assert!(up.contains("authors_best_book_id_books_fkey"));
    assert!(down.find(".drop_foreign_key(").unwrap() < down.find(".drop_table(").unwrap());
    assert!(
        books.contains("books_author_id_authors_fkey"),
        "inline key kept"
    );

    let lib = fs::read_to_string(migrations.join("lib.rs")).unwrap();
    assert!(lib.find("create_authors_table").unwrap() < lib.find("create_books_table").unwrap());
    del_env("RUNIQUE_TEST");
}

#[tokio::test]
async fn test_run_index_sur_chaque_fk() {
    set_env("RUNIQUE_TEST", "1");
    let entities = temp_dir("run_fk_idx_ent");
    let migrations = temp_dir("run_fk_idx_mig");
    fs::write(
        entities.join("shelf.rs"),
        r#"model! { Shelf, table: "shelves", pk: id => i32, { label: text } }"#,
    )
    .unwrap();
    fs::write(
        entities.join("book.rs"),
        r#"model! { Book, table: "books", pk: id => i32, {
                shelf_id: int [required],
                owner_id: int [required, unique],
                lender_id: int [required],
            },
            relations: {
                belongs_to: shelf via shelf_id [cascade],
                belongs_to: shelf via owner_id,
                belongs_to: shelf via lender_id,
            },
            meta: { indexes: [(lender_id, shelf_id)] } }"#,
    )
    .unwrap();
    run(
        entities.to_str().unwrap(),
        migrations.to_str().unwrap(),
        false,
    )
    .unwrap();

    let books = create_file(&migrations, "books");
    assert!(books.contains("\"idx_books_shelf_id\""), "{books}");
    assert!(
        !books.contains("idx_books_owner_id"),
        "unique column already indexed"
    );
    assert!(
        !books.contains("\"idx_books_lender_id\""),
        "covered by the declared index"
    );
    del_env("RUNIQUE_TEST");
}
