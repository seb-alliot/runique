//! An upload through `save`, outside the admin: the account form of
//! `user_profile` (`avatar: image [upload_to: "avatars/"]`). Files live on
//! disk, out of the test's transaction: each test removes the ones it made.
use super::form_save::named_request;
use crate::entities::user_profile;
use runique::config::static_files::resolve_media_root;
use runique::prelude::runique_users::ActiveModel as UserActiveModel;
use runique::prelude::*;
use runique::runique_test::{TestFailure, runique_test};
use sea_orm::DbErr;
use std::path::{Path, PathBuf};

/// A 1×1 PNG: the field checks the content, not only the extension.
const PNG: &[u8] = &[
    0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52,
    0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F, 0x15, 0xC4,
    0x89, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0x00, 0x01, 0x00, 0x00,
    0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE,
    0x42, 0x60, 0x82,
];

/// Removes, when dropped, the files and staging folders a test made.
#[derive(Default)]
struct OnDisk(Vec<PathBuf>);

impl Drop for OnDisk {
    fn drop(&mut self) {
        // Files first, then the folders they emptied (only removed when empty).
        for path in &self.0 {
            let _ = std::fs::remove_file(path);
        }
        for path in &self.0 {
            let _ = std::fs::remove_dir(path);
        }
    }
}

impl OnDisk {
    /// A file as `parse_multipart` leaves it: `{MEDIA_ROOT}/.staging-<tag>/<name>`.
    fn staged(&mut self, tag: &str, name: &str, bytes: &[u8]) -> String {
        let dir = Path::new(&resolve_media_root()).join(format!(".staging-{tag}"));
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join(name);
        std::fs::write(&file, bytes).unwrap();
        self.0.push(file.clone());
        self.0.push(dir);
        file.to_string_lossy().into_owned()
    }

    /// `{MEDIA_ROOT}/<relative>`, removed at the end.
    fn stored(&mut self, relative: &str) -> PathBuf {
        let file = Path::new(&resolve_media_root()).join(relative);
        self.0.push(file.clone());
        if let Some(dir) = file.parent() {
            self.0.push(dir.to_path_buf());
        }
        file
    }
}

/// A name no other run uses, for the files and the account.
fn unique(prefix: &str) -> String {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("{prefix}_{}_{nanos}", std::process::id())
}

async fn account(db: &ADb, username: &str, avatar: Option<&str>) -> Result<Pk, DbErr> {
    let id = UserActiveModel {
        username: Set(username.to_string()),
        email: Set(format!("{username}@example.com")),
        password: Set("not-a-real-hash".to_string()),
        is_active: Set(false),
        is_dirigeant: Set(false),
        is_admin: Set(false),
        is_superuser: Set(false),
        ..Default::default()
    }
    .insert(db)
    .await?
    .id;
    if let Some(avatar) = avatar {
        user_profile::ActiveModel {
            id: Set(id),
            avatar: Set(Some(avatar.to_string())),
            ..Default::default()
        }
        .update(db)
        .await?;
    }
    Ok(id)
}

async fn avatar_of(db: &ADb, id: Pk) -> Result<Option<String>, DbErr> {
    Ok(user_profile::Entity::find_by_id(id)
        .one(db)
        .await?
        .and_then(|row| row.avatar))
}

/// Validates the account form for `username`, `avatar` sent as given, then
/// saves it on the row `{id}`. `Err` when it didn't validate.
async fn save_profile(
    db: &ADb,
    id: &str,
    username: &str,
    avatar: &str,
) -> Result<Result<(), String>, DbErr> {
    let email = format!("{username}@example.com");
    let req = named_request(
        db,
        "profile",
        &[("id", id)],
        &[
            ("username", username),
            ("email", &email),
            ("avatar", avatar),
        ],
    );
    let form =
        match ValidationForm::try_form(req.form::<user_profile::Form>(), &req, "profile").await {
            Ok(form) => form,
            Err(form) => return Ok(Err(format!("invalid: {:?}", form.get_form().errors()))),
        };
    Ok(match form.save(db).await {
        Ok(_) => Ok(()),
        Err(form) => Err(format!("refused: {:?}", form.get_form().errors())),
    })
}

fn fail(message: String) -> DbErr {
    DbErr::Custom(message)
}

#[tokio::test]
async fn an_upload_is_moved_out_of_staging_and_its_path_saved() -> Result<(), TestFailure> {
    runique_test::<ADb>(super::ENV, async |db| {
        let mut disk = OnDisk::default();
        let username = unique("upload");
        let id = account(db, &username, None).await?;
        let name = format!("{}.png", unique("avatar"));
        let staged = disk.staged(&unique("t"), &name, PNG);
        let stored = disk.stored(&format!("avatars/{name}"));

        save_profile(db, &id.to_string(), &username, &staged)
            .await?
            .map_err(fail)?;

        let avatar = avatar_of(db, id).await?;
        if avatar.as_deref() != Some(format!("avatars/{name}").as_str()) {
            return Err(fail(format!("avatar saved as {avatar:?}")));
        }
        if !stored.is_file() {
            return Err(fail(format!("{} should exist", stored.display())));
        }
        if Path::new(&staged).exists() {
            return Err(fail("the staged file should be gone".into()));
        }
        Ok(())
    })
    .await
}

#[tokio::test]
async fn an_update_without_a_new_file_keeps_the_stored_one() -> Result<(), TestFailure> {
    runique_test::<ADb>(super::ENV, async |db| {
        let username = unique("keep");
        let id = account(db, &username, Some("avatars/kept.png")).await?;
        save_profile(db, &id.to_string(), &username, "")
            .await?
            .map_err(fail)?;
        match avatar_of(db, id).await?.as_deref() {
            Some("avatars/kept.png") => Ok(()),
            other => Err(fail(format!("the avatar became {other:?}"))),
        }
    })
    .await
}

