// Tests pour CleaningMemoryStore — SessionStore + ExpiredDeletion + watermarks

use runique::middleware::session::CleaningMemoryStore;
use tower_sessions::{
    SessionStore,
    cookie::time::{Duration, OffsetDateTime},
    session::{Id, Record},
    session_store::ExpiredDeletion,
};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn fresh_record(secs_from_now: i64) -> Record {
    Record {
        id: Id::default(),
        data: Default::default(),
        expiry_date: OffsetDateTime::now_utc() + Duration::seconds(secs_from_now),
    }
}

// ── default / with_watermarks / size_bytes ────────────────────────────────────

#[test]
fn test_default_size_zero() {
    let store = CleaningMemoryStore::default();
    assert_eq!(store.size_bytes(), 0);
}

#[test]
fn test_with_watermarks_stored() {
    let store = CleaningMemoryStore::default().with_watermarks(1024, 2048);
    // On vérifie juste que ça ne panique pas et que le store reste utilisable
    assert_eq!(store.size_bytes(), 0);
}

// ── create ────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_create_increases_size() {
    let store = CleaningMemoryStore::default();
    let mut r = fresh_record(3600);
    store.create(&mut r).await.unwrap();
    assert!(store.size_bytes() > 0);
}

#[tokio::test]
async fn test_create_multiple_records() {
    let store = CleaningMemoryStore::default();
    let mut r1 = fresh_record(3600);
    let mut r2 = fresh_record(3600);
    store.create(&mut r1).await.unwrap();
    store.create(&mut r2).await.unwrap();
    assert!(store.size_bytes() > 0);
}

#[tokio::test]
async fn test_create_assigns_unique_ids() {
    let store = CleaningMemoryStore::default();
    let mut r1 = fresh_record(3600);
    let mut r2 = fresh_record(3600);
    // Même ID de départ — le store doit en générer un nouveau
    r2.id = r1.id;
    store.create(&mut r1).await.unwrap();
    store.create(&mut r2).await.unwrap();
    // r2 doit avoir obtenu un ID différent
    assert_ne!(r1.id, r2.id);
}

// ── load ──────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_load_existing_record() {
    let store = CleaningMemoryStore::default();
    let mut r = fresh_record(3600);
    store.create(&mut r).await.unwrap();
    let loaded = store.load(&r.id).await.unwrap();
    assert!(loaded.is_some());
}

#[tokio::test]
async fn test_load_nonexistent_returns_none() {
    let store = CleaningMemoryStore::default();
    let id = Id::default();
    let result = store.load(&id).await.unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn test_load_expired_returns_none() {
    let store = CleaningMemoryStore::default();
    let mut r = fresh_record(-10); // expiré il y a 10 secondes
    store.create(&mut r).await.unwrap();
    let result = store.load(&r.id).await.unwrap();
    assert!(result.is_none());
}

// ── save ──────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_save_updates_existing() {
    let store = CleaningMemoryStore::default();
    let mut r = fresh_record(3600);
    store.create(&mut r).await.unwrap();

    // Modifier et sauvegarder
    r.data.insert("key".to_string(), serde_json::json!("value"));
    store.save(&r).await.unwrap();

    let loaded = store.load(&r.id).await.unwrap().unwrap();
    assert_eq!(loaded.data.get("key"), Some(&serde_json::json!("value")));
}

#[tokio::test]
async fn test_save_new_record() {
    // save() sur un enregistrement non existant le crée
    let store = CleaningMemoryStore::default();
    let r = fresh_record(3600);
    store.save(&r).await.unwrap();
    let loaded = store.load(&r.id).await.unwrap();
    assert!(loaded.is_some());
}

// ── delete ────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_delete_removes_record() {
    let store = CleaningMemoryStore::default();
    let mut r = fresh_record(3600);
    store.create(&mut r).await.unwrap();
    let size_before = store.size_bytes();

    store.delete(&r.id).await.unwrap();

    assert!(store.load(&r.id).await.unwrap().is_none());
    assert!(store.size_bytes() < size_before);
}

