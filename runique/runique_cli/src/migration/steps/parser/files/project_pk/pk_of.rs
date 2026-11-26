use super::*;

/// The type the features of `runique` give, in cargo's description of the
/// project (`cargo metadata`): `pk-uuid`, else `big-pk`, else `i32`, the
/// order the framework itself follows.
pub(super) fn pk_of(metadata: &str) -> Result<PkType> {
    let missing = || {
        anyhow!(t(runique_i18n::keys::makemigrations::error::PROJECT_PK_NO_RUNIQUE).into_owned())
    };
    let json: serde_json::Value = serde_json::from_str(metadata).map_err(|e| {
        anyhow!(tf(
            runique_i18n::keys::makemigrations::error::PROJECT_PK_CARGO,
            &[e.to_string()]
        ))
    })?;
    let id = json["packages"]
        .as_array()
        .and_then(|packages| {
            packages
                .iter()
                .find(|package| package["name"].as_str() == Some("runique"))
        })
        .and_then(|package| package["id"].as_str())
        .ok_or_else(missing)?;
    let features = json["resolve"]["nodes"]
        .as_array()
        .and_then(|nodes| nodes.iter().find(|node| node["id"].as_str() == Some(id)))
        .and_then(|node| node["features"].as_array())
        .ok_or_else(missing)?;
    let has = |name: &str| {
        features
            .iter()
            .any(|feature| feature.as_str() == Some(name))
    };
    Ok(if has("pk-uuid") {
        PkType::Uuid
    } else if has("big-pk") {
        PkType::I64
    } else {
        PkType::I32
    })
}