/// A path sent as text is not an upload: ignored, the stored avatar stays.
#[tokio::test]
async fn a_path_sent_as_text_is_never_taken_for_an_upload() -> Result<(), TestFailure> {
    runique_test::<ADb>(super::ENV, async |db| {
        let username = unique("text");
        let id = account(db, &username, Some("avatars/kept.png")).await?;
        for sent in [
            "/etc/passwd",
            "../../Cargo.toml",
            "avatars/someone_else.png",
        ] {
            let _ = save_profile(db, &id.to_string(), &username, sent).await?;
            match avatar_of(db, id).await?.as_deref() {
                Some("avatars/kept.png") => {}
                other => return Err(fail(format!("{sent:?} gave the avatar {other:?}"))),
            }
        }
        Ok(())
    })
    .await
}

#[tokio::test]
async fn a_file_that_is_no_image_is_refused_and_removed() -> Result<(), TestFailure> {
    runique_test::<ADb>(super::ENV, async |db| {
        let mut disk = OnDisk::default();
        let username = unique("fake");
        let id = account(db, &username, Some("avatars/kept.png")).await?;
        let staged = disk.staged(&unique("t"), "fake.png", b"not an image");

        if save_profile(db, &id.to_string(), &username, &staged)
            .await?
            .is_ok()
        {
            return Err(fail("a fake image should not validate".into()));
        }
        if Path::new(&staged).exists() {
            return Err(fail("the refused file should be removed".into()));
        }
        match avatar_of(db, id).await?.as_deref() {
            Some("avatars/kept.png") => Ok(()),
            other => Err(fail(format!("the avatar became {other:?}"))),
        }
    })
    .await
}

/// The profile form for `username` with `avatar`, validated, for the row `{id}`.
async fn validated_profile(
    db: &ADb,
    id: &str,
    username: &str,
    avatar: &str,
) -> Result<ValidationForm<user_profile::Form>, DbErr> {
    let email = format!("{username}@example.com");
    let req = named_request(
        db,
        "profile",
        &[("id", id)],
        &[
            ("username", username),
            ("email", &email),
            ("avatar", avatar),
        ],
    );
    ValidationForm::try_form(req.form::<user_profile::Form>(), &req, "profile")
        .await
        .map_err(|form| fail(format!("invalid: {:?}", form.get_form().errors())))
}

/// `{MEDIA_ROOT}/<relative>`: where a stored path lives on disk.
fn on_disk(relative: &str) -> PathBuf {
    Path::new(&resolve_media_root()).join(relative)
}

/// What the developer writes to replace a file: read the stored path before
/// the save, remove that file once the save went through.
#[tokio::test]
async fn the_developer_removes_the_replaced_file_after_the_save() -> Result<(), TestFailure> {
    runique_test::<ADb>(super::ENV, async |db| {
        let mut disk = OnDisk::default();
        let old_name = format!("{}.png", unique("old"));
        let old_file = disk.stored(&format!("avatars/{old_name}"));
        std::fs::create_dir_all(old_file.parent().unwrap()).unwrap();
        std::fs::write(&old_file, PNG).unwrap();

        let username = unique("replace");
        let id = account(db, &username, Some(&format!("avatars/{old_name}"))).await?;
        let name = format!("{}.png", unique("new"));
        let staged = disk.staged(&unique("t"), &name, PNG);
        let new_file = disk.stored(&format!("avatars/{name}"));

        // The developer's handler.
        let old = avatar_of(db, id).await?;
        let form = validated_profile(db, &id.to_string(), &username, &staged).await?;
        let Ok(form) = form.save(db).await else {
            return Err(fail("the save should go through".into()));
        };
        let new = form.get_form().fields["avatar"].value().to_string();
        if let Some(old) = old.filter(|old| *old != new) {
            std::fs::remove_file(on_disk(&old)).map_err(|e| fail(e.to_string()))?;
        }

        let avatar = avatar_of(db, id).await?;
        if avatar.as_deref() != Some(format!("avatars/{name}").as_str()) {
            return Err(fail(format!("avatar saved as {avatar:?}")));
        }
        if old_file.exists() {
            return Err(fail("the replaced file should be gone".into()));
        }
        if !new_file.is_file() {
            return Err(fail("the new file should be in place".into()));
        }
        Ok(())
    })
    .await
}

/// What the developer writes when the save is refused: the form comes back
/// holding the uploaded file's path, which the developer removes.
#[tokio::test]
async fn the_developer_removes_the_upload_of_a_refused_save() -> Result<(), TestFailure> {
    runique_test::<ADb>(super::ENV, async |db| {
        let mut disk = OnDisk::default();
        let username = unique("ghost");
        let name = format!("{}.png", unique("orphan"));
        let staged = disk.staged(&unique("t"), &name, PNG);
        let stored = disk.stored(&format!("avatars/{name}"));

        // The developer's handler, on a row that doesn't exist.
        let form = validated_profile(db, "2147483000", &username, &staged).await?;
        let Err(form) = form.save(db).await else {
            return Err(fail("a missing row should be refused".into()));
        };
        let uploaded = form.get_form().fields["avatar"].value().to_string();
        if uploaded != format!("avatars/{name}") {
            return Err(fail(format!("the form holds {uploaded:?}")));
        }
        std::fs::remove_file(on_disk(&uploaded)).map_err(|e| fail(e.to_string()))?;

        if stored.exists() {
            return Err(fail("the upload should be gone".into()));
        }
        Ok(())
    })
    .await
}