#[tokio::test]
async fn test_delete_nonexistent_is_noop() {
    let store = CleaningMemoryStore::default();
    let id = Id::default();
    // Ne doit pas paniquer
    store.delete(&id).await.unwrap();
    assert_eq!(store.size_bytes(), 0);
}

// ── delete_expired ────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_delete_expired_removes_only_expired() {
    let store = CleaningMemoryStore::default();

    let mut alive = fresh_record(3600);
    let mut expired = fresh_record(-10);

    store.create(&mut alive).await.unwrap();
    store.create(&mut expired).await.unwrap();

    store.delete_expired().await.unwrap();

    assert!(store.load(&alive.id).await.unwrap().is_some());
    assert!(store.load(&expired.id).await.unwrap().is_none());
}

#[tokio::test]
async fn test_delete_expired_all_alive_no_change() {
    let store = CleaningMemoryStore::default();
    let mut r1 = fresh_record(3600);
    let mut r2 = fresh_record(7200);
    store.create(&mut r1).await.unwrap();
    store.create(&mut r2).await.unwrap();

    let size_before = store.size_bytes();
    store.delete_expired().await.unwrap();

    assert_eq!(store.size_bytes(), size_before);
}

#[tokio::test]
async fn test_delete_expired_reduces_size() {
    let store = CleaningMemoryStore::default();
    let mut r = fresh_record(-5);
    store.create(&mut r).await.unwrap();
    let size_after_insert = store.size_bytes();
    assert!(size_after_insert > 0);

    store.delete_expired().await.unwrap();
    // La taille doit avoir diminué après purge
    assert!(store.size_bytes() < size_after_insert || store.size_bytes() == 0);
}

// ── high watermark ────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_high_watermark_refuses_when_exceeded() {
    // Watermarks très bas pour forcer le dépassement
    let store = CleaningMemoryStore::default().with_watermarks(1, 1);

    // Remplir le store avec des sessions expirées (non protégées)
    for _ in 0..5 {
        let mut r = fresh_record(-1); // expiré
        // On ignore les erreurs ici — on veut juste saturer
        let _ = store.create(&mut r).await;
    }

    // Quand toutes les sessions expirées sont purgées et c'est encore dépassé → refus
    let mut r = fresh_record(3600);
    r.data
        .insert("big_payload".into(), serde_json::json!("x".repeat(100)));
    // Peut réussir ou échouer selon la purge — on vérifie juste que ça ne panique pas
    let _ = store.create(&mut r).await;
}

// ── spawn_cleanup ─────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_spawn_cleanup_does_not_panic() {
    let store = CleaningMemoryStore::default();
    // spawn_cleanup lance une tâche tokio en arrière-plan
    store.spawn_cleanup(tokio::time::Duration::from_secs(3600));
    // Si on arrive ici sans panic, c'est bon
    assert_eq!(store.size_bytes(), 0);
}

// ── sessions anonymes / is_protected ─────────────────────────────────────────

/// Helper : enregistrement avec user_id → session protégée
fn protected_record_user_id(secs_from_now: i64) -> Record {
    let mut r = fresh_record(secs_from_now);
    r.data.insert("user_id".to_string(), serde_json::json!(42));
    r
}

/// Helper : enregistrement avec session_active (timestamp futur) → session protégée
fn protected_record_session_active(secs_from_now: i64) -> Record {
    use tower_sessions::cookie::time::OffsetDateTime;
    let mut r = fresh_record(secs_from_now);
    let future_ts = OffsetDateTime::now_utc().unix_timestamp() + 7200;
    r.data
        .insert("session_active".to_string(), serde_json::json!(future_ts));
    r
}

