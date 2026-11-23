pub(super) mod done;
pub(super) mod placed;
pub(super) mod prepared;
pub(super) mod replaced;
pub(super) mod waiting;

/// An action this step never takes: a fault in the batch itself, not in the
/// files it writes.
fn wrong_step<T>() -> std::io::Result<T> {
    Err(std::io::Error::new(
        std::io::ErrorKind::InvalidInput,
        runique_i18n::t(runique_i18n::keys::atomic::error::WRONG_STEP).into_owned(),
    ))
}
