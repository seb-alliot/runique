use super::*;

/// The type the features of `runique` give, in cargo's description of the
/// project (`cargo metadata`): `pk-uuid`, else `big-pk`, else `i32`, the
/// order the framework itself follows.
pub(crate) fn pk_of(metadata: &str) -> Result<PkType> {
    let features = features_of(metadata)?;
    let has = |name: &str| features.iter().any(|feature| feature == name);
    Ok(if has("pk-uuid") {
        PkType::Uuid
    } else if has("big-pk") {
        PkType::I64
    } else {
        PkType::I32
    })
}