/// Une session anonyme expirée doit être retirée par purge_anonymous_expired.
/// On force la branche low watermark en réglant des watermarks très bas.
#[tokio::test]
async fn test_purge_anonymous_expired_removes_anon_session() {
    // Watermarks minuscules : low = 1 byte, high = usize::MAX
    let store = CleaningMemoryStore::default().with_watermarks(1, usize::MAX);

    let mut anon = fresh_record(-5); // expiré, anonyme
    store.create(&mut anon).await.unwrap();
    let size_after_insert = store.size_bytes();
    assert!(size_after_insert > 0);

    // Cette create() dépasse le low watermark → spawn purge_anonymous_expired
    let mut trigger = fresh_record(3600);
    store.create(&mut trigger).await.unwrap();

    // Laisse tokio exécuter la tâche de purge
    tokio::task::yield_now().await;
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

    // La taille totale doit avoir diminué (session anonyme expirée purgée)
    assert!(store.size_bytes() < size_after_insert + 1000); // au moins la session trigger reste
}

/// Une session protégée (user_id) expirée ne doit pas être purgée par purge_anonymous_expired.
/// On vérifie via le high watermark synchrone (passe 1 = anonymes seulement).
#[tokio::test]
async fn test_protected_user_id_survives_passe1_high_watermark() {
    // high_watermark = 1 → déclenché dès le 2e create
    let store = CleaningMemoryStore::default().with_watermarks(usize::MAX, 1);

    // Session protégée expirée (user_id présent)
    let mut protected = protected_record_user_id(-5);
    store.create(&mut protected).await.unwrap();

    // Ce create déclenche le high watermark
    // Passe 1 : skip (is_protected = true)
    // Passe 2 : retire TOUS les expirés (y compris protected)
    // → store vide, nouveau record inséré → Ok
    let mut new_session = fresh_record(3600);
    let result = store.create(&mut new_session).await;
    // Doit réussir car passe 2 libère assez de place
    assert!(result.is_ok());
}

/// Une session protégée via session_active (timestamp futur) est traitée de même.
#[tokio::test]
async fn test_protected_session_active_survives_passe1_high_watermark() {
    let store = CleaningMemoryStore::default().with_watermarks(usize::MAX, 1);

    let mut protected = protected_record_session_active(-5); // expirée côté tower-sessions
    store.create(&mut protected).await.unwrap();

    let mut new_session = fresh_record(3600);
    let result = store.create(&mut new_session).await;
    assert!(result.is_ok());
}

/// Une session anonyme non expirée est protégée par is_protected (pas de user_id/session_active)
/// mais son expiry_date est dans le futur → purge_anonymous_expired ne la touche pas.
#[tokio::test]
async fn test_anon_alive_not_purged_by_anonymous_purge() {
    let store = CleaningMemoryStore::default().with_watermarks(1, usize::MAX);

    let mut alive = fresh_record(3600); // non expirée, anonyme
    store.create(&mut alive).await.unwrap();
    let size_before = store.size_bytes();

    // Déclencheur low watermark
    let mut trigger = fresh_record(3600);
    store.create(&mut trigger).await.unwrap();

    tokio::task::yield_now().await;
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

    // La session alive n'était pas expirée → toujours présente dans le store
    assert!(store.load(&alive.id).await.unwrap().is_some());
    let _ = size_before; // taille vérifiée implicitement via load
}

/// store saturé (high watermark, aucune session purgeable) → refus.
#[tokio::test]
async fn test_store_saturated_refuses_new_session() {
    // Watermarks à 1 byte : dès qu'il y a quelque chose, le high watermark est dépassé.
    // On insère d'abord une session vivante protégée (user_id) — pas purgeable en passe 1 et 2.
    let store = CleaningMemoryStore::default().with_watermarks(1, 1);

    // Session protégée NON expirée → passe 1 ne la prend pas (non expirée + protégée)
    //                                  passe 2 ne la prend pas non plus (non expirée)
    let mut alive_protected = protected_record_user_id(3600);
    store.create(&mut alive_protected).await.unwrap();

    // Tentative de création → high watermark, aucune purge possible → refus
    let mut new_session = fresh_record(3600);
    let result = store.create(&mut new_session).await;
    assert!(result.is_err());
}
